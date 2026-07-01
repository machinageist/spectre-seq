<!--
Author: Jeff
Date: 2026-06-22
Description: Terse handoff for the Ableton-style Geist DAW overhaul
Notes: Branch state, committed milestones, and the uncommitted in-flight B5 work
-->

# Geist DAW — Ableton Overhaul Handoff

## State
- Branch: `claude/ableton-studio-overhaul` (off `main`, not merged/PR'd).
- Last commit: `c378660 Architecture alignment (native devices under crates/, VST3-only) + B5 dynamic FX chain`.
- **Working tree is CLEAN.** The previously-uncommitted architecture-alignment + B5 slices were
  verified green (`cargo check --workspace` + `cargo test --workspace` both exit 0) and committed
  together as `c378660` on 2026-06-30.
- Architecture alignment (now committed): native device crates moved `plugins/` -> `crates/`,
  dormant first-party CLAP export files removed, workspace excludes CLAP/LV2 host scaffolds.
  Docs trail: `INITIAL_PLAN.md`, `PROPOSED_FILE_TREE.md`, `docs/architecture.md`,
  `docs/architecture/native-vst-internal-devices.md`, `docs/vst_hosting.md`.

## Loop continuation — where the /loop left off (2026-06-30)
Running `/loop` to resume `docs/gpt_mega_prompt.md`. Iteration log + the precise next slice:
- **Iter 1 (done):** oriented, verified green, committed the in-flight checkpoint `c378660`.
- **Iter 2 (done) — B5 loose end #3: EQ + Saturator in the FX palette/pool.** Added `FxKind::{Eq,
  Saturator}` + UI `EffectKind::{Eq,Saturator}`; `fx.rs` pools (EQ = single peaking band via
  `ParametricEq::new(1)`, params 0=freq/1=gain_db/2=q recomputed together in `apply_eq_band` since
  `Biquad::set_peaking` needs all three; Saturator 0=drive/1=out/2=mix, Tanh curve, stateless so
  untouched in `prepare`); studio maps + `FX_DEFAULTS` (6 rows) + the two mirror caches
  (`fx_on`/`fx_param`) grown 4->6 (indexed by kind index); `plugin_rack` menu; `session.rs`
  `fx_kind_to/from_f32` (Eq=4,Saturator=5). **Correction vs Iter-1 plan:** did NOT grow
  session `FX_COUNT` (it drives the *legacy* per-kind param-ID layout `base + t*FX_COUNT`; growing
  it would misalign old-project migration) — legacy match now skips Eq/Saturator via `continue`.
  Verified: `cargo test -p geist-daw`(65)/`-p geist-fx`(145)/`-p geist-dsp`(15)/`-p geist-ui`(57)
  green; clippy clean bar the known pre-existing `engine.rs:1403`; touched files rustfmt-clean.
  New test `fx::tests::eq_and_saturator_process_when_enabled`.
- **Iter 3 (done) — B6: session-slot save/load (M5 follow-up).** `TrackSession.session_slots:
  Vec<SessionSlotSession{scene,notes}>` persisted via a reserved clip-id block
  `SESSION_CLIP_ID_BASE = u64::MAX - MAX_SCENES - 1` (just below `STEP_CLIP_ID`); `to_project`
  writes one reserved MIDI clip per created slot (empty-but-filled slots persist too),
  `from_project` decodes them via `session_scene_of(id)` without leaking into arrangement clips.
  `studio.rs` `to_session` gathers filled slots + notes; `apply_session` tears down engine slots
  (`StopSlot` + `RemoveSessionNote` for tracked notes) then rebuilds (`CreateSessionSlot` +
  `AddSessionNote`) and restores the grid/`session_notes`. Slot length is fixed (`SESSION_CLIP_LEN`
  = `SESSION_SLOT_LEN_BEATS` = 4.0), so not stored per slot. Verified: `cargo test -p geist-daw` 66
  green (new `session::tests::session_launcher_slots_round_trip`); clippy clean bar the known
  `engine.rs:14xx`; touched files rustfmt-clean.
- **Iter 4 (done) — B6: arrangement loop-region drag.** Foundations already existed
  (`geist-timeline` `LoopRegion` + loop-aware `Playhead`; engine's `transport.advance()` already
  folds into the loop; UI `Transport` already had `loop_enabled/start/end`). Wired the gap:
  `EngineCommand::SetLoop{enabled,start_beats,end_beats}` (engine converts beats->samples via
  `tempo_map().beats_to_samples` and calls `set_loop`/`clear_loop`); studio mirror + `emit_engine_diff`
  send it; `arrangement.rs` `draw` takes `&mut Transport` and a ruler `Sense::click_and_drag`
  (anchor beat stashed in egui temp state) sets `loop_start/end/enabled` with grid snapping — a bare
  ruler click clears the loop; `shell.rs` passes `&mut session.transport`. Verified: `cargo test -p
  geist-daw` 67 (new `engine::tests::set_loop_wraps_the_playhead`) + `-p geist-ui` 57 green; clippy
  clean bar known `engine.rs:14xx`; my edits add no new rustfmt diffs (arrangement.rs/shell.rs were
  already non-fmt-clean in HEAD — left per surgical rule).
  **Skipped mixer/browser context menus** as low-value: mixer strips already reset via fader/knob
  double-click + expose M/S/R toggles (a menu would be redundant); browser "insert" needs catalog
  redesign (studio catalog lists fixed Filter/Delay/Reverb, not addable character effects) — design
  ambiguity, deferred.
- **Iter 5 (done) — loop-region persistence.** `StudioSession.{loop_enabled,loop_start_beats,
  loop_end_beats}` persisted as macros-node params `PARAM_LOOP_ENABLED/START/END` = 3/4/5 (global,
  like `PARAM_GAIN`=2); `to_session` reads `session.transport`, `apply_session` sends `SetLoop` +
  restores transport/mirror. Verified: `cargo test -p geist-daw` 68 (new
  `session::tests::arrangement_loop_round_trips`; existing round-trip test now also covers loop);
  clippy clean bar known `engine.rs:14xx`; touched files rustfmt-clean.
- **Iter 6 (done) — record-into-session-slot (M5 follow-up).** In Session view, an armed track's
  live playing records into its selected slot (created/filled if needed) instead of a new
  arrangement clip. New `session_record_target: Option<(u8,u8)>` (mutually exclusive with
  `record_target`); `sync_recording` branches on `MainView::Session` at record start/stop;
  `note_event` routes captures to `commit_recorded_session`, which folds each note into the slot
  loop phase via `session_slot_pos(record_start, note_start_rel, SESSION_CLIP_LEN)` (rem_euclid),
  overdub/non-destructive, and adds `AddSessionNote` + updates `session_notes`(+mirror). Verified:
  `cargo test -p geist-daw` 69 (new `studio::tests::session_slot_pos_folds_into_the_loop_phase`);
  clippy clean bar known `engine.rs:14xx`; studio.rs rustfmt-clean. UX (arm/record/hear-loop) needs
  a human `cargo run` to fully QA — the loop-phase math is unit-tested.
- **NEXT SLICE — reverb decay live control (harder; architectural).** `ReverbNode::set_decay_seconds`
  only takes effect on `prepare()`, and the `Copy`-only rtrb command ring can't carry a rebuilt
  `ReverbNode`. Needs an off-thread rebuild + a separate object/return ring (or a
  partitioned-convolution reverb whose decay is settable live). Investigate the rtrb ring setup in
  `control.rs`/`engine.rs` before committing. Alternatively node-graph drag-to-connect (large: the
  modular graph view is representative-only and may not be wired to audio).
  **The core Ableton-overhaul + session launcher (M1–M5) is now complete; only these two harder,
  lower-priority items remain — a natural point to consider winding down and reassessing against
  `docs/gpt_mega_prompt.md`.**
- Plan + per-slice checklist: `~/.claude/plans/keep-working-on-this-tranquil-hickey.md`.
- Memory: `~/.claude/projects/-Users-machinageist-geist-daw/memory/geist-ableton-overhaul.md`.

## 2026-06-30 native-device/VST-only plan position
- User direction: Rust-first DAW, custom native DSP/device architecture, VST3 hosting only for
  third-party plugins, and no first-party plugin binaries.
- Current implementation position: phases 0-9 are represented in code/docs at first-vertical-slice
  level; `geist-vst-host` remains a boundary scaffold and is not required for the first slice.
- Active native devices: `crates/geist-synth`, `crates/geist-fx`, `crates/geist-modular`.
- Phase 1 continuation landed explicit core time newtypes in `crates/geist-core/src/time.rs`:
  `SampleTime`, `BeatTime`, `Seconds`, `PpqTick`, `BarBeat`.
- Phase 1 continuation also landed internal device primitives in `crates/geist-core/src/devices.rs`:
  `DeviceKind`, `DeviceDescriptor`, `DeviceState`, plus `DeviceId` in `ids.rs`.
- Graph/device continuation landed `geist-graph::node::AudioDevice` with descriptor, parameter,
  latency, state, and load-state hooks above the existing realtime `AudioNode` trait.
- Shelved historical crates: `crates/geist-clap-host`, `crates/geist-lv2-host` are excluded from
  workspace builds and should not receive feature work unless Jeff explicitly reverses policy.
- Current docs-cleanup slice: stale planning docs are being aligned to `docs/gpt_mega_prompt.md` so
  native/internal device work cannot collide with historical plugin-suite/CLAP/LV2 language.
- Next coherent cleanup after docs alignment: rename remaining source/module-level `plugin_rack`
  identifiers only when it is worth the churn; user-facing docs should say device chain/rack unless
  they mean third-party hosted VSTs.
- 2026-06-30 docs collision-prevention pass updated project-local skills/agents plus canonical docs:
  `.claude/skills/README.md`, `.claude/skills/geist-daw-working-context.md`,
  `.claude/skills/geist-plugin-hosting.md`, `.claude/skills/geist-ui-workflow.md`,
  `.claude/agents/README.md`, `.claude/agents/geist-plugin-host-agent.md`,
  `INITIAL_PLAN.md`, `PROPOSED_FILE_TREE.md`, `docs/adr/001-clap-over-vst.md`,
  `docs/architecture/native-vst-internal-devices.md`, `docs/ui_ux_principles.md`, and
  `docs/vst_hosting.md`.
- Docs guardrail now says: use "plugin" for third-party hosted VSTs only; use device/internal
  device/device chain/rack for Geist synths, effects, MIDI tools, modulators, and modular utilities.
- Docs validation for this pass: `git diff --check` passed; targeted stale-term searches for old
  rack/CLAP-first/first-party-plugin/time-newtype phrases returned no live repo-wide matches;
  deterministic Python doc-presence checks passed.

## Committed (done)
- **M1–M5**: Ableton layout (left browser / center Arrange⇄Session / toggle mixer / bottom Clip⇄Device),
  Play/Stop/Pause/Record, synth FM + coarse/fine pitch + polyphony + per-track LFO,
  phaser/flanger/distortion DSP + fixed FX slots, session clip launcher.
- **B1** `d69f33f`: piano roll full MIDI 0–127, opens on C4.
- **B2** `bd75082`: per-view adjustable grid + snap + gridlines (piano roll + arrangement).
- **B3** `6c7ec20`/`cf85cba`: note quantize (Cmd/Ctrl+U) + session launch quantization + playing-scene readback.
- **B4** `2bcede6`: snapshot undo/redo (`app/geist-daw/src/history.rs`), Cmd/Ctrl+Z / +Shift+Z.
- **B6** `3a5d7bc`: piano-roll velocity editing (Alt-drag).

## B5 dynamic FX chain (COMMITTED in c378660; not visually QA'd)
Duplicate-capable, reorderable per-track effects chain. The design from the prior handoff is now implemented:
- `control.rs`: `FxSlot{kind,instance}`, `FX_CHAIN_MAX=8`; commands `SetFxChain{track,slots,len}` (whole-chain replace, like ClearPattern) + `SetFxOn`/`SetFxParam` now **instance-addressed**.
- `fx.rs`: `FxChain` holds a per-kind **instance pool** (`FX_INSTANCE_POOL=4`, one per channel) + ordered `character_chain: [FxSlot; FX_CHAIN_MAX]`; `set_character_chain` rebuilds order; delay+reverb stay the fixed tail. No audio-thread alloc (pools built in `new`/`prepare`).
- `studio.rs`: `handle_rack_intents` parses `add_effect:{name}` (+ existing rack remove/reorder intents); `fx_chain_len`/`fx_chain_slots` mirror diffed to `SetFxChain`; `default_character_chain()`.
- `plugin_rack.rs`: "+ Add Effect" menu (Distortion/Phaser/Flanger/Chorus).
- `session.rs`: `FxSession{kind,instance,on,params}` + `TrackSession.fx_chain`; persisted (param bases 700/730/760/800). Legacy per-kind `fx_on`/`fx_param` project params are load-only migration input and are no longer written for new saves.

### B5 loose ends before committing
1. **Resolved in this slice:** `TrackSession.fx_chain` is now authoritative for saved character-FX state; `TrackSession.fx_on`/`fx_param` were removed, new project saves no longer write legacy params, and old per-kind params migrate into the default chain on load.
2. **Automated coverage added:** duplicate processing, no-grow command traffic, and rack duplicate/reorder/remove chain serialization are covered by tests; visual/manual UI QA is still pending.
3. **EQ + Saturator** — DONE (Iter 2): added to `FxKind`/`EffectKind`, the FX pool, palette menu, and persistence.
4. Confirm `FX_INSTANCE_POOL=4` vs `FX_CHAIN_MAX=8` is intended (8 ordered slots, 4 instances/kind).
5. Land as one coherent commit (engine+control+studio+session), then tick plan B5 + update memory.

## Remaining (B6 gaps)
- Session-slot save/load — DONE (Iter 3, `SessionSlotSession` + reserved session clip-id block).
- Loop-region drag in arrangement — DONE (Iter 4, `SetLoop` + ruler drag).
- Loop-region persistence — DONE (Iter 5, `PARAM_LOOP_*` macros params).
- Record-into-session-slot — DONE (Iter 6, `session_record_target` + loop-phase folding).
- Mixer/browser context menus — deferred (low-value / design-ambiguous; see Loop continuation).
- Lower priority (harder/architectural): reverb decay live control, node-graph drag-to-connect.

## Validate / run
- 2026-06-30 post-architecture validation:
  - `cargo check --workspace` passed.
  - `cargo test --workspace` passed.
  - `cargo metadata --no-deps --format-version 1` lists active crates only: no `geist-clap-host` or `geist-lv2-host`.
  - `cargo clippy --workspace --all-targets` exited 0 with one warning in pre-existing dirty app code:
    `app/geist-daw/src/engine.rs:1403` (`clippy::needless_range_loop`).
  - Phase 1 time/device continuation: `cargo test -p geist-core` passed (45 tests), and
    `cargo check -p geist-daw` plus `cargo check --workspace` passed after targeted `rustfmt` on the touched core files.
  - Graph/device continuation: `cargo test -p geist-graph` passed (34 tests), and
    `cargo check -p geist-vst-host -p geist-synth -p geist-fx -p geist-daw` passed after targeted `rustfmt` on touched graph files.
  - B5 dual-FX-state cleanup: added `session::tests::project_persists_character_fx_chain_without_legacy_parallel_state`, `session::tests::legacy_per_kind_character_fx_params_load_into_default_chain`, and `studio::tests::character_chain_tracks_duplicate_reorder_and_remove`; `cargo test -p geist-daw` passed (64 tests), `cargo check -p geist-daw` passed, `cargo clippy -p geist-daw --all-targets` passed with the known pre-existing `engine.rs:1403` warning, and `rustfmt --check app/geist-daw/src/session.rs app/geist-daw/src/studio.rs` passed.
- General gate: `cargo test --workspace`; `cargo clippy --workspace --all-targets` should return no new warnings from the architecture slice.
- Visual/audio QA (egui can't be screenshotted here): `cargo run -p geist-daw --release`. `--classic` must still open the old GUI.
- Per-slice gate: `cargo build/clippy/test -p <crate>`; commit each slice.

## Cautions
- Never allocate or lock in `FxChain::process()`; pools are prebuilt, not handed over a ring.
- Don't half-land B5 — it spans engine, control, UI, studio mirror, and persistence together.
- This file is uncommitted unless Jeff asks; the B5 source changes are also uncommitted.
- Legacy saved projects can still contain old per-kind FX params; keep the load-only migration path until project schema migration policy is formalized.
