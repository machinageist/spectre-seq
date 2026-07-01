<!--
Author: Jeff
Date: 2026-06-30
Description: Intended Rust workspace tree for the native-device/VST-host DAW architecture
Notes: Native devices live under crates/ and are not plugin binaries
-->

# Proposed File Tree

```text
geist-daw/
├── Cargo.toml                          # workspace root; crates/* plus app, with CLAP/LV2 excluded
├── Cargo.lock
├── CLAUDE.md
├── INITIAL_PLAN.md
├── PROPOSED_FILE_TREE.md
├── HANDOFF.md                          # current cross-session state and validation trail
│
├── assets/
│   ├── themes/                         # calm/minimal UI themes
│   ├── presets/                        # native device presets, not plugin presets
│   └── wavetables/                     # native wavetable assets
│
├── xtask/                              # build/test/package automation
│   ├── Cargo.toml
│   └── src/main.rs
│
├── app/
│   └── geist-daw/                      # native app shell and controller/runtime wiring
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── init.rs
│           ├── startup.rs
│           ├── engine.rs               # current app-level realtime engine; extract only if needed
│           ├── control.rs              # bounded UI/controller -> audio commands
│           ├── fx.rs                   # current internal device chain runtime
│           ├── session.rs              # session/project-facing app state
│           ├── project.rs              # app save/load bridge
│           ├── studio.rs               # UI/controller state bridge
│           ├── gui.rs
│           ├── graph_view.rs
│           ├── recorder.rs
│           └── history.rs
│
├── crates/
│   ├── geist-core/                     # dependency-light shared primitives
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── ids.rs                  # NodeId, PortId, TrackId, ClipId, ParamId, ProjectId, DeviceId
│   │       ├── config.rs               # AudioConfig and callback block shape
│   │       ├── context.rs              # borrowed ProcessContext; no callback allocation
│   │       ├── errors.rs               # GeistError/GeistResult
│   │       ├── events.rs               # MIDI/note/parameter/transport events
│   │       ├── params.rs               # parameter descriptors and ranges
│   │       ├── port.rs                 # typed ports and compatibility checks
│   │       ├── signal.rs               # signal-rate classification
│   │       ├── devices.rs              # internal/hosted device descriptors and state envelopes
│   │       ├── time.rs                 # SampleTime, BeatTime, Seconds, PpqTick, BarBeat
│   │       └── transport.rs            # atomic transport snapshots and beat/sample conversion
│   │
│   ├── geist-audio-backend/            # platform audio I/O behind an app-owned trait
│   │   └── src/
│   │       ├── backend.rs
│   │       ├── bridge.rs
│   │       ├── cpal_backend.rs
│   │       ├── device.rs
│   │       ├── stream.rs
│   │       └── lib.rs
│   │
│   ├── geist-graph/                    # editable graph -> immutable process plan
│   │   └── src/
│   │       ├── graph.rs
│   │       ├── edge.rs
│   │       ├── node.rs                 # AudioNode process trait plus internal AudioDevice surface
│   │       ├── topology.rs
│   │       ├── process_list.rs
│   │       ├── swap.rs
│   │       ├── channel.rs
│   │       └── nodes/
│   │
│   ├── geist-dsp/                      # pure native Rust DSP primitives
│   │   └── src/
│   │       ├── math.rs
│   │       ├── rng.rs
│   │       ├── simd.rs
│   │       ├── osc/                    # phasor, sine, wavetable, PolyBLEP, noise
│   │       ├── env/                    # ADSR/AHDSR/follower
│   │       ├── filter/                 # SVF, ladder, biquad, comb
│   │       ├── lfo/
│   │       └── fx/                     # native DSP algorithms
│   │
│   ├── geist-timeline/                 # transport, tempo map, tracks, clips, scheduling
│   ├── geist-automation/               # automation lanes and modulation matrix
│   ├── geist-project/                  # versioned project schema, serialization, assets, autosave
│   ├── geist-config/                   # workflow profiles, templates, keybindings
│   ├── geist-ui/                       # command/snapshot UI; not audio-thread state
│   │
│   ├── geist-synth/                    # flagship internal synth device; not a plugin
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── daw_node.rs             # internal graph wrapper
│   │       └── engine/                 # voice, voice pool, osc stack, filter stack, params, mod matrix
│   │
│   ├── geist-fx/                       # native internal effects devices; not plugins
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── io.rs
│   │       ├── delay/
│   │       ├── reverb/
│   │       ├── chorus/
│   │       ├── saturator/
│   │       └── eq/
│   │
│   ├── geist-modular/                  # native internal modular utility devices; not plugins
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── math.rs
│   │       ├── logic.rs
│   │       ├── signal.rs
│   │       ├── timing.rs
│   │       ├── sample_hold.rs
│   │       └── util.rs
│   │
│   ├── geist-vst-host/                 # the only active external plugin host boundary
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── scanner.rs              # VST3 discovery; never on audio thread
│   │       ├── bundle.rs               # .vst3 bundle layout
│   │       ├── module.rs               # dynamic library loading boundary
│   │       ├── descriptor.rs           # class/factory info
│   │       ├── host_app.rs             # host identity/callback shell
│   │       ├── instance.rs             # lifecycle/state shell
│   │       └── plugin_node.rs          # VST wrapper as internal graph node
│   │
│   ├── geist-clap-host/                # shelved historical crate; excluded from workspace
│   └── geist-lv2-host/                 # shelved historical crate; excluded from workspace
│
├── docs/
│   ├── architecture.md
│   ├── architecture/
│   │   └── native-vst-internal-devices.md
│   ├── realtime_rules.md
│   ├── vst_hosting.md                  # VST-only host boundary notes
│   ├── plugin_sdk.md                   # historical title; describes native node/device development
│   ├── ui_ux_principles.md
│   ├── ui_interaction_model.md
│   ├── ui_configuration_model.md
│   └── adr/
│       ├── 001-clap-over-vst.md        # VST3-only host decision
│       ├── 002-arcswap-graph-swap.md
│       ├── 003-cbor-project-format.md
│       └── 004-egui-first-wgpu-later.md
│
└── tests/
    ├── integration/
    └── audio_golden/
```

## Tree invariants

- No first-party code under `crates/geist-synth`, `crates/geist-fx`, or `crates/geist-modular` exports VST, CLAP, AU, LV2, or standalone plugin binaries.
- `plugins/` is not an active workspace member. It should stay empty or be removed after `.DS_Store` cleanup.
- External plugin support is VST3-only and isolated to `crates/geist-vst-host`.
- CLAP/LV2 crates are historical, excluded from the workspace, and should not receive new feature work.
- UI code emits commands and renders snapshots; it does not own audio-thread state.
