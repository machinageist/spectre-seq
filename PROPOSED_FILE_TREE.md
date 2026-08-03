```
geistdaw/
├── Cargo.toml                          # workspace root; members = all crates
├── Cargo.lock
├── rust-toolchain.toml                 # pin nightly for std::simd
├── .cargo/
│   └── config.toml                     # target-specific linker flags, RUSTFLAGS
├── .github/
│   └── workflows/
│       ├── ci.yml                      # test + clippy + fmt on all platforms
│       ├── release.yml                 # build + package release artifacts
│       └── bench.yml                   # criterion benchmarks on push to main
├── assets/
│   └── workflows/
│       ├── default.toml                # built-in default workflow profile
│       ├── modular.toml                # graph-first modular sound-design workflow
│       ├── songwriting.toml            # arrangement-first writing workflow
│       ├── mixing.toml                 # meter/routing-focused mix workflow
│       └── performance.toml            # macro/live-performance workflow
├── xtask/                              # cargo xtask build automation
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── build_plugins.rs
│       ├── package_release.rs
│       └── run_benchmarks.rs
│
├── crates/
│   │
│   ├── spectre-core/                     # shared primitives; no dependencies on other crates
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── ids.rs                  # opaque newtypes; the durable family rejects zero
│   │       ├── time.rs                 # MusicalTime, TICKS_PER_QUARTER (960 PPQ)
│   │       ├── port.rs                 # PortType enum, PortDescriptor, PortDirection
│   │       ├── signal.rs               # Signal enum (Audio, CV, Gate, NoteEvent, Parameter)
│   │       ├── config.rs               # AudioConfig { sample_rate, block_size, channels }
│   │       ├── context.rs              # ProcessContext<'a>: buffers, events, transport snapshot
│   │       ├── events.rs               # NoteEvent, MidiEvent, TransportEvent
│   │       ├── params.rs               # ParamId, ParamInfo, ParamValue, ParamRange
│   │       ├── transport.rs            # AtomicTransport, TransportState, TempoMap
│   │       └── errors.rs               # GeistError, GeistResult
│   │
│   ├── geist-document/                 # canonical app-thread project truth; depends only on spectre-core
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── identity.rs             # IdSequence, IdentityAllocator; checked monotonic allocation
│   │       └── arrangement.rs          # Arrangement, ClipEntity, ArrangementTrack, ClipLocation
│   │
│   ├── spectre-graph/                    # audio process graph; depends on spectre-core
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── node.rs                 # AudioNode trait definition
│   │       ├── graph.rs                # Graph struct: nodes, edges, port registry
│   │       ├── edge.rs                 # Edge { src: PortId, dst: PortId }, type validation
│   │       ├── topology.rs             # topological sort, cycle detection, delay insertion
│   │       ├── process_list.rs         # compiled flat Vec<ProcessStep>; what audio thread runs
│   │       ├── swap.rs                 # rtrb SPSC Executor ownership handoff; see ADR 002
│   │       ├── channel.rs              # rtrb ring buffers for param changes + metering
│   │       ├── nodes/
│   │       │   ├── mod.rs
│   │       │   ├── delay_node.rs       # auto-inserted one-block delay for feedback loops
│   │       │   ├── passthrough.rs      # identity node; useful for testing + routing
│   │       │   ├── mixer.rs            # n-input summing node
│   │       │   └── monitor.rs          # metering node; writes peak/RMS to ring buffer
│   │       └── tests/
│   │           ├── topology_tests.rs
│   │           ├── cycle_tests.rs
│   │           └── routing_tests.rs
│   │
│   ├── geist-audio-backend/            # platform audio I/O abstraction
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── backend.rs              # AudioBackend trait
│   │       ├── device.rs               # AudioDevice, DeviceInfo
│   │       ├── stream.rs               # AudioStream, StreamConfig, XrunCounter
│   │       ├── cpal_backend.rs         # cpal wrapper (all platforms, default)
│   │       ├── pipewire_backend.rs     # direct PipeWire (Linux, feature-gated)
│   │       ├── jack_backend.rs         # JACK (Linux/macOS, feature-gated)
│   │       └── tests/
│   │           └── roundtrip_test.rs   # latency + xrun validation
│   │
│   ├── spectre-dsp/                      # pure DSP; no FFI, no I/O, no allocations in hot path
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── simd.rs                 # SIMD abstractions; feature-gated behind `simd` feature
│   │       ├── math.rs                 # fast_tanh, polyblep, lerp, db_to_linear, etc.
│   │       ├── osc/
│   │       │   ├── mod.rs
│   │       │   ├── polyblep.rs         # bandlimited saw, square, tri
│   │       │   ├── wavetable.rs        # wavetable engine with morph + linear interp
│   │       │   ├── sine.rs             # phase-accumulator sine
│   │       │   └── noise.rs            # white, pink, crackle
│   │       ├── filter/
│   │       │   ├── mod.rs
│   │       │   ├── svf.rs              # state-variable filter (LP/HP/BP/notch)
│   │       │   ├── ladder.rs           # Moog ladder approximation
│   │       │   ├── biquad.rs           # direct form II biquad (EQ building block)
│   │       │   └── comb.rs             # comb filter for flanging/chorus
│   │       ├── env/
│   │       │   ├── mod.rs
│   │       │   ├── adsr.rs             # ADSR with curve shapes
│   │       │   ├── ahdsr.rs            # AHDSR extended envelope
│   │       │   └── follower.rs         # envelope follower (peak + RMS)
│   │       ├── lfo/
│   │       │   ├── mod.rs
│   │       │   ├── lfo.rs              # free + tempo-synced LFO
│   │       │   └── stepseq.rs          # step sequencer LFO shape
│   │       ├── fx/
│   │       │   ├── mod.rs
│   │       │   ├── delay.rs            # stereo delay with feedback + filtering
│   │       │   ├── reverb.rs           # FFT convolution reverb (rustfft)
│   │       │   ├── chorus.rs           # stereo chorus/flanger
│   │       │   ├── saturator.rs        # waveshaper with multiple curves
│   │       │   └── eq.rs               # parametric EQ (chains biquads)
│   │       └── benches/
│   │           ├── osc_bench.rs
│   │           ├── filter_bench.rs
│   │           └── reverb_bench.rs
│   │
│   ├── geist-clap-host/                # CLAP plugin host
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── scanner.rs              # filesystem scan; populates plugin DB
│   │       ├── db.rs                   # sled/sqlite plugin metadata cache
│   │       ├── bundle.rs               # .clap bundle loading, entry point resolution
│   │       ├── instance.rs             # ClapInstance: init, activate, process, destroy
│   │       ├── plugin_node.rs          # ClapPluginNode implements AudioNode
│   │       ├── params.rs               # parameter discovery, get/set, flush
│   │       ├── gui.rs                  # raw-window-handle plugin GUI embedding
│   │       ├── state.rs                # save/restore plugin state (opaque bytes)
│   │       └── ffi/
│   │           ├── mod.rs
│   │           └── host_impl.rs        # clap_host_t vtable implementation
│   │
│   ├── spectre-lv2-host/                 # LV2 plugin host (lower priority)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── world.rs                # lilv world wrapper
│   │       ├── scanner.rs
│   │       ├── instance.rs
│   │       └── plugin_node.rs          # LV2PluginNode implements AudioNode
│   │
│   ├── geist-timeline/                 # legacy arena timeline; D8 deletes what geist-document replaces
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                  # prelude re-exports the relocated document/core types
│   │       ├── transport.rs            # play/pause/record/loop state machine
│   │       ├── tempo.rs                # TempoMap: BPM automation, time signature changes
│   │       ├── track.rs                # Track { id, clip_ids, armed, muted, soloed }
│   │       ├── clip.rs                 # Clip enum: AudioClip, MidiClip, AutomationClip
│   │       ├── arena.rs                # Arena<T> allocator for clips
│   │       ├── pattern.rs              # Pattern: note grid for piano roll / step seq
│   │       ├── playhead.rs             # sample-accurate playhead position
│   │       └── commands.rs             # command objects for undo/redo
│   │
│   ├── geist-automation/               # automation lanes + modulation matrix
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── lane.rs                 # AutomationLane: breakpoint curve over timeline
│   │       ├── curve.rs                # CurveSegment: linear, exponential, bezier
│   │       ├── matrix.rs               # ModMatrix: Vec<ModRoute>
│   │       ├── route.rs                # ModRoute { src: PortId, dst: PortId, depth, bipolar }
│   │       ├── evaluator.rs            # per-block curve evaluation + mod sum resolution
│   │       └── tests/
│   │           └── mod_sum_tests.rs
│   │
│   ├── spectre-project/                  # save/load; project format versioning
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── schema.rs               # ProjectFile struct; SCHEMA_VERSION: u32
│   │       ├── serialize.rs            # CBOR serialization (ciborium crate)
│   │       ├── migrate.rs              # version migration table
│   │       ├── asset_map.rs            # relative path + blake3 hash for audio files
│   │       ├── autosave.rs             # background autosave thread + crash recovery
│   │       └── settings.rs             # global settings (TOML via toml crate)
│   │
│   ├── spectre-config/                   # user/project config; workflow profiles; no audio callback work
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── schema.rs               # versioned WorkflowProfile and Settings structs
│   │       ├── loader.rs               # built-in/user/project config precedence
│   │       ├── validate.rs             # diagnostics + safe fallback rules
│   │       ├── keybindings.rs          # shortcut/controller binding schema
│   │       ├── commands.rs             # declarative command aliases to typed UICommand intents
│   │       └── templates.rs            # track/rack/graph/project template definitions
│   │
│   └── geist-ui/                       # UI layer; depends on all other crates
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── app.rs                  # top-level App struct; owns ProjectState Arc; loads workflow TOML
│           ├── state.rs                # UIState: selected object, active lens, workflow profile snapshot
│           ├── commands.rs             # UICommand enum; workflow aliases and file loads resolve to typed intents
│           ├── renderer.rs             # renderer trait + workflow-derived frame plan
│           ├── egui_renderer.rs        # egui adapter scaffold consuming UIState frame plan
│           ├── views/
│           │   ├── mod.rs              # workflow-derived WorkspaceSurface and action chips
│           │   ├── arrangement.rs      # arrangement lens surface model
│           │   ├── mixer.rs            # mix lens surface model
│           │   ├── node_graph.rs       # build lens surface model
│           │   ├── piano_roll.rs       # piano roll action helpers
│           │   ├── plugin_rack.rs      # shape lens surface model
│           │   ├── browser.rs          # browser lens surface model
│           │   └── modulation.rs       # modulation lens surface model
│           ├── widgets/
│           │   ├── mod.rs              # workflow-derived WorkspaceWidgets for tabs, panels, buttons
│           │   ├── knob.rs
│           │   ├── fader.rs
│           │   ├── meter.rs            # peak + RMS level meter
│           │   ├── cable.rs            # bezier cable renderer for node graph
│           │   ├── waveform.rs         # audio clip waveform display
│           │   └── piano.rs            # mini keyboard widget
│           └── assets/
│               ├── fonts/              # embedded via include_bytes!
│               └── icons/              # embedded SVG icons
│
├── plugins/
│   │
│   ├── geist-synth/                    # flagship wavetable/subtractive synth
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── engine/
│   │       │   ├── mod.rs
│   │       │   ├── voice.rs            # per-voice state: oscs, filters, envs
│   │       │   ├── voice_pool.rs       # polyphony manager + steal modes
│   │       │   ├── osc_stack.rs        # 2× wavetable oscs with unison/detune
│   │       │   ├── filter_stack.rs     # 2× SVF in series/parallel with FM routing
│   │       │   ├── mod_matrix.rs       # internal mod matrix (env/lfo → any param)
│   │       │   └── params.rs           # all parameter definitions + ranges
│   │       ├── daw_node.rs             # implements AudioNode for DAW-internal use
│   │       └── clap_plugin.rs          # implements CLAP ABI for standalone use
│   │
│   ├── geist-fx/                       # effects bundle
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── reverb/
│   │       │   ├── engine.rs
│   │       │   ├── daw_node.rs
│   │       │   └── clap_plugin.rs
│   │       ├── delay/
│   │       │   ├── engine.rs
│   │       │   ├── daw_node.rs
│   │       │   └── clap_plugin.rs
│   │       ├── chorus/
│   │       │   └── ...
│   │       ├── saturator/
│   │       │   └── ...
│   │       └── eq/
│   │           └── ...
│   │
│   └── geist-modular/                  # utility nodes: the routing glue
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── math.rs                 # Add, Multiply, Abs, Clip, Rescale nodes
│           ├── logic.rs                # AND, OR, NOT, comparator, flip-flop
│           ├── signal.rs               # Mux, Demux, Attenuverter, DC offset
│           ├── timing.rs               # Clock divider, gate delay, slew limiter
│           ├── sample_hold.rs          # Sample & Hold, Track & Hold
│           └── clap_plugins.rs         # registers every node as a CLAP plugin
│
├── app/
│   └── geist-daw/                      # main binary; wires all crates together
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── init.rs                 # startup: audio backend, graph, UI
│           ├── engine.rs               # top-level engine: owns graph + backend + timeline
│           └── ipc.rs                  # optional: OSC/socket control surface protocol
│
└── docs/
    ├── architecture.md                 # high-level system overview
    ├── ui_ux_principles.md             # product interaction principles and UI validation checklist
    ├── ui_interaction_model.md          # concrete UI lenses, selection behavior, and first prototype slice
    ├── ui_configuration_model.md        # configurable workflow profiles and safe UI config model
    ├── realtime_rules.md               # the audio thread contract; contributor law
    ├── plugin_sdk.md                   # how to write a native geist node
    ├── clap_hosting.md                 # CLAP host implementation notes
    └── adr/                            # architecture decision records
        ├── 001-clap-over-vst.md
        ├── 002-arcswap-graph-swap.md
        ├── 003-cbor-project-format.md
        └── 004-egui-first-wgpu-later.md
```
