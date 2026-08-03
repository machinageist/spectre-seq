// =============================================================================
// File: crates/spectre-project/src/autosave.rs
// Layer: project persistence
// Purpose: Atomic saves, crash-recovery scanning, background autosave thread
// Status: Implemented; temp+rename writes and an RAII autosaver handle.
// Notes: Saves write a sibling temp file then rename over the target so a crash
//        never leaves a half-written project. The autosaver snapshots live state
//        on a worker thread; stopping or dropping the handle joins it cleanly.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::schema::ProjectFile;
use crate::serialize::{to_cbor, ProjectError};

// Extension marking a recoverable autosave sidecar
const AUTOSAVE_EXT: &str = "geist-autosave";
// Default cadence for the background autosaver
pub const DEFAULT_AUTOSAVE_INTERVAL: Duration = Duration::from_secs(60);
// Granularity at which the worker checks for a stop request
const POLL_SLICE: Duration = Duration::from_millis(10);

// Atomically write a project as CBOR: temp file then rename into place
pub fn atomic_write_cbor(
    project: &ProjectFile,
    path: impl AsRef<Path>,
) -> Result<(), ProjectError> {
    let path = path.as_ref();
    let bytes = to_cbor(project)?;
    let tmp = temp_sibling(path);
    std::fs::write(&tmp, &bytes)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

// Build a unique sibling temp path next to the target on the same filesystem
fn temp_sibling(path: &Path) -> PathBuf {
    let pid = std::process::id();
    let mut name = path
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    name.push(format!(".{pid}.tmp"));
    match path.parent() {
        Some(dir) => dir.join(name),
        None => PathBuf::from(name),
    }
}

// Autosave sidecar path for a project name inside a directory
pub fn autosave_path(dir: impl AsRef<Path>, project_name: &str) -> PathBuf {
    dir.as_ref().join(format!("{project_name}.{AUTOSAVE_EXT}"))
}

// Scan a directory for autosave files left behind by a prior session
pub fn find_recovery_files(dir: impl AsRef<Path>) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|x| x.to_str()) == Some(AUTOSAVE_EXT) {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

// RAII handle to a background autosave worker
pub struct Autosaver {
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Autosaver {
    // Spawn a worker that snapshots and atomically writes every interval
    // `source` produces the current project state when the timer fires
    pub fn start<F>(path: impl Into<PathBuf>, interval: Duration, source: F) -> Self
    where
        F: Fn() -> ProjectFile + Send + 'static,
    {
        let path = path.into();
        let running = Arc::new(AtomicBool::new(true));
        let flag = running.clone();
        let handle = thread::spawn(move || {
            while flag.load(Ordering::Relaxed) {
                if !sleep_interruptibly(&flag, interval) {
                    break;
                }
                let snapshot = source();
                let _ = atomic_write_cbor(&snapshot, &path);
            }
        });
        Self {
            running,
            handle: Some(handle),
        }
    }

    // Stop the worker and wait for it to finish its current cycle
    pub fn stop(mut self) {
        self.shutdown();
    }

    // Signal stop and join; idempotent across stop() and Drop
    fn shutdown(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for Autosaver {
    fn drop(&mut self) {
        self.shutdown();
    }
}

// Sleep up to `interval` in slices; return false if stop is requested
fn sleep_interruptibly(flag: &AtomicBool, interval: Duration) -> bool {
    let mut waited = Duration::ZERO;
    while waited < interval {
        if !flag.load(Ordering::Relaxed) {
            return false;
        }
        let slice = POLL_SLICE.min(interval - waited);
        thread::sleep(slice);
        waited += slice;
    }
    flag.load(Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialize::load_from_path;
    use std::time::{SystemTime, UNIX_EPOCH};

    // Create and return a unique temp directory
    fn temp_dir(tag: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("spectre_{tag}_{nanos}"));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn atomic_write_reloads_and_leaves_no_temp() {
        let dir = temp_dir("atomic");
        let path = dir.join("proj.spectre");
        atomic_write_cbor(&ProjectFile::new("atomic"), &path).unwrap();

        let back = load_from_path(&path).unwrap();
        assert_eq!(back.meta.name, "atomic");

        // The directory should hold only the final file, no .tmp residue
        let names: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["proj.spectre".to_string()]);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn autosave_path_uses_recovery_extension() {
        let p = autosave_path("/tmp/x", "song");
        assert_eq!(p, PathBuf::from("/tmp/x/song.geist-autosave"));
        assert_eq!(p.extension().unwrap(), "geist-autosave");
    }

    #[test]
    fn find_recovery_files_filters_by_extension() {
        let dir = temp_dir("recovery");
        atomic_write_cbor(&ProjectFile::new("a"), autosave_path(&dir, "a")).unwrap();
        atomic_write_cbor(&ProjectFile::new("b"), autosave_path(&dir, "b")).unwrap();
        // A normal project file must not be reported as a recovery candidate
        atomic_write_cbor(&ProjectFile::new("c"), dir.join("c.geist")).unwrap();

        let found = find_recovery_files(&dir);
        assert_eq!(found.len(), 2);
        assert!(found
            .iter()
            .all(|p| p.extension().unwrap() == "geist-autosave"));

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn autosaver_writes_then_stops_cleanly() {
        let dir = temp_dir("worker");
        let path = autosave_path(&dir, "session");
        let saver = Autosaver::start(path.clone(), Duration::from_millis(20), || {
            ProjectFile::new("auto")
        });

        // Poll up to ~3s for the first autosave to land
        let mut wrote = false;
        for _ in 0..150 {
            if path.exists() {
                wrote = true;
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        saver.stop();
        assert!(wrote, "autosave file was never written");

        let recovered = load_from_path(&path).unwrap();
        assert_eq!(recovered.meta.name, "auto");

        std::fs::remove_dir_all(&dir).ok();
    }
}
