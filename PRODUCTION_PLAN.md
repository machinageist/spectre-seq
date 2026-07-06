<!--
Author: Jeff (drafted with Claude)
Date: 2026-07-03
Description: Implementation plan from the current vertical slice to production-ready
Notes: Successor planning layer above INITIAL_PLAN.md's phase status; targets the
       serious electronic musician moving off Ableton. Milestones ordered by
       switcher value, smallest-risk-first inside each. Decision gates marked JEFF.
-->

# Geist DAW — Production Readiness Plan

## Product bar

A serious electronic musician can move a real workflow off Ableton: play and
arrange with their existing VST3 library, automate everything, record and edit
audio and MIDI without friction, trust the engine under load, and never lose a
project. Geist does not need Ableton feature parity — it needs a coherent,
fast, stable core with the modular/sound-design identity the founding scope
defines.

## Verified starting position (2026-07-03)

- Workspace: 586 tests green; clippy clean bar one known warning; release builds.
- Realtime contract: documented (`docs/realtime_rules.md`), pool/ring
  discipline throughout, allocator-level enforcement test in place.
- Interactability: clips, piano-roll notes, effects, browser, session slots all
  support click-move / double-click-create / Delete / drag-and-drop.
- Headless smoke: debug + release binaries stream live audio (cpal) cleanly.
- Known engine seams: app engine is a fixed-track `SynthProcessor`;
  `geist-graph`'s compiled-plan/swap machinery is tested but unused by the live
  path; `geist-vst-host` is a scaffold (scanner/bundle/descriptor unit-tested,
  no end-to-end hosting); `geist-automation` is tested but unsurfaced in UI.

## Milestone P1 — stability and correctness floor

Outcome: nothing a switcher hits in week one feels broken.

1. **[JEFF gate] Reverb decay live control.** Decide: (a) off-thread IR
   rebuild + object swap over a ring (keeps the convolution sound; the
   asset-return ring is the proven pattern), or (b) algorithmic reverb
   (comb/allpass; decay is a per-block parameter). Then: Decay knob in the
   Reverb rack slot + `SetReverbDecay` command + tests.
   Verify: `cargo test -p geist-daw -p geist-fx`; audio QA.
2. Debug-assert RT guard in the live callback (not just tests): wrap
   `process_block` in a debug-build thread marker; `debug_assert` hooks in
   `push_capped`-class helpers. Verify: `cargo test --workspace`, debug run.
3. Xrun surfacing: the engine counts xruns (`stream.xruns()`); show the count
   + a health color in the transport strip. Verify: UI test + forced-load QA.
4. Latency accounting groundwork: devices report `latency_samples`; sum through
   the chain; log/display only (compensation lands with the graph in P6).
   Verify: unit tests on chain totals.

## Milestone P2 — VST3 hosting end-to-end (the switcher blocker)

Outcome: a user's existing VST3 instruments/effects load, sound, persist, and
show their editors. This is Phase 8 finished for real.

1. Scan cache schema (`geist-vst-host`): bundle paths + class descriptors +
   file hashes, serialized via `geist-project` blob conventions; rescan off the
   audio thread. Verify: `cargo test -p geist-vst-host` with fixture dirs.
2. Instance lifecycle against a real `.vst3` on this Mac (component +
   controller init, bus arrangement, activate). Verify: dev-box integration
   test behind `--ignored` (needs local plugin fixtures).
3. Process adapter as an internal device: wrap the prepared instance as a
   graph/device node obeying the callback contract (prealloc buffers, no
   COM calls in-process except the process call itself). Verify: offline
   render test with a known freeware VST3 fixture.
4. Parameter mapping: descriptor list -> rack knobs; `IComponentHandler`
   edits flow back as commands. Verify: param round-trip test.
5. State persistence: `IComponent::get/setState` blobs into the existing
   opaque-blob project slots (ADR 003). Verify: save/load round-trip.
6. Editor window: host the plugin GUI in a native child window from the UI
   thread. **[JEFF QA]** — needs windowed QA. Verify: manual.
7. Rack/browser integration: scanned plugins appear in the browser (searchable,
   draggable like native devices — "plugin" naming per guardrail). Verify:
   browser test + GUI QA.

## Milestone P3 — automation surfaced

Outcome: any knob can be automated in the arrangement; `geist-automation`
stops being dormant.

1. Automation lane model in the UI timeline (per-track lanes bound to
   ParamIds), drawing + breakpoint editing with the existing
   click/drag/Delete grammar. Verify: `cargo test -p geist-ui`.
2. Engine-side automation rendering: resolve lane curves to per-block param
   values through `geist-automation`; commands only carry lane edits
   (precompiled curves swap like other prepared state). Verify: offline render
   test with a ramped cutoff; allocator-guard test extended.
3. Record automation: knob moves while transport records write breakpoints.
   Verify: app test simulating knob traffic during roll.
4. Persistence + undo integration (lanes in session schema + EditSnapshot).
   Verify: round-trip + undo tests.

## Milestone P4 — audio clips first-class

Outcome: sample-based workflows work: drop a WAV in, trim it, balance it.

1. Import: drag an audio file from the OS onto an arrangement lane creates an
   asset + clip (decode off-thread via the recorder's WAV path; add format
   support as needed). Verify: app test with fixture WAV; GUI QA for the drop.
2. Clip gain + fades (in/out ramps applied at mix time from clip params —
   pool-friendly, no allocation). Verify: golden render test.
3. Clip start-offset/trim (window into the asset). Verify: render test.
4. Waveform preview in clips (background analysis thread; cached peaks).
   Verify: unit test on the peak reducer.
5. **[deferred, flag only]** time-stretch/warp — large DSP scope; decide after
   P6.

## Milestone P5 — session view completion

Outcome: live performance workflow holds up.

1. Scene model (names, per-scene launch), follow actions (next/stop/random
   with chance). Verify: engine tests for follow-action scheduling.
2. Clip launch modes (trigger/gate/toggle) + per-slot quantize override.
   Verify: engine tests.
3. Record-arm ergonomics: count-in, fixed-length record option. Verify: tests.
4. Session -> arrangement capture (record scene launches into the timeline).
   Verify: app test.

## Milestone P6 — modular identity: the graph goes live

Outcome: the node graph stops being representative and becomes the real
routing surface — the founding differentiator.

1. **[JEFF gate] Adoption strategy:** migrate the live engine onto
   `geist-graph`'s compiled process plan + swap (the crate is already tested),
   or keep the fixed-track engine and drive graph edits into it. Recommend the
   former; the fixed-track engine becomes one prebuilt graph.
2. Graph edits -> commands -> compiled-plan swap off-thread (ADR 002 pattern).
   Verify: swap tests + allocator guard.
3. Drag-to-connect in the node view with typed-port validation feedback
   (`ui_interaction_model.md` grammar). **[JEFF QA]** Verify: view tests +
   manual.
4. Latency compensation using P1.4's accounting. Verify: golden alignment test.
5. Modulation routes as first-class overlays (CV/mod signals through
   `geist-modular`). Verify: render tests.

## Milestone P7 — sound-design depth

Outcome: the reasons to leave Ableton, not just parity.

1. Wavetable editor first slice (frame list, waveform display, import
   single-cycle WAV, normalize/DC/smooth; editing operates on `geist-synth`'s
   wavetable types). Verify: `cargo test -p geist-synth` + GUI QA.
2. `geist-midi-tools` crate: scale lock + arpeggiator as internal devices on
   the track MIDI path (sample-accurate event buffers). Verify: event-offset
   tests.
3. Device growth per founding list as pull demands (compressor/limiter next —
   mix-bus table stakes). Verify: per-effect DSP tests.

## Milestone P8 — ship polish

Outcome: trustworthy daily driver.

1. Autosave + crash recovery (project snapshots on a timer thread; reopen
   prompt). Verify: kill-and-recover test.
2. Project bundles per founding scope (`.project/` with Audio/, PluginStates/,
   Backups/). Verify: round-trip tests.
3. Keybinding editor + command palette (config model already supports both).
   Verify: config tests + GUI QA.
4. Embedded/branded fonts + theme pass. **[JEFF taste]**
5. Packaging: macOS app bundle + codesign/notarize; Linux build check.
   Verify: `cargo bundle`/CI artifacts.
6. Delete `geist-clap-host`/`geist-lv2-host` after **[JEFF confirm]**.

## Standing gates (every slice)

- `cargo test --workspace` green; clippy no new warnings; touched files
  rustfmt-clean; per-crate targeted tests for the layer touched.
- The allocator-guard test extends to cover any new engine-side state.
- Realtime rules checklist (`docs/realtime_rules.md`) for every new
  `EngineCommand`.
- HANDOFF.md iteration log per session; INITIAL_PLAN.md phase status updated
  when a milestone lands.

## Sequencing note

P1 -> P2 are strictly first: stability floor, then the plugin library — those
two decide whether a switcher stays past the first week. P3/P4/P5 can
interleave (independent surfaces). P6 before P7's modulation-heavy work. P8
runs as a trailing lane once P2 exists.
