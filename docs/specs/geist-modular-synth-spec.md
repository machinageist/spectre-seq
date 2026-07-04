<!--
Author: Jeff
Date: 2026-07-03
Description: Clean-room functional spec for a Geist first-party semi-modular synth, derived from the public Phase Plant manual
Notes: Behavior-only spec written from public documentation; no vendor code or assets referenced; implementation must use original naming, UI, and DSP
-->

# Phase Plant — Clean-Room Behavioral Spec for Geist Modular Synth

## Clean-room statement

- Source material is the publicly published Kilohearts Phase Plant documentation and directly referenced public Kilohearts documentation for modulation, Snapins, host behavior, presets, samples, wavetables, and curve/LFO/remap editors.
- No decompilation, binary inspection, source code, presets, samples, wavetables, content banks, UI assets, screenshots, private docs, or non-public behavior were used.
- This spec records public observable behavior, data relationships, parameter semantics, routing semantics, and implementation implications. Geist must implement original DSP, parameter names where required by product policy, UI layout, visual styling, copy, assets, defaults, and content.
- `PP` marks facts sourced from public documentation. `Geist` marks intended implementation mapping or policy decisions.

## Provenance and public URLs

Fetched 2026-07-03 from public Kilohearts docs:

1. `PP` Phase Plant: https://kilohearts.com/docs/phase_plant
2. `PP` Modulation: https://kilohearts.com/docs/modulation
3. `PP` Kilohearts Essentials / Snapins: https://kilohearts.com/docs/snapins
4. `PP` Curves, LFOs, Remaps, and Shapes: https://kilohearts.com/docs/curves_lfos_remaps
5. `PP` Samples, Grains, and IRs: https://kilohearts.com/docs/samples
6. `PP` Wavetables: https://kilohearts.com/docs/wavetables
7. `PP` Presets: https://kilohearts.com/docs/presets
8. `PP` Host Plugins: https://kilohearts.com/docs/host_plugins

## 0. Documentation coverage matrix

| Public document | Public sections covered in this spec | Coverage status |
|---|---|---|
| Phase Plant | Overview; User Interface; Generator Area; Analog Oscillator; Granular Generator; Noise Generator; Sample Player; Wavetable Oscillator; Distortion Effect; Filter Effect; Non-Linear Filter Effect; Mix Routing; Aux Routing; Envelope Output; Curve Output; Generator Groups; Audio Rate Modulation; Effect Lanes; Voice Settings; Unison Settings | Exhaustive behavioral coverage of all public text in scope |
| Modulation | Overview; Modulator Lane; Envelope; LFO; LFO Table; Curve; Random; Audio Follower; Pitch Tracker; Note; Pressure; Velocity; Pitch Wheel; Note Gate; MIDI CC; MPE Timbre; Remap; Lower Limit; Upper Limit; Scale; Sample & Hold; Triggering | Exhaustive behavioral coverage of all public text in scope |
| Snapins | Overview; Designed to be combined; Common controls; 3-Band EQ; Bitcrush; Channel Mixer; Chorus; Comb Filter; Compressor; Compactor; Delay; Distortion; Dynamics; Ensemble; Filter; Flanger; Formant Filter; Frequency Shifter; Gain; Gate; Haas; Ladder Filter; Limiter; Nonlinear Filter; Phase Distortion; Phaser; Pitch Shifter; Resonator; Reverb; Reverser; Ring Mod; Stereo; Shaper; Tape Stop; Trance Gate; Transient Shaper | Exhaustive feature matrix because Phase Plant effect lanes host Snapins |
| Curves, LFOs, Remaps, and Shapes | Overview; Editors; Looping; Control Point Tool; Free Draw Tool; Stepped Draw Tool | Covered for LFO, Curve Output, Curve modulator, Remap, Shaper, and curve behavior |
| Samples, Grains, and IRs | Overview; File formats; Sample Editor; Tools; Selection Tool; Pan Tool; Zoom Tool; Stereo Mode; Grid settings; Top menu; Normalize; Remove DC; Reverse; Convert to Mono/Stereo | Covered for Sample Player and Granular Generator asset/editor behavior |
| Wavetables | Overview; Wavetable Editor; Tools; Selection Tool; Morph Tool; Pen Tool; Brush Tool; Wave Tool; Harmonic Edit Tool; Filter Tool; Sample Conversion; Effects; Fixes | Covered for Wavetable Oscillator and LFO Table asset/editor behavior |
| Presets | Overview; Preset Browser | Covered for public preset metadata, browsing, searching, saving, favorites, folders, and content bank visibility |
| Host Plugins | Overview; Topbar; Macros; Lanes; Groups; Modulation; Automation | Covered for macro, lane, group, topbar, and automation-slot implications |

## 1. Product-level architecture

### 1.1 High-level model

- `PP` Phase Plant is a modular synthesizer combining analog oscillators, samples, wavetables, granular synthesis, and noise with effects, modulation, and audio-rate modulation.
- `PP` The sound-design surface is divided into generator area, Snapin/effect lanes, modulation area, top bar/macros, voice settings, and master controls.
- `PP` Signal-generation path: generator area creates/modulates/filters sound, output modules send generated audio to effect lanes, effect lanes route serially or in parallel, and master controls shape final pitch/gain behavior.
- `Geist` Represent one patch as a schema-versioned device graph containing:
  - generator groups and modules;
  - per-voice audio graph edges;
  - output module bus sends;
  - three effect lanes with lane routing;
  - modulation slots and modulation edges;
  - macro controls and automation exposure;
  - voice/unison settings;
  - asset references for samples, wavetables, curves, and presets.

### 1.2 Top bar, patch metadata, macros, keyboard, undo/redo

| Feature | `PP` public behavior | Geist requirement |
|---|---|---|
| Patch/preset metadata | Top bar shows patch/preset name, author, and description. Host-plugin docs say metadata can be edited by single-clicking info in the top bar. | Store name/author/description in Geist preset metadata. Editing UX must be original but expose the same metadata fields. |
| Preset browser toggle | Browse button opens/closes the preset browser. | Provide a browser/open action with equivalent data behavior, not copied layout. |
| Favorites | Host topbar heart can like loaded preset, placing it in Favorites. | Store per-user favorite state outside immutable factory/public libraries. |
| Undo/redo | Undo/Redo buttons; host docs also mention keyboard shortcuts and separate DAW/plugin undo queues. | Device edits must enter Geist plugin/device undo queue. Do not assume DAW undo integration covers modular edits. |
| Eight macros | Eight macro knobs sit under patch name; each macro can route to any parameter and to multiple parameters. Macros can be renamed; new name is stored in preset and reflected to DAW automation. | Implement exactly eight patch macro controls for Phase-Plant-compatible behavior. Macro is a modulation source with renameable label, automation ID, and arbitrary fan-out. |
| On-screen keyboard | Keyboard icon reveals a keyboard used to play notes and display incoming MIDI notes. | Optional UI panel may emit note events and visualize incoming note state. Must not copy graphics. |
| Master pitch | Controls pitch of all generator modules at once; useful for LFO vibrato. Cents fine-tunes with 100 cents per semitone. | Apply as global pitch transform before generator pitch calculation. Expose semitone+cents or equivalent high-resolution pitch parameter. |
| Bend range | Sets pitch bend range in semitones for pitch wheel; not stored in presets, but saved in project as global parameter. | Store bend range in project/session state, not preset payload. |
| Master gain | Final patch gain fader controls patch volume. | Apply after lane/master sum. Parameter must be automatable. |

## 2. Generator area — exhaustive public behavior

### 2.1 Generator area invariants

- `PP` The generator area is the place where sound is generated, modulated, and filtered before effects.
- `PP` It is modular: modules are added one by one from an add-module area/menu.
- `PP` First generator insertion automatically creates a group and an output module.
- `PP` Maximum generator-area capacity is 32 modules.
- `PP` Modules can be moved and copied by drag/drop; holding ctrl/command while dragging copies.
- `PP` Every generator-area module displays a visualization of its output. Visualization data is generated through the same signal path as sound, but uses a fixed frequency and does not track played note.
- `PP` There are five public sound-source generator module families: Analog Oscillator, Noise Generator, Granular Generator, Sample Player, Wavetable Oscillator.
- `PP` All sound-source generator modules are keytracked: frequency depends on the played note.
- `Geist` Capacity, graph validation, and preset format must enforce a hard limit of 32 generator-area modules unless a future Geist-only mode explicitly opts out.
- `Geist` Visualization must be derived from Geist DSP output, not from static images or vendor assets. Visualization fixed-frequency behavior must be documented in UI code so users do not infer note tracking from scopes.

### 2.2 Generator auto-routing and group boundaries

| Routing rule | `PP` public behavior | Geist implementation requirement |
|---|---|---|
| Source mixing | Generator modules automatically route themselves by mixing their output on top of the signal coming from above. | In a group, sound-source modules contribute to the current group signal by summing with upstream signal. |
| Input-dependent modules | Some modules require input from above; if placed at top of group a red arrow indicates missing input. | Validate processors/utilities with no upstream signal and surface a non-audio-breaking warning. Warning artwork must be original. |
| Group-only placement | No generator module can exist outside a generator group. | Preset schema must nest every generator module under a group. |
| Group routing break | Group header breaks automatic routing. Audio never automatically flows between groups. | Compile each group as an isolated per-voice subgraph. No implicit edge crosses group boundaries. |
| Cross-group routing | Audio can be routed between groups using audio-rate modulation system. | Cross-group edges must be explicit graph edges, with feedback-cycle validation and delay policy. |
| Layering use case | Separate groups can act as layered sounds with their own output modules. | Multiple groups may send independently to effect lanes/master/sideband. |
| Modulator-source group use case | Groups can isolate modules that act as audio-rate modulation sources. | Permit groups with output disabled if they feed audio-rate modulation only. |

### 2.3 Common generator parameters

| Parameter | `PP` public semantics | Data/DSP/realtime implications |
|---|---|---|
| Level | Amplitude of generator; volume scaled from 0% to 200%. | Store normalized scalar with displayed percent. Audio-rate modulatable target. Applies before source is mixed into group signal. |
| Pitch: semitone + cent | Pitch offset from played note. +12.00 = octave up, -12.00 = octave down. Cent component fine-tunes interval. | Frequency ratio `2^((semi + cents/100)/12)` before harmonic/shift. Audio-rate target for exponential FM/vibrato; can detune. |
| Harmonic | Frequency multiplier. x4 = two octaves up. x0 turns off keytracking; frequency then set with Shift. | Apply after pitch ratio. Audio-rate target for linear FM; preferred over Shift for pitch-invariant waveform under modulation. Zero means base keytracked term contributes no frequency. |
| Shift | Fixed detune in Hz after pitch and harmonic. Can be positive or negative. Can push final frequency below zero; negative frequency is allowed and generator runs backward “in a sense.” | Accumulate in Hz. Do not clamp at zero. Negative phase increments must be supported for oscillator/sample/wavetable/grain position semantics where relevant. Audio-rate target for linear FM. |
| Phase | Has fixed phase offset in degrees plus per-note random phase range in degrees. Fixed component sets oscillator start phase. Random component adds randomness for each note; with unison, each unison voice randomizes individually. | Store two values: offset and random range. On voice start and unison subvoice start, draw deterministic seeded random phase inside documented range. Audio-rate target for phase modulation/classic FM. |

### 2.4 Generator module feature matrix

| Module | Produces audio? | Requires upstream input? | Keytracked? | Supports oscillator unison? | Can be audio-rate source/target? | Asset dependency |
|---|---:|---:|---:|---:|---:|---|
| Analog Oscillator | Yes | No | Yes | Yes | Yes | None |
| Noise Generator | Yes | No | Public docs say all generator modules are keytracked; specific keytracked modes are stepped and smooth keytracked noise | Not stated | Yes | None |
| Granular Generator | Yes | No | Yes | Not stated | Yes | Sample |
| Sample Player | Yes | No | Yes | Yes | Yes | Sample |
| Wavetable Oscillator | Yes | No | Yes | Yes | Yes | Wavetable |
| Distortion Effect | Processor | Yes | N/A | No | Yes | None |
| Filter Effect | Processor | Yes | N/A | No | Yes | None |
| Non-Linear Filter Effect | Processor | Yes | N/A | No | Yes | None |
| Mix Routing | Utility/processor | Yes | N/A | No | Yes, especially Level | None |
| Aux Routing | Utility/router | Explicit audio-rate input plus optional upstream signal | N/A | No | Yes | None |
| Envelope Output | Output | Yes | N/A | No | Send target; envelope target depends on standard envelope model | Standard envelope data |
| Curve Output | Output | Yes | N/A | No | Send target; curve target depends on curve model | Curve data |

## 3. Sound-source generators

### 3.1 Analog Oscillator

| Feature/parameter | `PP` public behavior | Geist requirement |
|---|---|---|
| Purpose | Reproduces classic waveforms with high fidelity; suited to subtractive synthesis and FM. | Implement original antialiased oscillators. Do not clone DSP algorithms. |
| Waveform | Selects sawtooth, pulse, triangle, or sine. | Store enum with four choices. |
| Sync | Simulates oscillator sync by running oscillator at a higher frequency while resetting phase to zero at normal frequency. | Separate “sync oscillator frequency” relation from reset/base frequency. Reset must happen at base-note period. |
| Pulse width (PW) | Adjusts pulse width of pulse waveform. | Applies only to pulse; define behavior for other waveforms as no-op or hidden in Geist UI. |
| Common params | Level, Pitch, Harmonic, Shift, Phase. | All common generator modulation targets apply. |
| Unison | Supports unison. | Use oscillator-unison path, not global stack duplication. Output is mixed internal unison signal. |
| Audio-rate modulation | Analog parameters can be source/target in generator area. | Phase/Harmonic/Shift/Level/Pitch must accept sample-rate graph inputs. |

### 3.2 Noise Generator

| Feature/parameter | `PP` public behavior | Geist requirement |
|---|---|---|
| Purpose | Synthesizes inharmonic elements such as drums or wind. | Implement noise as original random/noise DSP. |
| Noise type | Colored noise, stepped keytracked noise, smooth keytracked noise. Two of the three modes are keytracked. | Store enum. Keytracked modes must derive frequency/rate from note pitch. |
| Slope | For colored noise, adjusts spectral falloff from flat/white through -3 dB/oct pink to -6 dB/oct brown. | Parameter must cover these documented landmarks. Interpolation is Geist-defined. |
| Stereo | Blends between mono and stereo noise. | At mono, L/R must be identical; increasing value decorrelates or widens using original implementation. |
| Seed | Stable = same noise sequence for each note. Random = different sequence each time. | Voice start must use deterministic per-note seed for Stable and fresh/reproducible-per-render seed for Random. Offline bounce determinism must be considered. |
| Common params | Common generator pitch parameters apply because docs state all generator modules are keytracked, with keytracking most audible in keytracked modes. | Keep pitch path available for audio-rate modulation even if colored-noise pitch influence is limited by mode. |

### 3.3 Sample Player

| Feature/parameter | `PP` public behavior | Geist requirement |
|---|---|---|
| Purpose | Uses sampled audio as a building block; playback speed and pitch vary by played note. | Implement sample source with pitch-ratio playback, interpolation, looping, and project asset references. |
| Loading | User can browse for a sample or drag/drop onto module. Bundled sample library exists publicly but vendor content is out of scope. | Support browse/import/drag-drop of user/project samples. Do not ship/copy vendor samples. |
| Sample view behavior | Shows loaded sample, permits dragging offset and loop points. | Geist UI may show waveform and editable markers with original design. |
| Sample selector | Shows current sample name. Click opens browser. Up/down arrows step through sample library. Browser can add user folders. | Store asset ID/path, display user-facing name, support library locations and next/previous navigation if Geist has a library. |
| Save button | Saves current sample; root, offset, and current loop are saved in sample file. | Geist policy: do not silently mutate user audio. Provide explicit save/export. Prefer sidecar metadata in project/preset unless user chooses destructive write. |
| Root | Fundamental frequency/root pitch must be set for correct pitched playback. Public docs describe visual tuning with two sample portions one cycle apart; when blue/grey align with one cycle between white bars sample is in tune. | Store root note/frequency metadata per module or sample sidecar. UI may offer original visual tuning aid. |
| Offset | Adjusts start position of playback in sample. | Start read pointer at offset on note start, subject to loop/reverse mode. Audio/control modulation semantics must be realtime safe. |
| Loop Mode: Infinite | Loops forwards forever. | Playback wraps loop region while voice exists. |
| Loop Mode: Sustain | Loops forwards until key release, then continues without looping. | Requires envelope/gate release event to switch from looped to one-shot continuation. |
| Loop Mode: Ping Pong | Alternates loop region forwards and backwards. | Direction flips at loop boundaries. |
| Loop Mode: Reverse | Plays loop only backwards. | Read pointer direction is reverse within loop. Define note-start pointer from public loop semantics. |
| Start | Loop start point. | Stored in sample-time units; UI displays in seconds/samples/percent as Geist chooses. |
| Length | Loop length. Visual zoom around loop start/end can help align curves. Shift-drag fine-tunes in source product. | Store loop length independently from sample length. Provide precise editing affordance. |
| X-Fade | Crossfade across loop boundary to avoid clicks/pops; parameter adjusts crossfade region length. | DSP must crossfade wrapped samples in loop modes and handle ping-pong/reverse without discontinuities. |
| Unison | Supports oscillator unison. | Internal unison duplicates sample playback voices and mixes them before module output. |
| File formats | Public sample docs: Kilohearts factory samples are FLAC; user can load WAV and AIFF. | Geist should support WAV/AIFF and optionally FLAC. Asset importer must preserve channels/rate and project portability. |

### 3.4 Granular Generator

| Feature/parameter | `PP` public behavior | Geist requirement |
|---|---|---|
| Purpose | Plays several short sample snippets called grains. Grains start at current play cursor position and are pitched by played note. | Implement grain scheduler per voice with sample asset, pitch ratio, cursor, envelope, spawn modes, and randomness. |
| Loading | Browse or drag/drop any sample; bundled sample library exists. | Same sample asset policy as Sample Player. |
| Visualization | Shows loaded sample, draggable play cursor, and all playing grains. | Use original visualization showing cursor and active grains from actual engine state. |
| Sample selector | Shows loaded sample name; opens sample browser; up/down step samples; browser can add user folders. | Same browser/asset behavior as Sample Player. |
| Edit button | Opens sample editor. Saving through editor saves current root note in sample file. | Provide sample editor or external editor integration. Prefer explicit metadata writes. |
| Play Cursor | Controls position where grains start. | Cursor in normalized/sample-time units; can be modulation target. |
| Grain Length | Time length in milliseconds. Length keytracking makes high notes use shorter grains so grains cover same sample section regardless of note. | Scheduler must convert length ms to samples and optionally scale by pitch ratio. |
| Grain Envelope | Per-grain amplitude envelope stretched over grain lifetime. Attack and decay times plus attack/decay curvature are adjustable. | Envelope function evaluated inside each grain. Must be independent from voice amp envelope. |
| Grain Spawn Rate: Free Rate | Grains spawned at fixed Hz. | Scheduler interval = 1/rate, sample-clock accurate enough for audio. |
| Grain Spawn Rate: Synced Rate | Grains spawned at tempo-synced note length. | Scheduler follows host tempo/transport with musical subdivisions. |
| Grain Spawn Rate: Density | Spawn rate computed automatically to hit target number of simultaneous grains; changing grain length changes spawn rate. | Maintain approximate concurrency target `spawn_rate = density / grain_length` subject to jitter/randomness. |
| Root | Fundamental frequency/root pitch set for correct pitch. Visual one-cycle alignment aid as in Sample Player. | Store root metadata and use in pitch-ratio calculation. |
| Additional public features | Public Phase Plant text documents randomization dimensions: position, timing, pitch, level, pan, reverse probability; chord/picking behavior can spawn multiple pitch choices; warm-start behavior begins voices with grains already active. | Implement these as explicit parameters only where public names/semantics are available from docs. If exact ranges or chord lists are not public, use Geist-defined ranges and original labels. |

### 3.5 Wavetable Oscillator

| Feature/parameter | `PP` public behavior | Geist requirement |
|---|---|---|
| Purpose | Versatile generator able to replicate arbitrary waveforms; modulating frame creates movement. | Implement original wavetable oscillator with interpolation/bandlimiting. |
| Wavetable dimensions | Wavetable contains 256 frames; each frame is 2048 samples. Compatible WAV/FLAC length is 256 × 2048 = 524,288 samples. | Internal data model must support exactly this compatibility profile for imported PP-style tables; Geist may store additional metadata. |
| Library/import | Public docs mention factory library, compatible external sources, and editor. | Do not copy factory library. Support import of compatible user WAV/FLAC where licensing permits. |
| Wavetable selector | Shows name, opens browser, up/down steps, user folders can be added. | Store asset ref and library location; no vendor browser UI copying. |
| Frame | Selects current frame to play. | Continuous or stepped frame interpolation is Geist-defined unless public docs specify; expose modulation target. |
| Bandlimit | Applies very sharp internal low-pass filter to wavetable before phase modulation. Used to tame aliasing under heavy phase modulation. | Bandlimit processing must occur pre-phase-modulation in signal order. Exact filter design is original. |
| Unison | Supports oscillator unison. | Internal unison duplicates wavetable readout and mixes. |
| Audio-rate modulation | Can be source/target in generator area. | Phase, frequency-related parameters, level, and frame if implemented as target must be sample-safe or appropriately smoothed. |

## 4. In-stack effects and routing modules

### 4.1 Distortion Effect module

| Parameter/feature | `PP` public behavior | Geist requirement |
|---|---|---|
| Role | Lightweight generator-area effect distorting incoming signal from modules above. Similar to Distortion Snapin. Can be audio-rate source and target. | Per-voice processor inside generator group. Requires upstream input. |
| Missing input | Red arrow in source product indicates no input if placed without module above. | Graph validator warning. |
| Type | Overdrive, Saturate, Foldback, Sine, Hard Clip, Quantize. | Store enum with these algorithm families. Implement original transfer functions. |
| Drive | Boosts input, causing heavier distortion. | Pre-shaper gain. Audio-rate modulatable. |
| Bias | Adds DC offset before distortion; can prevent hollow/uninteresting sound. | Pre-shaper DC offset. Needs DC-safety downstream. |
| Spread | Adds variable amount of bias to left and right channels for subtle stereo widening. | Channel-differential bias. |
| Mix | Dry/wet mix; lower lets unmodified signal pass. | Equal-power or linear mix is Geist-defined; must be automatable. |

### 4.2 Filter Effect module

| Parameter/feature | `PP` public behavior | Geist requirement |
|---|---|---|
| Role | Applies filters to upstream signal; similar to Filter Snapin; audio-rate source/target. | Per-voice filter processor. |
| Type | Low pass, band pass, high pass, notch, low shelf, peak, high shelf. | Store enum. |
| Cutoff | Operating frequency; for low-pass, -3 dB point. | Parameter in Hz/note-scaled UI; audio-rate modulation must be stable. |
| Q | High values resonate at cutoff. | Resonance/Q parameter. |
| Gain | Applies to low shelf, peak, high shelf. | Hide/disable for irrelevant types or make no-op. |
| Slope | 1x = classic 2-pole; 2x behaves like 4-pole. | Implement slope modes at least 1x/2x. |
| Missing input | Public red-arrow behavior. | Validator warning. |

### 4.3 Non-Linear Filter Effect module

| Parameter/feature | `PP` public behavior | Geist requirement |
|---|---|---|
| Role | Applies nonlinear filter varieties to upstream signal; similar to Nonlinear Filter Snapin; audio-rate source/target. | Per-voice nonlinear filter processor. |
| Type | Low pass, band pass, high pass, notch, all pass in Phase Plant module. Public Snapin page lists low pass, band pass, high pass, notch. | Generator-area module must include all pass because Phase Plant docs state it. |
| Cutoff | Operating frequency. | Stable under modulation. |
| Q | High values resonate at cutoff. | Resonance/Q parameter. |
| Drive | Overdrives filter, making nonlinear behavior more prominent. | Nonlinear input/core drive. |
| Mode | Clean has no nonlinearities and does not color signal. All other modes distort/color differently. | Include Clean plus additional original Geist nonlinear flavors. Exact non-clean mode names are not in public text; do not invent compatibility claims. |
| Missing input | Public red-arrow behavior. | Validator warning. |

### 4.4 Mix Routing module

| Parameter/feature | `PP` public behavior | Geist requirement |
|---|---|---|
| Role | Utility like an effect applying gain to signal. Commonly mixes output of generators above it for audio-rate modulation. | Group-signal gain/summing tap. |
| Auto input | Uses signal coming from modules above. | Requires upstream group signal; output replaces/continues group signal after gain/invert. |
| Level | Adjusts amplitude. Can be target for audio-rate modulation, enabling amplitude/ring modulation. | Sample-rate parameter input; multiplication can cross zero if modulation drives it. |
| Invert | Inverts signal/turns phase upside down. | Multiply by -1 after level or combine sign into gain. |

### 4.5 Aux Routing module

| Parameter/feature | `PP` public behavior | Geist requirement |
|---|---|---|
| Role | Similar to Mix, but does not automatically use only upstream signal; expects input routed via audio-rate modulation. | Explicit audio input node with optional mix into current group stream. |
| Cross-stack use | Can send audio to different parts of generator stack, including different group. | Permit explicit graph edge from any generator-area source to Aux input if no illegal zero-delay cycle. |
| Mixing | Aux mixes its output onto signal coming from above, like a generator module. | Output = upstream signal + processed explicit input. |
| Latency | Adds one sample latency for technical reasons. | Model as mandatory one-sample delay on Aux explicit input. This is also Geist feedback/cycle safety primitive. |
| Level/Invert | Work like Mix. | Same semantics as Mix. |

## 5. Output modules

### 5.1 Shared output-module behavior

- `PP` At least one output module is required in the module stack to create audible sound.
- `PP` Output modules can turn output off completely; disabled output is not heard but may still be used for modulations.
- `PP` Gain changes sent signal volume.
- `PP` Pan pans sent signal left/right.
- `PP` Send To selects one of the effect lanes to the right, master bus, or sideband bus for sidechain input in Snapins that support it.
- `PP` Out toggle is enabled by default and routes to effect lane 1.
- `Geist` Output module is the explicit voice-to-bus bridge. It must emit both audio bus sends and optional modulation taps. Output-off must mute bus send only, not necessarily disable source availability for audio-rate modulation.

### 5.2 Envelope Output

| Feature/parameter | `PP` public behavior | Geist requirement |
|---|---|---|
| Envelope | Applies a standard Kilohearts amplitude envelope to incoming signal. | Implement original ADSR/standard envelope behavior sufficient for amplitude shaping. If exact KH envelope curves/ranges are not public here, use Geist envelope model and document differences. |
| Routing | Sends enveloped signal to lane/master/sideband. | Envelope is before bus send and after upstream group signal. |
| Gain/Pan/Out/Send To | Shared output behavior. | Store in output module schema. |

### 5.3 Curve Output

| Feature/parameter | `PP` public behavior | Geist requirement |
|---|---|---|
| Curve envelope | Similar to Envelope Output but uses editable curve with set length for volume shaping. | Implement curve playback as amp multiplier before bus send. |
| Curve selector | Lets user choose from many logically sorted curve folders. | Geist may ship original curves; do not copy vendor curve library. User curve browser stores asset refs. |
| Editor | Pen button opens curve editor. | Original editor implementing public curve operations. |
| Loop handles | Two handles set loop points within curve. | Store loop start/end. Permit equal handles: public curve docs say this can force curve to stay at that value until loop ends. |
| Loop mode | Loop symbol selects loop mode. Public curve docs enumerate Off, Infinite, Sustain, Ping Pong, Reverse. | Implement all five curve loop modes. |
| Lock behavior | Curve modulator has lock for loop settings; Curve Output docs do not state lock. | Do not assume Curve Output has lock unless Geist adds its own. |
| Gain/Pan/Out/Send To | Shared output behavior. | Store in output module schema. |

## 6. Generator groups

| Feature | `PP` public behavior | Geist requirement |
|---|---|---|
| Organization | Groups organize modules and can be renamed for readability. | Group has stable ID and user label. |
| Generator-specific function | Unlike mostly cosmetic Snapin/modulator groups, generator groups affect routing. | Group boundary breaks implicit audio edges. |
| Mandatory grouping | Generator modules cannot be outside a group. | Schema validation. |
| Move/copy | Groups can be moved/copied as a whole; ctrl/command drag copies. | Preserve internal relative order, modulation edges, asset refs, and group outputs on copy with fresh IDs. |
| Minimize | Host docs say all groups can be minimized. | UI state may store minimized flag; no DSP effect. |
| Layering | Layers can live in separate groups with their own outputs. | Multiple group outputs may sum/routings by bus. |
| Modulation-source isolation | Groups can isolate modules acting as modulation sources. | Groups may have no audible output if used only for audio-rate modulation. |

## 7. Audio-rate modulation

### 7.1 Public behavior

- `PP` Phase Plant supports audio-rate modulation between all modules in the generator area.
- `PP` Audio-rate modulation is intended for FM patches and related techniques.
- `PP` Source product gesture: hover source module, plus appears, enter modulation mode, viable target plus icons appear, drag on target parameter; modulated target turns green.
- `PP` Audio-rate modulations are color-coded green. Control-rate modulations are orange. Modulations scaling other modulations are yellow.
- `PP` Generator-section values update every sample (“audio rate”). Control-rate modulators output less often, typically every 64 samples depending on DAW/settings. Around 100 Hz or higher, control-rate changes are likely affected by aliasing/static; fast modulation should use generator audio-rate modulation.
- `Geist` UI gesture/color can differ. Behavior must be graph-edge based at sample rate, not automation-lane based.

### 7.2 Audio-rate technique matrix

| Technique | `PP` target | `PP` public result | Geist DSP implication |
|---|---|---|---|
| Classic FM / phase modulation | Phase value of a generator | Classic FM effect; benefit of never causing generator to go out of tune. | Add modulator signal to phase accumulator/read phase. Preserve base frequency. |
| Linear FM via Harmonic | Harmonic multiplier | Linear FM; preferred because it creates same resulting waveform for all played pitches. DC in source can detune. | Multiply base keytracked frequency by modulated harmonic. Allow negative/zero results per frequency rules. |
| Linear FM via Shift | Shift value | Linear FM; DC in source can detune. | Add modulator in Hz to final frequency. |
| Ring modulation / AM | Level of a generator or Mix module | Multiplies source and target waveforms together. | Apply source signal as gain modulation at sample rate. |
| Exponential FM | Pitch value | Hard to control and almost always causes detuning; useful with slow source for vibrato. | Convert modulated pitch semitones/cents to exponential frequency ratio. |
| Cross-group routing | Aux input or viable target in another group | Sends audio between groups. | Explicit edge, one-sample Aux delay where Aux is used. |

### 7.3 Modulation-edge data requirements

- `Geist` Every audio-rate route stores source module ID/output tap, target module ID/parameter, depth/amount, polarity/scale if exposed, and enabled state.
- `Geist` Audio-rate routes participate in voice graph compilation. They are per-voice unless source/target semantics explicitly cross voices via global unison or bus processing.
- `Geist` Cycles are illegal unless routed through an explicit delay primitive such as Aux one-sample latency. Error messages must name the edge that creates the cycle.
- `Geist` Offline rendering must match realtime rendering for deterministic sources when seeds and transport are fixed.

## 8. Effect lanes and Snapin hosting

### 8.1 Lane behavior

| Feature | `PP` public behavior | Geist requirement |
|---|---|---|
| Count | Three effect lanes follow generator area. | Provide exactly three Phase-Plant-style lanes in compatibility device. |
| Host content | Lanes host Snapin effect chains. Each lane can hold multiple Snapins. | Geist effect modules must be original implementations. Snapin names in this spec are public reference categories, not required product labels. |
| Adding | Add by clicking empty lane bottom or spaces above/between effects. | Original insertion UX; data model supports insertion at index. |
| Routing topology | Lanes can route serially or in parallel. Send To can select one of the lanes to the right or master. | Directed acyclic lane routing left-to-right or master. No routing to previous lane. |
| Poly button | Runs all effects in that lane polyphonically, per voice, instead of after voice mix. | Lane can execute either per-voice before merge or post-merge. |
| Poly order constraint | Poly mode can only be enabled left-to-right. If lane 2 is polyphonic, lane 1 must also be. | Enforce prefix constraint: poly lanes are a contiguous prefix starting at lane 1. |
| Mute | Mutes lane. | Bypass/silence lane output according to solo/mute rules. |
| Solo | Mutes all other lanes. | Multiple solos policy Geist-defined; at least one solo mutes non-solo lanes. |
| Gain | Changes lane output volume. | Post-chain lane gain before send. |
| Mix | Output mix from 0% unprocessed to 100% fully processed with Snapins. | Lane dry path must be captured at lane input and blended after chain. |
| Send To | Sends lane result to lane to the right or master. | Store lane destination enum. |

### 8.2 Snapin common controls

| Feature | `PP` public behavior | Geist requirement |
|---|---|---|
| Power | Header checkbox enables plugin; off bypasses and lets incoming sound through unaffected. | Every effect module has enabled/bypass with latency compensation policy. |
| Minimize | Modules can collapse except stand-alone mode. | UI state only. |
| Preset bar | Header menu shows preset controls, current preset name, step through presets in folder, full browser folder icon. | Optional per-effect preset layer with original content. |
| Randomize | Dice randomizes all controls. | If implemented, must use Geist parameter ranges and not vendor randomization. |
| CPU efficiency/design | Snapins designed to be simple, stackable building blocks. | Effects should be realtime-safe, bounded allocation, and composable. |

### 8.3 Exhaustive Snapin/effect feature matrix for lane modules

| Effect family | `PP` public purpose | Public parameters and semantics to spec/implement if included in Geist |
|---|---|---|
| 3-Band EQ | Three-band EQ with adjustable split frequencies. | Splits: low/mid and mid/high cutoffs. Low/Mid/High: gains for bass/midrange/treble bands. |
| Bitcrush | Lo-fi sampler-like rate/bit-depth distortion. | Rate: downsample to minimum 200 Hz. Bits: amplitude quantization. ADC Q: lower adds low-frequency dissonant aliasing. DAC Q: lower adds high-frequency dissonant aliasing. Dither: adds noise to reduce quantization distortion. Mix: dry/wet. |
| Channel Mixer | Stereo channel amplitude and cross-mix. | Left to Left: scale incoming L to L from -1 inverted through 0 silent to 1 passthrough. Right to Left: scale R to L. Left to Right: scale L to R. Right to Right: scale R to R. |
| Chorus | Stereo/presence by mixing delayed versions. | Delay: average delayed-voice delay. Rate: delay variation rate. Depth: delay variation amount. Spread: stereo width, lower toward mono. Mix: dry/wet. Taps: number of chorus voices. |
| Comb Filter | Mixes signal with delayed copy creating repeated peaks/troughs. | Cutoff: distance between peaks. Mix: dry/wet. Polarity: plus peak at 0 Hz, minus trough at 0 Hz. Stereo: flips polarity for right channel for mono-compatible widening. |
| Compressor | Lowers volume when signal is loud. | Attack: time to lower volume over threshold. Release: time to return under threshold. Mode: RMS power vs Peak waveform following. Ratio: amount of reduction; example 1:2 lowers halfway between input and threshold. Threshold. Makeup. VU meter: input, threshold, attenuation. Sidechain: attenuation calculated from secondary input but applied to main input. |
| Compactor | Ducking, clipping, limiting; sidechain ducking or no-sidechain limiting. | Attack: how far ahead audio is ducked; higher starts earlier. Hold. Release. Threshold: sidechain above threshold silences input; without sidechain limits input to threshold. Mode: RMS, PEAK per-sample, ISP inter-sample peaks. Range: gain-reduction range; >100% exaggerates. Stereo: 0% channels processed same, 100% independently. Sidechain. |
| Delay | Echo by delaying input. | Delay: time before delayed sound, ms or beat fraction. Sync Mode: tempo sync. Feedback: delayed sound feeds back for exponential decay. Pan. Ping-Pong: swaps L/R in feedback; with pan bounces. Duck: lowers delay output when input high. Mix. |
| Distortion | Distortion with multiple algorithms. | Drive. Bias. Spread. Type: overdrive, saturate, foldback, sine, hard clip, quantize. Dynamics: preserves input dynamics otherwise distortion may force max volume. Mix. |
| Dynamics | Upward/downward compression and expansion with graph mapping input to output; moving disc shows current levels. | Low Threshold. Low Ratio: upward compression/expansion below low threshold. High Threshold. High Ratio: downward compression above high threshold. Attack. Release. Knee. In Gain. Out Gain. Mix. |
| Ensemble | Illusion of many unison voices via delayed copies, phase modulation, and delay modulation detune. | Voices. Detune: speed of delay modulation affecting detune. Spread: pans voices. Mix. Motion: modulation pattern. |
| Filter | Common filters. | Type: low pass, band pass, high pass, notch, low shelf, peak, high shelf. Cutoff: operating frequency; LP -3 dB point. Q: resonance at cutoff. Gain: shelf/peak gain. Filter slope: 1x classic 2-pole; public lane docs do not state 2x but generator filter does. |
| Flanger | Mixes audio with slightly delayed version; optional phase shift for infinite barberpole-style flanging up/down. | Delay: minimum delay. Depth: modulation depth added to delay. Rate. Scroll: enables phase offset and motion. Offset: phase offset dry/wet. Motion: phase-offset modulation rate. Spread: stereo spread affecting delay modulation and phase offset. Feedback. Mix. |
| Formant Filter | Boosts two frequencies to mimic vowels. | Vowel Selector: selects two boost frequencies. Q: boost power/narrowness. Lows: allow lows. Highs: allow highs. |
| Frequency Shifter | Shifts all frequencies up/down by fixed amount, ruining harmonic content and sounding dissonant. | Shift: amount all frequencies shift. |
| Gain | Volume change. | Gain in dB. VU Meter: current output level left/right. |
| Gate | Passes audio only above threshold. | Attack: open time. Hold: minimum open time. Release: close time. Threshold. Tolerance: hysteresis requiring level to drop under threshold before closing. Range: attenuation when closed. Look-ahead: 5 ms look-ahead, transient benefit with latency. Flip: reverse gate attenuates when open. VU Meter: input, threshold/tolerance, state. Sidechain: transient detection from secondary input but effect on main input. |
| Haas | Stereo widening by delaying left or right channel. | Channel: which channel to delay. Delay: delay time. |
| Ladder Filter | Classic-hardware-style low-pass ladder filters. | Cutoff. Resonance. Topology: transistor ladder or diode ladder; diode has gentler rolloff after cutoff; topology differs under saturation. Saturate. Drive in saturate mode. Bias in saturate mode. |
| Limiter | Prevents volume over threshold. | In gain. Out gain. Threshold. Release: return time after limiting peak. VU Meter: input, threshold, attenuation. |
| Nonlinear Filter | Filters with internal nonlinear color/distortion. | Type: low pass, band pass, high pass, notch. Cutoff. Q. Drive. Mode: Clean has no nonlinearities/no color; other modes distort/color differently. |
| Phase Distortion | Offsets phases of individual harmonics, amount controlled by signal itself like FM feedback. | Drive. Normalize: input-gain-insensitive. Tone: filters modulation to reduce high-frequency noise. Bias: constant phase offset to harmonics. Spread: L/R phase-offset spread. Mix. |
| Phaser | Series of moving spectral peaks/troughs. | Order: filter order, more pronounced with more peaks/troughs. Cutoff. Depth: cutoff modulation depth. Rate. Spread: L/R cutoff modulation phase offset for stereo. Mix. |
| Pitch Shifter | Changes input pitch up/down. | Pitch in semitones. Jitter: random pitch; high gives unison-like effect. Grain Size: length of grains used by processor. Mix. |
| Resonator | Adds harmonic resonance. | Pitch: resonance frequency. Decay: ring-out time after input silent. Intensity. Timbre: all harmonics saw-like or odd harmonics square-like. Mix. |
| Reverb | Simulated space. | Decay. Dampen: high frequencies decay faster. Size: room size from closet to church; public docs warn modulation except Macro can crackle. Width: 100% wet L/R uncorrelated. Early: early/late reflection balance; higher brighter/more responsive. Mix. |
| Reverser | Delayed reversed sections mixed with dry. | Delay time: section length to delay/reverse; e.g. 1/4 plays every beat reversed 1/4 bar later. Sync: tempo sync. Crossfade: ramp in/out percentage to avoid pops. Mix. |
| Ring Mod | Modulates input with internal generator or secondary input. | Bias: positive bias to secondary input. Rectify: positive/negative rectification of secondary input. Mix. Frequency: internal oscillator base frequency or internal noise filter cutoff. Spread: shifts internal-generator frequency L/R for stereo. |
| Stereo | Stereo width/pan with balance/correlation display. | Mid: mono part volume. Width: stereo part volume; needs stereo info. Pan. Stereo Meter: balance/correlation; red means correlation <0 and mono compatibility risk. |
| Shaper | Remaps incoming level using custom graph for distortion. | Drive. Mix. Overflow: beyond curve edges repeat, hold edge, or mirror-repeat. DC Filter: removes DC offset introduced by distortion. Uses shaper curve editor behavior from curve docs. |
| Tape Stop | Simulates tape motor stopping/starting. | Play: motor state. Stop Time. Start Time. Curve: speed curve for start/stop. |
| Trance Gate | Programmable rhythmic volume sequence. | Pattern Select: eight pattern slots. Pattern Editor: toggle steps; drag to tie steps. Length. Attack/Decay/Sustain/Release amplitude envelope. Mix. Resolution: step length. |
| Transient Shaper | Dynamics focused on initial transient. | Attack: transient amp/attenuation. Pump: attenuation after transient. Sustain: sustain amp/attenuation. Speed: higher snappier, lower smoother. Clip: clips output to 0 dB. Sidechain: transient detection from secondary input but effect on main. |

## 9. Voice settings and unison

### 9.1 Voice settings

| Feature | `PP` public behavior | Geist requirement |
|---|---|---|
| Glide Enable | Enables glide/portamento; new voice pitch slides from last pitch. | Pitch source must retain last-note pitch state. |
| Glide Time | How quickly voice glides to target pitch. | Time constant/ramp duration parameter. Exact curve is Geist-defined. |
| Glide Mode Always | Glide happens between all notes. | Apply regardless of legato gap. |
| Glide Mode Legato | Glide only when a new note is played while previous note still held; no gap. | Requires held-note set tracking. |
| Polyphony | Number of active voices. If all voices in use, oldest and quietest voice is recycled. Polyphony 1 makes patch monophonic. | Voice allocator must rank by age and level; deterministic tie-break. |
| Mono retrig | In monophonic retrig mode, every new note retriggers. | Restart voice modulators/envelopes on every note. |
| Mono legato | In monophonic legato mode, new notes are not retriggered while previous held; retrigger only after gap. | Preserve running voice state across legato pitch change. |
| CPU/mix implication | Too many voices can muddy mix and hurt CPU. | Provide voice limits and profiling; avoid unbounded allocation. |
| Master Pitch/Cents | Controls all generator pitches; 100 cents per semitone. | Apply globally as described above. |
| Bend Range | Project-global, not preset-stored. | See topbar/master. |
| Master Gain | Final patch volume. | See topbar/master. |

### 9.2 Oscillator unison vs global unison

| Type | `PP` public behavior | Geist requirement |
|---|---|---|
| Oscillator unison | Implemented in Analog, Sampler, and Wavetable. Generates several waveforms internally; module output is all waveforms mixed. If used as modulation source, mixed output is the modulation signal. Resource efficient and recommended first. | Internal subvoice fan-out inside module. Audio-rate source tap is post-unison mix. |
| Global unison | Creates several parallel voices for every note. Includes whole generator stack and effect lanes set to polyphonic. Allows unison for FM patches and polyphonic effects. | Duplicate complete per-note voice graph including generator stack and poly lanes. Non-poly lanes process post-merge. |
| Granular/noise support | Public docs do not list Granular or Noise as oscillator-unison modules. | Do not claim oscillator-unison support for those modules unless Geist adds a non-compatibility feature. |

### 9.3 Unison modes and parameters

| Mode/parameter | `PP` public behavior | Geist requirement |
|---|---|---|
| Unison - Hard | All voices same pitch with slight detune; all start at same phase. | Phase offsets equal except common/random phase rules. |
| Unison - Smooth | Same pitch with slight detune; voices start at different phases. | Random/distributed phase per subvoice. |
| Unison - Synthetic | Same pitch with slight detune; phases evenly spread. | Even phase distribution across subvoices. |
| Creative - Frequency Stack | Each additional voice frequency shifted by a multiple of original note. Bias knob replaced by Range. Range -100% = all voices same; 0% = multiples [2,3,4,5,...]. | Implement frequency multiplier sequence with Range interpolation. |
| Creative - Pitch Stack | Each additional voice pitch shifted by a multiple of original note. Bias replaced by Range. Range -100% = all voices same; 0% = each voice exactly one octave higher than previous; +100% = two octaves higher than previous. | Implement per-voice semitone offsets from Range. |
| Creative - Shepard | Creates Shepard tone. Voices spread octaves apart; farthest notes lowered in volume. Loudest octave determined by Center knob replacing Bias. Each C sounds same regardless of played octave. | Implement octave-wrapped Shepard distribution with Center parameter. |
| Chords | Several common chord modes tune additional voices into that chord scale. Chords replace Bias with Balance that shifts volume emphasis to lower or higher notes. | Public docs do not enumerate chord list; implement Geist chord modes separately or source from additional public doc if later found. |
| Voice Count | Number of simultaneous unison voices. | Integer count, bounded for CPU. |
| Detune | Amount of detune for each voice. | Applies according to mode. |
| Spread | Pans voices for wider stereo. | Per-subvoice pan law. |
| Blend | Balance between detuned voices and main voice. | Mix dry/main vs unison voices. |
| Bias | Shifts pitch of flat/sharp voices; negative detunes farther from center, positive tunes closer. Replaced by Range/Center/Balance in specific modes. | Store optional per-mode parameter union. |

## 10. Control-rate modulation system

### 10.1 Routing, timing, colors, modulation list behavior

| Feature | `PP` public behavior | Geist requirement |
|---|---|---|
| Modulatability | Almost all parameters in Kilohearts ecosystem can be modulated. | Mark every supported parameter as modulatable unless unsafe; document exceptions. |
| Add modulators | Modulators are modules in horizontal bottom lane; up to 32 modulator modules. | Enforce 32-slot modulator lane for compatibility. |
| Macro integration | Macros use same modulation system and route like other modulators. | Macro routes and modulator routes share edge data model. |
| Connection gesture | Source plus enters target mode; orange knobs appear; drag sets level. | UI may differ; must support source-target-depth route creation. |
| Route edit | Modulation knob visible at source; target knob appears on hover; drag adjusts level; double-click disconnects. | Provide route editing and deletion. |
| Timing | Control-rate modulators update less often than every sample, typically every 64 samples depending on host/settings. | Control-rate engine block size configurable; audio-rate targets cannot be faked by control-rate for FM. |
| Aliasing warning | Around 100 Hz or higher, control-rate modulation likely aliases/becomes static. | UI/docs should steer users to audio-rate modulation for fast modulation. |
| Colors | Orange control-rate, green audio-rate, yellow modulation scaling another modulation. | Geist can choose own colors but must represent route type distinctly. |
| Context menu list | Right-click control shows detailed modulation rows: enable toggle, source name, amount, curvature line, delete x, and bounds arrow. Bounds can be set instead of amount for precision. Bounds shown per modulation; total of all modulations can exceed individual bounds. | Store route enabled, source label, amount, curvature/remap, target bounds/depth. Allow multiple routes per target and sum them before domain handling. |
| Depth scaling | All modulators' output depth can be scaled by dragging animated output display; depth itself can be modulated. | Every modulator has output gain/depth parameter that is modulatable, creating modulation-of-modulation. |
| Output range | Most modulators can switch unipolar + (0..1), bipolar +/- (-1..+1), inverted - (1 down to 0). | Store output range mode per modulator. |
| Per-voice/global display | Animated display shows most recent voice in blue and other voices grey; some modulators have global value for non-voice-tied parameters. | Engine must distinguish per-voice modulator state and global state. UI visualization original. |

### 10.2 Modulator family matrix

| Modulator | `PP` public behavior | Parameters/semantics | Geist requirement |
|---|---|---|---|
| Envelope | Envelope modulation source using standard Kilohearts envelope. | Trigger target restarts envelope. Right-click opens triggering menu. Seamless option restarts from current value rather than zero. | Implement envelope state per voice/global as needed; include Seamless restart mode. |
| LFO | Low-frequency oscillator for oscillating/rhythmic modulation; editable shapes. | Trigger restarts. Frequency in Hz free-running or note length synced. Phase offset. Shape selected from organized predefined shapes or edited. | Implement free/synced oscillator, phase, shape asset/editor. Do not copy vendor shapes. |
| LFO Table | Uses wavetable as modulation source, cycling through 256 LFO shapes in one modulator. Useful for evolving/generative patterns; can combine with Random. | Trigger restarts. Frame scrolls through 256 frames. Smooth reduces scope/depth of wavetable; higher flattens curve. Uses wavetable editor. | Reuse wavetable data model; output is low/control-rate curve from selected frame. |
| Curve | Similar to envelope but uses selectable editable curve. | Trigger restarts. Modulator Rate sets speed; total cycle time displayed in seconds. Freewheeling or DAW-sync. Lock preserves loop settings when loading curves. Pen opens curve editor. Loop handles/mode. | Implement curve playback with rate/sync/lock/loop. |
| Random | Stream of random values; polyphonic patches get different random values per voice. | Trigger target. Voice mode: Unison = voices triggered together and global use same random sequence; Independent = every voice/global independent, matching Phase Plant v1.8 and earlier presets. Frequency in Hz/synced note length. Jitter randomizes timing. Smooth blends random numbers. Chaos adjusts step length; lower chaos makes next value closer; at 0 updates only on triggers and holds. | Deterministic seeded random engine with voice mode. Transport/render reproducibility. |
| Audio Follower | Tracks amplitude of incoming audio and outputs modulation. | Input selectable: Lane 1-3, Master, Sideband. Gain. Attack rise time. Release fall time. Can convert audio-rate sources to control-rate but does not make control targets sample-rate. | Envelope follower reading bus taps with attack/release smoothing. |
| Pitch Tracker | Tracks pitch of incoming audio and outputs modulation. | Input selectable Lane 1-3, Master, Sideband. Keyboard graph vertical sliders set range and center frequency. Sensitivity: low stable, high faster. | Pitch detector with selectable source bus and range/center mapping. |
| Note | Played note as modulation source. Range matches many cutoff/frequency knobs; 100% modulation makes them perfectly track played note. Polyphonic pitch bend affects Note modulator. | No extra parameters documented here. | Map MIDI note + per-note bend to normalized pitch control. |
| Pressure | Reflects pressure/aftertouch data. Sources include channel pressure MIDI CC and polyphonic aftertouch; VST3 exposes Polyphonic Expression slot. MPE compatible and can differ per note if DAW supports. | Source selection implied by input availability. | Support channel aftertouch, poly aftertouch, and MPE pressure where host API permits. |
| Velocity | Tracks note-on velocity; can also track release velocity where controllers support it. | No params documented here. | Store note-on/release velocity per voice. |
| Pitch Wheel | Tracks keyboard/controller pitch wheel. In MPE, tracks only global-channel pitch wheel; poly bends are Note modulator. | No params documented here. | Separate global pitch wheel from per-note pitch expression. |
| Note Gate | Note-on and note-off messages generate modulation signals. | Gate value follows note state. | Per-voice gate and global gate as needed. |
| MIDI CC | Any MIDI continuous controller. On instantiate, asks user to move controller; detected controller binds. Click bottom text field to assign another. | CC number binding. | MIDI learn and explicit rebinding. |
| MPE Timbre | Uses third-axis polyphonic MPE control, typically finger movement up/down on key. | No params documented here. | Support MPE timbre/slide per note if host data available. |
| Remap | Maps incoming values to custom output shape. Can be inserted between LFO and generator to reshape LFO. Uses scales/curves and freely editable curve editor. | Input comes via modulation routing; curve editor data. | Treat as modulation processor node with input edge(s) and curve asset. |
| Lower Limit | Clamps another modulator so output never goes below set value. White bar moved vertically; graph illustrates clamp. Limits can be modulated. | Lower limit parameter. | Modulation processor with modulatable threshold. |
| Upper Limit | Clamps another modulator so output never goes above set value. White bar moved vertically; graph illustrates clamp. Limits can be modulated. | Upper limit parameter. | Modulation processor with modulatable threshold. |
| Scale | Adjusts output of another modulator with vertical handle; bottom multiplier changes order of magnitude. Scale factor can be modulated. | Scale factor and multiplier. | Modulation processor multiplying input. |
| Sample & Hold | Samples input signal when triggered and holds until next trigger. | Input connected by sending modulation to center graph. Trigger connected to trigger arrow. When trigger rises past threshold, samples input at that moment. Threshold changeable by right-clicking trigger arrow. | Two-input modulation processor: sampled input and trigger/gate with threshold. |

### 10.3 Triggering behavior

| Trigger feature | `PP` public behavior | Geist requirement |
|---|---|---|
| Trigger button | Many modulators have trigger button `>`; it is a gate, on while pressed and off when released. | Model trigger as gate signal with rising/falling edges. |
| Restart | Gate-on restarts modulator from zero. | Reset phase/time or envelope state unless Seamless envelope says restart from current value. |
| Sustain action | Gate remains on; envelopes or LFOs/Curves with sustain loop mode perform sustain action until gate release. | Curve/envelope loop mode must read gate state. |
| Default note trigger | By default trigger gate controlled by note-on/note-off. | Per-voice trigger follows note events. |
| Modulated trigger override | When modulation is attached to trigger, notes stop triggering it and only modulation/clicks trigger unless behavior changed. | Trigger mode Auto must implement override. |
| Sensitivity | Trigger menu adjusts sensitivity for modulated trigger. | Store threshold/sensitivity per trigger input. |
| Auto | Default; notes do not trigger when modulation attached. Otherwise selects Always or Legato depending on Phase Plant voice polyphony. | Auto policy depends on attached trigger route and mono/poly settings. |
| Never | Notes never trigger; can create free-running LFO. | Disable note-driven trigger. |
| Always | Every note-on triggers. | Trigger on every note-on. |
| Legato | First note in legato group triggers; no retrigger until all notes released. | Requires held-note counter. |

## 11. Curves, LFOs, remaps, and shape editors

### 11.1 Shared shape concepts

- `PP` LFO uses one-dimensional shape as waveform for low-frequency control.
- `PP` Curve, Remap, and Shaper use similar one-dimensional data but interpret it differently.
- `PP` LFOs naturally loop; final control point is also first in a sense. Curve has a clear beginning and end. Because of interpretation differences, source product cannot load one type's data in another editor.
- `Geist` Store typed shape assets: `lfo_shape`, `curve_shape`, `remap_shape`, `shaper_shape`. Do not allow silent cross-type loading unless converted explicitly.

### 11.2 Editor common behavior

| Editor feature | `PP` public behavior | Geist requirement |
|---|---|---|
| Variants | Popup editors for one-dimensional shapes have LFO, Remap, Shaper, and Curve variants with shared functionality. | Use shared editor engine with type-specific constraints. |
| Workspace | Main workspace area with curve. | Original UI. |
| Toolbar | Left toolbar; Control Point Tool default; other tool buttons select modes. | Tool state stored in UI only. |
| Grid controls | Horizontal/vertical grid divisions; magnet toggles grid snapping; ctrl/cmd temporarily toggles snapping. | Snap system with temporary override. |
| Zoom/pan | Scroll wheel on horizontal axis zooms; click-drag axis pans. | Provide equivalent navigation. |
| Loop selector | Present in Curve Editor only, not LFO/Remap/Shaper. | Enforce by shape type. |

### 11.3 Curve looping modes

| Loop mode | `PP` public behavior | Geist requirement |
|---|---|---|
| Off | No loop; curve plays once exactly as drawn then holds last control point value. | Playback end holds final value. |
| Infinite | Loops left-to-right through loop area until voice dies; curve right of loop end never plays. | Ignore post-loop region. |
| Sustain | Loops left-to-right through loop area until note release, then continues from current position and ignores loop handle; region right of loop end is release phase. | Gate release exits loop and continues. |
| Ping Pong | Loops left-to-right then reverses direction, bouncing until voice dies; region right of loop end never plays. | Bidirectional loop. |
| Reverse | Goes through loop area left-to-right then loops right-to-left until voice dies; region right of loop end never plays. | Initial forward pass then reverse loop. |
| Equal handles | Both handles can be same point, forcing curve to stay at that value until loop ends. | Allow zero-length loop as hold point. |

### 11.4 Shape editor tools

| Tool | `PP` public behavior | Geist requirement |
|---|---|---|
| Control Point Tool | Drag points to move. Double-click curve/empty area creates point. Double-click point removes. Shift while moving = finer adjustment. Ctrl toggles snapping. Drag curve segment modifies slope; near a point adjusts slope to/from that point, middle adjusts both points. | Implement point CRUD, fine edit, snap override, slope handles/curvature. |
| Control point context | Right-click point opens numerical coordinate entry and smoothing toggle. Ctrl-double-click toggles smoothing. | Store smoothing per point and numeric coordinate edit. |
| Multi-select | Drag empty area or hold shift to select multiple points. Selected points can move together or be deleted with delete/backspace. | Multi-selection operations. |
| Free Draw Tool | Freely draw any shape. | Sample/densify freehand path into control/shape data. |
| Stepped Draw Tool | Always snaps to horizontal grid and draws vertical bars regardless of snapping; used like step sequencer. | Step drawing independent of snap toggle. |

## 12. Wavetable public asset/editor behavior

### 12.1 Wavetable concepts and editor structure

| Feature | `PP` public behavior | Geist requirement |
|---|---|---|
| Wavetable definition | Series of equal-length waveforms in a list; active waveform is a frame. Frame modulation changes waveform during playback. | Data model with frames and per-frame waveform/spectrum. |
| Uses | Wavetable Oscillator and LFO Table. | Shared asset type used by audio and modulation contexts. |
| Factory wavetables | Public docs mention a large factory selection. | Do not copy. Geist ships original/user content only. |
| Editor launch | Pen next to wavetable selector opens editor for current wavetable. | Original editor. |
| Top mini view | Miniature whole wavetable for navigation and keyframes. | Optional equivalent overview. |
| Waveform view | Shows selected frame waveform. | Required for editing. |
| Spectrum view | Shows magnitudes of partials for selected frame. | Required if spectral tools are implemented. |
| Pan/zoom | Waveform/spectrum pan/zoom via axes/scroll. | Equivalent navigation. |
| Modal tools | Many tools enter non-destructive mode with options bar. Edits affect sound while playing. Done commits, other tool commits, Cancel discards. | Transactional edit session with live preview and commit/cancel. |
| Keyframe animation | Most modal tools support keyframes across wavetable. Keyframes placed by clicking frame and editing; marker appears; drag/drop keyframes; double-click deletes. | Tool-parameter automation across frames with interpolation. |
| Region crossfade handles | Tools on waveform/spectrum sections often have handles to crossfade edited region edges. | Region edits include edge fade. |

### 12.2 Wavetable editor tools/effects/fixes

| Feature | `PP` public behavior | Geist requirement |
|---|---|---|
| Selection Tool | Select parts of wavetable/waveform/spectrum; copy/paste. Pasting enters keyframe-enabled transform mode. | Selection/copy/paste with transform preview. |
| Morph Tool | Crossfades between frames with keyframes. Intermediate frames filled by crossfade. Linear or spectral morph. | Implement linear and spectral morph modes if editor included. |
| Pen Tool | Draw curve between control points; animatable across wavetable. Interaction like LFO editor. Zoom, pan, magnet snap, grid resolution handles. | Reuse curve editor concepts. |
| Brush Tool | Freehand draw waveform; each frame drawn becomes keyframe; in-between frames linearly interpolated. | Freehand waveform edit with keyframes. |
| Wave Tool | Inserts standard waveform. Phase, frequency, and position adjustable and keyframe-animatable. | Standard oscillator-shape insertion. |
| Harmonic Edit Tool | Directly edit partials in spectrum by freehand drawing. Small widgets under partials adjust phases when zoomed. Interpolated like Brush. | Spectrum magnitude and phase editing. |
| Filter Tool | Applies filter to wavetable. Standard filter types; freely set slope; all params keyframe-animatable. Can bake filter sweeps. | Offline wavetable-domain filtering. |
| Sample Conversion | Drag/drop sample or File > Create from Sample. Keyframe animated modal tool; old data remains underneath and Mix blends. Visual input sample over wavetable. Root pitch auto-detected or manual; per-keyframe pitch bend offset for non-static pitch/formant-like effects. Source parameter maps keyframes to sample locations. Phase drift mitigated by phase-alignment strategies. | Import sample-to-wavetable tool with root detection/manual override, source mapping, mix, pitch bend, and phase alignment options. |
| Automatic EQ | Flattens spectrum toward gentle slope while preserving fine detail; cleans frames with varying spectral energy. | Modal wavetable effect. |
| Frame Blend | Blends adjacent frames; example frame 5 becomes mix of frames 2-8 when distance covers those. Distance parameter controls amount. | Neighbor-frame smoothing. |
| Comb Filter | Frequency-domain comb filter with spectrum-warp possibilities normal comb filter cannot do. | Spectral comb effect. |
| Disperse | Adds phase shift to partials increasing for higher partials, similar to Disperser plugin. | Frequency-dependent phase shift. |
| Distortion | Wavetable distortion with Drive, Bias, Mix and six selectable types. | Offline shaper on frames. |
| Phase Offset | Adds phase offset; blend between linearly frequency-dependent waveform offset and same-angle shift of every partial. | Phase transform. |
| Power Sync | Novel sync-like wavetable effect. | Original sync-like frame transform; exact algorithm not public. |
| Rectify | Flips negative waveform part positive. | Absolute-value style transform. |
| Reset Phases | Sets all partial phases to specified value; blended. | Phase reset with mix. |
| Self FM | Phase-modulates waveform with itself. | Offline self-PM transform. |
| Sine FM | Phase-modulates waveform with sine. | Offline sine-PM transform. |
| Squarify | Removes even harmonics, mimicking square wave harmonic profile. | Zero even partials. |
| Sync | Applies oscillator-sync-like effect to waveform. | Offline sync transform. |
| Tilt EQ | Applies sloped EQ curve. | Spectral tilt transform. |
| Normalize fix | Removes DC and normalizes RMS volume of all frames, ensuring no peak exceeds 100%. | Immediate destructive/editor operation with undo. |
| Remove DC fix | Removes DC from wavetable. | Immediate operation. |
| Invert fixes | Invert Time plays waveform backwards; Invert Amplitude inverts phase. | Time reversal and sign inversion. |
| Align Fundamentals | Sets first-harmonic phase in all frames identical to selected frame. | Phase alignment. |
| Align All Phases | Sets all harmonic phases identical to selected frame. | Phase alignment. |
| Align Frames | Phase-shifts each frame preserving phase ratios to maximize correlation to selected frame. | Correlation-based frame alignment. |

## 13. Sample public asset/editor behavior

| Feature | `PP` public behavior | Geist requirement |
|---|---|---|
| Shared sample type | Samples used by Sample Generator, Granular Generator, and Convolver; same file type/data structure but often purpose-specific. | Asset type can be reused, with module-specific metadata. |
| File formats | Factory samples FLAC; user can load uncompressed WAV and AIFF. | Support WAV/AIFF; FLAC optional but recommended for parity. |
| Editor launch | Anywhere a sample can be loaded, pen opens sample editor. | Original editor or explicit external workflow. |
| Editor purpose | Basic deleting/gain/fades/clean-up, not heavy sound effects. | Keep editor lightweight. |
| Selection Tool | Select region; modify boundaries; zoom for accuracy. Three handles over selection: middle adjusts gain; side handles create fade in/out over full selection and adjust gain/fade curvature. | Region selection, gain, fade curves. |
| Pan Tool | Drag sample view to pan; axes can pan even without Pan tool. | View navigation. |
| Zoom Tool | Drag view to zoom in/out; scroll over sample/axis zooms. | View navigation. |
| Stereo Mode | Default overlays stereo channels and edits both equally. Stereo controls isolate channels for one-channel edits. | Channel edit mode. |
| Grid settings | Upper-right grid behavior controls; magnet enables snapping. | Grid/snap state. |
| Top menu | Common save/copy/delete plus Operations menu. | Menu/action equivalents with undo. |
| Normalize | Applies to selection or full sample if no active selection; highest peak set exactly 0 dB. | Peak normalize operation. |
| Remove DC | Applies to selection or full sample; centers biased sample. | DC offset removal. |
| Reverse | Reverses selection or full sample. | Reverse operation. |
| Convert Mono/Stereo | Collapses stereo to mono or duplicates mono into stereo as appropriate. | Channel conversion. |

## 14. Presets, content, and automation

### 14.1 Preset browser and content behavior

| Feature | `PP` public behavior | Geist requirement |
|---|---|---|
| Factory presets | Every Kilohearts plugin ships with factory presets; count varies. | Geist must not copy vendor factory presets. Provide original presets if desired. |
| Browser opening | Host plugins have immediate Browse button; Snapins expose hidden topbar via arrow. | Device and effect preset browsers may differ but support same data operations. |
| Loading | Click loads preset. Double-click loads and closes. Arrow keys/buttons step current folder. | Browser selection/activation and previous/next commands. |
| Locations | Favorites, Factory, User, additional folders via Add Location. Remove added location with x on hover. | Location model with immutable factory/original content, user folder, favorites, extra paths. |
| Favorites | Heart icon favorites presets; Favorites starts empty. | User metadata. |
| Search | Free-text search over name, description, author; all words matched any order; quoted phrases exact. `by:Author Name` searches author. | Index metadata and implement token/phrase/author filters. |
| Tags | `#` in descriptions interpreted as clickable/searchable tags; search `#tag`. | Parse/store tags from description or explicit tag field. |
| Navigation | Back/forward arrows like browser through folders/searches. | Browser history. |
| Default preset | Right-click preset context menu can set default preset for plugin; new instances load it. | User preference per device/effect. |
| Close | Browse button, X, Escape, or double-click preset closes. | Equivalent close actions. |
| Save | Host topbar Save stores changes; New clears current patch. Save popup selects location; write-protected Factory/Favorites omitted. Name/Author editable. Create Subfolder organizes presets. | Preset save flow with write-protection, metadata edit, new/clear, folder creation. |
| Content Banks | Relevant installed content banks show in left listing for all content types. | Geist content packs may integrate as read-only or user-managed libraries. |

### 14.2 Preset payload contents

`Geist` A modular synth preset must capture:

- patch metadata: name, author, description, tags;
- generator groups, module order, parameters, group names/minimized UI state;
- audio-rate modulation routes and enabled/depth/curvature data;
- output modules and send destinations;
- effect lanes, effect modules, lane poly/mute/solo/gain/mix/send routing;
- voice settings except public non-preset Bend Range;
- unison settings at oscillator and global levels;
- modulator lane slots, modulator parameters, trigger modes, per-route amounts/bounds/curves;
- macro labels, values, route assignments, automation identities;
- asset references for samples, wavetables, curves/shapes, but not embedded third-party/vendor assets unless user explicitly chooses project collect/embed and license permits;
- schema version and migration metadata.

### 14.3 Automation

| Feature | `PP` public behavior | Geist requirement |
|---|---|---|
| Always-available params | Master Pitch and macros automate normally. | Expose stable host automation IDs. |
| Modular params | DAW cannot know complete dynamic parameter list; 64 automation slots expose selected module params. | Provide 64 assignable automation slots for dynamic parameters in compatibility mode. |
| Assign/clear | Right-click knob/button assign to slot; slot takes function name; clear removes. | Parameter context action or equivalent. |
| Macro rename | Macro renamed label reflected to DAW for automation. | Update automation display name safely. |

## 15. Realtime, DSP, data, and UI implications

### 15.1 Realtime/audio engine

- `Geist` Generator modules, in-stack effects, audio-rate modulation, Aux delay, output modules, and polyphonic effect lanes must run without allocation, locks, file I/O, or UI dependencies on the audio thread.
- `Geist` Control-rate modulation may update per block; audio-rate modulation must update per sample.
- `Geist` Parameter smoothing is allowed for stability except where public behavior depends on discontinuity/retrigger (sample offset at note start, trigger edges, loop boundaries, hard sync resets).
- `Geist` Voice allocator must be deterministic: when polyphony is exceeded, recycle oldest and quietest voice with deterministic tie-break.
- `Geist` Sideband/sidechain bus must be available to effects whose public docs support sidechain or secondary input.

### 15.2 Data model

- `Geist` Use stable UUIDs or graph IDs for modules, groups, lanes, modulators, routes, automation slots, and assets.
- `Geist` Store order explicitly for stack/lane/modulator arrays.
- `Geist` Store public compatibility limits: 32 generator modules, 32 modulators, 3 effect lanes, 8 macros, 64 automation slots.
- `Geist` Asset references must support relative project paths, library IDs, missing-file diagnostics, and collect/export workflows.
- `Geist` Sample/wavetable/curve editor operations must integrate undo/redo and avoid destructive writes unless explicit.

### 15.3 Project/session storage

- `Geist` Preset excludes Bend Range per public docs; project/session stores it.
- `Geist` User-added library folders, favorites, default preset choices, and content-bank registrations are user preferences/content database entries, not patch DSP state.
- `Geist` Presets should remain portable by warning on external asset references and offering collect/embed where licensing allows.

### 15.4 UI/original-design constraints

- `Geist` Do not copy Phase Plant layout, screenshots, module art, colors, exact icons, text snippets beyond necessary public parameter labels in internal specs, or vendor content organization.
- `Geist` Public gestures can inform required capability, but Geist should present original graph/handle/context UI.
- `Geist` Visualizations must be generated by Geist DSP/state and must not use vendor images.
- `Geist` Warnings for missing input, missing assets, illegal cycles, unsupported file formats, sidechain absence, and automation-slot conflicts must be explicit and actionable.

## 16. Geist mapping notes

| Public concept | Geist architecture mapping |
|---|---|
| Generator stack | Per-voice subgraph in `geist-graph`; implicit intra-group series/sum edges compiled from ordered modules. |
| Group header | Subgraph boundary and implicit-routing reset. |
| Audio-rate modulation | Sample-rate graph edge tagged as parameter/phase/frequency/level modulation. |
| Aux one-sample latency | Explicit delay node on Aux input; allowed cycle breaker. |
| Output module | Voice-to-bus send with amp shaper, gain, pan, out toggle, destination. |
| Effect lane Poly | Determines voice merge point. Poly lanes execute per voice; non-poly lanes execute on mixed bus. |
| Sideband | Named bus available as secondary input to compatible effects/modulators. |
| Modulator lane | Control-rate graph with per-voice/global state and up to 32 slots. |
| Macro | Control-rate modulation source with DAW automation and renameable label. |
| Remap/limits/scale/sample-hold | Modulation processor nodes, not audio processors unless explicitly repurposed by Geist. |
| Presets/assets | Schema-versioned patch plus external asset map. |

## 17. Deliberate non-goals and exact gaps

### 17.1 Non-goals

- No Kilohearts source code, assets, factory presets, samples, wavetables, curves, screenshots, icons, or private documentation.
- No cloning of UI layout, visual identity, or proprietary DSP algorithms.
- No claim of binary/preset compatibility with Phase Plant unless a separate legal/product decision defines it.
- No Rust code changes in this documentation task.

### 17.2 Exact public-doc gaps that must not be guessed

| Gap | Exact reason | Allowed Geist action |
|---|---|---|
| Exact numeric ranges/defaults for many parameters | Public pages often describe semantics without ranges/default values. | Choose Geist ranges/defaults based on usability and DSP safety; document them as Geist decisions, not PP facts. |
| Exact envelope segment curves/ranges for “standard Kilohearts envelope” | Referenced but not fully specified in fetched public docs except behavior around trigger/sustain. | Implement Geist envelope model; if needed, consult additional public `basic_usage#kilohearts_envelopes` later. |
| Exact non-clean Nonlinear Filter mode names/algorithms | Public text says clean plus other modes that distort/color, but does not enumerate in fetched docs. | Provide original modes; do not imply matching names/algorithms. |
| Exact chord unison list | Public text says several common chord modes but does not list them. | Implement original chord modes or source from additional public docs if available. |
| Exact oscillator/filter/distortion algorithms | Public docs describe observable purpose and parameters, not implementation. | Use original DSP achieving comparable behavioral roles. |
| Vendor libraries/content | Public docs mention bundled samples, wavetables, curves, presets, and content banks, but assets are not documentation and are out of clean-room scope. | Ship only original/user-licensed content. |
| Source product UI colors/icons/gestures | Public docs describe some colors and gestures, but copying visual design is out of scope. | Implement equivalent capabilities with original UI. |

## 18. Warnings for implementers

- `Geist` Do not “summarize away” a public parameter: every parameter listed in the matrices above needs a schema field, DSP/control behavior, or an explicit product decision to omit it.
- `Geist` Distinguish three modulation rates: audio-rate generator modulation, control-rate modulation, and host automation. They have different timing and aliasing behavior.
- `Geist` Global unison duplicates the whole generator stack and poly lanes; oscillator unison only duplicates one source internally. Mixing these up changes FM/modulation results.
- `Geist` Aux routing has one-sample latency; omitting it can create illegal zero-delay feedback or non-public behavior.
- `Geist` Lane poly mode is a left-to-right prefix. Enabling lane 2 poly implies lane 1 poly; enabling lane 3 poly implies lanes 1 and 2 poly.
- `Geist` Output-off does not necessarily mean “not usable as modulation”; public docs state disabled output may still be used for modulations.
- `Geist` Reverb Size public docs warn that non-macro modulation can crackle; if Geist smooths it, document that as an intentional improvement rather than public parity.
