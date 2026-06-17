## Phase 0 — Foundations (Weeks 1–2)

- Create cargo workspace; establish `geist-core` crate first, nothing depends on nothing
- Define canonical types: `NodeId(u64)`, `PortId(u64)`, `PortType` enum, `AudioConfig`, `ProcessContext<'a>`
- `ProcessContext` holds: sample rate, block size, input/output buffer slices, event queue slice — all borrowed, zero allocation
- Enforce real-time constraints via type system: `ProcessContext` is `!Send`; audio callback is `extern "C"`-compatible
- Define `AudioNode` trait: `fn process(&mut self, ctx: &mut ProcessContext)`
- Pin crate-level lints: `#![deny(unsafe_code)]` in safe crates, explicit `#![allow(unsafe_code)]` only in FFI crates
- Set up CI (GitHub Actions): `cargo test`, `cargo clippy`, `cargo build --release` for linux/macos/windows

---

## Phase 1 — Audio Graph Engine (Weeks 3–6)

- Implement `geist-graph`: directed graph of `AudioNode` instances connected by typed port edges
- Topological sort on every patch change; result is a flat `Vec<ProcessStep>` — this is the only representation the audio thread sees
- Feedback loops get automatic one-block delay; detect cycles, insert `DelayNode` transparently
- All graph mutations happen on the app thread; result is atomically swapped via `Arc<ArcSwap<ProcessGraph>>` — audio thread never locks
- Communication: `rtrb` ring buffer for UI→audio parameter changes; `crossbeam-channel` bounded for audio→UI metering
- Port types are enforced at connection time; mismatched types are a compile-time error where possible, runtime `Result` otherwise
- Write unit tests for: cycle detection, topological order, buffer routing correctness, one-block delay insertion
- Benchmark: graph with 128 nodes must sort and swap in under 1ms on the app thread

---

## Phase 2 — Audio Backend (Weeks 5–7, overlaps Phase 1)

- `geist-audio-backend`: one trait, multiple platform impls hiding behind it
- Wrap `cpal` (Apache 2.0) initially; the trait boundary means it can be replaced per-platform later
- Platform priority: PipeWire (Linux) → JACK → ALSA; CoreAudio (macOS); WASAPI (Windows)
- `AudioBackend::start()` takes ownership of the compiled `ProcessGraph` and drives it from the audio callback — no DAW logic on this thread
- Expose: sample rate negotiation, buffer size, device enumeration, xrun detection and reporting
- Validate: round-trip latency test, xrun counter under load, no allocations in callback (verify with `assert_no_alloc` crate in debug builds)

---

## Phase 3 — DSP Primitives (Weeks 6–10)

- `geist-dsp`: pure Rust, no FFI, no I/O — just math operating on `&mut [f32]` slices
- Implement per module, each independently testable and benchmarkable:
  - **Oscillators**: bandlimited saw/square/tri via PolyBLEP; wavetable engine with linear interp
  - **Filters**: state-variable filter (SVF) canonical implementation; Moog ladder approximation
  - **Envelopes**: ADSR/AHDSR with configurable curves (linear, exponential, custom LUT)
  - **LFO**: free/synced, phase-accumulator design, all waveforms
  - **Effects**: convolution reverb (FFT-based, `rustfft`), stereo delay with feedback, waveshaper/saturator
- All DSP structs are `#[repr(C)]` — prepares them for SIMD and future CLAP FFI reuse
- Add SIMD acceleration via `std::simd` (nightly) or `packed_simd2` for hot paths; gate behind a feature flag
- Criterion benchmarks for every hot path: oscillator, filter, convolution

---

## Phase 4 — CLAP Host (Weeks 9–13)

- `geist-clap-host`: wraps `clap-sys` raw FFI in a safe Rust API
- Scanner: walk `~/.clap`, `/usr/lib/clap`, platform-standard paths; load `.clap` bundles; cache metadata in a `sled` or `sqlite` DB
- Each loaded plugin instance is a `ClapPluginNode` implementing `AudioNode` — slots into the process graph transparently
- Handle: audio i/o, parameter get/set, parameter flush, note events, MIDI, save/restore state (blob of bytes)
- GUI: plugins that provide their own GUI get a raw window handle via `raw-window-handle` crate; your UI provides the parent
- Isolation: each plugin runs in its own thread during `init`/`destroy`; audio processing is inline on audio thread (CLAP contract)
- LV2 host in `geist-lv2-host`: lower priority, same `AudioNode` interface; use `lv2` crate (ISC licensed)

---

## Phase 5 — Plugin Suite (Weeks 10–18, parallel track)

- Each plugin in `plugins/` has three layers: `engine/` (pure DSP), `daw_node.rs` (implements `AudioNode`), `clap_plugin.rs` (implements CLAP ABI)
- Same `engine/` compiles into both; zero duplication
- Build `geist-synth` first — it's the flagship and exercises the most DSP infrastructure:
  - 2× wavetable oscillators with morph, unison, detune
  - 2× SVF filters in series/parallel with FM routing
  - Modulation matrix: any envelope/LFO output → any parameter; implemented as a `Vec<ModRoute>` resolved per block
  - Polyphonic with per-voice state; voice allocation with steal modes
- Build `geist-fx` second: reverb, delay, chorus, saturator, EQ — each as a standalone CLAP
- Build `geist-modular`: utility nodes (math ops, signal mux, attenuverter, slew limiter, logic gates, sample-and-hold) — these are the glue for "any signal to any input"
- CLAP binaries built with `cargo build --release --features clap-plugin`; DAW-internal nodes built without that feature

---

## Phase 6 — Timeline Engine (Weeks 14–20)

- `geist-timeline`: transport state machine, clip model, pattern sequencer
- Transport: BPM (with automation), time signature, play/pause/record/loop — state is an `Arc<AtomicTransport>` readable from audio thread
- Clip types: audio clip (sample playback via `rubato` for pitch/time), MIDI clip (note event sequence), automation clip (breakpoint envelope)
- Pattern sequencer: step sequencer + piano roll data model; emits `NoteEvent` stream into the graph
- All clips live in an `Arena<Clip>`; the timeline holds only IDs and positions — separation of data from layout
- Undo/redo: command pattern, `Vec<Box<dyn Command>>`; every mutation is a reversible command object

---

## Phase 7 — Automation & Modulation (Weeks 16–22)

- `geist-automation`: unifies automation lanes and real-time modulation into one system
- Automation lane: breakpoint curve over timeline position; evaluated per-block, outputs `f32` into parameter target
- Modulation source: any `PortType::CV` output in the graph can target any `PortType::Parameter` input — this IS the modular routing feature
- Modulation matrix stored as `Vec<ModRoute { src: PortId, dst: PortId, depth: f32, bipolar: bool }>`
- All parameter values are: `base_value + sum(active_mod_routes × depth)`, clamped per parameter spec
- Modulation is per-block (not per-sample) for non-audio signals; audio-rate modulation is just a CV audio port connection — the graph handles it uniformly

---

## Phase 8 — UI (Weeks 20–32)

- Start with `egui` (MIT) + `eframe` for rapid prototype; design UI behind a trait so the renderer is swappable
- Implement views in priority order: mixer → node graph → piano roll → arrangement → plugin rack
- Node graph view is the signature UI: render graph with bezier cables, port type-colored, drag-to-connect, rubber-band select
- All UI state is derived from app state; UI never owns ground truth — it reads from `Arc<RwLock<ProjectState>>` and sends commands
- Parameter controls (knobs, faders) send `ParameterChange` commands; audio thread reads via `rtrb` with no locking
- Metering: audio thread writes peak/RMS into lock-free ring buffer; UI reads and draws at 60fps independently
- Font/icon assets are embedded at compile time via `include_bytes!` — zero runtime asset loading
- Long-term: replace `egui` with custom `wgpu` renderer; the trait boundary makes this non-breaking

---

## Phase 9 — Project Format & Persistence (Weeks 24–28)

- `geist-project`: serialization of the entire DAW state to disk
- Format: `CBOR` (compact binary) or `MessagePack` for the main project file; human-readable `TOML` for settings
- Project file contains: graph topology, all parameter values, all clip data, plugin state blobs (opaque bytes from CLAP), automation curves
- Audio files are referenced by relative path + content hash (blake3); never embedded — project portability is the user's responsibility
- Versioning: project format has a `u32` schema version; forward-compatibility via unknown-field skipping; breaking changes bump major version
- Autosave: write to a `.geist-autosave` temp file on a background thread every 60s; crash recovery on next launch

---

## Phase 10 — Polish, Distribution, Community (Weeks 30+)

- `cargo xtask` for all build tasks: build plugins, run benchmarks, package releases, generate CLAP metadata
- Cross-compilation: `cross` crate for Linux→Windows/macOS builds in CI; test on all three in CI matrix
- Plugin distribution: individual CLAP binaries as GitHub Release artifacts; bundle installer script (shell + PowerShell)
- Documentation: `mdbook` for user docs; `rustdoc` for API; architecture decision records (ADRs) in `docs/adr/`
- Community: public RFC process for breaking API changes; plugin SDK published as its own crate so third parties can write native nodes
