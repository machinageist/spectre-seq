<!--
Author: Jeff
Date: 2026-07-12
Description: Current verified state of Spectre
Notes: Claims here link to live evidence; optimistic language is prohibited
-->

# Status

- **Status:** accepted
- **Last verified:** 2026-08-25
- **Scope:** current implementation, documentation, and research state
- **Decision authority:** Jeff
- **Upstream sources:** workspace tests, `../01-requirements/traceability.md`, research ledgers
- **Downstream dependents:** `NEXT.md`, `../06-plans/current-milestone.md`
- **Supersedes:** all removed prototype-era status and handoff material
- **Superseded by:** none
- **Open decisions:** milestone-gated decisions in `../01-requirements/decision-gates.md`; rows 19-22 ratified 2026-08-09
- **Known gaps:** the only callback-headroom measurement Spectre owns is macOS on a three-node plan, and R4-4 makes the plan up to 34 nodes, so decision 23's Linux debt is now wider than when it was written; no Linux audio device has ever been opened; `docs/03-architecture/dsp-device-io.md` asserts `Gain` smooths and the shipped `Gain` does not, open as D-R3; `./spectre`'s transport button, engine status cluster, Retry control, and Shape sliders have not been exercised by hand, so neither landed slice has run its manual protocol

## Repository state

The prototype implementation and all prototype-only plans, assets, CI, audits, archives, feature lanes, and agent scaffolding were removed on 2026-07-12. The former namespaced workspace was promoted to the repository root. Git history retains committed historical material.

The project was renamed from Geist to Spectre on 2026-08-09, aligning the code with the `spectre-seq` repository. The rename covered all six crate directories and package names (`spectre-core`, `spectre-dsp`, `spectre-graph`, `spectre-project`, `spectre-offline`, `spectre-app`), the `./spectre` launcher and its binary, in-app strings, the research taxonomy tags `SPECTRE-CANDIDATE`/`SPECTRE-REQ`, the local agent skills, and all documentation. It was a pure identifier and prose rename: no behavior, contract, schema, or fixture content changed, and the checked-in project fixtures never carried the old name. The copyright holder `machinageist` is unrelated to the product name and is unchanged.

The active workspace contains:

- `spectre-core`: stable IDs, explicit time types, tempo and meter maps, transport, bounded event ordering, and parameter descriptors;
- `spectre-dsp`: planar-buffer processing contract, a callback-safe runtime parameter seam, bounded note events, deterministic tone source, Pulse instrument, Gain, Saturator, the `SumBus` summing device, and the R4-6 pair — `Filament`, a one-voice phase-warped instrument behind a linear amplitude contour, and `Gloam`, a stereo one-pole damper whose corner opens with the signal's own level;
- `spectre-graph`: app-thread editable graph and immutable compiled plan (GRAPH-001 split) with validated compilation, implicit-cycle diagnostics, and measured allocation-free execution;
- `spectre-project`: versioned JSON envelope, semantic validation after decode, atomic command transactions, bounded undo/redo, and the R4-4 track model — an ordered `TrackList` with stable identity, mute/solo/level, and the graph builder that turns it into an instrument → track gain → sum → master render path;
- `spectre-offline`: deterministic project inspection, a Pulse → Gain → Saturator fixture and a Filament → Gloam voice chain rendered through the compiled plan, including an offline snapshot entrypoint that requires the complete four-parameter fixture and validates exact backend identities and authoritative values before processor construction;
- `spectre-audio`: audio backend trait seam with validated stream configuration, a deterministic pump-driven null backend for CI and offline, a cpal implementation behind a default-on feature, the RT-002 split-lane control transport over a bounded wait-free SPSC ring, a callback bridge that drives the existing compiled plan, timestamped MIDI ingress, and off-thread health telemetry;
- `spectre-app`: native egui shell with backend-derived Build/Shape device surfaces, stable project-instance device focus, an owned offline device-parameter snapshot seam, a selection-aware feedback-report seam, and the R4-1 live engine host (`engine.rs`) that compiles a plan from the app snapshot, opens the default device through the `AudioBackend` seam, and owns the stream on the UI thread.

`./spectre` launches the graphical interaction prototype. Build shows the native device signal path and offers a visible `Open in Shape` action on every card. The action atomically preserves track selection, focuses that existing device by stable `ObjectId`, and enters Shape; Shape renders only the selected device using backend parameter descriptors and setters. Ordinary lens changes and parameter edits preserve device focus. App parameter edits can be transferred as an owned snapshot to deterministic offline rendering independently of focus, where the complete fixed fixture is validated before values construct processors in the immutable compiled plan. **R4 slice 1 landed 2026-08-24 and this is now a live engine.** On its first frame `./spectre` compiles a plan from the app snapshot, queries the default device's own sample rate through the seam, opens it at 256 frames, and moves the `RenderBridge` into the render closure. Play sends a transport command and one held audition note through the RT-002 lanes; Stop sends all-notes-off and a Stop command. The app sends first and mutates second, so the UI transport changes only when the render thread will see the same change. The transport bar reports the backend, device, rate, and block size, exposes the render thread's own counters on hover, and reads `ENGINE RUNNING` only once `blocks_rendered > 0` — an opened stream that has not called back reads `ENGINE OPENED`.

Status is `implemented`, not `verified`: the manual protocol in the R4-1 spec §5.4 has not run, and no one has confirmed by ear that sound leaves the speakers. What is evidenced is that the app's own `open_default` reaches a real driver and renders through it — the `#[ignore]`d `app_engine_opens_a_real_device_and_renders` drill opened an M-Audio AIR 192|6 through CoreAudio at its native 88 200 Hz and rendered 88 blocks with 0 xruns, 0 plan errors, 0 frame-capacity rejections, 0 contaminated nodes, and 0 stream errors. **R4 slice 2 landed the same day and closed the first of those limits.** `AudioProcessor` gained one required `set_parameter`; `CompiledPlan::set_parameter` addresses a live node's processor; a frozen, binary-searched route table maps an RT-002 target to a node and key and is itself added to `rt_guard`'s scanned modules. The bridge applies the drained lane once per block before `process`, so a block sees one coherent parameter set, and a Shape edit now changes what the render thread produces. **One limit remains:** the audition voice is scaffolding that R4-5 replaces with clip playback. **One conflict is open rather than resolved:** the accepted device contract asserts `Gain` smooths and the shipped `Gain` does not — slice 2 declined to change either side by assertion, and it stands as D-R3. No VST3 host, recording path, or project editing canvas exists; the Record button is disabled and says so. A live CoreAudio driver has been opened and driven by the bridge from the slice 7 drill, and, since R4-1, through the app's own engine path as well.

The accepted JSON project codec and 960-PPQ `BeatTicks` representation have checked-in R1 fixtures. Tempo conversion evidence now covers signed pre-roll, fractional piecewise boundaries, 24-hour positions, unrounded-anchor accumulation, and nearest-tick sample quantization without claiming impossible one-sample arbitrary-sample round trips.

## Validation

The current gate is:

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

Latest full-gate result (2026-08-25, after R4 slices 1, 2, 4, and 6): formatting, strict Clippy, and all **299** tests pass with **two** `#[ignore]`d hardware drills, both run by hand on macOS — the audio crate's `hardware_lifecycle_drill` (175 blocks, 0 xruns, worst headroom 0.974, `frame_capacity_rejections=0`) and the app's `app_engine_opens_a_real_device_and_renders` (88 blocks at 88 200 Hz, 0 xruns, worst headroom 0.970). R4-1 added 14 counted tests (12 in `crates/spectre-app/tests/live_engine.rs`, 2 in `crates/spectre-audio/tests/backend_seam.rs`) and R4-2 added 9 more (3 in `live_engine.rs`, 3 in `crates/spectre-dsp/tests/devices.rs`, 2 in `crates/spectre-graph/tests/graph_plan.rs`, 1 in `crates/spectre-app/tests/app_model.rs`). The live-engine set proves the app's plan renders bit-identically to `render_app_snapshot` over the same snapshot, that Play produces a nonzero peak and Stop returns exact silence, that an opened stream never reads as running before its first callback, that a Play and Stop landing in one block are refused into counted silence with no stuck note, and — with a re-declared guarding allocator and its own positive control — that the app's render closure allocates nothing across 16 blocks.

**R4 slices 4 and 6 landed after that count was first written, and the 299 above includes them.** Slice 4 added the track model, the `SumBus`, and the track → master graph builder; slice 6 added `Filament` and `Gloam`. Slice 6's evidence is 25 counted tests: 15 device-contract tests in `crates/spectre-dsp/tests/devices.rs` (30 total, up from 15) and 3 device-boundary RT tests in its new `device_rt.rs`, 3 app-model tests, 2 offline harness tests, 1 containment test, and 1 allocation-guarded plan test in `crates/spectre-audio/tests/rt_guard.rs` that drives a `Filament → Gloam` plan through the null backend's callback and records zero allocations across seven blocks. That last one was falsified before being trusted: a deliberate `vec!` inserted into `Gloam::process` made it fail, and removing it made it pass again.

Slice 6 arrived on a branch that predated slices 1, 2, and 4, and its integration is recorded here because it changed shipped code. It had defined its own `ParameterError` in a `runtime_parameter` module and implemented `set_parameter` as an inherent method on both devices, because `AudioProcessor::set_parameter` did not exist at its branch point. Both were reconciled onto the seam slice 2 landed: the duplicate module is gone and both devices implement the trait method. `spectre-offline`'s `render_plan` also carried its own copy of the peak/FNV-1a fold; it now returns through `report_from`, so the crate holds one definition of the equivalence walk. Three hand-wired tests that compute their own reference inside the test crate still pass unchanged, which is what proves the extraction changed no bit.

**Neither new device is reachable from a track yet.** `TrackInstrument` still has one variant, `Pulse`, so `Filament` and `Gloam` appear on the Build surface and render in the offline harness but are not in the plan `./spectre` runs. A Shape edit to either is stored by the model and reported as not having reached live audio, which is what the status line says. Making them selectable is R4-5's and R4-7's, not slice 6's.

**Corrected 2026-08-24 after R4-1's implementation review.** This paragraph previously also listed "a refused transport send leaves the UI unchanged" among what the set proved. It did not: the test asserting it built an `AppModel`, never passed it to the rule under test, and asserted that a fresh prototype was not playing — an assertion that cannot fail. The rule was correct in `main.rs`, but `main.rs` is a binary target unreachable from `crates/spectre-app/tests/`, so it had no coverage at all. The rule now lives in `spectre_app::engine::toggle_transport` and carries three falsifiable tests, one per branch. Two further review findings were fixed in the same pass: the smoke line's `engine=` field was a hard-coded literal, so the assertion guarding the headless path could not fire either, and it is now derived from the shell's own engine.

The previous full-gate result (2026-08-09, after the Geist → Spectre rename and R3 slices 2-9): formatting, strict Clippy, and all 230 tests passed with one ignored hardware drill; the selected-device headless launch check and offline self-test pass. The app evidence includes 27/27 model tests and 1/1 process smoke test; the offline harness retains 21/21 tests. Selection tests cover deterministic Pulse focus, atomic valid drill-in, invalid-ID rollback of lens/selection/device state, all-lens and parameter-edit continuity, selected descriptor and parameter identity, focus-independent complete snapshots with correct edited-value attribution, renderer-neutral Build/Shape presentation, recoverable UI-thread error reporting, and selected-device feedback/smoke reporting. Existing snapshot tests continue to cover validated nonzero `ObjectId` decode at the public snapshot/render boundary, stable project-instance device/parameter identity, private-field DTO access through getters, canonical constructor containment of NaN/infinities/out-of-range input, exact signed-zero/subnormal publication, read-only app schema with identity-based value attribution and explicit invariant errors, exact complete fixture membership, duplicate/alias/partial/unknown/mismatched snapshot rejection, exact hand-wired render equivalence for all four mappings, backend-default equivalence, and deterministic repeated rendering. Native descriptor tests also pin the documented `f32` normalization and boundary policy. The existing R2 silence, impulse, allocation, and deterministic-hash evidence remains on the unchanged compiled-plan process path.

The rename changed no test count and no assertion: the same 155 tests pass before and after, and the pre-rename run on 2026-08-09 reproduced the 2026-08-06 result exactly. The only source churn beyond identifiers was rustfmt reordering `use` blocks, because `spectre_*` sorts differently than `geist_*`.

R0/R1 exited 2026-07-17 and R2 (offline graph) exited 2026-08-09 on its four render gates. R3 (live shell) exited 2026-08-09 with all ten rows closed; R4 (credible alpha) is the active milestone and opens with no implementation. Two of R3's rows closed on macOS hardware alone under decision 23: cpal opened an M-Audio AIR 192|6 through CoreAudio for 173 driver callbacks with 0 xruns and 0.990 worst-case headroom. No Linux audio device has been opened, so no Linux support is claimed and decision 1's co-first-class commitment remains undischarged; the drill is carried to R4 as debt. R2's exit does not claim full GRAPH-002 satisfaction: only implicit-cycle rejection exists, and explicit priced feedback edges stay gated at decision row 7 before R11.

The R1 exit disposition is complete: CORE-004's atomic-save API design is accepted via the project-persistence contract (implementation at R4, crash qualification at R5), and CORE-001 remains implemented with reorder evidence explicitly gated on the first persisted collection (R4) and migration evidence on the first schema migration (R5).

## Product and requirements

The product vision, requirement seed, decision defaults, roadmap, and current milestone are accepted. Decisions explicitly assigned to later intakes remain gated there rather than blocking R1.

## Research

- 29 unique ledger sources across 10 products.
- Four timestamped FL Studio action-sequence observations.
- Two Ableton thematic self-reports, not action-sequence evidence.
- No frequency, convergence, or priority claim is authorized by the current corpus.
- Visible-session Bitwig and Ableton evidence remains the highest-value workflow gap.