<!--
Author: Jeff
Date: 2026-09-07
Description: User-supplied final-product source for Spectre
Notes: Original pasted text preserved below; authority and interpretation are defined by product-contract.md
-->

# Final-product source

- **Status:** accepted (capability direction only)
- **Last verified:** 2026-09-07
- **Scope:** supplied final-product capabilities; not an implementation plan or first-release promise
- **Decision authority:** Jeff
- **Upstream sources:** user paste supplied during the 2026-09-07 architecture interview
- **Downstream dependents:** `product-contract.md`, `capability-map.md`
- **Supersedes:** nothing independently; amendments are specified in `product-contract.md`
- **Superseded by:** none
- **Open decisions:** optional examples, numerical targets, technical prescriptions and release allocation
- **Known gaps:** source uses reference-product names, aspirational absolutes, and mixed normative language

Jeff accepted this as final-product scope with phased delivery and separate review of technical
prescriptions. Preserve required capabilities; do not turn “eventually”, “where practical”,
examples, suggested layouts, or algorithm/API sketches into unconditional release gates.

## Original supplied text

Absolutely. I’d frame this as **“a DAW platform with a modular synthesis environment embedded inside it”**, rather than simply “Ableton + VCV Rack + Serum.” That distinction matters because an agent needs explicit subsystem boundaries, APIs, real-time constraints, and acceptance criteria.

 Below is a spec you could hand directly to a software-building agent.

 # All-in-One DAW — Master Product Specification

 ## 1\. Product vision

 Build a professional, cross-platform digital audio workstation combining:

 - **Ableton-style** clip launching, session workflow, arrangement, recording, editing, mixing, and automation.
- **Serum 2-style** modern wavetable synthesis, modulation, unison, filtering, envelopes, LFOs, effects, and visual feedback.
- **VCV Rack-style** modular synthesis, with freely routable modules and patch cables.
- A complete professional **mixer, effects, routing, mastering, MIDI, audio, and plugin environment**.

 The application should feel like **one coherent instrument/workstation**, not three applications bolted together.

 ### Target platforms

 - Windows
- macOS
- Linux
- x64
- ARM64 where practical

 ### Primary audio formats

 - WAV
- AIFF
- FLAC
- MP3 import
- 16/24/32-bit PCM
- 44.1–192 kHz
- mono/stereo
- multichannel internally where architecture permits

---

 # 2\. Core architecture

 The system should be built around a **real-time audio engine**, with every major subsystem communicating through well-defined APIs.

 ### Core subsystems

```
Application
│
├── Project System
├── UI Framework
│
├── Audio Engine
│   ├── Graph Engine
│   ├── Mixer
│   ├── Routing
│   ├── DSP Scheduler
│   ├── Automation
│   └── Latency Compensation
│
├── Timeline / Arrangement Engine
├── Session / Clip Launcher
├── MIDI Engine
├── Instrument Engine
│   ├── Wavetable Synth
│   ├── Modular Synth
│   ├── Sampler
│   └── External Plugins
│
├── FX Engine
├── Recording Engine
├── Editing Engine
├── File / Media Manager
├── Plugin Host
└── Preset / Library System
```

 The **audio graph** should be the underlying abstraction connecting tracks, instruments, effects, buses, sends, returns, modular patches, and master processing.

---

 # 3\. Audio engine

 This is the most important subsystem.

 ## Requirements

 - Real-time low-latency processing.
- Lock-free audio thread wherever possible.
- No memory allocation on the real-time audio thread.
- Deterministic DSP scheduling.
- Multithreaded processing.
- Automatic CPU-core distribution.
- SIMD optimization.
- Sample-accurate event scheduling.
- Sample-accurate automation.
- Variable audio buffer sizes.
- Offline rendering.
- Freeze/bounce functionality.
- Plugin delay compensation.
- Track latency compensation.
- Automatic graph dependency resolution.

 ### Target performance

 At 48 kHz:

 - 32-sample buffer: professional low-latency operation
- 64 samples: primary low-latency target
- 128/256/512/1024 samples: fully supported

 The engine should gracefully degrade rather than glitch when CPU load becomes excessive.

---

 # 4\. Project system

 A project contains:

```
Project
├── Tempo Map
├── Time Signature Map
├── Markers
├── Tracks
├── Groups
├── Buses
├── Returns
├── Master
├── Arrangement
├── Session
├── Automation
├── MIDI
├── Audio Files
├── Plugin State
├── Modular Patches
├── Presets
└── Project Metadata
```

 Projects must save and restore **exactly**, including:

 - plugin states
- synth patches
- modular patches
- automation
- routing
- clip positions
- warp settings
- mixer state
- sends
- sidechains
- MIDI mappings
- UI state where appropriate

 Autosave and crash recovery are mandatory.

---

 # 5\. Track system

 Support:

 - Audio tracks
- MIDI tracks
- Instrument tracks
- Hybrid audio/MIDI tracks
- Return tracks
- Group tracks
- Bus tracks
- Master track
- Modular tracks

 Each track has:

```
Input
→ Input Processing
→ Instrument
→ FX Chain
→ Sends
→ Output
```

 Features:

 - Arm
- Solo
- Mute
- Record monitoring
- Input selection
- Output selection
- Volume
- Pan
- Stereo width
- Phase invert
- Mono/stereo switching
- Automation
- Metering
- Color
- Naming
- Track icons
- Track grouping
- Track freezing
- Track bouncing

---

 # 6\. Arrangement view

 Full linear DAW timeline.

 ## Features

 - Unlimited tracks
- Audio clips
- MIDI clips
- Automation lanes
- Tempo automation
- Time signature changes
- Markers
- Locators
- Loop regions
- Punch in/out
- Comping
- Crossfades
- Slip editing
- Split
- Consolidate
- Duplicate
- Time stretch
- Time compression
- Ripple editing
- Multi-track editing
- Snap/grid
- Quantization
- Groove templates

 ### Editing

 Provide professional non-destructive editing:

 - Trim
- Crop
- Fade in/out
- Crossfade
- Reverse
- Normalize
- Gain
- Pitch shift
- Time stretch
- Warp
- Clip envelopes
- Clip automation

---

 # 7\. Clip launcher / Session view

 This should be a first-class subsystem rather than merely another timeline mode.

 Each track contains scenes of clips.

```
         Scene 1    Scene 2    Scene 3
Track 1   [Drums]    [Drums2]   [Break]
Track 2   [Bass]     [Bass2]    [Bass3]
Track 3   [Synth]    [Synth2]   [Synth3]
Track 4   [Vocal]    [---]      [Vocal2]
```

 ## Requirements

 - Launch clips
- Launch scenes
- Quantized launching
- Immediate launching
- Stop clips
- Stop tracks
- Follow actions
- Clip looping
- Clip legato
- Clip launch quantization
- Launch velocity
- MIDI triggering
- Keyboard triggering
- Hardware controller triggering

 ### Follow actions

 Support:

 - Next
- Previous
- Random
- Random other
- First
- Last
- Any
- Repeat
- Stop

 with probability and timing controls.

---

 # 8\. Audio recording

 Professional multitrack recording.

 Support:

 - Multiple simultaneous inputs
- Input monitoring
- Punch recording
- Loop recording
- Pre-roll
- Count-in
- Take lanes
- Comping
- Overdub
- Replace recording
- Destructive/non-destructive modes
- Automatic file naming
- Automatic file organization

 Recording must continue reliably under heavy CPU load.

---

 # 9\. MIDI engine

 Support:

 - MIDI input/output
- MIDI CC
- MIDI notes
- Pitch bend
- Aftertouch
- Polyphonic aftertouch
- MPE
- Program changes
- MIDI clock
- MIDI timecode where appropriate
- MIDI learn
- MIDI mapping
- MIDI recording
- MIDI quantization
- MIDI transformation

 ### MIDI editor

 Piano roll with:

 - Velocity
- Note length
- Probability
- Chance
- Groove
- Transpose
- Scale snapping
- Chord tools
- Arpeggiator
- Humanization
- Note repeat
- MPE editing

---

 # 10\. Wavetable synthesizer

 This should be a **serious flagship instrument**, not a toy synth.

 Think Serum-class architecture.

 ## Oscillators

 At minimum:

 ### Oscillator A

 - Wavetable selection
- Wavetable browser
- Wavetable position
- Unison
- Detune
- Stereo spread
- Phase
- Random phase
- Warp
- FM
- AM
- RM
- Sync

 ### Oscillator B

 Same capabilities.

 ### Sub oscillator

 - Sine
- Triangle
- Saw
- Square
- Custom waveform
- Octave selection
- Level
- Stereo mode

 ### Noise oscillator

 - Sample-based noise
- Procedural noise
- Custom noise samples
- Pitch tracking
- Key tracking
- One-shot mode
- Loop mode
- Level
- Filter

---

 # 11\. Wavetable engine

 Support:

 - Single-cycle waveforms
- Multi-frame wavetables
- 2D wavetable scanning
- 3D wavetable concepts where practical
- User-imported wavetables
- Wavetable generation
- Wavetable editing
- Crossfading
- Spectral interpolation
- Harmonic morphing

 ### Wavetable editor

 Allow users to:

 - Draw waveforms
- Add/remove frames
- Smooth
- Normalize
- Quantize
- Harmonic edit
- FFT/spectral edit
- Morph between frames
- Generate from audio
- Import WAV
- Export WAV

---

 # 12\. Wavetable synthesis modes

 Include:

 - Classic wavetable
- FM
- AM
- Ring modulation
- Hard sync
- Phase distortion
- Wavefolding
- Frequency warping
- Formant-style warping
- Spectral morphing
- Bend
- Mirror
- Quantize
- Harmonic stretch

 Each oscillator should expose modulation targets for these parameters.

---

 # 13\. Filters

 At minimum:

 - Low-pass 12/24 dB
- High-pass 12/24 dB
- Band-pass
- Notch
- Comb
- State-variable
- Ladder
- Diode
- MS20-style
- Formant
- Peak
- Multi-mode

 Filter parameters:

 - Cutoff
- Resonance
- Drive
- Key tracking
- Mix
- Slope
- Modulation

 Allow serial and parallel filter routing.

---

 # 14\. Synth modulation system

 This should be one of the strongest parts of the application.

 Sources:

 - LFO
- Envelope
- Velocity
- Note
- Key tracking
- Mod wheel
- Aftertouch
- MPE
- MIDI CC
- Random
- Noise
- Audio envelope follower
- Step sequencer
- Macro
- Other modulation sources

 Targets should be essentially any automatable synth parameter.

 Use a **modulation matrix plus direct visual modulation assignment**.

 Example:

```
LFO 1 → Filter Cutoff +42
Env 2 → Wavetable Position +65
Velocity → Amp +20
MPE Y → FM Amount +35
Random → Pan ±15
```

---

 # 15\. Envelopes

 Support:

 - ADSR
- DAHDSR
- Multi-stage envelopes
- Looping envelopes
- Curve control
- Tempo synchronization
- Retrigger
- Legato mode

---

 # 16\. LFOs

 Each LFO:

 - Free-running
- Retrigger
- Note-triggered
- Tempo sync
- Hz mode
- Multiple shapes
- Custom drawing
- Step mode
- Smoothing
- Phase
- Unipolar/bipolar

---

 # 17\. Polyphony

 Support:

 - Mono
- Poly
- Legato
- Portamento
- Glide
- Retrigger
- Voice stealing
- Voice priority
- Unison
- Per-voice modulation

 Target:

 **At least 64 voices per synth instance**, hardware permitting.

---

 # 18\. Modular synthesis environment

 This is the VCV-inspired portion.

 The modular environment should use the same audio engine as the DAW.

 ## Patchable modules

 ### Sources

 - VCO
- Wavetable oscillator
- FM oscillator
- Noise
- Sampler
- Granular oscillator
- Audio input
- MIDI input

 ### Control

 - ADSR
- Envelope
- LFO
- Random
- S&H
- Sequencer
- Clock
- Divider
- Quantizer
- MIDI/CV conversion

 ### Utilities

 - Mixer
- VCA
- Attenuator
- Mult
- Switch
- Crossfader
- Comparator
- Logic
- Slew limiter

 ### Filters

 - LP
- HP
- BP
- Notch
- Comb
- Ladder
- Formant

 ### Effects

 - Delay
- Reverb
- Chorus
- Flanger
- Phaser
- Distortion
- Bitcrusher
- Compressor
- EQ

 ### Routing

 Any compatible module output can connect to any compatible input.

---

 # 19\. Modular patch cables

 Patch cables should provide:

 - Visual connection
- Signal type indication
- Cable colors
- Cable hiding
- Cable rerouting
- Multiple connections where valid
- Input normalization
- Signal visualization
- Voltage/audio range handling

 Support:

 - Audio-rate signals
- Control-rate signals
- Gates
- Triggers
- MIDI-derived signals
- Modulation signals

---

 # 20\. Modular module SDK

 This is **extremely important** if this is being built by an agent.

 Create a public/internal module API allowing developers to create modules.

 Something conceptually like:

```
class Module {
public:
    virtual void prepare(const AudioSpec&);
    virtual void process(ProcessContext&);
    virtual void reset();

    InputPort* audioInput();
    OutputPort* audioOutput();

    Parameter* parameter(const char* id);
};
```

 The actual API can be different, but the architecture needs to be designed around extensibility from day one.

 Modules should be able to define:

 - Inputs
- Outputs
- Parameters
- GUI
- DSP
- Presets
- State serialization
- Automation
- MIDI behavior

---

 # 21\. Full effects suite

 Build a complete native effects library.

 ## EQ

 - 3-band EQ
- 4-band EQ
- Parametric EQ
- Dynamic EQ
- Linear-phase EQ
- Mid/Side EQ
- Spectrum analyzer

 ## Dynamics

 - Compressor
- Multiband compressor
- Limiter
- Brickwall limiter
- Gate
- Expander
- Transient shaper
- De-esser
- Dynamic EQ
- Sidechain compressor

 ## Saturation/distortion

 - Soft clip
- Hard clip
- Tape
- Tube
- Transistor
- Wavefolder
- Bitcrusher
- Sample-rate reducer
- Waveshaper

 ## Time effects

 - Delay
- Ping-pong delay
- Multi-tap delay
- Grain delay
- Tape delay
- Chorus
- Flanger
- Phaser

 ## Reverb

 At minimum:

 - Algorithmic reverb
- Plate
- Room
- Hall
- Chamber
- Spring

 Eventually:

 - Convolution reverb
- IR browser
- User IR import

 ## Modulation

 - Chorus
- Flanger
- Phaser
- Tremolo
- Auto-pan
- Ring modulator
- Frequency shifter

---

 # 22\. Creative effects

 Include:

 - Granulator
- Spectral delay
- Spectral freeze
- Pitch shifter
- Harmonizer
- Vocoder
- Resonator
- Looper
- Stutter
- Glitch
- Beat repeat
- Reverse
- Tape stop
- Vinyl simulation
- Frequency shifter

---

 # 23\. Mixer

 The mixer needs to be a professional console.

 Every channel:

```
Input
 ↓
Gain
 ↓
Gate
 ↓
EQ
 ↓
Compressor
 ↓
FX Inserts
 ↓
Sends
 ↓
Fader
 ↓
Pan
 ↓
Output
```

 But the routing must also allow arbitrary reordering.

 ## Mixer features

 - Unlimited channels
- Groups
- Buses
- Returns
- Inserts
- Sends
- Pre/post-fader sends
- Sidechain inputs
- Parallel processing
- External hardware I/O
- VCA-style control
- Link groups
- Channel linking
- Solo-safe
- Mute groups
- Metering
- Peak/RMS/LUFS

---

 # 24\. Mastering

 Master channel should support:

 - EQ
- Dynamic EQ
- Compressor
- Multiband compressor
- Saturation
- Stereo imaging
- Limiter
- Loudness meter
- Spectrum analyzer
- Correlation meter
- True peak detection

 Export:

 - WAV
- AIFF
- FLAC
- MP3
- Multiple sample rates
- Multiple bit depths
- Dither
- Normalization
- Loudness normalization

---

 # 25\. Automation

 Everything should be automatable.

 Support:

 - Track automation
- Clip automation
- Plugin parameters
- Synth parameters
- Modular parameters
- Mixer parameters
- Send levels
- Tempo
- Time signature

 Automation modes:

 - Read
- Write
- Touch
- Latch
- Trim

 Automation editing:

 - Bezier curves
- Linear
- Step
- Smoothing
- Quantization
- Copy/paste
- Scale
- Transform

---

 # 26\. Plugin support

 Support major plugin ecosystems appropriate to each platform.

 Priority:

 - VST3
- CLAP
- AU on macOS
- AAX only if licensing/business requirements make sense

 Plugin hosting needs:

 - Plugin scanning
- Plugin sandboxing where possible
- Plugin bypass
- Plugin latency reporting
- Plugin state saving
- Plugin presets
- Sidechains
- MIDI plugins
- Instrument plugins
- Crash isolation

---

 # 27\. Routing architecture

 This needs to be **more powerful than conventional DAWs**.

 Users should be able to route:

```
Track A
   ↓
Synth
   ↓
FX
   ↓
Bus
   ↓
Return
   ↓
Master
```

 but also:

```
Audio Input
 ↓
Modular Patch
 ├── Synth
 ├── Envelope follower
 ├── Filter
 └── FX
       ↓
      Mixer
```

 and:

```
Track A → Track B
Track B → Track C
Track C → Bus
```

 while preventing illegal feedback loops unless explicit feedback routing is implemented.

---

 # 28\. Sidechain system

 Every suitable processor should be able to accept an independent sidechain.

 Examples:

 - Kick → Bass compressor
- Vocal → Music ducking
- Audio → Envelope follower
- Modular audio → Synth modulation
- External input → FX

 Sidechain source should be selectable independently from the main signal path.

---

 # 29\. Warp/time-stretch engine

 Professional-quality time manipulation.

 Modes:

 - Beats
- Tones
- Texture
- Re-Pitch
- Complex
- Complex Pro
- Granular

 Features:

 - Warp markers
- Transient detection
- Tempo sync
- Pitch preservation
- Formant preservation
- Offline high-quality processing

---

 # 30\. Sampler

 A full sampler should be included.

 Features:

 - Sample start/end
- Looping
- Crossfade
- Reverse
- Pitch
- Time stretch
- Filter
- Amp envelope
- Filter envelope
- LFO
- Key mapping
- Velocity mapping
- Round robin
- Multi-sample instruments
- Velocity layers
- Release samples

---

 # 31\. Granular engine

 Include a dedicated granular instrument/effect.

 Parameters:

 - Grain size
- Density
- Position
- Position randomization
- Pitch
- Pitch randomization
- Spread
- Pan
- Envelope
- Direction
- Freeze
- Time stretch

---

 # 32\. Browser / library

 Unified content browser:

```
Instruments
├── Wavetables
├── Synth Presets
├── Modular Patches
├── Samples
├── Drums
├── One Shots
├── Loops
├── MIDI
├── FX Presets
└── Projects
```

 Features:

 - Search
- Tags
- Favorites
- Preview
- Collections
- User libraries
- Metadata
- Drag-and-drop
- Similar preset discovery

---

 # 33\. Macro system

 Create global macros that can control arbitrary parameters.

 Example:

```
MACRO 1 "MOVEMENT"

Filter Cutoff       +40%
Reverb Mix          +20%
LFO Rate             +15%
Wavetable Position   +60%
Delay Feedback       +10%
```

 Support:

 - 8–16 macros per device
- Macro ranges
- Inverted mappings
- Curves
- Min/max
- Multiple targets

---

 # 34\. UI architecture

 The UI should support multiple workflows.

 ### Main views

 - Arrangement
- Session
- Mixer
- Synth
- Modular
- Piano Roll
- Sample Editor
- Browser
- Device Chain

 Users should be able to:

 - Resize panels
- Detach windows
- Full-screen devices
- Zoom
- Hide panels
- Save layouts
- Customize shortcuts

---

 # 35\. Synth UI

 The wavetable synth should have a premium visual interface.

 Suggested layout:

```
┌──────────────────────────────────────────────┐
│ PRESET / BROWSER                             │
├───────────────┬───────────────┬──────────────┤
│ OSC A         │ FILTER        │ OSC B        │
│ Wavetable     │ Cutoff        │ Wavetable    │
│ WT Position   │ Resonance     │ WT Position  │
│ Warp          │ Drive         │ Warp         │
├───────────────┴───────────────┴──────────────┤
│              MODULATION MATRIX               │
├──────────────────────────────────────────────┤
│ ENV 1 │ ENV 2 │ LFO 1 │ LFO 2 │ MACROS       │
├──────────────────────────────────────────────┤
│                 FX SECTION                   │
└──────────────────────────────────────────────┘
```

 Visualizations:

 - Wavetable waveform
- Spectrum
- Filter response
- Envelope curves
- LFO curves
- Modulation indicators
- Voice visualization

---

 # 36\. Modular UI

 The modular view should behave like an actual virtual hardware rack.

 Features:

 - Infinite/large canvas
- Zoom
- Pan
- Module snapping
- Module resizing where supported
- Patch cables
- Cable highlighting
- Searchable module browser
- Module categories
- Favorites
- Presets
- Sub-patches
- Collapse/expand modules

---

 # 37\. Clip system

 Each clip should have:

 - Name
- Color
- Gain
- Pitch
- Warp
- Loop
- Start/end
- Fades
- Automation
- Launch settings
- Follow actions

 Audio clips also need:

 - Waveform rendering
- Transient markers
- Warp markers
- Spectral view optionally

 MIDI clips:

 - Notes
- Velocity
- CC
- MPE
- Probability
- Groove

---

 # 38\. Groove system

 Support:

 - Swing
- Groove templates
- MIDI groove extraction
- Audio groove extraction
- Timing percentage
- Velocity percentage
- Randomization

---

 # 39\. Performance / live mode

 The application should work as a live performance instrument.

 Requirements:

 - Extremely fast clip launching
- MIDI controller mapping
- Scene launching
- Tempo control
- Tap tempo
- Crossfader
- Performance macros
- Emergency audio bypass
- CPU monitoring
- Plugin bypass
- Setlist/project loading

---

 # 40\. Undo/redo

 Everything user-facing should support undo.

 This includes:

 - Audio edits
- MIDI edits
- Mixer changes
- Routing
- Synth changes
- Modular patching
- Automation
- Plugin state
- Project operations

 Use a robust command/history architecture rather than ad-hoc undo implementations.

---

 # 41\. Accessibility

 Support:

 - Keyboard navigation
- Scalable UI
- High contrast
- Screen-reader-compatible controls where practical
- Colorblind-safe meters/indicators
- Custom shortcuts

---

 # 42\. Crash recovery

 The DAW must be designed around failure.

 Implement:

 - Autosave
- Incremental project backups
- Crash recovery
- Audio file recovery
- Plugin crash isolation
- Safe mode
- Plugin blacklist
- Corrupt-project recovery
- Automatic diagnostic reports

---

 # 43\. Testing requirements

 The agent should not simply implement features—it needs to continuously test the DAW.

 ### Unit tests

 Every DSP component should have tests for:

 - NaN
- Infinity
- Denormals
- Silence
- Extreme gain
- Extreme resonance
- Sample-rate changes
- Buffer-size changes
- Reset
- State restoration

 ### Audio tests

 Use reference renders to verify:

 - Oscillators
- Filters
- Envelopes
- LFOs
- Effects
- Mixer
- Automation
- Time stretching
- Export

 ### Stress tests

 Examples:

 - 500 tracks
- 1000 clips
- 100 synth voices
- 100+ plugin instances
- Huge modular patches
- Maximum automation density
- Long recordings
- Large projects

---

 # 44\. Audio quality requirements

 DSP must prioritize:

 - No unexpected clipping
- Stable oscillators
- Proper anti-aliasing
- Oversampling where appropriate
- Denormal protection
- High-quality interpolation
- Accurate filters
- Stable feedback networks
- Proper dithering
- True-peak-aware processing

 Every nonlinear effect should have configurable oversampling where appropriate.

---

 # 45\. Preset/state architecture

 Every device should have a serializable state.

 Conceptually:

```
{
  "device": "wavetable_synth",
  "version": 4,
  "parameters": {},
  "modulation": {},
  "oscillators": {},
  "filters": {},
  "effects": {},
  "macros": {}
}
```

 The actual format can be binary or another structured format, but it must support **version migration**.

 Old projects must remain loadable after future software updates.

---

 # 46\. Agent-development architecture

 This is the part I'd emphasize most if you're giving this to an autonomous coding agent.

 **Do not tell the agent to build the whole DAW at once.**

 Require it to work in vertical slices.

 ### Phase 1 — Foundation

 - Build application shell
- Audio device abstraction
- Audio callback
- Project format
- Track model
- Basic graph
- Basic mixer
- WAV playback
- WAV recording

 ### Phase 2 — DAW core

 - Arrangement
- Clips
- MIDI
- Piano roll
- Automation
- Mixer
- Routing
- Export

 ### Phase 3 — Synth

 - Oscillators
- Wavetables
- Filters
- Envelopes
- LFOs
- Modulation matrix
- Unison
- Polyphony
- Synth UI

 ### Phase 4 — Effects

 Build each effect as an independent DSP module.

 ### Phase 5 — Session

 - Clip launcher
- Scenes
- Follow actions
- Quantized launching

 ### Phase 6 — Modular

 - Module API
- Patch graph
- Cables
- Modular UI
- Core module collection

 ### Phase 7 — Plugin ecosystem

 - VST3
- CLAP
- AU
- Plugin scanning
- Plugin state
- Latency compensation

 ### Phase 8 — Professionalization

 - Performance optimization
- Crash recovery
- Autosave
- Accessibility
- Stress testing
- Profiling
- Packaging
- Installers

---

 # 47\. Critical engineering rule

 I'd give the coding agent this explicit instruction:

 > **Every subsystem must be usable before the next subsystem is built. Never create mock UI pretending that functionality exists. A button labeled “Reverb,” “Wavetable,” “Record,” or “Launch” must either perform its actual function or be explicitly marked as unimplemented.**

 And:

 > **Do not replace DSP with placeholders, random waveform generators, timers, fake meters, or UI simulations. The objective is a functioning audio workstation, not a visual prototype.**

---

 # 48\. Definition of “done”

 A feature is **not complete** when:

```
UI exists
```

 It is complete only when:

```
UI
 ↓
State model
 ↓
Audio/MIDI implementation
 ↓
Automation
 ↓
Undo/redo
 ↓
Serialization
 ↓
Project reload
 ↓
CPU-safe realtime behavior
 ↓
Tests
 ↓
Documentation
```

 all work.

---

 # 49\. Suggested master architecture

 The ultimate architecture should look roughly like:

```
                         ┌───────────────┐
                         │   PROJECT     │
                         │    ENGINE     │
                         └───────┬───────┘
                                 │
             ┌───────────────────┼───────────────────┐
             │                   │                   │
       ┌─────▼─────┐       ┌─────▼─────┐       ┌─────▼─────┐
       │ ARRANGER  │       │  SESSION  │       │    MIDI   │
       └─────┬─────┘       └─────┬─────┘       └─────┬─────┘
             │                   │                   │
             └───────────────────┼───────────────────┘
                                 │
                         ┌───────▼────────┐
                         │   AUDIO GRAPH  │
                         └───────┬────────┘
                                 │
       ┌─────────────────────────┼────────────────────────┐
       │                         │                        │
 ┌─────▼─────┐             ┌─────▼─────┐            ┌─────▼─────┐
 │ INSTRUMENT│             │   MODULAR  │            │   PLUGIN  │
 │   ENGINE  │             │   ENGINE   │            │   HOST    │
 └─────┬─────┘             └─────┬─────┘            └─────┬─────┘
       │                         │                        │
       │                   ┌─────▼─────┐                  │
       │                   │  MODULES  │                  │
       │                   └───────────┘                  │
       │                                                  │
 ┌─────▼──────────────────────────────────────────────────▼─────┐
 │                         MIXER / FX                           │
 └──────────────────────────────┬───────────────────────────────┘
                                │
                         ┌──────▼──────┐
                         │    MASTER   │
                         └──────┬──────┘
                                │
                         ┌──────▼──────┐
                         │ AUDIO DEVICE│
                         └─────────────┘
```

 ## The really important design decision

 I'd make **everything a node in the same audio graph**.

 That means the following aren't fundamentally different things:

 - a Serum-style synth
- a VCV-style oscillator
- a compressor
- a modular patch
- a DAW track
- a return bus
- a plugin
- a sampler
- a master chain

 They're all **processing nodes with inputs, outputs, parameters, state, and automation**.

 That one architectural decision gets you much closer to the “Ableton + VCV + Serum in one package” idea without creating three incompatible engines.

 If you're giving this to an AI coding agent, I'd also turn this into a **formal `SPEC.md` \+ phased implementation plan + repository architecture + acceptance-test checklist** next. That would make it substantially more useful than a product-feature wishlist.