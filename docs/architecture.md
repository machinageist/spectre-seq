<!--
Author: Jeff
Date: 2026-08-01
Description: System overview of the Geist DAW workspace as implemented — crates, dependency direction, and the app/audio thread split.
Notes: Describes what runs today. Planned structure is marked planned; where code contradicts an older plan, the code wins.
-->

# Geist DAW Architecture

## Scope

This document states the system as built. `PROPOSED_FILE_TREE.md` and
`INITIAL_PLAN.md` describe an intended shape the implementation has diverged from
in several places. Those divergences are called out here, and the implementation
is authoritative.

## Workspace layout

Root `Cargo.toml` declares `members = ["xtask", "crates/*", "plugins/*", "app/geist-daw"]`
— **17 workspace members**:

- **12 library crates** under `crates/`
- **3 first-party plugin crates** under `plugins/`
- **1 binary**, `app/geist-daw`
- **1 build tool**, `xtask` (an empty `fn main`; scaffold)

### `crates/`

| Crate | Role | State |
| --- | --- | --- |
| `spectre-core` | Shared primitives: `AudioConfig`, `ProcessContext`, IDs, ports, events, params, signal, transport, errors | Implemented; zero workspace dependencies |
| `geist-graph` | Graph model, topology/scheduling, plan compilation, block executor, executor swap, built-in nodes | Implemented; not on the app's audio path (see below) |
| `geist-dsp` | Oscillators, filters, envelopes, LFOs, fx primitives, FFT, math/SIMD helpers, benches | Implemented incrementally |
| `geist-audio-backend` | Backend abstraction, `BlockBridge`, cpal backend, capture ring, xrun counter | cpal implemented; JACK and PipeWire are scaffolds |
| `geist-timeline` | Transport, tempo map, playhead, arena clips/patterns, legacy `Timeline`, canonical `Arrangement`/`ClipEntity`, identity | Mixed; see "Competing arrangement authorities" |
| `geist-automation` | Curves, lanes, routes, modulation matrix, evaluator | Implemented; **no consumer in the app binary** |
| `geist-project` | On-disk project schema, CBOR serialize, TOML settings, blake3 asset map, migration, autosave | Implemented; **zero workspace dependencies** — a standalone serde DTO tree |
| `spectre-config` | Workflow profile schema, TOML loader, keybindings, command intents, validation | Implemented |
| `geist-ui` | UI state, typed selection, commands, egui renderer, views, widgets, theme | Implemented incrementally |
| `geist-vst-host` | VST3 host over raw `vst3` COM bindings; scanner, bundle, module, instance, `VstPluginNode` | Implemented, compile-checked; FFI crate, does not deny unsafe |
| `geist-clap-host` | CLAP host over `clap-sys`; scanner, bundle, cache db, params, state, gui, instance, `ClapPluginNode` | Substantially implemented; **not on the active plan** per ADR 001 |
| `geist-lv2-host` | LV2 host | Scanner only; `world.rs` and `instance.rs` are scaffolds |

### `plugins/`

| Crate | Role | State |
| --- | --- | --- |
| `geist-synth` | Flagship synth engine exposed as an `AudioNode` (`daw_node.rs`) | Implemented as a two-oscillator prototype; `clap_plugin.rs` is a scaffold |
| `geist-fx` | Delay, reverb, chorus, saturator, EQ graph nodes | Implemented |
| `geist-modular` | Utility node families: math, logic, signal, timing, sample/hold | Implemented; **no consumer in the app binary**; `clap_plugins.rs` deferred |

### `app/geist-daw`

| Module | Role |
| --- | --- |
| `main.rs` | Entry point; eframe window, or `--headless` demo |
| `init.rs` | Picks a device-compatible `AudioConfig` and starts the cpal output stream |
| `startup.rs` | Launch options and workflow-profile resolution (app-thread file I/O) |
| `engine.rs` | The running engine: sequencer, per-track arrangement, `SynthProcessor` block processor |
| `fx.rs` | Hard-wired per-track `FxChain` (delay then reverb) |
| `control.rs` | The app/audio boundary: `EngineCommand` ring, asset ring, scope ring, meters, beat clock |
| `studio.rs` | The real front-end: lens shell over `geist-ui`, diffing a `SessionModel` mirror into `EngineCommand`s |
| `gui.rs`, `graph_view.rs` | Minimal window and node-graph view |
| `session.rs`, `project.rs` | Mapping app state to and from `geist-project`'s `ProjectFile` |
| `recorder.rs` | App-thread drain of the capture ring; WAV write |
| `ipc.rs` | Scaffold (OSC/socket control surface) |

## Dependency direction

Actual `[dependencies]` edges between workspace members:

```
spectre-core        (no workspace deps)
spectre-config      (no workspace deps)
geist-dsp         (no workspace deps; rustfft)
geist-project     (no workspace deps; serde/ciborium/toml/blake3)
geist-lv2-host    (no workspace deps)
xtask             (no workspace deps)

geist-graph          -> spectre-core
geist-audio-backend  -> spectre-core
geist-timeline       -> spectre-core
geist-automation     -> spectre-core
geist-ui             -> spectre-config

geist-vst-host       -> spectre-core, geist-graph
geist-clap-host      -> spectre-core, geist-graph
geist-synth          -> spectre-core, geist-graph, geist-dsp
geist-fx             -> spectre-core, geist-graph, geist-dsp
geist-modular        -> spectre-core, geist-graph

app/geist-daw        -> spectre-core, geist-graph, geist-audio-backend,
                        geist-synth, geist-fx, geist-timeline,
                        geist-project, geist-ui, spectre-config
```

The direction is acyclic and `spectre-core` is the only shared root. Two edges are
worth naming because they are surprising:

- **`geist-project` depends on nothing in the workspace.** It is a parallel serde
  data model, not a projection of an in-memory one. Converting between it and live
  app state is `app/geist-daw/src/session.rs`'s job.
- **`geist-ui` depends only on `spectre-config`.** It does not see `spectre-core` or
  `geist-timeline`; it owns its own renderer-facing `SessionModel` and
  `TimelineModel` types. The app binary is the only place UI types and engine types
  meet.

Five workspace members build and test but contribute nothing to the running
binary: `geist-automation`, `geist-modular`, `geist-vst-host`, `geist-clap-host`,
`geist-lv2-host`.

### Planned: `geist-document`

`docs/changes/project-document/SPEC.md` introduces a new dependency-low crate
`crates/geist-document` to own the canonical app-thread `ProjectDocument`, with
`spectre-core` as its only dependency. **It is being implemented concurrently and is
not in the tree at the time of writing.** Treat every `geist-document` reference
in this document as in-progress, not landed.

## Thread model

Four kinds of thread exist at runtime.

| Thread | Owner | Work |
| --- | --- | --- |
| Main / app | eframe native event loop (`main.rs`) | UI, session state, project I/O, recorder drain, config, all mutation |
| Audio output | cpal (`crates/geist-audio-backend/src/cpal_backend.rs:180`) | `BlockBridge::render` -> `SynthProcessor::process_block` |
| Audio input | cpal (`crates/geist-audio-backend/src/cpal_backend.rs:237`) | Pushes interleaved capture frames into a lock-free ring |
| Autosave worker | `geist_project::autosave::Autosaver` | 60 s snapshot writes; never touches audio state |

The app thread owns mutation; the audio thread consumes bounded queues and
publishes atomics. The primitives carrying that boundary:

- `rtrb` SPSC rings — `EngineCommand` (256), `AudioAsset` (64), scope samples
  (8192), input capture (`crates/geist-audio-backend/src/stream.rs`), and the
  graph-swap pair in `crates/geist-graph/src/swap.rs`.
- Latest-value atomics — `LevelMeter` (`AtomicU32` bit-cast `f32`,
  `app/geist-daw/src/control.rs:114`), `BeatClock` (`AtomicU64` bit-cast `f64`,
  `app/geist-daw/src/control.rs:137`), `XrunCounter` (`AtomicU64`,
  `crates/geist-audio-backend/src/stream.rs:65`).

`docs/realtime_rules.md` is the contract for what may run on the audio callback,
including the places the current code breaks it.

### Block sizing

`BlockBridge` (`crates/geist-audio-backend/src/bridge.rs`) adapts the backend's
arbitrary interleaved callback size to a fixed channel-major block, carrying the
remainder across callbacks. cpal is opened with `BufferSize::Default` because not
every platform honors a fixed size. All bridge scratch is allocated once in
`BlockBridge::new` on the app thread.

### Clock domains

Input and output are **two independent cpal streams with no shared clock**. The
recorder drains the capture ring on the app thread at UI frame rate. There is no
drift correction, no resampling between the domains, and no latency compensation.
A unified clocking or explicit drift-correction contract is roadmap Milestone 8.

## The audio path as built

```
cpal output callback
  |- BlockBridge::render                     re-blocks to a fixed channel-major block
      |- SynthProcessor::process_block        app/geist-daw/src/engine.rs:575
          |- drain the asset ring
          |- drain the EngineCommand ring     notes, transport, macros, clips
          |- per track (fixed 3):
          |    |- Sequencer::advance_to_beat  step grid -> NoteEvent
          |    |- Arrangement::advance        placed MIDI clips -> NoteEvent
          |    |- SynthNode::process          geist-synth
          |    |- Arrangement::mix_audio      placed audio clips from the asset store
          |    \- FxChain::process            app/geist-daw/src/fx.rs
          |- sum tracks, apply master gain
          \- publish meter, beat clock, scope samples
```

Everything in that path is hand-wired. There is no compiled graph in it.

## What is not wired up

These are load-bearing gaps, not omissions from this document.

1. **The compiled graph is not on the audio path.** `app/geist-daw/src/engine.rs`
   imports exactly one item from `geist-graph` — `geist_graph::node::AudioNode`
   (line 18). It never constructs a `Graph`, never calls `compile`, never holds an
   `Executor`, and never uses `graph_swap`. `geist-graph`'s topology, compilation,
   executor, and swap are exercised only by that crate's own tests and its bench
   (`crates/geist-graph/benches/graph_bench.rs`). The running DAW and the graph
   engine are, today, two separate systems.
2. **The engine is a fixed three-track path.** `NUM_TRACKS: usize = 3`
   (`app/geist-daw/src/engine.rs:70`) with a fixed `TRACK_BASE_MIDI` array. Tracks
   cannot be added or removed at runtime, and each track's device chain is the
   hard-coded `FxChain`, not a graph.
3. **Typed ports are validation metadata only.** `spectre_core::port::can_connect`
   (`crates/spectre-core/src/port.rs:60`) rejects direction, type, and channel-count
   mismatches at connect time. But the executor routes every edge through one flat
   `Vec<f32>` pool, and notes and parameter changes are **global slices handed to
   every node** (`crates/geist-graph/src/process_list.rs:205-214`). There is no
   per-port event routing, no control-rate buffer, no polyphonic lane, and no rate
   conversion. Typed multi-rate routing is roadmap Milestone 3.
4. **Automation is not connected.** `geist-automation` implements curves, lanes,
   and an evaluator, and nothing depends on it.
5. **No plugin host is instantiated.** ADR 001 makes VST3 the active target;
   `geist-vst-host` is implemented and compile-checked, but the binary never loads
   it. `geist-clap-host` is substantially implemented (bundle, cache db, params,
   state, gui, instance, `ClapPluginNode`, FFI layer) despite ADR 001 describing it
   as a shelved scaffold — "shelved" is accurate about the *plan*, not about the
   code. `geist-lv2-host` really is scanner-plus-scaffolds. `docs/clap_hosting.md`
   and `docs/plugin_sdk.md` remain scaffolds on purpose: documenting the CLAP host
   as authoritative would contradict ADR 001.
6. **Only cpal exists as a backend.** `jack_backend.rs` and `pipewire_backend.rs`
   are pseudocode scaffolds. Output is f32-only; input runs at the device's native
   format.
7. **`xtask` is empty** — an empty `fn main`. There is no packaging, release, or
   CI-driver tooling behind it.
8. **`app/geist-daw/src/ipc.rs` is a scaffold.** No control-surface protocol.

## Competing arrangement authorities

Five types currently hold overlapping durable arrangement or project truth. This is
the central problem `docs/changes/project-document/SPEC.md` exists to fix.

| Owner | Path | Consumers today |
| --- | --- | --- |
| `geist_timeline::Timeline` | `crates/geist-timeline/src/track.rs:91` | None outside its own crate; legacy arena handles, sample placement |
| `geist_document::Arrangement` | `crates/geist-document/src/arrangement.rs` | None yet; canonical `ClipEntity` model, unwired. Relocated out of `geist-timeline` by slice D1 and still re-exported from `geist_timeline::prelude` for the compatibility window |
| `app::engine::Arrangement` | `app/geist-daw/src/engine.rs:259` | The audio thread; the only one that makes sound |
| `geist_ui::model::TimelineModel` | `crates/geist-ui/src/model.rs:295` | Passed `&mut` into `views::arrangement`, so the view mutates it directly |
| `app::session::StudioSession` | `app/geist-daw/src/session.rs` | The de-facto persistence model, in float beats |

The app binary uses exactly one item from `geist-timeline`: `Transport`
(`app/geist-daw/src/engine.rs:20`). Neither canonical arrangement model is
reachable from the running DAW.

Resolution is roadmap Milestone 2: `geist_document::arrangement::Arrangement`
becomes the single authority, the other four become projections or are deleted, and
each is deleted only when its four criteria in the SPEC hold.

## Where documentation and code disagree

Recorded so a future session does not re-derive them:

- `INITIAL_PLAN.md:29` and `PROPOSED_FILE_TREE.md:53` specify `ArcSwap` for graph
  publication. The implementation is an rtrb SPSC ownership handoff, and
  `arc-swap` is not a dependency. See ADR 002 for why.
- `INITIAL_PLAN.md:128` leaves the project encoding open between CBOR and
  MessagePack; the filename of ADR 003 asserts CBOR. The accepted project-format
  decision settles the package shape but **not** the encoding. See ADR 003.
- ADR 001 calls `geist-clap-host` a shelved scaffold. The crate is substantially
  implemented. The plan decision stands; the description of the code does not.
- `PROPOSED_FILE_TREE.md` predates the `geist-document` decision and the vocabulary
  relocation into `spectre-core`. Per the roadmap it is revised only after authority
  boundaries are accepted, so expect it to lag.

## Accepted changes the code has not caught up to

Recorded 2026-08-03, when `docs/changes/typed-realtime-graph/SPEC.md` and
`docs/changes/canonical-clip-content/SPEC.md` were accepted. Each of these is a
decision the code contradicts today. None is a bug report; they are scheduled work.

- **`PortType::Meter` leaves the connectable set.** `crates/spectre-core/src/port.rs:19`
  declares it as a routable variant. Decision 4 makes meters node-declared outlets
  backed by atomic cells, never edges. `SignalDomain` will have six variants.
- **`SignalRate` finally gains a consumer.** `crates/spectre-core/src/signal.rs:23` has
  had no reader outside its own tests. Decision 1 makes rate a declared per-port
  property that the compiler validates and the executor acts on.
- **The silent one-block cycle conversion becomes the explicit SCC path.**
  `crates/geist-graph/src/process_list.rs:54` discards `.feedback`;
  `crates/geist-graph/src/topology.rs:93` never fails and `:136` skips back-edges.
  Slice T6 turns that into a rejection and gives `topological_order`
  (`topology.rs:18`) its first real caller; slice T6a adds the sub-block scheduler.
  See ADR 006.
- **Device instances leave the executor.** `Executor::new`
  (`crates/geist-graph/src/process_list.rs:150`) moves node instances out of the
  graph, which is why a graph edit resets all DSP state. Decision 8 moves them to an
  audio-thread `DeviceTable`. See ADR 005.
- **`ClipEntity.duration` becomes `ClipExtent`.** `crates/geist-document/src/arrangement.rs`
  types it as `MusicalTime`. Decision 4a makes it `Musical | Source` so unwarped audio
  keeps a sample-domain length and a tempo edit mutates no clip record.
- **`rehome_clip`, `ClipLocation`, and `RemovedClip` change shape.** Decision 6a
  forbids overlap on an arrangement lane and derives order from start, which removes
  the explicit index parameter those three carry.
- **Clip content moves to its own aggregate.** `arrangement` is re-scoped to placement;
  a new `clips` aggregate owns records. Decision 1 amends the accepted aggregate table
  in `docs/changes/project-document/SPEC.md`.

## Validation

- `cargo test --workspace` is the workspace gate.
- `cargo check -p <crate>` is the minimum per-slice gate.
- `crates/geist-graph/benches/graph_bench.rs` targets sub-millisecond compile and
  swap for a 128-node chain.
- `crates/geist-dsp/src/benches/` holds the DSP primitive benches.
- Reproducible realtime performance fixtures around the 48 kHz / 128-frame baseline
  and the 64-frame stress mode are **planned** (roadmap Milestone 3), not
  established.
