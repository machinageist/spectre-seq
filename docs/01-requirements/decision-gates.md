<!--
Author: Jeff
Date: 2026-07-11
Description: Unresolved product/architecture decisions with recommendations and safe defaults
Notes: Safe defaults accepted by Jeff through delegated decision authority on 2026-07-12
-->

# Decision Gates

- **Status:** accepted
- **Last verified:** 2026-07-12
- **Scope:** every decision the audits and mandate flagged as open, with recommendation, reversibility, and safe default
- **Decision authority:** Jeff
- **Upstream sources:** product vision, implementation evidence, and reference research
- **Downstream dependents:** requirements ledger, architecture contracts, R0+ implementation
- **Supersedes:** removed prototype decision material
- **Superseded by:** none
- **Open decisions:** only rows explicitly gated at a future milestone
- **Known gaps:** benchmark and license-audit evidence still missing for several rows

Legend: **SD** = safe default adopted so work can proceed; **GATE** = must be ratified by Jeff before the stated milestone ships.

| # | Decision | Recommendation | Reversibility | Status |
|---:|---|---|---|---|
| 1 | First-class platform order | macOS and Linux co-first-class; Windows at beta. | Medium | **Accepted** |
| 2 | Workspace posture | Repository root is the only workspace; prototype code is removed. | High | **Implemented** |
| 3 | Project encoding | Versioned JSON envelope for R1 because it is inspectable and preserves unknown fields; reconsider binary packing only with measured need. | Medium | **Accepted** |
| 4 | Stable ID strategy | Deterministic seeded 64-bit IDs with zero excluded and project-scoped uniqueness validation; creation sequence owns ordering. | Medium | **Accepted** |
| 5 | Canonical pitch/time types | `SampleTime(i64)`, fixed-point `BeatTicks(i64)` at 960 ticks/beat, and explicit pitch newtypes when pitch enters scope; conversions defined once. | Medium | **Accepted** |
| 6 | f32 vs f64 processing | f32 buffers, f64 for accumulators/coefficients where numerically warranted, documented per module | Medium | **SD adopted** |
| 7 | Graph feedback policy | Explicit one-block delay on declared feedback edges (all three studied modular systems price feedback in explicit unit delays); no implicit cycles | Medium | **SD adopted** for design work; GATE before R11 modular surface |
| 8 | UI stack | egui for the R4 shell; accessibility audit before beta; custom rendering only if measured limits justify it. | Low once beta ships | **Accepted** |
| 9 | VST3 bindings + license | Decide at R8 intake with a fresh audit of SDK tag v3.8.0_build_66 (pinned commit + submodule licenses) vs third-party Rust bindings; no decision now | Medium | GATE at R8 intake |
| 10 | Plugin crash isolation | Out-of-process scanning from day one; in-process hosting with containment at R8; full process isolation evaluated post-1.0 | Medium | **SD adopted** |
| 11 | Sample decode/streaming deps | symphonia for decode (pure Rust) evaluated first; OS codecs as fallback; decision recorded at R7 | High | **SD adopted** for planning |
| 12 | Time-stretch strategy | Own phase-vocoder/transient path long-term (DSPREF-JOS-SASP); no third-party stretch library in the identity path; interim: no stretch until R7 | Medium | **SD adopted** |
| 13 | Undo architecture | Command-pattern transactions over the project model with grouped edits; no state-snapshot diffing | Low after R5 | **SD adopted**, GATE at R5 exit |
| 14 | Autosave/recovery model | Journaled autosave to sidecar + atomic rename saves; recovery drill required at R5 exit | Medium | **SD adopted** |
| 15 | Small synth vs flagship relationship | R4 ships a deliberately small original synth; R11 designs the flagship and modular identity from accepted requirements without prototype-code reuse. | High now | **Accepted** |
| 16 | Reference-product numeric limits | No copied limits (e.g., wavetable frame sizes, unison caps); every numeric bound in Geist needs its own rationale row in the requirements ledger | n/a | **SD adopted** (standing rule) |
| 17 | Accessibility baseline | Keyboard-complete operation and screen-reader labels on all commands/params by beta; scoped audit at R4 | Low if deferred | GATE before beta |
| 18 | Config/scripting boundary | Declarative, versioned, validated config; no embedded scripting language pre-1.0 | High | **SD adopted** |

Rows marked **SD adopted** proceed now and are re-opened only by evidence. GATE rows block their named milestone, not current work.
