<!--
Author: Jeff
Date: 2026-07-11
Description: Spectre requirements ledger — stable normative requirements with provenance and acceptance evidence
Notes: Seeded with foundation requirements for R0-R1; grows only with provenance
-->

# Requirements Ledger

- **Status:** accepted
- **Last verified:** 2026-08-24
- **Scope:** accepted-for-work normative requirements; each row carries provenance and required evidence
- **Decision authority:** Jeff
- **Upstream sources:** `docs/00-product/vision.md`; `docs/01-requirements/decision-gates.md`; `docs/02-reference-research/*observations*.md`
- **Downstream dependents:** architecture contracts, rebuild roadmap, implementation slices, traceability
- **Supersedes:** removed prototype requirement material
- **Superseded by:** none
- **Open decisions:** see decision gates
- **Known gaps:** the R0/R1 foundation families plus the R4 ENGINE bounds are seeded; SEQ/REC/VST/UI families await their milestone intakes

Format: `ID | requirement (MUST/SHOULD/MAY) | provenance | acceptance evidence | status`.
Statuses: `proposed`, `accepted`, `implemented`, `verified`.

## RT — realtime safety

| ID | Requirement | Provenance | Acceptance evidence | Status |
|---|---|---|---|---|
| RT-001 | Audio-callback-reachable code MUST NOT allocate, deallocate, take blocking locks, perform I/O, log, or panic across the callback boundary. | accepted realtime product contract | allocation/lock guard wrapping every reachable callback path in CI | accepted |
| RT-002 | All control↔render communication MUST use bounded wait-free structures with defined overflow policy and off-thread reclamation of retired state. | accepted realtime product contract | concurrency tests incl. loom/model tests on the chosen structure. **Extended 2026-08-24 (R4-2):** the parameter lane is now consumed end to end, so the evidence covers application and not only transport — `a_shape_edit_changes_live_audio_and_nothing_stays_pending` asserts a UI edit changes rendered output with `parameters_pending == 0`, and `every_fixture_parameter_routes_and_a_sweep_coalesces` asserts 500 writes to one target coalesce to a single application | accepted |
| RT-003 | The engine MUST flush denormals (FTZ/DAZ or equivalent) and contain NaN/Inf by isolating the offending node and surfacing a diagnostic, outputting silence rather than noise. | OBS-VCV-VOLT-006 (0-on-NaN precedent); mandate §12.1 | injection tests: NaN/Inf/denormal fixtures per node type | accepted 2026-08-09; implemented in `CompiledPlan::process` with a software FTZ-equivalent flush |

## TIME — time, tempo, transport

| ID | Requirement | Provenance | Acceptance evidence | Status |
|---|---|---|---|---|
| TIME-001 | Spectre MUST use explicit distinct time types (sample time, musical beats, wall seconds) with single-definition conversions; raw numeric time in APIs is prohibited. | decision gate 5 | type-level API review + conversion property tests | implemented |
| TIME-002 | Musical ranges MUST be half-open `[start, end)` and events at identical sample offsets MUST have a total deterministic ordering (transport > note-off > note-on > CC > param). | mandate §12.2 | property tests over boundary/loop/seek cases | implemented |
| TIME-003 | The tempo map MUST convert deterministic piecewise tempo between absolute `BeatTicks` positions at 960 PPQ and integer samples over signed pre-roll and projects of at least 24 hours. Segment durations MUST accumulate with unrounded absolute anchors and round once at integer-sample conversion. `BeatTicks → samples → BeatTicks` MUST preserve the tick; arbitrary `samples → BeatTicks → samples` MUST select the nearest tick within one-half local samples-per-tick plus one-half sample. Absolute-position deltas MUST telescope across tempo boundaries; independently rounded durations are not required to be associative. | mandate §8.2; decision gate 5; OBS-AB12-WARP-002, OBS-AB12-AUTO-010 | 24-hour exact/fractional fixtures, boundary and pre-roll tests, round-once discriminator, monotonicity, nearest-tick bounds | verified |
| TIME-004 | Time-signature changes MUST carry numerator 1–99 and denominator in {1,2,4,8,16} as a starting envelope, with Spectre-rationale review before widening. | OBS-AB12-ARR-001, OBS-AB12-SES-002 (both products converge); gate 16 (no blind copying — envelope adopted with rationale: covers observed practice) | serde + validation tests | implemented |
| TIME-005 | Transport MUST be a deterministic state machine (stopped/playing/recording × loop) whose transitions are testable without an audio device. | accepted R1 contract | state-machine unit + property tests | implemented |

## CORE — identity and project envelope

| ID | Requirement | Provenance | Acceptance evidence | Status |
|---|---|---|---|---|
| CORE-001 | Every user-visible object (track, clip, device, parameter, marker…) MUST have a stable 64-bit ID unique within its project, preserved across save/load, undo, reorder, and migration. | mandate §8.1; gate 4 | ID-stability tests across round-trip and mutation sequences. R1 disposition (2026-07-17): save/load and undo identity are evidenced; reorder evidence is gated on the first persisted object collection (R4 intake) and migration evidence on the first real schema migration (R5). Not verifiable before then. | implemented |
| CORE-002 | Parameters MUST carry stable identity, typed range, default, display mapping, and unit; normalized value semantics are defined once in spectre-core. | mandate §8.2 device model; OBS-BW53-AUTO-* (override model needs identity) | parameter descriptor unit tests + API review | implemented |
| CORE-003 | The project envelope MUST carry an explicit schema version from the first byte written; unknown newer fields MUST be preserved on rewrite where feasible. | accepted project-safety contract | round-trip fixtures incl. newer-schema preservation test | verified |
| CORE-004 | Saves MUST be atomic (write-new + rename) with no partially written project ever observable. | mandate §12.6; vision project-safety pillar | crash-injection save tests at R5; API design review at R1 — completed 2026-07-17 via the accepted [project-persistence contract](../03-architecture/project-persistence.md) (boundaries, save algorithm, failure vocabulary, target-state guarantees, test seam) | accepted |

## ENGINE — live engine bounds (R4 intake)

Every row here exists because PROD-003 requires each numeric limit to carry its own rationale in
this ledger, and decision 16 prohibits copying a bound from a reference product. None of these
values is derived from any other product.

| ID | Requirement | Provenance | Acceptance evidence | Status |
|---|---|---|---|---|
| ENGINE-001 | The live engine MUST request a driver block of 256 frames. Rationale: 256 is the only block size for which Spectre holds its own measured hardware evidence — the macOS qualification in `../06-plans/current-milestone.md`. Re-open when R4-3's Linux qualification produces a second measurement. | PROD-003; decision 16; the macOS qualification record | `ENGINE_BUFFER_FRAMES` in `crates/spectre-app/src/engine.rs`, exercised by every test in `crates/spectre-app/tests/live_engine.rs` and by the app drill on real hardware | implemented |
| ENGINE-002 | The compiled plan MUST reserve twice the requested block. Rationale: the seam asks cpal for a fixed buffer size but no Spectre evidence proved every host honors it, so a larger-than-requested block was unproven-absent. Any block beyond the margin still lands in the existing counted-silence refusal. | PROD-003; decision 16; `RenderBridge`'s frame-capacity refusal | `ENGINE_PLAN_FRAME_MARGIN`; `the_plan_reserves_more_frames_than_the_requested_block`. **Measured 2026-08-24:** the macOS drill and the app drill both report `frame_capacity_rejections=0`, so on this host the request is honored; the margin remains because one host is not every host | implemented |
| ENGINE-004 | A control channel MUST refuse a registered target set larger than 1 024. Rationale: `ParameterReader::drain` walks every registered slot on every block, so without a cap the per-block cost of the callback path grows without limit as a project grows, which RT-001's bounded clause does not permit. The value is deliberately generous rather than tuned, because Spectre has no measurement that would justify a tuned value and a generous bound still discharges the obligation that the number be bounded at all. What is computable is the memory: 1 024 targets reserve 69 632 B, negligible against the plan's own channel pool. R4's complete accepted device scope registers four. | PROD-003; decision 16; RT-001's bounded-work clause | `MAX_PARAMETER_TARGETS` and `ControlError::TooManyTargets` in `crates/spectre-audio/src/control.rs`, checked in `control_channel` before any slot is allocated | implemented |
| ENGINE-003 | The null backend MUST report 48 000 Hz as its synthetic device rate. Rationale: the null device has no hardware format, and 48 000 is the rate every existing `spectre-audio` test already asserts against, so the synthetic default keeps deterministic tests on the number they already use. | PROD-003; decision 16 | `NULL_SAMPLE_RATE` in `crates/spectre-audio/src/null.rs`; `null_backend_reports_its_fixed_rate_and_refuses_unknown_devices` | implemented |

The R4-1 audition voice constants (`AUDITION_VOICE_ID`, `AUDITION_CHANNEL`, `AUDITION_NOTE`,
`AUDITION_VELOCITY`) are deliberately not ledger rows: they are the values the offline fixture
already renders, not bounds on anything, and R4-5 removes the audition voice entirely when clip
playback replaces it.

## GRAPH — render graph (R2 intake, seeded now)

| ID | Requirement | Provenance | Acceptance evidence | Status |
|---|---|---|---|---|
| GRAPH-001 | The editable graph and the compiled render plan MUST be distinct types; the callback executes only immutable compiled plans. | accepted graph and realtime contract | type-level separation + integration test that the app path uses the plan | implemented |
| GRAPH-002 | Feedback edges MUST be explicit and priced with exactly one render-quantum delay; implicit cycles MUST fail graph validation with a diagnostic. | gate 7; OBS-VCV-VOLT-006, OBS-PP-ARCH-002 (one-sample-delay precedents, adapted to block scope with rationale) | validation unit tests + cycle fixtures | proposed |

## PROD — product-level

| ID | Requirement | Provenance | Acceptance evidence | Status |
|---|---|---|---|---|
| PROD-001 | Spectre MUST keep timeline and performance-launcher playback authority explicit per track, with a visible "return authority" affordance at track and global scope. | OBS-AB12-SES-005 + OBS-BW53-LAUNCH-001 (cross-product convergence) | spec + interaction tests at R10 | proposed |
| PROD-002 | Automated parameters MUST expose distinct visible states for automated / manually-overridden, with an explicit restore action. | OBS-AB12-AUTO-004 + OBS-BW53-AUTO-002 (convergent pattern) | UI-model tests at R9 | proposed |
| PROD-003 | Every numeric limit in Spectre MUST have its own rationale recorded in this ledger; copied vendor limits are prohibited. | gate 16; clean-room methodology | ledger review at each milestone exit | proposed |
