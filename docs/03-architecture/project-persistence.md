<!--
Author: Jeff
Date: 2026-07-16
Description: Project load and atomic-save API contract for the application thread
Notes: R1 design authority for R4 persistence and R5 crash-recovery qualification
-->

# Project Persistence Contract

- **Status:** accepted for R4/R5 implementation
- **Last verified:** 2026-07-16
- **Scope:** synchronous project-file load and atomic replacement boundaries
- **Decision authority:** Jeff
- **Upstream sources:** [CORE-001, CORE-003, and CORE-004](../01-requirements/requirements-ledger.md), [decision gates 3, 4, and 14](../01-requirements/decision-gates.md), [rebuild roadmap](../06-plans/rebuild-roadmap.md)
- **Downstream dependents:** `geist-project`, app project lifecycle, autosave/recovery, persistence tests
- **Supersedes:** no prior active project-persistence architecture contract
- **Superseded by:** none
- **Open decisions:** exact OS API mapping and qualified filesystem matrix at R5 intake
- **Known gaps:** filesystem implementation, crash injection, autosave journal, recovery, migration, and missing-media handling are not part of R1

## Boundary and execution model

`geist-project` will expose two blocking boundaries with these design-level signatures:

```rust
pub fn load_project(path: &Path) -> Result<ProjectEnvelope, LoadError>;
pub fn save_project_atomic(
    path: &Path,
    snapshot: &ProjectEnvelope,
) -> Result<SaveReceipt, SaveError>;
```

The final Rust signatures MAY use a named immutable snapshot type as the project model grows, but MUST preserve these semantics.

- Both calls are synchronous and app-thread-only. The caller chooses when blocking file work is safe.
- Neither call is audio-thread-safe or callback-reachable. Loading, validation, encoding, allocation, file I/O, synchronization, replacement, and cleanup MUST never run on the audio thread.
- The project crate MUST NOT spawn hidden background threads, own process-global persistence state, or conceal asynchronous work behind either call.
- The app layer MUST serialize saves to the same normalized target path. Concurrent same-target saves inside the project crate are unsupported; the crate MUST NOT add a global lock or background coordinator to make them appear safe.
- A successful load returns a complete validated value. The caller MUST keep the current live project unchanged until that result is available and accepted.
- A save consumes an immutable app-thread snapshot. It MUST NOT read mutable UI, transport, or engine state during persistence.

## Load contract

`load_project(path)` performs one ordered operation:

1. open the named file for reading;
2. read its complete bytes;
3. decode the versioned envelope and apply the readable-schema gate;
4. run reusable semantic validation over the complete decoded envelope;
5. return the envelope only if every step succeeds.

`LoadError` is actionable at the design level:

```rust
pub enum LoadError {
    Open { source: io::Error },
    Read { source: io::Error },
    SchemaTooNew { found: u32, max_readable: u32 },
    Malformed { source: CodecError },
    InvalidProject { reason: ValidationError },
}
```

Implementations MAY retain the path and richer diagnostics in each variant. They MUST preserve the distinction between inaccessible I/O, malformed encoding, unsupported newer schema, and semantically invalid project data. No failure returns a partial project or mutates the destination file.

Semantic validation is a trust boundary. Derived deserialization is not sufficient: it can construct values without validated constructors. Validation MUST cover all envelope invariants available at that schema, including nonzero and project-wide unique object IDs once persisted object collections exist.

## Save result and failure vocabulary

A successful save reports that the new bytes and replacement metadata reached the required synchronization points:

```rust
pub struct SaveReceipt {
    pub target_state: TargetState, // always ReplacedDurable on Ok
}

pub enum TargetState {
    Unchanged,
    ReplacedDurable,
    ReplacedDurabilityUncertain,
}

pub enum SaveStage {
    ValidateSnapshot,
    EncodeSnapshot,
    CreateTemporary,
    WriteTemporary,
    SyncTemporary,
    ReplaceTarget,
    SyncParentDirectory,
}
```

`TargetState` describes the named destination relative to its state when the call began:

- `Unchanged`: an existing target retains its prior bytes, or a previously absent target remains absent.
- `ReplacedDurable`: the complete encoded snapshot replaced the target, the temporary file was synchronized before replacement, and the parent-directory synchronization completed.
- `ReplacedDurabilityUncertain`: the complete new snapshot is the visible target in the running system, but parent-directory synchronization failed, so a crash may lose or roll back the directory entry.

`SaveError` preserves semantic versus I/O failures and the exact failure stage:

```rust
pub enum SaveError {
    InvalidProject { reason: ValidationError },
    Encode { source: CodecError },
    Io {
        stage: SaveStage,
        target_state: TargetState,
        source: io::Error,
    },
}
```

Validation maps to `SaveStage::ValidateSnapshot` conceptually and encoding maps to `SaveStage::EncodeSnapshot`; their dedicated variants avoid pretending they are filesystem errors. Implementations MAY add non-lossy context, but MUST NOT collapse `ReplacedDurabilityUncertain` into an ordinary failed save or report it as `Unchanged`.

## Atomic-save algorithm

`save_project_atomic(path, snapshot)` MUST execute in this order:

1. Run the same reusable semantic validation required after load.
2. Encode the complete snapshot into memory.
3. Only after validation and encoding succeed, select a unique temporary filename in the target's existing parent directory and create it with exclusive `create_new` semantics.
4. Write the entire encoded buffer with `write_all`; a short write is not success.
5. Call `sync_all` on the temporary file and close or otherwise release handles as required by the replacement API.
6. Atomically replace the target with the synchronized sibling temporary file. Never delete, truncate, or move aside the destination first.
7. Synchronize the parent directory to make the replacement metadata crash-durable on the qualified platform/filesystem.
8. Return `SaveReceipt { target_state: ReplacedDurable }` only after every required synchronization succeeds.

Validation and encoding happen before any destination or sibling temporary path is touched. A serialization failure therefore cannot alter the old target or leave a temporary file.

The temporary name MUST be collision-resistant within the parent, MUST NOT reuse a predictable fixed name, and MUST rely on exclusive creation rather than a check-then-create sequence. Because the temporary file is a sibling, replacement remains on one filesystem. Existing destination permissions and metadata are not implicitly preserved unless a later accepted contract explicitly requires and tests them.

## Failure guarantees and cleanup

The replacement operation is the commit point.

| Failure point | Required target state | Temporary-file rule |
|---|---|---|
| validation or encoding | `Unchanged` | no temporary file was created |
| temporary create, write, or `sync_all` | `Unchanged` | best-effort cleanup |
| atomic replacement | `Unchanged` | best-effort cleanup |
| parent-directory synchronization after replacement | `ReplacedDurabilityUncertain` | no temporary path remains after successful replacement |

Before the commit point, the original destination bytes or prior absence MUST remain observable. Cleanup failure MUST NOT replace or hide the primary `SaveError`; it MAY be attached as secondary diagnostic context. Best-effort cleanup runs synchronously and MUST NOT be deferred to a hidden worker.

After successful replacement, readers observe either the complete old file or the complete new file, never an encoded prefix. A parent-directory synchronization failure does not restore the old file and MUST NOT trigger a second replacement attempt. The caller receives `ReplacedDurabilityUncertain`, keeps the in-memory project dirty, and decides whether to retry a fresh save or warn the user.

## Platform qualification

macOS and Linux are co-first-class targets. R4 implementation MUST use a same-directory atomic replacement primitive whose documented behavior replaces an existing path without a delete-first window, then attempt parent-directory synchronization. It MUST fail explicitly rather than silently weaken the contract when the platform or filesystem cannot provide a required step.

Normal-process atomicity is not proof of crash durability. R5 MUST qualify the exact macOS and Linux OS APIs and supported filesystem matrix with fault and crash/recovery drills. Filesystems or mounts whose rename or directory-synchronization semantics cannot satisfy this contract remain unsupported for the durable guarantee and require an explicit product decision; they MUST NOT be reported as `ReplacedDurable`.

## Test seam

Production persistence uses the concrete qualified filesystem path. A private, test-only filesystem adapter MAY model open/create, `write_all`, file `sync_all`, replacement, parent-directory synchronization, and cleanup. It exists only under test configuration or behind a private generic seam; it MUST NOT become application-global state or a public pluggable filesystem API.

Fault injection MUST be deterministic at every `SaveStage` and verify:

- validation/encoding failures touch no filesystem path;
- exclusive temporary creation retries name collisions without truncating another file;
- write and temporary-sync failures preserve the exact old target or absence;
- replacement never exposes a partial destination and never deletes first;
- parent-sync failure reports `ReplacedDurabilityUncertain` with the new file visible;
- cleanup is attempted, and cleanup failure never masks the primary error;
- simultaneous same-target operations are prevented by the app-level owner rather than serialized by hidden project-crate state.

The adapter proves control flow. R5 tests against real qualified filesystems prove OS and crash behavior.

## Milestone ownership

- **R1:** accept this API, state machine, and failure contract. No filesystem API is implemented in this slice.
- **R4:** implement app-thread `load_project` and `save_project_atomic`, reusable semantic validation, normal save/reload integration, and real-filesystem tests for successful replacement and ordinary injected failures.
- **R5:** add crash-injection qualification, parent-directory durability evidence, journaled autosave, recovery selection, migrations, salvage policy, and missing-media diagnostics. R5 owns the supported filesystem matrix and recovery UX.

R4 save/reload MUST use the atomic boundary; R5 adds proof and recovery behavior rather than replacing it with a second save path.

## Current codec qualification

The current `geist_project::from_bytes` path performs semantic validation after decoding. The current `geist_project::to_bytes` path only serializes and **does not revalidate an in-memory envelope**. An in-memory value can therefore become semantically invalid after construction and still encode successfully today. Future save implementation MUST extract or expose one reusable semantic validator and call it before encoding and before touching the destination, while load calls the same validator after decode.

## Acceptance checklist

- Public behavior distinguishes load I/O, malformed data, newer schema, and semantic invalidity.
- Save validation and encoding precede all filesystem touches.
- A unique sibling temporary file uses exclusive creation, `write_all`, and file `sync_all`.
- Replacement is atomic and never delete-first.
- Parent-directory synchronization gates `ReplacedDurable` on qualified macOS/Linux filesystems.
- Every failure stage has the exact target-state guarantee above.
- Temporary cleanup is best effort and does not mask the primary error.
- No persistence work reaches the audio thread or a hidden background thread.
- Tests inject every stage through a private test-only seam; R5 separately proves crash behavior.
- Project-wide duplicate-ID validation lands with persisted object collections rather than being claimed by the current minimal envelope.
