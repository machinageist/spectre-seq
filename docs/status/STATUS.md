<!--
Author: Jeff
Date: 2026-07-11
Description: Current verified state of the Geist DAW rebuild by phase and subsystem
Notes: Claims here link to evidence; optimistic language is prohibited
-->

# Status

- **Status:** draft
- **Last verified:** 2026-07-11
- **Scope:** current verified rebuild-phase and subsystem state
- **Decision authority:** Jeff
- **Upstream sources:** `VALIDATION.md`, `subsystems.toml`, `../audits/*.md`, `../02-reference-research/source-ledger.json`, `../02-reference-research/workflow-field-study/workflow-observations.jsonl`
- **Downstream dependents:** `NEXT.md`, `../06-plans/current-milestone.md`
- **Supersedes:** narrative status sections of `HANDOFF.md` for rebuild-lane state
- **Superseded by:** none
- **Open decisions:** see `NEXT.md` blockers
- **Known gaps:** no requirements ledger, traceability ledger, decision-gates document, architecture contracts, quality gates, or rebuild roadmap exists yet

## Rebuild-phase state

| Phase | State | Evidence |
|---|---|---|
| A — forensic audit | drafted | six audits in `docs/audits/`, `subsystems.toml`, `VALIDATION.md` |
| B — documentation architecture | authority map accepted | `docs/README.md`; target hierarchy defined; most target documents intentionally absent |
| C — clean-room references | inventory-only for all ten references; workflow field study started | `source-ledger.json` (23 sources); 4 FL Studio + 2 Ableton Live workflow observations admitted |
| D — product North Star | not started | none |
| E — architecture contracts | not started | none |
| F — verification strategy | not started | none |
| G — rebuild roadmap | not started | none |
| H — R0/R1 implementation | not started | none |

The specification gate is not satisfied. The ground-up rebuild has not begun.

## Baseline gates (2026-07-11, macOS 27.0 arm64)

Full commands and durations in `VALIDATION.md`.

| Gate | Result |
|---|---|
| `cargo metadata --locked` | PASS |
| `cargo fmt --all -- --check` | FAIL (formatting drift, unmodified) |
| `cargo check --locked --workspace --all-targets --all-features` | PASS with warnings |
| `cargo clippy … -D warnings` | FAIL (dead code, one range-loop lint) |
| `cargo test --locked --workspace --all-features` | PASS with warning |

## Legacy-runtime subsystem maturity

`subsystems.toml` is the machine-readable ledger. Summary: the live callback path runs a fixed-track engine; the compiled graph, automation, stacksynth, and modular-rack crates are unit-tested in isolation and not application dependencies; external MIDI I/O is absent; recording is integrated but not exercised end-to-end. No subsystem is manually QA'd, stress-tested, or release-qualified.

## Field-workflow research state

- Sources discovered: 25 ledger records across 10 products; 6 promoted to `reviewed-workflow-source` (4 FL Studio, 2 Ableton Live).
- Two Bitwig artist interviews fully inspected 2026-07-11 with documented extraction shortfall: preference/friction statements only, no admissible action sequence; Bitwig needs visible-session video evidence. The `/latest/` guide's own version identity resolved as 5.3.
- Workflow observations admitted: 6 (`WF-FL-ARRANGE-001` … `WF-ABLETON-TSURUTA-006`), all parsing and cross-referencing cleanly (verified 2026-07-11).
- Ableton Live evidence is interview self-report only (`low` confidence); visible-session corroboration is still required.
- Shortcut-action map: 16 rows, all IDs resolve.
- Remaining candidate queues: 8 Ableton Live, 10 Bitwig Studio, 8 FL Studio, 12 REAPER.
- Bitwig Studio has zero reviewed sources; next extraction SHOULD target Bitwig Studio.
- No frequency, convergence, or priority claim is authorized by the current corpus.

## Parallel lanes

- **Modular-rack lane (pre-existing, separate from rebuild docs):** M3 code-complete, manual GUI verification pending, M4 engine integration next. Five pre-existing tracked modifications preserved uncommitted (`AGENTS/changes/modular-rack/PLAN.md`, `HANDOFF.md`, `app/geist-daw/src/control.rs`, `app/geist-daw/src/engine.rs`, `crates/geist-ui/src/widgets/knob.rs`).
- **Stacksynth lane:** S0–S2b landed in Git history; S2c next. Not an application dependency.
