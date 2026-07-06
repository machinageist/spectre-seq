<!--
Author: Jeff
Date: 2026-07-03
Description: Atomic implementation plan for the modular rack change
Notes: Executes AGENTS/changes/modular-rack/SPEC.md against docs/modular_rack_spec.md;
       tasks are ordered, one commit each, engine before UX before persistence
-->

# Modular Rack — Implementation Plan

Verification shorthand: `T <crate>` = `cargo test -p <crate>`, `W` = full
`cargo test --workspace` + clippy, `R` = launch app and exercise by hand.

## M1 — Signal contract (engine substrate)

1. **`geist-modular::standards`** — voltage constants and helpers from spec §2:
   `GATE_V = 10.0`, trigger len 1 ms, Schmitt (0.1/1.0 V rearm/fire) struct,
   pulse generator, `volts_to_hz` / `hz_to_volts` (C4 @ 0 V audio, 2 Hz @ 0 V
   LFO), `flush_non_finite`. Property-style unit tests pin the exact numbers
   (SPEC acceptance 2). → verify: `T geist-modular`.
2. **Retro-fit existing node families** — timing/logic/S&H nodes consume the
   shared Schmitt + pulse helpers instead of local thresholds; behavior pinned
   by existing 28 tests plus new threshold-edge tests. → verify: `T geist-modular`.
   **DONE 2026-07-03 (uncommitted):** Clock divider, flip-flop, sample & hold,
   and track & hold now use `standards::Schmitt` for trigger/gate hysteresis;
   threshold-edge tests added. Verified `cargo test -p geist-modular` (37 pass).
3. **Poly channel semantics** — helper implementing spec §3.3/§3.4 mapping
   (M=1 broadcast, M≥N index, 1<M<N zero-fill, 0 = unpatched) over
   `ProcessContext` channels + mono-summing rule; unit tests for all four
   cases (SPEC acceptance 3). → verify: `T geist-modular`.
   **DONE 2026-07-03 (uncommitted):** Added `get_poly_voltage`,
   `mono_audio_sum`, and `mono_cv_first` helpers, exported through
   `geist-modular::prelude`; tests cover broadcast, indexed mapping,
   short-source zero fill, unpatched zero fill, 16-channel cap, audio mono sum,
   and CV mono first-channel fallback. Verified `cargo test -p geist-modular`
   and `cargo check -p geist-modular`.

## M2 — Rack node set (playable minimum)

4. **Node registry** — one `rack_catalog()` in `geist-modular` (or a thin
   `geist-daw` module if cross-crate) listing constructible rack nodes with
   name, category tag (spec §10.3 vocabulary subset), port list. Existing
   utilities registered first. → verify: `T geist-modular` + registry test.
5. **Generator/processor adapters** — wrap `geist-dsp` osc/lfo/env/filter and
   a VCA as `AudioNode` rack nodes speaking v/oct + gate per M1 helpers.
   No new DSP; adapters only. Per-node smoke tests (440 Hz at A4 volts, env
   fires on gate edge). → verify: `T geist-modular`, `T geist-dsp` untouched.
6. **Bridge nodes** — transport clock node (24 PPQN + divided output off the
   existing transport snapshot), MIDI→CV node (v/oct, gate no-legato-retrig,
   velocity, RTRG; poly modes Rotate/Reuse/Reset per spec §6.4), rack-out node
   feeding the track chain. Poly-mode unit tests. → verify: `T geist-modular`,
   `T geist-graph`.

## M3 — Patching UX (Build lens)

7. **Drag-to-connect** — port hit-zones in `views/node_graph.rs`; drag
   out→in (either direction) draws a live cable; drop on a compatible port
   emits a connect intent; inputs single-cable (new cable replaces), outputs
   fan out; drag a cable end off a port to disconnect. Poly cables draw
   thicker (spec §3.1, §4.1). → verify: `T geist-ui` view-model tests + `R`.
   **DONE 2026-07-04 (`a7c84f0`):** GraphModel connect/disconnect helpers +
   Cable.channels; live cable with amber mismatch feedback (never refuses);
   input pickup re-routes; `graph_connect:`/`graph_disconnect:` intents
   emitted for task-10 engine sync. geist-ui 64+3, workspace 635, clippy
   clean. Manual `R` pass still owed at the M3 milestone gate.
8. **Node lifecycle** — browser items drop onto the graph canvas to add a
   rack node at the cursor (reuse the existing dnd intent pattern from the
   arrangement/rack work); Delete/Backspace removes the selected node and its
   cables, focus-gated like the rest of the app. → verify: `T geist-ui` + `R`.
   **DONE 2026-07-06:** GraphModel `add_node`/`remove_node`/`next_node_id`;
   click-select with accent ring, empty-canvas click deselects; browser
   payload dropped on the canvas adds a node at the pointer (generic In/Out
   ports until task-10 catalog wiring) with hover preview; Delete/Backspace
   removes the selected node + its cables (pointer-over-canvas + focus gated);
   emits `graph_add:`/`graph_remove_node:` intents for engine sync. geist-ui
   68+3, app 73, clippy clean. Manual `R` owed at the M3 gate.
9. **Param feel** — knob drag ladder (Shift fast / Ctrl slow / Ctrl+Shift
   finest), double-click reset to default, right-click numeric entry via the
   existing param-edit surface (spec §1.5, §4.4). → verify: `T geist-ui` + `R`.

## M4 — Integration and persistence

10. **Engine wiring** — rack patch compiles into the existing process graph
    via graph swap; allocator guard proves the hot path clean while a patch
    plays. → verify: `T geist-graph`, `T geist-audio-backend`, RT guard test.
11. **Project persistence** — rack nodes, positions, cables, and param values
    serialize with the project; load rebuilds the patch (SPEC acceptance 5).
    Round-trip test. → verify: `T geist-project` + `R` save/reload.
12. **End-to-end gate** — the SPEC acceptance-1 patch played by hand; then
    `W`, headless + release smoke gates, HANDOFF iteration log entry, and
    update `docs/modular_rack_spec.md` status line. → verify: `W` + `R`.

## Decisions taken (flag to Jeff if wrong)

- This plan executes `PRODUCTION_PLAN.md` P6; P6.4 latency compensation stays
  in the production plan, not here. Both gates resolved by Jeff 2026-07-03:
  task 10 migrates the live engine onto `geist-graph`'s compiled plan + swap;
  task 7 keeps any-out-to-any-in patching with feedback-only validation
  (see SPEC non-goals). No open gates — tasks execute in order.
- Rack lives in the existing Build-lens node graph over the existing process
  graph — no separate rack device/document. Cheapest path; matches Phase 9's
  "partially implemented" surface.
- Module presets (`.vcvm`-style JSON, spec §11) deferred to a follow-up
  change; project persistence covers session continuity first.
- Poly cable channel count capped at 16 to match the spec and SIMD width
  planning.

## Task→agent map

M1–M2 `geist-audio-dsp-agent` / `geist-core-graph-agent` (task 6 bridge),
M3 `geist-ui-project-agent`, M4 tasks 10 → core-graph, 11 → ui-project,
12 → `geist-reviewer` before the milestone commit.
