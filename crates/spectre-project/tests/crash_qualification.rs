// Author: Jeff
// Date: 2026-08-29
// Description: R5 crash qualification for CORE-004 — a killed save never leaves a partial project
// Notes: R4-7's fault injection covers six of seven SaveStage values, and every one of them
//   UNWINDS: the injected error returns, the temporary is removed, and the function reports a
//   TargetState. A power loss does none of that. CORE-004's acceptance evidence is named
//   "crash-injection save tests at R5" precisely because the unwinding kind does not discharge it.
//
//   The kill point is randomized rather than chosen, because the interesting window -- inside
//   rename(2) -- cannot be addressed by name from outside the process. Many trials sample it; each
//   trial's ASSERTION is exact regardless of where the kill landed, which is what makes a random
//   probe sound evidence rather than a coin flip.

use spectre_project::{from_bytes, load_project, ProjectEnvelope};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

const TRIALS: usize = 24;
const SEED: u64 = 0x0043_5241_5348_0001;

fn scratch(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("spectre-crash-{name}"));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("scratch directory");
    path
}

// Locate the example binary cargo built beside this test.
//
// CARGO_BIN_EXE_* is set for [[bin]] targets only, and crash_saver is deliberately an example:
// making it a bin would put a test fixture into a library crate's shipped surface. The full gate's
// all-target Clippy step builds examples before tests. A standalone run must first use
// `cargo build --locked -p spectre-project --example crash_saver`. The test executable lives at
// target/<profile>/deps/<name>-<hash>, so the example is two levels up in examples/
fn saver_exe() -> PathBuf {
    let mut path = std::env::current_exe().expect("the test executable has a path");
    path.pop(); // deps/
    path.pop(); // <profile>/
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

// Start a saver and wait until it has completed at least one save, then return it.
//
// A fixed sleep before the kill is load-dependent and this drill proved it: under
// `cargo test --workspace`, with many test binaries competing, the saver is starved and a 60 ms
// window that samples the cycle on an idle machine samples nothing at all. Waiting for this
// child's completed-save signal makes the kill window relative to its own progress rather than a
// destination that may belong to an earlier trial
fn saver_past_first_save(path: &Path, seed: u64, torn: bool) -> Child {
    let mut child = spawn_saver(path, seed, torn);
    let stdout = child
        .stdout
        .take()
        .expect("the saver handshake pipe must be present");
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
        Ok(Ok((0, _))) => Some("the saver closed stdout before completing a save".to_owned()),
        Ok(Ok((_, line))) => Some(format!(
            "the saver emitted an unexpected handshake: {line:?}"
        )),
        Ok(Err(error)) => Some(format!("the saver handshake could not be read: {error}")),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            Some("the saver completed no save within 30 s; it is starved or broken".to_owned())
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            Some("the saver handshake reader stopped without a result".to_owned())
        }
    };
    if let Some(failure) = failure {
        let _ = child.kill();
        let _ = child.wait();
        panic!("{failure}");
    }
    child
}

fn spawn_saver(path: &Path, seed: u64, torn: bool) -> Child {
    let exe = saver_exe();
    let mut command = Command::new(exe);
    command.arg(path).arg(seed.to_string());
    if torn {
        command.arg("torn");
    }
    command
        .stdout(Stdio::piped())
        .spawn()
        .expect("the crash_saver example must build and run")
}

// Read the project the saver writes, by asking the saver itself rather than by restating its
// content here -- two descriptions of one fixture is how they drift apart
fn expected(directory: &Path) -> ProjectEnvelope {
    let settled = directory.join("settled.spectre");
    // One completed save, then stop: the signal is emitted after the whole reference artifact
    // exists, so no load-dependent grace period is needed
    let mut child = saver_past_first_save(&settled, SEED, false);
    let _ = child.kill();
    let _ = child.wait();
    load_project(&settled).expect("an unkilled saver must leave a loadable project")
}

#[test]
fn a_killed_save_never_leaves_a_partial_project() {
    let directory = scratch("partial");
    let reference = expected(&directory);
    let target = directory.join("take.spectre");

    let mut killed_with_target_present = 0_usize;
    for trial in 0..TRIALS {
        // Wait for a completed save, THEN kill at a random phase of the next cycle. The offsets
        // are coprime-ish multipliers rather than a PRNG so a failing trial is reproducible by
        // its index
        let mut child = saver_past_first_save(&target, SEED, false);
        std::thread::sleep(Duration::from_micros((trial as u64 * 7_919) % 23_000));
        child.kill().expect("the saver must be killable");
        child.wait().expect("the saver must be reapable");

        if !target.exists() {
            // Killed before the first rename completed. The requirement says nothing about a
            // project that was never written, so this trial is inconclusive rather than passing
            continue;
        }
        killed_with_target_present += 1;

        // The whole requirement: whatever is at the destination is a WHOLE project
        let bytes = std::fs::read(&target).expect("the target exists, so it reads");
        let decoded = from_bytes(&bytes).unwrap_or_else(|error| {
            panic!(
                "trial {trial}: the destination held a partial project after a kill: {error}\n\
                 {} bytes",
                bytes.len()
            )
        });
        assert_eq!(
            decoded, reference,
            "trial {trial}: the destination decoded but is not the project the saver writes"
        );
    }

    assert!(
        killed_with_target_present >= TRIALS / 2,
        "only {killed_with_target_present} of {TRIALS} trials had a destination to check; the \
         kill window is missing the save entirely and the drill proves little"
    );
}

// A temporary left behind by a crash is expected -- the cleanup never ran. What is NOT allowed is
// a leftover that a loader would accept as the project, or one that collides with the target name
#[test]
fn a_crash_leaves_no_temporary_that_could_pass_for_the_project() {
    let directory = scratch("temps");
    let target = directory.join("take.spectre");

    for trial in 0..TRIALS {
        let mut child = saver_past_first_save(&target, SEED, false);
        std::thread::sleep(Duration::from_micros((trial as u64 * 6_143) % 23_000));
        child.kill().expect("the saver must be killable");
        child.wait().expect("the saver must be reapable");
    }

    let target_name = target.file_name().expect("the target has a name");
    let mut leftovers = 0_usize;
    for entry in std::fs::read_dir(&directory).expect("the directory reads") {
        let entry = entry.expect("an entry reads");
        let name = entry.file_name();
        if name == target_name {
            continue;
        }
        leftovers += 1;
        // The contract's own naming rule: a temporary is a sibling carrying the temp suffix,
        // never the target's own name
        let text = name.to_string_lossy().into_owned();
        assert!(
            text.contains("spectre-tmp"),
            "an unexpected file survived a crash and is not a recognizable temporary: {text}"
        );
        assert_ne!(
            entry.path(),
            target,
            "a temporary took the target's own path, so a loader would open it as the project"
        );
    }
    // Not an assertion that leftovers exist -- a run that never crashed mid-write would leave
    // none, and that is also correct. This only records that the check had material when it did
    println!("crash leftovers inspected: {leftovers}");
}

// The negative control. A crash drill that cannot observe a torn write is not evidence that the
// save is atomic -- it is evidence that the kill window misses the write entirely, which would
// make both tests above pass against ANY implementation.
//
// crash_saver's "torn" mode truncates the destination and writes it in 64-byte chunks, the way a
// naive save would. Killing that must produce at least one destination that fails to decode.
#[test]
fn the_drill_detects_a_save_that_is_not_atomic() {
    let directory = scratch("torn");
    let target = directory.join("take.spectre");

    let mut torn_observed = 0_usize;
    let mut present = 0_usize;
    for trial in 0..TRIALS {
        let mut child = saver_past_first_save(&target, SEED, true);
        std::thread::sleep(Duration::from_micros((trial as u64 * 5_407) % 23_000));
        child.kill().expect("the saver must be killable");
        child.wait().expect("the saver must be reapable");

        if !target.exists() {
            continue;
        }
        present += 1;
        let bytes = std::fs::read(&target).expect("the target exists, so it reads");
        if from_bytes(&bytes).is_err() {
            torn_observed += 1;
        }
    }

    assert!(present > 0, "no trial produced a destination at all");
    assert!(
        torn_observed > 0,
        "{present} destinations were inspected and every one decoded cleanly, so this drill \
         cannot tell an atomic save from a truncating one and the two tests above prove nothing"
    );
}
