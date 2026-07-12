<!--
Author: Jeff
Date: 2026-07-11
Description: Reproducible forensic baseline for the pre-rebuild Geist DAW repository
Notes: Records observed evidence without treating passing compilation or tests as product readiness
-->

# Repository Baseline

- **Status:** draft
- **Last verified:** 2026-07-11
- **Scope:** repository state, workspace topology, available tooling, and mutation-free validation at rebuild intake
- **Decision authority:** Jeff
- **Upstream sources:** Git repository at `1e5314d1b9bb0d27698f468b442158ba9c4952be`; Cargo metadata; commands listed below
- **Downstream dependents:** `architecture-drift.md`, `runtime-reachability.md`, `test-and-quality-gaps.md`, `reuse-disposition.md`, `../status/subsystems.toml`
- **Supersedes:** no prior forensic baseline
- **Superseded by:** none
- **Open decisions:** rebuild branch/worktree isolation; Rust distribution and pinned channel; active-workspace migration posture
- **Known gaps:** dependency license/advisory audit, benchmark execution, application launch, audio/MIDI hardware QA, release build, and non-macOS qualification remain unperformed

## Evidence boundary

This document records what was observed on one machine. It does not establish release readiness, professional usability, realtime safety across the callback closure, VST3 compatibility, project recoverability, or cross-platform support.

## Repository identity and preservation

| Item | Observed value |
|---|---|
| Root | `/Users/machinageist/geist-daw` |
| Branch | `claude/ableton-studio-overhaul` |
| HEAD | `1e5314d1b9bb0d27698f468b442158ba9c4952be` |
| Remote | `origin` = `git@github.com:machinageist/geist-daw.git` |
| Worktrees | one, at the repository root |
| Submodules | none |
| Tags | none |
| Tracked files | 299 |
| Tracked Rust files | 219 |
| Tracked Markdown files | 48 |

Five unstaged tracked modifications predated this audit and are treated as user work:

- `AGENTS/changes/modular-rack/PLAN.md`
- `HANDOFF.md`
- `app/geist-daw/src/control.rs`
- `app/geist-daw/src/engine.rs`
- `crates/geist-ui/src/widgets/knob.rs`

At intake the diff contained 306 insertions and 18 deletions. There were no staged changes and no non-ignored untracked files. Baseline commands did not alter that set or `Cargo.lock`. No branch, tag, commit, reset, cleanup, or formatter write was performed.

`CLAUDE(1).md` and `HANDOFF(1).md` are absent from the current tree. `git log --all -- <path>` returned no history for either exact path.

## Host and toolchain

| Item | Observed value |
|---|---|
| Host | macOS 27.0, build 26A5378j |
| Architecture | arm64 / `aarch64-apple-darwin` |
| Cargo | 1.96.1, Homebrew |
| rustc | 1.96.1 (`31fca3adb`, 2026-06-26), Homebrew |
| Repository pin | `rust-toolchain.toml`: `channel = "nightly"` |
| rustup | unavailable |

The repository requests an unversioned nightly channel, but the executing `cargo` and `rustc` are Homebrew binaries and `rustup` is unavailable. The successful baseline therefore does not prove that the declared toolchain pin is honored or reproducible.

## Active Cargo workspace

`cargo metadata --locked --format-version 1` reported 16 active packages:

- application/build: `geist-daw`, `xtask`;
- core/runtime: `geist-core`, `geist-graph`, `geist-audio-backend`, `geist-timeline`, `geist-automation`, `geist-project`, `geist-config`;
- UI: `geist-ui`;
- sound/device work: `geist-dsp`, `geist-fx`, `geist-synth`, `geist-stacksynth`, `geist-modular`;
- plugin boundary: `geist-vst-host`.

The root manifest uses `crates/*` and explicitly excludes `crates/geist-clap-host` and `crates/geist-lv2-host`. Their source remains in the repository but is not built by workspace-wide Cargo gates. The active application directly depends on eleven local crates, making it a broad integration owner rather than a narrow shell.

Declared non-library targets include:

- binary: `app/geist-daw`;
- binary: `xtask`;
- examples: `geist-ui/examples/studio.rs`, `geist-ui/examples/widget_gallery.rs`;
- integration tests: `geist-config/tests/bundled_profiles.rs`, `geist-synth/tests/params_descriptors.rs`, `geist-ui/tests/workflow_render.rs`;
- benchmarks: three `geist-dsp` Criterion targets and one `geist-graph` Criterion target.

No Cargo features were declared by active local packages in the metadata output. The apparent `--all-features` baseline therefore did not exercise alternate local feature configurations.

## Build, test, and quality baseline

| Command | Result | Duration | Evidence |
|---|---:|---:|---|
| `cargo metadata --locked --format-version 1` | pass | 3.10s | 16 active local packages; dependencies downloaded to external Cargo cache |
| `cargo fmt --all -- --check` | fail | 0.30s | widespread formatting drift; check-only |
| `cargo check --locked --workspace --all-targets --all-features` | pass with warnings | 31.38s | dead-code warnings in `geist-daw` |
| `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings` | fail | 4.69s | dead code plus `needless_range_loop` |
| `cargo test --locked --workspace --all-features` | pass with warning | 62.61s | unit, integration, and doctest targets completed; one dead-code warning |

No `#[ignore]` declarations were found in tracked Rust sources. Passing tests are evidence for the named test fixtures only. They are not evidence of application launch, hardware I/O, real plugin operation, realtime closure safety, stress tolerance, crash recovery, or platform qualification.

Strict Clippy failed on the pre-existing dirty application changes:

- `app/geist-daw/src/control.rs`: unused `FilterMode` variants, `FilterRoute::Parallel`, and filter command variants;
- `app/geist-daw/src/engine.rs:1508`: `clippy::needless_range_loop`.

Formatting failed across both dirty and otherwise untouched files, so the repository did not enter this audit with a passing formatting gate.

## Unsafe and FFI inventory

Tracked unsafe declarations occur in:

- `app/geist-daw/src/alloc_guard.rs`: test allocator instrumentation;
- `crates/geist-vst-host`: active VST3 dynamic-loading and COM/FFI boundary;
- `crates/geist-clap-host`: extensive excluded legacy CLAP boundary.

The LV2 source tree is mostly pseudocode/scaffolding by direct inspection of its comments and is excluded from the active workspace. A full safety review must distinguish active code, excluded legacy code, tests, and dependency-provided unsafe code.

## CI, release, and stewardship baseline

Initial tracked-file inventory found no root `LICENSE`, `COPYING`, `CONTRIBUTING`, `SECURITY`, `.github` workflow, `.cargo` policy, `deny.toml`, `Cross.toml`, `Makefile`, or `Justfile`.

`xtask/src/main.rs`, `xtask/src/package_release.rs`, and `xtask/src/run_benchmarks.rs` explicitly identify themselves as pseudocode scaffolds. The executable `xtask` main currently performs no work. Packaging, release automation, benchmark orchestration, dependency policy, security reporting, and contributor policy are therefore not established by the current tree.

## Baseline conclusions

1. The workspace compiles and its current automated tests pass on one arm64 macOS environment.
2. The workspace is not quality-gate clean: formatting and strict Clippy fail.
3. The declared Rust toolchain is not demonstrably the toolchain used.
4. Excluded CLAP/LV2 trees remain as architectural residue but are not covered by workspace checks.
5. Passing test volume must not be translated into integrated, platform-qualified, release-qualified, or professional-ready status.
6. No existing user work was modified while establishing this baseline.
