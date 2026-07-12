<!--
Author: Jeff
Date: 2026-07-11
Description: Latest reproducible validation evidence for the Geist DAW rebuild audit
Notes: Replace results only after rerunning the exact commands in a recorded environment
-->

# Validation

- **Status:** draft
- **Last verified:** 2026-07-11
- **Scope:** latest forensic baseline commands and results
- **Decision authority:** Jeff
- **Upstream sources:** `../audits/repository-baseline.md`
- **Downstream dependents:** `STATUS.md`, `NEXT.md`, `subsystems.toml`, release gates
- **Supersedes:** ad hoc optimistic workspace-green claims in legacy planning and handoff documents
- **Superseded by:** none
- **Open decisions:** canonical toolchain provisioning and CI matrix
- **Known gaps:** no launch, hardware, plugin-fixture, stress, benchmark, release-build, packaging, recovery-drill, or cross-platform validation

## Environment

- Date: 2026-07-11
- Host: macOS 27.0 (26A5378j), arm64
- `cargo 1.96.1 (356927216 2026-06-26)` via Homebrew
- `rustc 1.96.1 (31fca3adb 2026-06-26)` targeting `aarch64-apple-darwin`
- `rustup`: unavailable
- Repository: `/Users/machinageist/geist-daw`
- Branch: `claude/ableton-studio-overhaul`
- HEAD: `1e5314d1b9bb0d27698f468b442158ba9c4952be`
- Pre-existing worktree state: five unstaged tracked modifications; no staged or non-ignored untracked files

## Results

| Gate | Exact command | Result | Duration | Qualification |
|---|---|---:|---:|---|
| Metadata | `cargo metadata --locked --format-version 1` | PASS | 3.10s | dependency/workspace resolution only |
| Formatting | `cargo fmt --all -- --check` | FAIL | 0.30s | widespread formatting drift; no files changed |
| Compilation | `cargo check --locked --workspace --all-targets --all-features` | PASS WITH WARNINGS | 31.38s | compile evidence on this host only |
| Static analysis | `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` | FAIL | 4.69s | dead code and one range-loop diagnostic |
| Tests | `cargo test --locked --workspace --all-features` | PASS WITH WARNING | 62.61s | current automated fixtures only |

## Diagnostics retained

Strict Clippy failed on:

- unused filter-mode and routing variants in `app/geist-daw/src/control.rs`;
- unused filter-related command variants in `app/geist-daw/src/control.rs`;
- `clippy::needless_range_loop` at `app/geist-daw/src/engine.rs:1508`.

The test command emitted a dead-code warning for `FilterMode::Notch`. No tracked Rust source contains `#[ignore]`.

## Preservation verification

After the commands:

- the same five pre-existing paths remained modified;
- the diff remained 306 insertions and 18 deletions;
- `Cargo.lock` retained its original timestamp and size;
- no staged or non-ignored untracked files appeared from the baseline;
- no repository file was formatted or otherwise rewritten.

The new audit/status documents created after that comparison are attributable to the rebuild audit and must be distinguished from the five pre-existing modifications.

## Claims this evidence does not support

These results MUST NOT be described as:

- workspace green;
- realtime-safe;
- integrated end to end;
- manually QA'd;
- platform-qualified;
- release-qualified;
- production-ready;
- professional-ready.
