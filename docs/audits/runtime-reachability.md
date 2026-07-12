<!--
Author: Jeff
Date: 2026-07-11
Description: Source-traced reachability and maturity audit of the active Geist DAW runtime
Notes: Classifications describe the highest evidenced state, not the ambition or file-header status claim
-->

# Runtime Reachability

- **Status:** draft
- **Last verified:** 2026-07-11
- **Scope:** active application path from startup through audio processing, UI control, persistence, and teardown
- **Decision authority:** Jeff
- **Upstream sources:** `repository-baseline.md`; active Rust source at HEAD plus the five preserved pre-existing modifications
- **Downstream dependents:** `architecture-drift.md`, `test-and-quality-gaps.md`, `reuse-disposition.md`, `../status/subsystems.toml`
- **Supersedes:** unqualified implementation claims in source headers and legacy handoffs
- **Superseded by:** none
- **Open decisions:** canonical graph/device runtime, MIDI backend, project authority, autosave integration, VST3 host strategy, recording durability
- **Known gaps:** no application launch, hardware QA, real VST3 fixture, callback stress test, or crash-recovery drill was performed during this audit

## Classification scale

The ledger uses only: `absent`, `documented only`, `scaffolded`, `unit-tested in isolation`, `integrated but not exercised end-to-end`, `end-to-end exercised in a controlled fixture`, `manually QA'd on named platforms`, `stress-tested`, and `release-qualified`.

A subsystem receives the highest state supported by direct evidence. A source header saying “implemented” or “validated on real plugins” is not verification evidence by itself.

## Active startup path

1. `app/geist-daw/src/main.rs:43-65` parses workflow arguments, resolves UI state, and calls `init::start`.
2. `app/geist-daw/src/init.rs:33-40` opens only the default CPAL output device and derives one fixed 512-frame `AudioConfig`.
3. `init.rs:41-66` constructs a fixed vector of tracks, each owning a `SynthNode`, local modulation state, effects, sequencer, arrangement, and session state; it then creates `SynthProcessor`.
4. `init.rs:67-74` wraps that processor in `BlockBridge` and starts CPAL output.
5. `init.rs:76-84` attempts a separate best-effort input stream; failure is silently reduced to `None`.
6. The GUI path moves the engine/control/recorder into either `GeistApp` or `StudioApp`; the headless path loops forever and reports meters/xruns.

The application does not construct a `geist_graph::Graph`, compile a process list, create a graph executor, instantiate `geist_automation`, scan VST3 bundles, instantiate VST3 plugins, or open a MIDI backend during startup.

## Audio device and callback

### Device selection — integrated but not exercised end-to-end

- `init.rs:34-39` uses the default output only; there is no application selection UI or persisted selection in this path.
- `cpal_backend.rs:152-197` can open a named or default output, but `init::start` supplies no name.
- The stream requests `f32`, the selected channel count, selected sample rate, and CPAL default buffer size.
- Enumeration has a hardware-tolerant unit test, but startup was not launched during this audit.

### Callback entry — integrated but not exercised end-to-end

- CPAL invokes the closure at `crates/geist-audio-backend/src/cpal_backend.rs:179-189`.
- The closure calls `callback.render(&[], data, channel_count)`.
- `BlockBridge::render` at `bridge.rs:47-69` adapts arbitrary interleaved hardware callback lengths to fixed 512-frame channel-major blocks.
- Bridge tests exercise exact, partial, and larger-than-block callback sizes in controlled fixtures.

The output callback never receives captured input. Input is handled by an independent CPAL stream and drained later on the UI thread, so this is not a duplex render path.

## Live render path

### Fixed application engine — integrated but not exercised end-to-end

`SynthProcessor` is the live `BlockProcessor` (`engine.rs:835-894`). It owns:

- a fixed `Vec<Track>`;
- `geist_timeline::Transport`;
- preallocated per-track event vectors;
- one scratch block;
- a fixed-capacity audio-asset slot vector;
- the audio-thread side of UI command and asset rings.

During each block it:

- clears event vectors without dropping capacity;
- drains asset and command rings;
- mutates transport, track patches, patterns, clips, session slots, and mixer state directly;
- schedules local note events;
- calls native synth/effect objects directly;
- mixes into the master output and publishes meters/scope/clock state.

The steady-state allocation fixture is meaningful but does not prove every
failure path. In particular, `EngineSink::return_asset` can destroy an `Arc`
on the callback if its bounded return ring is saturated
(`app/geist-daw/src/control.rs:531-538`). Command and event saturation are
also handled by dropping work rather than surfacing a recoverable error.

Numerous unit tests invoke this processor directly. No launch or actual CPAL callback execution was recorded in the baseline, so the highest current evidence is integrated source plus controlled fixtures, not manual QA.

### Compiled graph — unit-tested in isolation

`geist-graph` supplies editable graph, topology, process-list execution, feedback handling, node lifecycle, swap, channels, and monitor nodes. Its tests execute compiled chains and swaps. However:

- `init.rs` never imports or constructs `Graph`, `ProcessList`, an executor, or `GraphSwap`;
- the active callback receives `SynthProcessor`, not a graph executor;
- application graph UI state therefore does not establish canonical audio routing.

The graph is not reachable from the active render path.

### Native devices and effects — integrated but not exercised end-to-end

The live `Track` directly owns `geist_synth::SynthNode`, app-local `FxChain`, LFO/modulation matrix, sequencer, arrangement, and session state (`engine.rs:738-772`). Native synth/effect unit tests and processor fixtures pass. This is integration into the fixed engine, not integration through the proposed first-party device/render-graph architecture.

`geist-stacksynth` and `geist-modular` are active workspace crates with substantial isolated tests, but the application manifest does not depend on either. They are not reachable from the live app.

## Time, sequencing, and events

### Transport — end-to-end exercised in a controlled fixture

The fixed processor owns `geist_timeline::Transport`, advances it per rendered block, applies play/pause/stop/tempo/loop commands, and publishes clock state. Processor and timeline tests exercise these paths. Hardware timing and discontinuity stress remain unverified.

### Arrangement/session/step sequencing — end-to-end exercised in a controlled fixture

The fixed engine contains separate app-local step, arrangement, and session structures and exercises command-to-audio behavior in processor tests. This proves controlled fixture behavior through `SynthProcessor`, not a general project/graph sequencing contract.

Scheduling is block-start accurate, not sample-accurate. Step events use offset
zero, quantized session transitions occur at block processing boundaries, and
the fixed processor passes an empty parameter-change slice to each synth
`ProcessContext` (`engine.rs:1368-1376`).

### External MIDI — absent

No active MIDI I/O crate or backend appears in Cargo metadata, and startup opens no MIDI device. UI keyboard and clip events become internal `NoteEvent` values, but that is not external MIDI ingestion, timestamping, MPE, sync, or controller support.

## Automation and modulation

### Automation crate — unit-tested in isolation

`geist-automation` has lane, curve, route, matrix, and evaluator tests. The `geist-daw` application does not depend on `geist-automation`, and no startup/render symbol instantiates it. Automation is not reachable from the live signal path.

### App-local modulation — end-to-end exercised in a controlled fixture

Each fixed track owns one LFO and `geist_synth::ModMatrix`. UI commands mutate an app-local patch, and processor tests verify selected destinations. This is a narrow synth modulation path, not the general automation/modulation architecture described in product documents.

## Recording and media

### Input capture — integrated but not exercised end-to-end

`cpal_backend.rs:207-255` opens a separate native-format input stream and pushes interleaved samples into a bounded ring. `init.rs:76-81` silently disables recording if input startup fails. No device selection, latency alignment, transport timestamp correction, or duplex synchronization is present in the traced path.

### Recording buffer and WAV write — integrated but not exercised end-to-end

`app/geist-daw/src/recorder.rs` drains captured data on the UI thread into a growing `Vec`, then writes a float WAV synchronously. Unit tests cover ring accumulation and WAV round trip. `studio.rs:1011-1015` writes a take after recording stops and reports failure as UI status.

The path lacks pre-created recording files, incremental disk writing, flush/dropout markers, disk-full tests, crash salvage, and timestamp/latency correction. Audio data exists only in memory until stop, so a crash during recording loses the take.

## VST3 hosting

### Host crate — unit-tested in isolation

`geist-vst-host` contains bundle discovery, dynamic module loading, factory/class descriptors, component/audio-processor lifecycle, and a graph `AudioNode` wrapper. The wrapper builds one input and one output bus and passes null parameter, event, and process-context pointers (`plugin_node.rs:102-117`). Tests cover scanning/path/descriptor/error helpers, not a redistributed real plugin fixture.

The application does not depend on `geist-vst-host`, does not scan or instantiate plugins, does not persist plugin state, and does not host plugin editors. Source comments claiming real-plugin validation have no linked fixture or recorded result in the repository. VST3 is therefore not reachable from the application and must not be described as supported.

## UI command and state flow

### UI shell — integrated but not exercised end-to-end

The GUI runs on the main thread through eframe. UI models send `EngineCommand` values through bounded control rings, while meters/scope/clock are reflected through atomics/rings. The Studio UI also keeps its own session/mixer/rack/timeline models and mirrors changes into engine commands.

This is an operational command path in source and controlled model tests, but the coexistence of UI session state, fixed-engine state, and serialized `StudioSession` means authority and reconciliation are not yet a defined single-source contract. No accessibility, focus, high-DPI, multi-window, plugin-editor, or keyboard-only manual protocol was executed.

## Project save, autosave, recovery, and reload

### Manual project save/load — integrated but not exercised end-to-end

`app/geist-daw/src/session.rs` converts a UI-owned `StudioSession` to/from `geist_project::ProjectFile`. `studio.rs:1785-1798` invokes synchronous save/load from UI actions. Round-trip tests cover a bounded demo schema.

The mapping encodes many application concepts as numeric parameters on a synthetic `geist-macros` graph node and uses reserved clip-ID ranges. This is not evidence of a durable canonical project model or compatibility policy.

### Atomic write/migration primitives — unit-tested in isolation

`geist-project` contains CBOR serialization, schema version checks, ordered migration helpers, content hashing, and sibling-temp-plus-rename writes. Tests cover happy-path round trips, garbage rejection, missing files, simple migration sequencing, and recovery-file discovery.

There is no evidence for fsync durability, directory sync, rollback after migration failure, unknown-field preservation, disk-full/permission/interrupted-write behavior, case-sensitive asset repair, or cross-version/cross-platform fixtures.

### Autosave/recovery application path — unit-tested in isolation

`geist_project::Autosaver` spawns and joins a worker and ignores write errors. No active application source instantiates `Autosaver` or calls `find_recovery_files`. Autosave and recovery are therefore library capabilities disconnected from the live app.

## Shutdown and destruction

### GUI shutdown — integrated but not exercised end-to-end

The engine owns CPAL stream objects. Returning from eframe drops the app, engine, and streams; CPAL stream RAII stops device activity. No explicit shutdown state machine, note flush, recording finalization, save prompt, worker coordination, callback quiescence proof, or teardown test is present.

### Headless shutdown — absent

`main.rs:96-113` loops forever and relies on external process interruption. It has no signal handling or orderly teardown path.

## Maturity summary

| Subsystem | Highest evidenced state | Reason |
|---|---|---|
| default output selection/start | integrated but not exercised end-to-end | live source path; no launch evidence |
| callback-size bridge | end-to-end exercised in a controlled fixture | bridge tests cover variable callback sizes |
| fixed `SynthProcessor` render | end-to-end exercised in a controlled fixture | command/scheduling/audio tests invoke active processor |
| compiled render graph | unit-tested in isolation | executor/swap tests; absent from startup |
| native synth/effects in fixed engine | end-to-end exercised in a controlled fixture | processor fixtures, not graph/device architecture |
| stacksynth/modular crates | unit-tested in isolation | not application dependencies |
| transport/sequencing | end-to-end exercised in a controlled fixture | processor and timeline fixtures; timing remains block-start accurate |
| external MIDI I/O | absent | no backend or startup path |
| general automation | unit-tested in isolation | app has no dependency |
| narrow synth modulation | end-to-end exercised in a controlled fixture | fixed-track processor tests |
| input capture/recording | integrated but not exercised end-to-end | source wiring and unit tests; no device run |
| durable recording/recovery | absent | memory-until-stop design; no salvage path |
| VST3 host crate | unit-tested in isolation | no app dependency or real fixture evidence |
| VST3 user workflow/editor/state | absent | no live path |
| UI command/reflection | integrated but not exercised end-to-end | source wiring and model tests |
| manual project save/load | integrated but not exercised end-to-end | UI invocation plus isolated round trips |
| autosave/recovery | unit-tested in isolation | not instantiated by app |
| GUI teardown | integrated but not exercised end-to-end | implicit RAII only |
| headless teardown | absent | infinite loop, no signal path |

No subsystem has evidence for manual QA on a named platform, stress testing, or release qualification in this audit.
