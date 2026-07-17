<!--
Author: Jeff
Date: 2026-07-11
Description: Geist requirements ledger — stable normative requirements with provenance and acceptance evidence
Notes: Seeded with foundation requirements for R0-R1; grows only with provenance
-->

# Requirements Ledger

- **Status:** accepted
- **Last verified:** 2026-07-16
- **Scope:** accepted-for-work normative requirements; each row carries provenance and required evidence
- **Decision authority:** Jeff
- **Upstream sources:** `docs/00-product/vision.md`; `docs/01-requirements/decision-gates.md`; `docs/02-reference-research/*observations*.md`
- **Downstream dependents:** architecture contracts, rebuild roadmap, implementation slices, traceability
- **Supersedes:** removed prototype requirement material
- **Superseded by:** none
- **Open decisions:** see decision gates
- **Known gaps:** only the R0/R1 foundation families are seeded; SEQ/REC/VST/UI families await their milestone intakes

Format: `ID | requirement (MUST/SHOULD/MAY) | provenance | acceptance evidence | status`.
Statuses: `proposed`, `accepted`, `implemented`, `verified`.

## RT — realtime safety

| ID | Requirement | Provenance | Acceptance evidence | Status |
|---|---|---|---|---|
| RT-001 | Audio-callback-reachable code MUST NOT allocate, deallocate, take blocking locks, perform I/O, log, or panic across the callback boundary. | accepted realtime product contract | allocation/lock guard wrapping every reachable callback path in CI | accepted |
| RT-002 | All control↔render communication MUST use bounded wait-free structures with defined overflow policy and off-thread reclamation of retired state. | accepted realtime product contract | concurrency tests incl. loom/model tests on the chosen structure | accepted |
| RT-003 | The engine MUST flush denormals (FTZ/DAZ or equivalent) and contain NaN/Inf by isolating the offending node and surfacing a diagnostic, outputting silence rather than noise. | OBS-VCV-VOLT-006 (0-on-NaN precedent); mandate §12.1 | injection tests: NaN/Inf/denormal fixtures per node type | proposed |

## TIME — time, tempo, transport

| ID | Requirement | Provenance | Acceptance evidence | Status |
|---|---|---|---|---|
| TIME-001 | Geist MUST use explicit distinct time types (sample time, musical beats, wall seconds) with single-definition conversions; raw numeric time in APIs is prohibited. | decision gate 5 | type-level API review + conversion property tests | implemented |
| TIME-002 | Musical ranges MUST be half-open `[start, end)` and events at identical sample offsets MUST have a total deterministic ordering (transport > note-off > note-on > CC > param). | mandate §12.2 | property tests over boundary/loop/seek cases | implemented |
| TIME-003 | The tempo map MUST convert deterministic piecewise tempo between absolute `BeatTicks` positions at 960 PPQ and integer samples over signed pre-roll and projects of at least 24 hours. Segment durations MUST accumulate with unrounded absolute anchors and round once at integer-sample conversion. `BeatTicks → samples → BeatTicks` MUST preserve the tick; arbitrary `samples → BeatTicks → samples` MUST select the nearest tick within one-half local samples-per-tick plus one-half sample. Absolute-position deltas MUST telescope across tempo boundaries; independently rounded durations are not required to be associative. | mandate §8.2; decision gate 5; OBS-AB12-WARP-002, OBS-AB12-AUTO-010 | 24-hour exact/fractional fixtures, boundary and pre-roll tests, round-once discriminator, monotonicity, nearest-tick bounds | verified |
| TIME-004 | Time-signature changes MUST carry numerator 1–99 and denominator in {1,2,4,8,16} as a starting envelope, with Geist-rationale review before widening. | OBS-AB12-ARR-001, OBS-AB12-SES-002 (both products converge); gate 16 (no blind copying — envelope adopted with rationale: covers observed practice) | serde + validation tests | implemented |
| TIME-005 | Transport MUST be a deterministic state machine (stopped/playing/recording × loop) whose transitions are testable without an audio device. | accepted R1 contract | state-machine unit + property tests | implemented |

## CORE — identity and project envelope

| ID | Requirement | Provenance | Acceptance evidence | Status |
|---|---|---|---|---|
| CORE-001 | Every user-visible object (track, clip, device, parameter, marker…) MUST have a stable 64-bit ID unique within its project, preserved across save/load, undo, reorder, and migration. | mandate §8.1; gate 4 | ID-stability tests across round-trip and mutation sequences | implemented |
| CORE-002 | Parameters MUST carry stable identity, typed range, default, display mapping, and unit; normalized value semantics are defined once in geist-core. | mandate §8.2 device model; OBS-BW53-AUTO-* (override model needs identity) | parameter descriptor unit tests + API review | implemented |
| CORE-003 | The project envelope MUST carry an explicit schema version from the first byte written; unknown newer fields MUST be preserved on rewrite where feasible. | accepted project-safety contract | round-trip fixtures incl. newer-schema preservation test | verified |
| CORE-004 | Saves MUST be atomic (write-new + rename) with no partially written project ever observable. | mandate §12.6; vision project-safety pillar | crash-injection save tests at R5; API design review at R1 | proposed |

## GRAPH — render graph (R2 intake, seeded now)

| ID | Requirement | Provenance | Acceptance evidence | Status |
|---|---|---|---|---|
| GRAPH-001 | The editable graph and the compiled render plan MUST be distinct types; the callback executes only immutable compiled plans. | accepted graph and realtime contract | type-level separation + integration test that the app path uses the plan | accepted |
| GRAPH-002 | Feedback edges MUST be explicit and priced with exactly one render-quantum delay; implicit cycles MUST fail graph validation with a diagnostic. | gate 7; OBS-VCV-VOLT-006, OBS-PP-ARCH-002 (one-sample-delay precedents, adapted to block scope with rationale) | validation unit tests + cycle fixtures | proposed |

## PROD — product-level

| ID | Requirement | Provenance | Acceptance evidence | Status |
|---|---|---|---|---|
| PROD-001 | Geist MUST keep timeline and performance-launcher playback authority explicit per track, with a visible "return authority" affordance at track and global scope. | OBS-AB12-SES-005 + OBS-BW53-LAUNCH-001 (cross-product convergence) | spec + interaction tests at R10 | proposed |
| PROD-002 | Automated parameters MUST expose distinct visible states for automated / manually-overridden, with an explicit restore action. | OBS-AB12-AUTO-004 + OBS-BW53-AUTO-002 (convergent pattern) | UI-model tests at R9 | proposed |
| PROD-003 | Every numeric limit in Geist MUST have its own rationale recorded in this ledger; copied vendor limits are prohibited. | gate 16; clean-room methodology | ledger review at each milestone exit | proposed |
