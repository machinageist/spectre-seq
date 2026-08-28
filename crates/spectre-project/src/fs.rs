// Author: Jeff
// Date: 2026-08-28
// Description: CORE-004's atomic project save and the bounded, validating load beside it
// Notes: App-thread only. Both calls are synchronous, allocate freely, and are never reachable
//   from an audio callback. Nothing here holds process-global state, a lock, or a target-keyed
//   queue: two saves to one destination are the app's problem to prevent, not this crate's to
//   serialize behind the caller's back.

use crate::{to_bytes, validate_envelope, ProjectEnvelope, ProjectError, ValidationError};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

// How many temporary names a save will try before giving up. The name mixes the process ID, the
// nanosecond field of the wall clock, and a monotonic counter, so a collision needs a leftover
// temporary whose three components all match. The contract requires the retry; the retry must be
// bounded, because this runs synchronously on the UI thread and an unbounded loop over a
// pathological directory would hang the window instead of failing. Eight is small enough that the
// worst case is imperceptible and large enough that an accidental collision cannot exhaust it
pub const SAVE_TEMP_NAME_ATTEMPTS: u32 = 8;

// Refusal ceiling on a project file's size. load_project reads a file that may be truncated,
// corrupt, or not a project at all before it can know anything about it; without a bound the read
// allocates whatever the file claims to be. The checked-in R1 fixture is 912 bytes and a schema-2
// project with these collections is a few kilobytes, so 64 MiB is four orders of magnitude of
// headroom while keeping the app-thread allocation bounded. It also bounds the validator's
// uniqueness set, which is sized by the document. A starting envelope for R4, expected to be
// revisited when a later milestone persists sample or media references
pub const MAX_PROJECT_FILE_BYTES: u64 = 64 * 1024 * 1024;

// Temporary-name suffix. Hidden and distinctive so a leftover is recognizable rather than mistaken
// for the musician's own file
const TEMP_SUFFIX: &str = "spectre-tmp";

// Name entropy only. Holds no target, no lock, and no ordering guarantee, so it is not the
// process-global persistence state the contract forbids
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

// Where a save stopped. Seven stages, because the caller's recovery differs at each one
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SaveStage {
    ValidateSnapshot,
    EncodeSnapshot,
    CreateTemporary,
    WriteTemporary,
    SyncTemporary,
    ReplaceTarget,
    SyncParentDirectory,
}

// What the destination holds after a save returns. The middle variant is the whole point: a
// failure before the rename leaves the old project exactly as it was
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetState {
    Unchanged,
    ReplacedDurable,
    ReplacedDurabilityUncertain,
}

// What a completed save guarantees. Always ReplacedDurable on Ok
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveReceipt {
    pub target_state: TargetState,
}

#[derive(Debug)]
pub enum SaveError {
    InvalidProject {
        reason: ValidationError,
    },
    Encode {
        source: serde_json::Error,
    },
    Io {
        stage: SaveStage,
        target_state: TargetState,
        source: io::Error,
    },
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidProject { reason } => write!(f, "project is not valid: {reason}"),
            Self::Encode { source } => write!(f, "project could not be encoded: {source}"),
            Self::Io {
                stage,
                target_state,
                source,
            } => {
                let state = match target_state {
                    TargetState::Unchanged => "the existing file is unchanged",
                    TargetState::ReplacedDurable => "the file was replaced",
                    TargetState::ReplacedDurabilityUncertain => {
                        "the file was replaced but may not survive a power loss"
                    }
                };
                write!(f, "save failed at {stage:?}: {source} — {state}")
            }
        }
    }
}

impl std::error::Error for SaveError {}

#[derive(Debug)]
pub enum LoadError {
    Open { path: PathBuf, source: io::Error },
    Read { path: PathBuf, source: io::Error },
    SchemaTooNew { found: u32, max_readable: u32 },
    Malformed { source: serde_json::Error },
    InvalidProject { reason: ValidationError },
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open { path, source } => write!(f, "cannot open {}: {source}", path.display()),
            Self::Read { path, source } => write!(f, "cannot read {}: {source}", path.display()),
            Self::SchemaTooNew {
                found,
                max_readable,
            } => write!(
                f,
                "project schema {found} is newer than this build can read (max {max_readable})"
            ),
            Self::Malformed { source } => write!(f, "malformed project: {source}"),
            Self::InvalidProject { reason } => write!(f, "invalid project: {reason}"),
        }
    }
}

impl std::error::Error for LoadError {}

// Read, decode, schema-gate, and semantically validate a complete project.
// No failure returns a partial project, and no failure path writes anything
pub fn load_project(path: &Path) -> Result<ProjectEnvelope, LoadError> {
    let bytes = read_bounded(path, MAX_PROJECT_FILE_BYTES)?;
    match crate::from_bytes(&bytes) {
        Ok(envelope) => Ok(envelope),
        Err(ProjectError::SchemaTooNew {
            found,
            max_readable,
        }) => Err(LoadError::SchemaTooNew {
            found,
            max_readable,
        }),
        Err(ProjectError::Malformed(source)) => Err(LoadError::Malformed { source }),
        Err(ProjectError::InvalidProject(reason)) => Err(LoadError::InvalidProject {
            reason: ValidationError { reason },
        }),
    }
}

// Read a whole file, refusing anything past `max` before allocating for it. The length is checked
// first and the read is still bounded at max + 1, so a file that grows between the two steps is
// refused rather than read
fn read_bounded(path: &Path, max: u64) -> Result<Vec<u8>, LoadError> {
    let file = File::open(path).map_err(|source| LoadError::Open {
        path: path.to_path_buf(),
        source,
    })?;
    let too_large = |path: &Path| LoadError::Read {
        path: path.to_path_buf(),
        source: io::Error::new(
            io::ErrorKind::InvalidData,
            format!("project file exceeds the {max}-byte maximum"),
        ),
    };

    let length = file
        .metadata()
        .map_err(|source| LoadError::Read {
            path: path.to_path_buf(),
            source,
        })?
        .len();
    if length > max {
        return Err(too_large(path));
    }

    let mut bytes = Vec::with_capacity(length as usize);
    file.take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(|source| LoadError::Read {
            path: path.to_path_buf(),
            source,
        })?;
    if bytes.len() as u64 > max {
        return Err(too_large(path));
    }
    Ok(bytes)
}

// Validate, encode, and atomically replace the target with a synchronized sibling
pub fn save_project_atomic(
    path: &Path,
    snapshot: &ProjectEnvelope,
) -> Result<SaveReceipt, SaveError> {
    save_with(&RealFs, path, snapshot)
}

// The filesystem operations a save performs, named so a test can fail any one of them.
// Private on purpose: the contract forbids this becoming application-global state or a public
// pluggable filesystem API
trait FsOps {
    type Handle;

    // Exclusive creation. O_CREAT|O_EXCL, so the kernel decides and an existing file is never
    // truncated by a check-then-create race
    fn create_new(&self, path: &Path) -> io::Result<Self::Handle>;
    fn write_all(&self, handle: &mut Self::Handle, bytes: &[u8]) -> io::Result<()>;
    fn sync_file(&self, handle: &mut Self::Handle) -> io::Result<()>;
    fn close(&self, handle: Self::Handle);
    fn replace(&self, from: &Path, to: &Path) -> io::Result<()>;
    fn sync_parent_dir(&self, path: &Path) -> io::Result<()>;
    fn remove(&self, path: &Path) -> io::Result<()>;
}

struct RealFs;

impl FsOps for RealFs {
    type Handle = File;

    fn create_new(&self, path: &Path) -> io::Result<File> {
        OpenOptions::new().write(true).create_new(true).open(path)
    }

    fn write_all(&self, handle: &mut File, bytes: &[u8]) -> io::Result<()> {
        handle.write_all(bytes)
    }

    fn sync_file(&self, handle: &mut File) -> io::Result<()> {
        handle.sync_all()
    }

    fn close(&self, handle: File) {
        drop(handle);
    }

    fn replace(&self, from: &Path, to: &Path) -> io::Result<()> {
        std::fs::rename(from, to)
    }

    // Flushes the directory entry, so the replacement itself survives a crash rather than only
    // the data it points at
    fn sync_parent_dir(&self, path: &Path) -> io::Result<()> {
        File::open(parent_of(path))?.sync_all()
    }

    fn remove(&self, path: &Path) -> io::Result<()> {
        std::fs::remove_file(path)
    }
}

// The target's existing parent, or the working directory for a bare filename. The temporary is
// always a sibling, so the rename never crosses a filesystem
fn parent_of(path: &Path) -> PathBuf {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

// A collision-resistant sibling name. Never fixed and never predictable: a fixed name would make
// two concurrent saves silently share one temporary
fn temp_path(path: &Path, attempt: u32) -> PathBuf {
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "project".into());
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.subsec_nanos())
        .unwrap_or(0);
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed).wrapping_add(
        // The attempt is folded in so a retry cannot reproduce the previous name even if the
        // clock and the counter both stand still
        u64::from(attempt),
    );
    parent_of(path).join(format!(
        ".{name}.{pid:x}-{nanos:x}-{counter:x}.{TEMP_SUFFIX}"
    ))
}

// The one save algorithm. Steps in the contract's required order; the failure of each names its
// own stage and says what the destination holds
fn save_with<F: FsOps>(
    fs: &F,
    path: &Path,
    snapshot: &ProjectEnvelope,
) -> Result<SaveReceipt, SaveError> {
    // 1 — validate. A semantically broken project can never reach the disk, and nothing has
    // been created when this fails
    validate_envelope(snapshot).map_err(|reason| SaveError::InvalidProject { reason })?;

    // 2 — encode. The complete bytes exist in memory before anything is created
    let bytes = to_bytes(snapshot).map_err(|error| match error {
        ProjectError::Malformed(source) => SaveError::Encode { source },
        // to_bytes returns only Malformed; the other variants belong to decoding
        other => SaveError::Io {
            stage: SaveStage::EncodeSnapshot,
            target_state: TargetState::Unchanged,
            source: io::Error::other(other.to_string()),
        },
    })?;

    // 3 — create the temporary, retrying a name collision without ever truncating another file
    let mut temp = PathBuf::new();
    let mut handle = None;
    let mut last = io::Error::new(io::ErrorKind::AlreadyExists, "no attempt was made");
    for attempt in 0..SAVE_TEMP_NAME_ATTEMPTS {
        let candidate = temp_path(path, attempt);
        match fs.create_new(&candidate) {
            Ok(opened) => {
                temp = candidate;
                handle = Some(opened);
                break;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => last = error,
            Err(error) => {
                return Err(SaveError::Io {
                    stage: SaveStage::CreateTemporary,
                    target_state: TargetState::Unchanged,
                    source: error,
                })
            }
        }
    }
    let Some(mut handle) = handle else {
        return Err(SaveError::Io {
            stage: SaveStage::CreateTemporary,
            target_state: TargetState::Unchanged,
            source: last,
        });
    };

    // Cleanup is synchronous, best effort, and never masks the primary error
    let cleanup = |fs: &F, temp: &Path| {
        let _ = fs.remove(temp);
    };

    // 4 — write. write_all loops until every byte is written; a short write is not success
    if let Err(source) = fs.write_all(&mut handle, &bytes) {
        fs.close(handle);
        cleanup(fs, &temp);
        return Err(SaveError::Io {
            stage: SaveStage::WriteTemporary,
            target_state: TargetState::Unchanged,
            source,
        });
    }

    // 5 — sync the contents to the device before any name points at them. Without this, a rename
    // can publish a file whose data is still only in the page cache
    if let Err(source) = fs.sync_file(&mut handle) {
        fs.close(handle);
        cleanup(fs, &temp);
        return Err(SaveError::Io {
            stage: SaveStage::SyncTemporary,
            target_state: TargetState::Unchanged,
            source,
        });
    }

    // 6 — release the handle before replacement
    fs.close(handle);

    // 7 — replace. rename(2) is atomic with respect to other processes: there is no window in
    // which the destination is missing or half-written. This is the commit point, and the
    // destination is never deleted, truncated, or moved aside first
    if let Err(source) = fs.replace(&temp, path) {
        cleanup(fs, &temp);
        return Err(SaveError::Io {
            stage: SaveStage::ReplaceTarget,
            target_state: TargetState::Unchanged,
            source,
        });
    }

    // 8 — sync the directory entry. The one failure that is not Unchanged: the replacement did
    // happen, so the caller is told the data is there but its durability is uncertain. No second
    // replacement is attempted, because the first one succeeded
    if let Err(source) = fs.sync_parent_dir(path) {
        return Err(SaveError::Io {
            stage: SaveStage::SyncParentDirectory,
            target_state: TargetState::ReplacedDurabilityUncertain,
            source,
        });
    }

    // 9 — receipt, only after every required synchronization succeeded
    Ok(SaveReceipt {
        target_state: TargetState::ReplacedDurable,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ProjectDoc, TrackList, ViewDoc, SCHEMA_VERSION};
    use serde_json::Map;
    use spectre_core::{IdGen, TempoMap, Transport};
    use std::cell::RefCell;

    // A valid schema-2 envelope. Modeled on the helper in src/lib.rs's own test module
    fn envelope() -> ProjectEnvelope {
        let mut ids = IdGen::new(0x0053_4156_4500);
        ProjectEnvelope {
            schema_version: SCHEMA_VERSION,
            project: ProjectDoc {
                id: ids.next_id(),
                name: "Save Test".into(),
                tempo_map: TempoMap::constant(120.0).unwrap(),
                transport: Transport::new(),
                id_gen_state: ids.state(),
                tracks: TrackList::new(),
                devices: Vec::new(),
                view: ViewDoc::default(),
                unknown: Map::new(),
            },
            unknown: Map::new(),
        }
    }

    // One recorded filesystem operation, with the path it named
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Call {
        CreateNew(PathBuf),
        WriteAll(usize),
        SyncFile,
        Close,
        Replace(PathBuf, PathBuf),
        SyncParentDir(PathBuf),
        Remove(PathBuf),
    }

    // Records every call in order and fails at one chosen stage. Private, per the contract:
    // an injection seam must never become application-global state or a public filesystem API
    struct FaultFs {
        log: RefCell<Vec<Call>>,
        fail_at: Option<SaveStage>,
        // Number of create_new calls that return AlreadyExists before one succeeds; usize::MAX
        // means every attempt collides
        collisions: usize,
        fail_remove: bool,
    }

    impl FaultFs {
        fn new() -> Self {
            Self {
                log: RefCell::new(Vec::new()),
                fail_at: None,
                collisions: 0,
                fail_remove: false,
            }
        }

        fn failing(stage: SaveStage) -> Self {
            Self {
                fail_at: Some(stage),
                ..Self::new()
            }
        }

        fn log(&self) -> Vec<Call> {
            self.log.borrow().clone()
        }

        fn creates(&self) -> Vec<PathBuf> {
            self.log()
                .into_iter()
                .filter_map(|call| match call {
                    Call::CreateNew(path) => Some(path),
                    _ => None,
                })
                .collect()
        }

        fn fails(&self, stage: SaveStage) -> bool {
            self.fail_at == Some(stage)
        }
    }

    fn injected() -> io::Error {
        io::Error::other("injected")
    }

    impl FsOps for FaultFs {
        // No real handle exists; the count is what the log records
        type Handle = ();

        fn create_new(&self, path: &Path) -> io::Result<()> {
            self.log.borrow_mut().push(Call::CreateNew(path.into()));
            let attempts = self.creates().len();
            if self.collisions == usize::MAX || attempts <= self.collisions {
                return Err(io::Error::new(io::ErrorKind::AlreadyExists, "collision"));
            }
            if self.fails(SaveStage::CreateTemporary) {
                return Err(injected());
            }
            Ok(())
        }

        fn write_all(&self, _handle: &mut (), bytes: &[u8]) -> io::Result<()> {
            self.log.borrow_mut().push(Call::WriteAll(bytes.len()));
            if self.fails(SaveStage::WriteTemporary) {
                return Err(injected());
            }
            Ok(())
        }

        fn sync_file(&self, _handle: &mut ()) -> io::Result<()> {
            self.log.borrow_mut().push(Call::SyncFile);
            if self.fails(SaveStage::SyncTemporary) {
                return Err(injected());
            }
            Ok(())
        }

        fn close(&self, _handle: ()) {
            self.log.borrow_mut().push(Call::Close);
        }

        fn replace(&self, from: &Path, to: &Path) -> io::Result<()> {
            self.log
                .borrow_mut()
                .push(Call::Replace(from.into(), to.into()));
            if self.fails(SaveStage::ReplaceTarget) {
                return Err(injected());
            }
            Ok(())
        }

        fn sync_parent_dir(&self, path: &Path) -> io::Result<()> {
            self.log.borrow_mut().push(Call::SyncParentDir(path.into()));
            if self.fails(SaveStage::SyncParentDirectory) {
                return Err(injected());
            }
            Ok(())
        }

        fn remove(&self, path: &Path) -> io::Result<()> {
            self.log.borrow_mut().push(Call::Remove(path.into()));
            if self.fail_remove {
                return Err(io::Error::other("cleanup also failed"));
            }
            Ok(())
        }
    }

    fn target() -> PathBuf {
        PathBuf::from("/tmp/spectre-fault/project.spectre")
    }

    // Decode a hand-built document that no constructor would accept. TrackList::insert refuses
    // a duplicate ObjectId at runtime, so only Deserialize can produce one
    fn envelope_from_json(project_patch: serde_json::Value) -> ProjectEnvelope {
        let mut value = serde_json::to_value(envelope()).unwrap();
        let object = value["project"].as_object_mut().unwrap();
        for (key, patched) in project_patch.as_object().unwrap() {
            object.insert(key.clone(), patched.clone());
        }
        serde_json::from_value(value).unwrap()
    }

    // U1
    #[test]
    fn validation_failure_touches_no_path() {
        let duplicate = serde_json::json!({
            "tracks": {
                "tracks": [
                    { "id": 77, "name": "A", "instrument": "Pulse",
                      "instrument_level": 0.3, "level": 1.0, "muted": false, "soloed": false,
                      "clips": { "placements": [], "lengths": [] } },
                    { "id": 77, "name": "B", "instrument": "Pulse",
                      "instrument_level": 0.3, "level": 1.0, "muted": false, "soloed": false,
                      "clips": { "placements": [], "lengths": [] } }
                ],
                "clips": [],
                "master_level": 1.0
            }
        });
        let fs = FaultFs::new();
        let result = save_with(&fs, &target(), &envelope_from_json(duplicate));

        assert!(matches!(result, Err(SaveError::InvalidProject { .. })));
        // The whole point: validation precedes every filesystem touch
        assert!(fs.log().is_empty());
    }

    // U2
    #[test]
    fn temporary_creation_failure_leaves_target_unchanged() {
        let fs = FaultFs::failing(SaveStage::CreateTemporary);
        let result = save_with(&fs, &target(), &envelope());

        assert!(matches!(
            result,
            Err(SaveError::Io {
                stage: SaveStage::CreateTemporary,
                target_state: TargetState::Unchanged,
                ..
            })
        ));
        assert!(!fs
            .log()
            .iter()
            .any(|call| matches!(call, Call::WriteAll(_) | Call::Replace(..))));
    }

    // U3
    #[test]
    fn temporary_name_collision_retries_without_truncating() {
        let fs = FaultFs {
            collisions: 2,
            ..FaultFs::new()
        };
        let result = save_with(&fs, &target(), &envelope());
        assert!(result.is_ok());

        let creates = fs.creates();
        assert_eq!(creates.len(), 3);
        // Three distinct names, so a retry never reuses the name that collided
        let mut unique: Vec<_> = creates.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), 3);
        // Every attempt is exclusive creation; nothing opens an existing path for writing
        assert!(creates.iter().all(|path| path != &target()));
    }

    // U4
    #[test]
    fn temporary_name_retries_are_bounded() {
        let fs = FaultFs {
            collisions: usize::MAX,
            ..FaultFs::new()
        };
        let result = save_with(&fs, &target(), &envelope());

        assert!(matches!(
            result,
            Err(SaveError::Io {
                stage: SaveStage::CreateTemporary,
                ..
            })
        ));
        // The bound is real: no unbounded loop on the UI thread
        assert_eq!(fs.creates().len(), SAVE_TEMP_NAME_ATTEMPTS as usize);
    }

    // U5
    #[test]
    fn write_failure_cleans_up_and_preserves_target() {
        let fs = FaultFs::failing(SaveStage::WriteTemporary);
        let result = save_with(&fs, &target(), &envelope());

        assert!(matches!(
            result,
            Err(SaveError::Io {
                stage: SaveStage::WriteTemporary,
                target_state: TargetState::Unchanged,
                ..
            })
        ));
        let log = fs.log();
        assert!(matches!(log.last(), Some(Call::Remove(_))));
        assert!(!log.iter().any(|call| matches!(call, Call::Replace(..))));
    }

    // U6
    #[test]
    fn temporary_sync_failure_preserves_target() {
        let fs = FaultFs::failing(SaveStage::SyncTemporary);
        let result = save_with(&fs, &target(), &envelope());

        assert!(matches!(
            result,
            Err(SaveError::Io {
                stage: SaveStage::SyncTemporary,
                target_state: TargetState::Unchanged,
                ..
            })
        ));
        let log = fs.log();
        assert!(log.iter().any(|call| matches!(call, Call::Remove(_))));
        assert!(!log.iter().any(|call| matches!(call, Call::Replace(..))));
    }

    // U7
    #[test]
    fn replacement_failure_never_deletes_first() {
        let fs = FaultFs::failing(SaveStage::ReplaceTarget);
        let result = save_with(&fs, &target(), &envelope());

        assert!(matches!(
            result,
            Err(SaveError::Io {
                stage: SaveStage::ReplaceTarget,
                target_state: TargetState::Unchanged,
                ..
            })
        ));
        // The destination is never deleted, truncated, or moved aside — only the temporary is
        assert!(!fs
            .log()
            .iter()
            .any(|call| matches!(call, Call::Remove(path) if path == &target())));
    }

    // U8
    #[test]
    fn parent_sync_failure_reports_durability_uncertain() {
        let fs = FaultFs::failing(SaveStage::SyncParentDirectory);
        let result = save_with(&fs, &target(), &envelope());

        assert!(matches!(
            result,
            Err(SaveError::Io {
                stage: SaveStage::SyncParentDirectory,
                target_state: TargetState::ReplacedDurabilityUncertain,
                ..
            })
        ));
        let log = fs.log();
        let replacements = log
            .iter()
            .filter(|call| matches!(call, Call::Replace(..)))
            .count();
        // Exactly one, and nothing after it: the replacement succeeded and is not retried
        assert_eq!(replacements, 1);
        let after = log
            .iter()
            .skip_while(|call| !matches!(call, Call::Replace(..)));
        assert!(!after.clone().any(|call| matches!(call, Call::Remove(_))));
    }

    // U9
    #[test]
    fn cleanup_failure_does_not_mask_primary_error() {
        let fs = FaultFs {
            fail_at: Some(SaveStage::WriteTemporary),
            fail_remove: true,
            ..FaultFs::new()
        };
        let result = save_with(&fs, &target(), &envelope());

        // Still the write failure, not the cleanup failure
        assert!(matches!(
            result,
            Err(SaveError::Io {
                stage: SaveStage::WriteTemporary,
                ..
            })
        ));
    }

    // U10
    #[test]
    fn bounded_read_refuses_oversize() {
        let path = std::env::temp_dir().join("spectre-bounded-read.json");
        std::fs::write(&path, vec![b'x'; 64]).unwrap();

        let error = read_bounded(&path, 16).unwrap_err();
        match error {
            LoadError::Read { source, .. } => {
                assert_eq!(source.kind(), io::ErrorKind::InvalidData);
            }
            other => panic!("expected a bounded-read refusal, got {other:?}"),
        }
        // The same file under the bound still reads
        assert_eq!(read_bounded(&path, 64).unwrap().len(), 64);
        std::fs::remove_file(&path).unwrap();
    }

    // U11
    #[test]
    fn ordering_is_validate_encode_create() {
        let fs = FaultFs::new();
        save_with(&fs, &target(), &envelope()).unwrap();

        let log = fs.log();
        assert!(matches!(log[0], Call::CreateNew(_)));
        assert!(matches!(log[1], Call::WriteAll(_)));
        assert!(matches!(log[2], Call::SyncFile));
        let sync = log
            .iter()
            .position(|c| matches!(c, Call::SyncFile))
            .unwrap();
        let replace = log
            .iter()
            .position(|c| matches!(c, Call::Replace(..)))
            .unwrap();
        // The data is durable before any name points at it
        assert!(sync < replace);
        // And the handle is released before the replacement, as the contract requires
        let close = log.iter().position(|c| matches!(c, Call::Close)).unwrap();
        assert!(close < replace);
    }

    // The temporary is always a sibling of the target, so the rename never crosses a filesystem
    #[test]
    fn the_temporary_is_a_hidden_sibling_of_the_target() {
        let fs = FaultFs::new();
        save_with(&fs, &target(), &envelope()).unwrap();

        let temp = fs.creates().pop().unwrap();
        assert_eq!(temp.parent(), target().parent());
        let name = temp.file_name().unwrap().to_string_lossy().into_owned();
        assert!(name.starts_with('.'), "{name}");
        assert!(name.ends_with(TEMP_SUFFIX), "{name}");
        assert!(name.contains("project.spectre"), "{name}");
    }

    // A bare filename saves into the working directory rather than into the filesystem root
    #[test]
    fn a_bare_filename_uses_the_working_directory() {
        let fs = FaultFs::new();
        save_with(&fs, Path::new("take.spectre"), &envelope()).unwrap();

        let temp = fs.creates().pop().unwrap();
        assert_eq!(temp.parent(), Some(Path::new(".")));
    }
}
