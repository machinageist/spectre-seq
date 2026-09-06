// Author: Jeff
// Date: 2026-09-06
// Description: R5 exit drill — a crash during autosave damages no project and offers whole work
// Notes: crash_qualification.rs kills a process saving THE PROJECT. This one kills a process
//   writing THE SIDECAR, which is the path the product actually runs between saves and which
//   makes a claim the other drill does not: journal.rs never opens the project file.
//
//   That claim has only ever been checked against a returned error. A crash returns nothing and
//   runs no cleanup, so it is the only way to find out whether an autosave can damage saved work
//   -- which would be the worst defect this milestone could ship, because the feature exists to
//   protect exactly that file.
//
//   The kill point is randomized for the same reason it is there: the interesting window is
//   inside the sidecar's own rename(2) and cannot be addressed by name from outside the process.

use spectre_project::recovery::{inspect, Recovery};
use spectre_project::{from_bytes, journal::journal_path, load_project, ProjectEnvelope};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const TRIALS: usize = 24;
const SEED: u64 = 0x0043_5241_5348_0002;

fn scratch(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("spectre-recovery-crash-{name}"));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("scratch directory");
    path
}

// See crash_qualification.rs: crash_saver is an example, not a bin, so CARGO_BIN_EXE_* is unset
fn saver_exe() -> PathBuf {
    let mut path = std::env::current_exe().expect("the test executable has a path");
    path.pop();
    path.pop();
    path.push("examples");
    path.push(if cfg!(windows) {
        "crash_saver.exe"
    } else {
        "crash_saver"
    });
    assert!(
        path.exists(),
        "the crash_saver example is not built at {}; build it with `cargo build --locked -p \
         spectre-project --example crash_saver` before this standalone test",
        path.display()
    );
    path
}

// Wait for the child's own completed-autosave handshake rather than sleeping a fixed time. Under
// a full workspace run the child competes with many test binaries and a window chosen on an idle
// machine samples nothing
fn saver_past_first_autosave(path: &Path) -> Child {
    let mut child = Command::new(saver_exe())
        .arg(path)
        .arg(SEED.to_string())
        .arg("autosave")
        .stdout(Stdio::piped())
        .spawn()
        .expect("the crash_saver example must build and run");
    let stdout = child.stdout.take().expect("the handshake pipe is present");
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let mut line = String::new();
        let result = BufReader::new(stdout)
            .read_line(&mut line)
            .map(|bytes| (bytes, line));
        let _ = sender.send(result);
    });
    let failure = match receiver.recv_timeout(Duration::from_secs(30)) {
        Ok(Ok((bytes, line))) if bytes > 0 && line.trim_end_matches(['\r', '\n']) == "saved" => {
            None
        }
        Ok(Ok((0, _))) => Some("the saver closed stdout before completing an autosave".to_owned()),
        Ok(Ok((_, line))) => Some(format!("unexpected handshake: {line:?}")),
        Ok(Err(error)) => Some(format!("the handshake could not be read: {error}")),
        Err(error) => Some(format!("the saver produced no handshake: {error}")),
    };
    if let Some(failure) = failure {
        let _ = child.kill();
        let _ = child.wait();
        panic!("{failure}");
    }
    child
}

// Ask the saver itself what a completed run leaves, rather than restating it here. Two
// descriptions of one fixture is how they drift apart
fn reference(directory: &Path) -> (Vec<u8>, ProjectEnvelope) {
    let settled = directory.join("settled.spectre");
    let mut child = saver_past_first_autosave(&settled);
    let _ = child.kill();
    let _ = child.wait();
    let project = std::fs::read(&settled).expect("the baseline project was written");
    let sidecar = load_project(&journal_path(&settled)).expect("one whole sidecar exists");
    (project, sidecar)
}

// The claim this milestone would be worst to get wrong: the thing that protects your work must
// not be able to destroy it
#[test]
fn a_crash_during_autosave_never_damages_the_saved_project() {
    let directory = scratch("project-intact");
    let (baseline, _) = reference(&directory);
    let target = directory.join("take.spectre");

    let mut killed_with_sidecar = 0_usize;
    for trial in 0..TRIALS {
        let mut child = saver_past_first_autosave(&target);
        std::thread::sleep(Duration::from_micros((trial as u64 * 7_919) % 23_000));
        child.kill().expect("the saver must be killable");
        child.wait().expect("the saver must be reapable");

        let project = std::fs::read(&target).unwrap_or_else(|error| {
            panic!("trial {trial}: the saved project is gone after an autosave crash: {error}")
        });
        assert_eq!(
            project, baseline,
            "trial {trial}: an autosave changed the saved project"
        );
        if journal_path(&target).exists() {
            killed_with_sidecar += 1;
        }
    }
    assert!(
        killed_with_sidecar >= TRIALS / 2,
        "only {killed_with_sidecar} of {TRIALS} trials left a sidecar; the kill window is \
         missing the autosave entirely and this drill proves little"
    );
}

// A sidecar is written by save_project_atomic, so it inherits the same all-or-nothing guarantee.
// A half-written sidecar would make recovery offer work that is not the musician's
#[test]
fn a_killed_autosave_never_leaves_a_partial_sidecar() {
    let directory = scratch("sidecar-whole");
    let (_, expected) = reference(&directory);
    let target = directory.join("take.spectre");

    let mut checked = 0_usize;
    for trial in 0..TRIALS {
        let mut child = saver_past_first_autosave(&target);
        std::thread::sleep(Duration::from_micros((trial as u64 * 6_143) % 23_000));
        child.kill().expect("the saver must be killable");
        child.wait().expect("the saver must be reapable");

        let sidecar = journal_path(&target);
        if !sidecar.exists() {
            // Killed before the first sidecar rename. Nothing is claimed about a file that was
            // never written, so this trial is inconclusive rather than passing
            continue;
        }
        checked += 1;
        let bytes = std::fs::read(&sidecar).expect("the sidecar exists, so it reads");
        let decoded = from_bytes(&bytes).unwrap_or_else(|error| {
            panic!(
                "trial {trial}: the sidecar held partial work after a kill: {error}\n{} bytes",
                bytes.len()
            )
        });
        assert_eq!(
            decoded, expected,
            "trial {trial}: the sidecar decoded but is not the work the saver journals"
        );
    }
    assert!(
        checked >= TRIALS / 2,
        "only {checked} of {TRIALS} trials had a sidecar to check"
    );
}

// The whole product path, end to end: crash, then the next launch offers the work and describes
// it. Recovery must never report a crash-written sidecar as "nothing to recover"
#[test]
fn the_next_launch_after_a_crash_offers_the_unsaved_work() {
    let directory = scratch("offered");
    let target = directory.join("take.spectre");

    let mut child = saver_past_first_autosave(&target);
    std::thread::sleep(Duration::from_millis(5));
    child.kill().expect("the saver must be killable");
    child.wait().expect("the saver must be reapable");
    assert!(
        journal_path(&target).exists(),
        "the crash left no sidecar, so the offer below could not be made"
    );

    match inspect(&target).expect("inspection succeeds after a crash") {
        Recovery::Available(offer) => {
            assert!(
                offer.autosaved.project.tracks.len() > offer.saved.project.tracks.len(),
                "the offer does not carry the unsaved work"
            );
            assert!(
                !offer.describe().is_empty(),
                "an offer was made with nothing said about it"
            );
            assert!(
                !offer.uncompared().is_empty(),
                "the offer claims its comparison was exhaustive"
            );
        }
        other => panic!("a crash-written sidecar was not offered back: {other:?}"),
    }

    // Inspection reports; it must not apply. Both files survive it untouched
    assert!(target.exists(), "inspection removed the saved project");
    assert!(
        journal_path(&target).exists(),
        "inspection consumed the sidecar it reported"
    );
}
