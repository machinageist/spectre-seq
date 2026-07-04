<!--
File: docs/realtime_rules.md
Layer: documentation
Purpose: the audio thread contract; contributor law
Status: Implemented 2026-07-03; filled in from the founding scope after the fidelity audit.
Contract: Keep comments terse, declarative, and synchronized with code.
-->

# Realtime Rules

The audio thread consumes precompiled state through bounded queues. Everything
here is binding; a change that needs an exception needs Jeff's sign-off first.

## Where the audio thread is

The cpal stream callback drives `BlockBridge`, which calls
`BlockProcessor::process_block` (`crates/geist-audio-backend/src/bridge.rs`).
The app's implementation is `SynthProcessor::process_block`
(`app/geist-daw/src/engine.rs`). Everything reachable from there is audio-thread
code: the command drain, `Arrangement`/`SessionClips` advance, track synth
nodes, `FxChain::process` (`app/geist-daw/src/fx.rs`), and every
`geist-synth`/`geist-fx`/`geist-modular`/`geist-dsp` process path. A prepared
VST process adapter joins this list when `geist-vst-host` instances reach the
graph.

## Forbidden on the audio thread

- Heap allocation — including `Vec` growth past capacity, `Box`/`Arc`
  construction, and format!/String work.
- Heap deallocation — dropping a `Box`, the last `Arc`, or any owning container.
  Dropping is as illegal as allocating; send owners back over a return ring.
- Blocking locks, file/network I/O, logging, panics crossing the boundary.
- Dynamic graph mutation, plugin scanning, project saving, UI calls.
- Unbounded loops; waiting on async work or other threads.

## Required patterns

- UI -> audio: `EngineCommand` (`Copy`) over the bounded rtrb ring in
  `app/geist-daw/src/control.rs`; saturated rings drop commands, never block.
- Audio -> UI: latest-value atomics (`LevelMeter`, `BeatClock`, session-scene
  bytes) plus the decimated scope ring; wait-free, lossy by design.
- Non-`Copy` payloads travel out-of-band: recorded buffers go UI -> audio as
  `AudioAsset { Arc<[f32]> }`; buffers the engine displaces bounce back over
  the asset-return ring and are dropped by `EngineControl::update_scope` on the
  UI thread.
- All engine containers are preallocated pools with `live`/cap flags:
  `Arrangement` clip pool, `SessionClips` slots, `FxChain` instance pools,
  `track_events` via `push_capped`. Add/remove flips flags; it never
  constructs or drops.
- `prepare()` sizes state on the app thread before the stream runs; `process()`
  only touches what `prepare()` built.
- Parameter changes mutate preallocated state in place (smoothing/clamping in
  DSP helpers). A change that requires rebuilding state (e.g. a convolution IR)
  must rebuild on the app thread and swap objects over a ring, returning the
  old object for off-thread drop.

## Enforced invariants and their tests

- Clip/note pools never grow or free: `engine::tests::arrangement_stays_within_capacity`,
  `engine::tests::processor_does_not_grow_under_command_traffic`.
- Displaced audio assets drop on the UI thread:
  `engine::tests::displaced_audio_asset_drops_on_the_ui_thread`.
- `SetBpm` replaces the origin tempo point without insertion:
  `tempo::tests::repeated_set_at_same_beat_does_not_accumulate` (capacity-pinned).
- FX pools process without growth: `fx` tests cover enabled-path processing;
  pools are built in `FxChain::new`/`prepare` only.

## Checklist for a new `EngineCommand`

1. The command is `Copy`; non-`Copy` payloads use a dedicated ring pair.
2. The handler only flips flags, writes into preallocated capacity, or mutates
   fixed-size state. No `new`, no `push` without a cap check, no `drop` of an
   owner (return it instead).
3. A test pins the no-growth/no-drop property where a helper can observe it.
4. If the handler cannot satisfy 2, the work belongs on the app thread with an
   object-swap ring — see the reverb-decay decision in `HANDOFF.md`.

## Allocator-level enforcement

Test builds of `geist-daw` install a counting global allocator
(`app/geist-daw/src/alloc_guard.rs`).
`engine::tests::audio_callback_is_allocation_free_in_steady_state` drives a
busy scene (placed clip, playing session slot, live notes, delay+reverb,
command churn) through `process_block` inside an
`alloc_guard::assert_no_alloc_scope` region and asserts zero allocator hits —
alloc, dealloc, and realloc all count. New engine work that touches the heap
in the callback fails this test.

## Known gaps

- The allocator guard covers the app's `process_block` path in tests; there is
  no runtime (debug-assert) guard inside the live cpal callback itself.
- Reverb decay live control is unimplemented pending Jeff's rebuild-vs-
  algorithmic decision (`HANDOFF.md`).
