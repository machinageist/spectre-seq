<!--
Author: Jeff
Date: 2026-07-11
Description: Spectre requirements ledger — stable normative requirements with provenance and acceptance evidence
Notes: Seeded with foundation requirements for R0-R1; grows only with provenance
-->

# Requirements Ledger

- **Status:** accepted
- **Last verified:** 2026-07-17
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

## DEV — native device numeric bounds (R4-6 intake)

Every row below is a bound introduced by `Filament` or `Gloam` (`crates/spectre-dsp/src/filament.rs`, `crates/spectre-dsp/src/gloam.rs`). None is taken, copied, or inferred from any reference product; where a value is chosen rather than derived, the row says so and carries a re-open trigger. This family discharges PROD-003 and decision 16 for the R4-6 devices.

| ID | Requirement | Provenance | Acceptance evidence | Status |
|---|---|---|---|---|
| DEV-001 | `filament.lean` MUST range over `0.0 … 1.0`. | The control **is** a normalized phase position — the breakpoint of the device's phase map. Spectre's existing oscillators already normalize phase to `[0, 1)` via `.fract()`, so the range is the domain itself rather than a chosen envelope. Both endpoints are reachable and both are proven safe by the strict-`<`/`else` argument in the module comment. | `filament_stays_finite_at_both_lean_endpoints` (`crates/spectre-dsp/tests/devices.rs`) | proposed |
| DEV-002 | `filament.lean` MUST default to `0.5`. | Derived, not chosen: `0.5` is the unique value at which the phase map is the identity and the oscillator is an exact sine. Both multiplications and divisions are by `0.5`, so the identity is bit-exact rather than approximate. | `filament_is_an_exact_sine_at_the_default_lean` asserts bit-equality against an independent sine | proposed |
| DEV-003 | `filament.rise_ms` and `filament.fall_ms` MUST range over `1.0 … 1000.0` ms. | Minimum `1.0` ms: the shortest contour a user can set must still be a ramp at every rate the engine can open. `spectre_audio::MIN_SAMPLE_RATE` is `8_000`, and `0.001 s × 8 000 Hz = 8` sample periods, so the contour is at least an eight-step ramp everywhere and the device cannot produce a full-scale amplitude step of its own. Maximum `1000.0` ms: two beats at 120 BPM, the tempo Spectre's own default project uses (`crates/spectre-offline/src/lib.rs`, `TempoMap::constant(120.0)`); a contour longer than two beats at the project default would outlast the gesture it shapes. **Re-open trigger:** the first project whose tempo makes 1000 ms shorter than one beat — any tempo below 120 BPM for a two-beat contour — or the first request for a pad-length release. | `filament_contour_minimum_is_at_least_eight_sample_periods`; `filament_release_reaches_exact_silence` | proposed |
| DEV-004 | `filament.rise_ms` and `filament.fall_ms` MUST default to `31.6` ms. | Derived by a stated rule: the geometric midpoint of the parameter's own declared range, rounded to one decimal. `sqrt(1.0 × 1000.0) = 31.6227766…` → `31.6`. The rule is used for every strictly-positive range in both devices so that no default in either device is a taste number. | descriptor default asserted through `every_native_parameter_meets_the_declared_normalized_round_trip_policy` | proposed |
| DEV-005 | `filament.level` MUST range over `0.0 … 1.0` with default `0.2`. | Range is the normalized amplitude domain. The default is Spectre's own existing instrument default, `PULSE_PARAMETERS[0]`'s `0.2`, reused so that swapping the alpha's instrument does not change its resting loudness. The geometric-midpoint rule degenerates to `0` on a range whose minimum is `0`, and a silent instrument default would be a fake surface, so this row states its own basis explicitly. | `new_device_setters_reach_the_rendered_signal` | proposed |
| DEV-006 | `gloam.damp_hz` MUST range over `20.0 … 20000.0` Hz. | Spectre's own already-accepted audible band: `TONE_PARAMETERS[0]` declares `20 … 20 000 Hz` for the frequency of a native source. Declaring a second, different audible band for a filter corner would create two definitions of one thing; this row reuses the one that exists. | `gloam_never_exceeds_its_input_peak` sweeps both endpoints | proposed |
| DEV-007 | `gloam.damp_hz` MUST default to `632.5` Hz. | Derived by the DEV-004 rule: the geometric midpoint of `20 … 20 000`, rounded to one decimal. `20 × 20 000 = 400 000`; `sqrt(400 000) = 632.4555…` → `632.5`. A one-pole corner has no more principled center than the geometric center of its own declared band, and a derived number is auditable where a taste number is not. | `gloam_at_zero_depth_is_a_plain_one_pole` computes its reference coefficient from this default | proposed |
| DEV-008 | `gloam.depth` MUST range over `0.0 … 1.0` with default `0.0`. | Range is the normalized fraction of the available corner travel. Default `0.0` is deliberate and is not the midpoint rule: at `0.0` the device is a plain static one-pole with a closed-form response, which is the least-surprise resting state and the reference state the zero-depth test asserts against. Level tracking then becomes attributable to a deliberate move rather than to the device's default. | `gloam_at_zero_depth_is_a_plain_one_pole` | proposed |
| DEV-009 | `gloam.track_ms` MUST range over `1.0 … 1000.0` ms with default `31.6`. | Same range and same derived default as DEV-003/DEV-004, for the same two reasons: 1 ms is at least eight sample periods at `MIN_SAMPLE_RATE`, so the follower is a filter rather than a bare rectifier, and 1000 ms is two beats at the project default tempo. Sharing the envelope with `filament.rise_ms` is intentional — one derivation, three parameters. | `filament_contour_minimum_is_at_least_eight_sample_periods` covers the follower minimum too | proposed |
| DEV-010 | `Filament` MUST have exactly one voice. | Polyphony requires a voice count *and* a voice-stealing policy, and both are limits Spectre has no evidence for. One voice also matches the shipped instrument, whose voice state is a single `Option<(u32, u8)>`. **Re-open trigger:** MIDI clips producing overlapping notes a user expects to hear together. | `filament_is_an_exact_sine_at_the_default_lean` renders one voice; the struct holds one `Option<(u32, u8)>` | proposed |
| DEV-011 | `Gloam` MUST be first order — one pole per channel. | One pole is the highest order whose stability needs no argument beyond "the coefficient is in `[0, 1]`": it is a convex combination of the current input and the previous output, so it cannot ring, cannot self-oscillate, and cannot exceed its input's peak, whatever the parameters do. A second order would introduce resonance, and resonance introduces a Q limit — a numeric bound with nothing behind it. **Re-open trigger:** the first accepted requirement that needs a resonant filter. | `gloam_never_exceeds_its_input_peak` across both damp endpoints and three depth settings | proposed |
| DEV-012 | `Gloam` MUST flush its recursive state at `f32::MIN_POSITIVE`. | Not a new number: it is the same predicate and the same constant the accepted RT-003 implementation already uses on plan output (`crates/spectre-graph/src/lib.rs`), applied to this device's internal state because the plan's flush sees output buffers and never a device's state. A second threshold would be a second definition of denormal. | `gloam_state_reaches_exact_zero_after_silence` | proposed |
| DEV-013 | A test asserting `Gloam` reaches exact silence MUST bound its search at 64 render quanta. | Derived from DEV-007's own default: at `damp_hz = 632.5` and 48 kHz the coefficient is `≈ 0.0794`, so the per-sample decay factor is `≈ 0.9206` and reaching `f32::MIN_POSITIVE ≈ 1.18e-38` from `1.0` needs `≈ 87.4 / 0.0827 ≈ 1 052` samples — about 4.1 quanta of 256 frames. 64 quanta is roughly 15× that, which is headroom for a slower corner without being unbounded. A test that waited forever could not fail. | `gloam_state_reaches_exact_zero_after_silence` asserts the bound is not hit | proposed |
