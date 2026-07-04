<!--
Author: Jeff
Date: 2026-07-03
Description: Clean-room behavioral spec for Serum 2-class synth planning, derived from public Xfer Records Serum 2 docs
Notes: Behavior-only spec; no vendor code, assets, presets, wavetables, samples, or private materials referenced
-->

# Serum 2 Clean-Room Behavioral Spec

Behavioral planning notes for a Serum 2-class hybrid synth inside Geist DAW.
This is a clean-room document: it records public, observable product behavior
and published parameter semantics only. Geist must implement original DSP,
original UI, original naming where appropriate, and original Geist factory content.

## Clean-room statement

- Sources are public Xfer Records Serum 2 product/support pages and the public
  “What’s New in Serum 2” PDF only.
- No decompilation, binary inspection, vendor source, vendor assets, vendor
  presets, vendor wavetables, vendor samples, screenshots, or private
  documentation were used.
- Factory-content names, artist lists, images, screenshots, graphic layout, and
  marketing copy are not copied into Geist.
- Feature labels are used only where needed to identify public behavior.
- “S2” below marks behavior sourced from the public Serum 2 docs. “Geist” notes
  are implementation/planning implications for an original instrument.

## Provenance and covered public sections

Fetched/checked 2026-07-03 from the public web. The support category itself
listed only the Serum 2 articles below; two older public Serum support articles
are included only because they document public behavior still referenced by
Serum/Serum 2 workflows.

| Source | URL | Public sections covered | Used for |
|---|---|---|---|
| Serum 2 product page | https://xferrecords.com/products/serum-2 | product overview; oscillator family cards; specs; system requirements; demo availability | top-level architecture, oscillator family claims, content counts, plugin formats, platform requirements, demo limit |
| “What’s New in Serum 2” PDF, manual version 1.0.0, March 17 2025 | https://static.xferrecords.com/Serum%202%20What%27s%20New.pdf | Credits/front matter; What’s New overview; Enhanced User Interface; Expanded Oscillators; New Oscillator Modes; Wavetable Enhancements; Sample Mode Highlights; More New Oscillator Highlights; Dual Expanded Filters; Improved Serum FX; New and Enhanced FX; New Signal Splitter Modules; Enhanced Modulation; Enhanced Modulation Matrix; New Serum Mixer; New Clip Sequencer; Advanced Arpeggiator; Enhanced Keyboard; Improved Presets Browser | page layout concepts, all named feature modules, oscillator/filter/FX/modulation/mixer/clip/arp/browser behavior |
| Serum 2 support category | https://support.xferrecords.com/category/45-serum-2 | category article list: Machine Authorization Problems; Upgrade FAQ; CPU optimization; Preset Previews; Converting Samples to Wavetables; Automatic Sample Root Note Mapping | confirms public Serum 2 support scope and article set |
| CPU optimization support article | https://support.xferrecords.com/article/51-serum2-sound-design-guidelines-for-optimizing-cpu-usage | Manage Unisons Effectively; Keep Unison Counts Low; Chorus vs. Unisons | unison CPU implications; FX-bus efficiency guidance |
| Preset previews support article | https://support.xferrecords.com/article/52-serum-2-preset-previews | Fallback Previews; custom previews; creating previews; user benefits; macro automation tip | preset-browser preview semantics; clip-as-preview workflow; macro automation in previews |
| Sample-to-wavetable support article | https://support.xferrecords.com/article/59-converting-samples-to-wavetables | Overview; converting from Multisample; converting from Sample oscillator; what happens during conversion; tips; after conversion | single-sample isolation; frequency-estimation conversion; wavetable-frame creation; post-conversion operations |
| Automatic root-note mapping support article | https://support.xferrecords.com/article/57-automatic-sample-root-note-mapping | naming format; fallback root; mapping behavior; octave convention; troubleshooting; tips | filename note detection; default C3; A3=440 Hz convention; pitch-adjust behavior; naming constraints |
| Upgrade FAQ support article | https://support.xferrecords.com/article/58-how-to-upgrade-from-serum-1-to-serum-2 | free upgrade eligibility; project update behavior; preset compatibility; simultaneous versions; Splice note; CPU/system note | non-auto project migration; backwards preset visibility; ratings transfer; Serum 1 asset availability note; CPU statement |
| Machine authorization support article | https://support.xferrecords.com/article/46-serum2-machine-authorization | DAW/editor login; account credentials; Serum 1 serial exclusion; machine authorization limits; wrong-account error; offline activation | licensing behavior, explicitly out of scope for Geist synth DSP |
| Third-party preset import support article | https://support.xferrecords.com/article/22-how-do-i-import-3rd-party-presets-in-to-serum | Show presets folder; place presets under Presets subfolder; rescan folders on disk | public import/rescan workflow; file schema remains undocumented |
| SerumFX support article | https://support.xferrecords.com/article/20-where-is-serumfx | bonus FX plugin availability; Splice timing; account location | separate FX plugin existence; not a first-party Geist synth requirement |

## Public coverage status

| Area | Coverage from public docs | Spec status | Exact gap reason where incomplete |
|---|---|---|---|
| Product/plugin identity | Product page and PDF identify a hybrid synth instrument for VST3/AU/AAX hosts on Windows/macOS. | Covered. | N/A |
| UI page model | PDF labels pages for oscillators, mixer, FX, matrix, global settings, browser/menu, undo/redo, volume, wheels, keyboard, arp, clips, voicing. | Covered at behavior level. | Exact widget layout, graphics, and screenshots intentionally excluded. |
| Primary oscillator count/families | Product page and PDF state three primary oscillators, plus sub and noise, with five primary modes. | Covered. | Per-parameter ranges/defaults absent from public sources. |
| Wavetable oscillator | Product/PDF state smooth interpolation, tuning modes, unison, phase memory, routing, copy/paste, dual warp, warp families. | Covered at feature level. | Complete warp list, algorithms, ranges, anti-aliasing details, and default values absent. |
| Sample oscillator | Product/PDF/support state points, loops, crossfade, loop modes, snap loop detection, rate, slicing, score extraction/playback, tails, warp, root-note mapping, conversion. | Covered at feature level. | Audio formats, slice-detection parameters, loop modes list, and ranges absent. |
| Multisample oscillator | Product/PDF/support state multisample arrays, pitch/intensity/articulations, SFZ loading, switch last played sample to single sample. | Covered at feature level. | Supported SFZ opcode subset, mapping editor behavior, streaming rules, and file schema absent. |
| Granular oscillator | Product/PDF state sample-based grains, up to 256 grains, envelope, window amount, timbre shifting, warp, granular parameters, sample-mode features. | Covered at feature level. | Exact grain-size/density/jitter/window parameters and ranges absent. |
| Spectral oscillator | Product/PDF state realtime/harmonic resynthesis, transient detection, independent time/pitch, high/low frequencies, scan rate, warp, sample-mode features. | Covered at feature level. | FFT/bin/window, transient algorithm, spectral controls/ranges absent. |
| Sub oscillator | PDF states dedicated sub oscillator similar to Serum 1. | Covered at minimal public level. | Waveforms, routing, octaves, phase, and ranges absent from Serum 2 public docs fetched. |
| Noise oscillator | PDF states dedicated noise oscillator as in Serum 1. | Covered at minimal public level. | Noise-file behavior, one-shot/loop/keytrack modes, and ranges absent from Serum 2 public docs fetched. |
| Filters | PDF states dual filters, one/both, series/parallel, cutoff/resonance mouse control, new virtual analog and other models, Clean Drive mode. | Covered at feature level. | Complete filter type list, drive range, routing math, stereo behavior, and cutoff/resonance ranges absent. |
| FX busses/effects/splitters | PDF states two busses, 13 effects, 3 splitters, rearrange, bypass, multiple instances, presets, direct graphical manipulation; names six effects and three splitter types. | Covered at feature level. | Seven effect names and all per-effect parameter lists absent. |
| Modulation matrix | PDF states expanded matrix, dynamic display, source/aux curves, remove/bypass/reorder. | Covered at feature level. | Depth math, polarity, transform curves, smoothing, per-destination units absent. |
| Envelopes/LFO/macros | PDF states 4 envelopes with BPM and invert-legato; up to 10 LFOs with tools, editor, independent grid, swing, direction, phase, 10x rate up to 1000 Hz, Lorenz/Rossler; 8 macros, macro destination, apply/delete. | Covered at feature level. | Envelope segment model, LFO shape set, chaos parameters, macro apply/delete implementation absent. |
| Mixer | PDF states oscillator/filter/FX/main/direct channels; routing, pan, mix, levels, graphical cutoff/resonance, FX bypass, balance routing. | Covered at feature level. | Numeric ranges, channel graph constraints, send laws, pan laws absent. |
| Global/voicing/keyboard | PDF states transpose, key, scale, swing, oscillator mapping, legato, portamento. | Covered at feature level. | Polyphony, mono/legato allocation, glide curves, pitch bend ranges, MPE absent. |
| Clip sequencer | PDF/support state piano roll, grid, MIDI editing, loop/offset markers, global/settings, banks, 12 slots, record Overdub/Extend, automation lanes, macros, key/scale/transpose/osc mapping, MIDI out, previews. | Covered at feature level. | MIDI file/clip schema, grid values, automation resolution, MIDI-out modes absent. |
| Arpeggiator | PDF states global settings, banks, 12 slots, pattern/shape, pattern editor when Shape=Pattern, transpose shift/range, playback offset/repeats/gate/chance, retrigger, velocity lane, MIDI out, play slot. | Covered at feature level. | Pattern lengths, mode list, rate/gate ranges, chance semantics absent. |
| Preset browser/content | Product/PDF/support state browser, library, folders, packs, categories/tags, ratings, search, metadata, previews, macro adjustment during previews, hybridize, auto-play, rescan, counts. | Covered at feature level. | Preset schema, metadata schema, tag ontology, preview storage schema absent. |
| Import/root-note/support behavior | Support pages state preset import/rescan, automatic root-note filename parsing, sample-to-wavetable conversion, upgrade compatibility, authorization. | Covered. | Binary/file schemas and license protocol details absent; licensing not needed for Geist synth. |

## 1. Top-level architecture (S2)

### 1.1 Product and host behavior

| Feature | Public observable behavior | Geist implementation implication | Public gap |
|---|---|---|---|
| Instrument role | Serum 2 is advertised as an advanced hybrid synthesizer instrument plugin. | Model as a synth instrument, not only an effect rack or sampler. | Standalone app behavior not documented in fetched public sources. |
| Host formats | Product page states VST3, AU, and AAX 64-bit availability. | Geist may expose equivalent internal module/plugin formats independently. | Exact plugin SDK versions and host quirks absent. |
| Platforms | Product page states Windows 10+ and macOS High Sierra or later for Intel, Big Sur or later for Apple Silicon. | Geist platform targets should be decided separately; use as comparison only. | CPU instruction requirements and minimum RAM absent. |
| Demo | Product page states a demo for OS X/macOS and Windows, limited to 15 minutes. | No DSP requirement for Geist. | Demo limitation mechanism absent and irrelevant. |
| Lifetime updates/free upgrade | Product/support pages state Serum 2 is a free upgrade for eligible Serum 1 owners. | No DSP requirement. | Commercial policy, not synth behavior. |

### 1.2 Main module map

| Module surface | Public features | Data/DSP/realtime implications | UI/project implications |
|---|---|---|---|
| Oscillator page | Three primary oscillators, sub, noise, envelopes, macros, wheels, clips, arp, keyboard, filters, LFOs, velocity/note modulation, voicing controls. | Primary oscillators are voice generators; filters may be voice or routed modules; modulation sources must be schedulable per voice and/or global. | Page should expose all sound sources and quick modulation surfaces without copying vendor layout. |
| Mixer page | Blend/balance sources and busses; channels for oscillators, filters, FX busses, main/direct outputs. | Represent as an explicit audio graph with levels/pan/routing and possible sends to filters/FX. | Project state must serialize routing, level, pan, bypass, and FX management state. |
| FX page | Expanded rack view; two busses; rack modules with reorder/bypass/multiple instances/presets. | FX are post/source/bus processors; multiple instances require stable module IDs and deterministic order. | UI must support drag reorder, bypass, preset selection, splitters, and direct graph manipulation. |
| Modulation matrix | Expanded matrix; dynamic visualizations; editable source and auxiliary curves; remove/bypass/reorder rows. | Modulation graph must support secondary scaling, per-row bypass, row order, and source visualization without audio-thread allocation. | Matrix state should be human-auditable and patch-serializable. |
| Global settings | Public overview labels global page/settings but does not enumerate all controls. | Keep separate from per-voice state; changes may affect scale, transpose, mapping, tempo-following modules. | Gap: complete global parameter list absent. |
| Preset browser | Search, folders, content library, packs, tags/categories, ratings, metadata, previews, macros during preview, menu actions. | Browser indexes metadata and preview clips separately from audio processing. | Browser DB must rescan and update without disrupting audio. |
| Undo/redo | PDF labels comprehensive undo/redo. | Mutating edit commands should be reversible. | Exact scope/stack depth absent. |
| Main menu | PDF labels main menu/common operations; support references show preset folder and rescan commands. | Menu commands are project/editor commands, not DSP. | Complete menu command list absent. |
| Main volume | PDF labels master/main volume. | Final gain stage after synth/FX; smooth changes. | Range and law absent. |
| Wheels | Pitch and mod wheels are exposed. | Treat as real-time MIDI/control sources. | Bend ranges and default mod mappings absent. |

## 2. Exhaustive public feature matrix

This matrix enumerates every feature named by the public sources and maps it to a
clean-room behavioral requirement. Where sources only name a feature, the spec
uses only that feature-level behavior and marks missing details explicitly.

| Domain | Feature | Public behavior | Geist mapping notes | Public gap |
|---|---|---|---|---|
| Architecture | Three primary oscillators | Three main oscillators are available, more than the previous two-oscillator design. | `hybrid_osc[3]` per voice, each with selectable engine mode. | Allocation limits/routing defaults absent. |
| Architecture | Dedicated sub oscillator | Sub oscillator is a dedicated sound source similar to Serum 1. | Separate low-cost oscillator source with independent mix/routing. | Waveform/mode/range details absent. |
| Architecture | Dedicated noise oscillator | Noise oscillator is a dedicated sound source. | Separate noise/sample-noise source with independent mix/routing. | Noise source list and playback semantics absent. |
| Osc modes | Wavetable | Central wavetable synthesis mode. | Continuous wavetable frame playback with original Geist interpolation/AA. | Table size, format, warp list, editor behavior absent. |
| Osc modes | Sample | Sample playback/manipulation oscillator. | Asset-backed sample engine; no allocation on audio thread. | Formats, interpolation, voice reset behavior absent. |
| Osc modes | Multisample | Arrays of samples across pitch, intensity, articulations. | Mapping layer for key/velocity/articulation zones. | SFZ subset and articulation switching absent. |
| Osc modes | Granular | Sample material split into grains and recombined. | Grain scheduler per voice; cap at/around public 256-grain capability if chosen. | Exact grain parameter model absent. |
| Osc modes | Spectral | Frequency/harmonic resynthesis with independent time/pitch. | Offline/streaming analysis cache plus real-time oscillator. | Analysis parameters, resynthesis model absent. |
| Wavetable | Smooth interpolation | Near-continuous frame positions; smooth transitions without morph tables. | Store frames and interpolate on read; avoid relying on vendor morph tables. | Interpolation curve/quality absent. |
| Wavetable | Tuning modes | Semitone, harmonics, ratio, and step modes for octave/semitone control. | Tuning parameter mode enum; render pitch from mode-specific units. | Ranges/defaults absent. |
| Wavetable | Preset navigation | Hover over oscillator preset menu and mouse wheel changes presets quickly. | Optional UI affordance; do not copy vendor menu design. | Preset format/schema absent. |
| Wavetable | Copy/paste oscillator controls | Copy/paste oscillators with or without modulation routings. | Copy engine state and optionally modulation edges targeting/from module. | Clipboard serialization absent. |
| Wavetable | Routing to multiple targets | Oscillator output can route easily to multiple targets. | Use graph fan-out/send model. | Allowed destination list and send laws absent. |
| Wavetable | Enhanced unison | Unison configuration is enhanced; support article warns high counts cost CPU. | Explicit per-osc unison voices/spread/detune abstractions; cost model. | Exact unison params/ranges absent. |
| Wavetable | Phase memory | Controls phase and phase randomization for new notes. | Per-osc phase-init state and randomization amount. | Memory/random distribution/range absent. |
| Wavetable | Warp modes | Filtering, FM, phase distortion, ring modulation, distortions, additional/new warps. | Ordered warp chain; original DSP per warp family. | Complete list, algorithms, modulation sources, ranges absent. |
| Wavetable | True FM | Public docs name true FM as a warp capability. | Audio-rate mod input path required if implemented. | Carrier/modulator definitions absent. |
| Wavetable | PD | Public docs name phase distortion, including product page PD and PDF PD from filters wording. | Phase-domain transform stage. | Exact source and transfer functions absent. |
| Wavetable | Ring mod | Product page names ring modulation. | Multiplicative/audio-rate modulation option. | Source selection/range absent. |
| Wavetable | Dual warp | Two warp modes can be active at once for any main oscillator type per product page/PDF. | Store two ordered warp slots per primary oscillator; deterministic order. | Interaction/order/defaults absent. |
| Sample | Start/end points | Start and end points are adjustable and modulatable. | Per-voice sample window with modulation-safe boundaries. | Units/ranges/smoothing absent. |
| Sample | Loop start/end | Loop start/end points adjustable and modulatable. | Loop region state; crossfade and bounds checks. | Behavior when modulation crosses invalid order absent. |
| Sample | Playback/loop modes | User can set loop mode. | Loop-mode enum. | Complete mode list absent. |
| Sample | Crossfade | Smooth loop transitions at loop points. | Crossfade read heads around loop boundary. | Crossfade shape/range absent. |
| Sample | Snap loop detection | Product page names snap loop detection. | Offline/editor analysis to suggest stable loop points. | Algorithm and controls absent. |
| Sample | Flexible loop modulation | Product page names flexible loop modulation. | Modulate loop boundaries and/or related parameters with smoothing. | Exact target set absent. |
| Sample | Rate control | Rate supports tape-stop-style speed effects. | Sample playback rate is modulatable and can approach/through slow speeds as designed. | Range, reverse behavior, pitch relation absent. |
| Sample | Slicing | Auto and manual slicing with advanced options. | Slice marker set, auto-detection offline/editor, manual edit UI. | Detection/range/options absent. |
| Sample | Realtime score extraction/playback | Product page states slicing includes realtime score extraction/playback. | Extract trigger/timing pattern from slices for playback; schedule in voice/clip layer. | Score representation absent. |
| Sample | Tails mode | Product page mentions tails mode; PDF states forward-reverse looping tails for real-time stretching. | Tail/stretch strategy separate from normal one-shot/loop. | Exact algorithm and params absent. |
| Sample | Warp modes | Sample mode has advanced warp modes including FM and PM; product page also lists FM/PD/Distortion types. | Apply warp chain to sample oscillator output/phase where applicable. | Complete sample-warp list/ranges absent. |
| Sample | Root note from filename | Sample loaded with note at filename end sets root note. | Filename parser at asset import/load time. | Flats/lowercase/non-Western notation absent. |
| Sample | Default root C3 | If filename lacks note, root note defaults to C3. | Asset metadata default root = C3 under Serum convention. | Whether user preferences can change default absent. |
| Sample | Pitch correction to C3 | Support says loaded note name is used to apply pitch adjustment so C3 key plays sample at C3. | Compute transposition from parsed root to keyboard note using A3=440 convention. | Exact tuning reference for non-A absent beyond A3=440. |
| Sample | Note naming syntax | A-G, optional sharp `#`, octave number at end before extension. | Regex against basename suffix. | Flats (`b`) not stated; spaces/special chars between note and octave are warned against. |
| Sample | Octave convention | MIDI note 69 / 440 Hz is A3. | Adopt explicit convention in importer or convert to Geist convention. | DAW display mismatch handling absent. |
| Sample | Troubleshooting controls | Support suggests OCT, SEM, FIN, CRS can be changed to match expectations. | Expose octave/semitone/fine/coarse pitch controls if mirroring behavior. | Exact ranges/defaults absent. |
| Multisample | Pitch ranges | Recordings span pitch ranges. | Key zones. | Split selection/crossfade absent. |
| Multisample | Intensity ranges | Recordings span intensity ranges. | Velocity or dynamic layers. | Velocity mapping details absent. |
| Multisample | Articulations | Recordings can include articulations. | Articulation dimension in mapping model. | Switching mechanism absent. |
| Multisample | SFZ loading | Product/PDF state loading/import via open, human-readable SFZ. | Implement a documented SFZ subset only after legal/technical scoping. | Supported opcode subset absent. |
| Multisample | Factory library | Vendor ships real-instrument library content. | Do not copy content; create Geist-owned content. | Content names/assets excluded. |
| Multisample | Switch to Single Sample | From multisample mode, play desired note/sample then choose switch to single sample to isolate last played sample. | Track last triggered sample zone; editor command creates single-sample oscillator asset reference. | Behavior with layers/round-robin absent. |
| Granular | Up to 256 grains | Public docs state audio samples can play with up to 256 grains. | Grain scheduler should cap/configure count, with CPU budgeting. | Whether 256 is per voice/osc/global absent. |
| Granular | Envelope customization | Granular mode exposes customizable envelope. | Grain amplitude envelope or oscillator amp envelope parameter. | Envelope shape/model absent. |
| Granular | Window amount | Controls grain amplitude window. | Windowing parameter in grain synthesis. | Window functions/range absent. |
| Granular | Timbre shifting | Timbre can be adjusted and modulated. | Spectral/formant-like or playback transform; implement original Geist semantics. | Algorithm/range absent. |
| Granular | Warp | Access to warping capabilities/advanced warp options. | Reuse primary oscillator dual-warp abstraction. | Which warps allowed absent. |
| Granular | Complete granular parameters | PDF claims complete granular control. | Provide granular parameter group; not enough public detail to mirror exact list. | Complete control list absent. |
| Granular | Sample-mode features | Start/end, looping, slicing also available. | Share sample-region and slice metadata with granular engine. | Per-feature interaction absent. |
| Spectral | Frequency-spectrum manipulation | Spectral mode manipulates spectrum of a sound. | Analysis data cache and spectral-domain controls. | Bin/harmonic model absent. |
| Spectral | Realtime harmonic resynthesis | Product page states realtime resynthesis at harmonic level. | Real-time oscillator from pre-analysis or streaming analysis. | Latency/window sizes absent. |
| Spectral | Transient detection | Product page names transient detection similar to advanced timestretching. | Preserve/shape transients separately from steady-state. | Algorithm absent. |
| Spectral | Independent time/pitch | Public docs state independent time and pitch control. | Time scan position/rate independent from pitch transposition. | Quality modes/ranges absent. |
| Spectral | Hi/Low frequencies | User can set sample high and low frequencies. | Spectral band limiting controls. | Units, slopes, ranges absent. |
| Spectral | Scan rate | Sets speed and direction of sample playback. | Signed scan-rate control. | Range, sync, smoothing absent. |
| Spectral | Warp | Access to Serum/primary warping options. | Reuse warp chain after/before resynthesis per original design. | Order/available warps absent. |
| Spectral | Sample-mode features | Start/end, looping, slicing also available. | Share sample metadata; define spectral scan over selected region. | Interaction details absent. |
| Filters | Dual filters | One or both filters can be used. | Two filter modules with independent state and enable/routing. | Voice/global placement absent. |
| Filters | Series routing | Filters can be routed in series. | Graph edge Filter1->Filter2 or chosen order. | Ordering/swap controls absent. |
| Filters | Parallel routing | Filters can be routed in parallel. | Split and sum paths. | Gain compensation/mix laws absent. |
| Filters | Cutoff/resonance direct control | Mouse manipulation controls cutoff and resonance. | UI gesture maps to cutoff/resonance parameters. | Parameter units/range absent. |
| Filters | New filter types | Virtual analog and other filter models are publicly named. | Pluggable filter model enum with original algorithms. | Complete model list absent. |
| Filters | Clean drive mode | Drive control includes Clean mode. | Drive mode enum; Clean mode likely less colored but exact behavior unknown. | Drive algorithm/range/default absent. |
| FX | Two FX busses | Two separate FX busses for flexibility. | `fx_bus[2]` global or routed busses with independent chains. | Pre/post/voice routing constraints absent. |
| FX | Expanded rack view | Rack can be expanded. | UI/editor feature only. | Dimensions/layout excluded. |
| FX | 13 effects | Public docs state 13 effects. | Effect registry must allow at least 13 if mirroring. | Seven effect identities absent from public PDF/product/support. |
| FX | 3 splitters | Public docs state three splitter modules. | Splitter modules as routing processors in chain. | Crossover slopes and routing UI absent. |
| FX | Reorder modules | Drag/drop reorder. | Stable module IDs; order mutation not on audio thread. | Exact drag behavior absent. |
| FX | Bypass modules | Individual module bypass. | Per-module bypass with click-free transition. | Bypass latency/compensation absent. |
| FX | Multiple instances | Multiple instances of one effect allowed. | Chain model supports repeated effect types. | Instance limits absent. |
| FX | Presets | Factory/user presets for racks and modules. | Preset layer for module state/rack state; original Geist presets. | File schema absent. |
| FX | Direct manipulation | Graphical displays allow direct adjustment. | Optional graph widgets tied to parameters. | Exact widgets excluded. |
| FX effect | Bode/frequency shifter | Named new frequency shifter effect. | Frequency-shifter DSP module. | Parameters/ranges absent. |
| FX effect | Convolution | Named convolution effect. | Convolver; impulse response management if implemented. | IR source/length/latency controls absent. |
| FX effect | Delay HQ | Delay has HQ mode, now default. | Delay quality-mode enum defaulting to high-quality in analogous design. | Delay params and HQ algorithm absent. |
| FX effect | Distortion Overdrive/DC bias | Distortion has Overdrive mode and DC bias control. | Distortion module with mode enum and bias parameter. | Transfer curve/ranges absent. |
| FX effect | Reverb types | Reverb includes Vintage, Nitrous, and Basin types. | Reverb algorithm type enum; names may be factual labels but Geist can choose original names. | Full parameter list absent. |
| FX effect | Utility | Utility effect is named. | Utility gain/pan/channel-style module likely, but public docs only name it. | Parameters absent; do not infer beyond utility role. |
| Splitter | L/H | Low/high splitter. | Two-band crossover/router. | Frequency/slope/phase behavior absent. |
| Splitter | L/M/H | Low/mid/high splitter. | Three-band crossover/router. | Crossovers/slopes absent. |
| Splitter | M/S | Mid/side splitter. | Encode/decode mid-side branches. | Width/gain behavior absent. |
| Mod sources | Envelopes | Envelopes modulate almost any parameter. | Modulation target abstraction across most parameters. | Exceptions/ranges absent. |
| Mod sources | LFOs | Low-frequency oscillator sources with enhanced editor. | Time-varying modulation sources; can reach audio-ish 1000 Hz. | Shape set absent. |
| Mod sources | Velocity/Note | UI labels more modulation options. | MIDI note/velocity source nodes. | Exact sources absent. |
| Mod sources | Wheels | Pitch and mod wheels visible. | MIDI/control source nodes. | Scaling/default mapping absent. |
| Mod sources | Macros | Control multiple parameters simultaneously. | Macro source nodes with fan-out destinations. | Range/display absent. |
| Mod sources | Osc/filter as sources | Any oscillator or filter can be a modulation source. | Audio-rate modulation source taps; graph scheduling required. | Tap point/rate/scaling absent. |
| Matrix | Expanded view | Matrix can be expanded. | UI state. | Layout excluded. |
| Matrix | Dynamic visualization | Matrix displays modulations dynamically. | Visualization reads modulation values safely off audio thread. | Exact visualization absent. |
| Matrix | Source curves | Editable source scale curves. | Per-row transfer curve on source value. | Curve data model absent. |
| Matrix | Aux source curves | Editable auxiliary source scale curves. | Secondary/modulating source curve per row. | Aux math/order absent. |
| Matrix | Remove mods | Remove unneeded modulation. | Delete row/edge. | Undo scope absent. |
| Matrix | Bypass mods | Bypass individual row/edge. | Active flag per modulation route. | Smoothing absent. |
| Matrix | Reorder mods | Drag handles reorder modulation rows. | Row order serialized. | Whether order affects math absent. |
| Envelopes | Four envelopes | Number of envelopes expanded to four. | `env[4]`. | Segment type/count absent. |
| Envelopes | BPM mode | Envelopes can follow host tempo. | Tempo-synced envelope time units. | Note values/range absent. |
| Envelopes | Invert Legato | Force envelope to trigger on note-on even when legato enabled. | Per-envelope trigger override in mono/legato mode. | Interaction with poly/overlap absent. |
| LFO | Up to 10 | Up to 10 LFOs; LFO 7-10 appear after LFO 6 assignment. | Lazy UI reveal; internal support for 10. | Assignment threshold details beyond LFO6 absent. |
| LFO | Drawing tools/editor | Dedicated editor and drawing tools. | Editable LFO shape data. | Tool list absent. |
| LFO | Independent grid | Graph grid has independent X/Y settings. | UI grid config stored with LFO. | Grid values absent. |
| LFO | Swing follow | LFOs can follow swing. | LFO timing can reference global swing. | Applies to which rates/shapes absent. |
| LFO | Directional playback | Directional playback supported. | Direction mode enum. | Mode list absent. |
| LFO | Phase | Phase can be set and modulated. | Phase offset parameter and modulation input. | Range likely cyclic but exact units absent. |
| LFO | 10x rate/1000 Hz | Rates up to 1000 Hz via 10x rate behavior. | High-rate modulation path; avoid assuming control-rate only. | Rate scale/defaults absent. |
| LFO | Presets/custom | Choose a preset or create own configuration. | LFO shape preset/user library. | File schema absent. |
| LFO | Chaos Lorenz/Rossler | Chaos modes include Lorenz and Rossler. | Deterministic chaotic modulators with seed/state. | Parameters/normalization absent. |
| Macros | Eight macros | Eight macro controls. | `macro[8]`. | Naming/range/display absent. |
| Macros | Macro as destination | Each macro can be a modulation destination. | Modulation can alter macro values before fan-out. | Feedback/ordering absent. |
| Macros | Apply/delete | Public docs mention ability to apply and delete a macro. | Editor commands for committing/removing macro routing. | Exact operation semantics absent. |
| Mixer | Oscillator channels | Set routing, panning, levels. | Channel strip per oscillator. | Pan law, level range absent. |
| Mixer | Filter channels | Set routing, panning, mix, levels; graphical cutoff/resonance. | Channel strip and filter UI. | Filter mix definition absent. |
| Mixer | FX bus channels | Set FX bus levels. | Return/send channel strips. | Send/return topology absent. |
| Mixer | Main/direct outputs | Set levels and manage effects. | Main output plus direct output paths. | Direct output count/host mapping absent. |
| Mixer | Manage routing | Choose channel routing and balance to filters/FX busses. | Routing matrix/graph. | Allowed graph cycles/feedback absent. |
| Mixer | Manage FX | Bypass individual FX modules from mixer. | Mixer has remote controls for FX module bypass. | Full remote-control scope absent. |
| Clips | Clip module activation/display | Clip module can be activated and shown via label/control. | Editor module with enable/display state. | UI layout excluded. |
| Clips | Piano roll | Input/edit MIDI in full-featured editor. | Internal MIDI clip editor. | Event schema/resolution absent. |
| Clips | Flexible grid | Grid can be set appropriately. | Grid division setting. | Values/snap rules absent. |
| Clips | Global/settings | Module and clip parameters; factory/user clip banks and clips. | Clip preset/bank library. | File schema absent. |
| Clips | 12 slots | Each clip bank offers 12 slots. | Fixed slot count per bank for analogous module. | Slot switching behavior absent. |
| Clips | Loop/offset markers | Loop and offset markers set playback region/timing offset. | Clip timeline markers. | Marker constraints absent. |
| Clips | Record modes | Record clips in Overdub or Extend mode. | Capture incoming MIDI/automation with selectable mode. | Exact overwrite/extend rules absent. |
| Clips | Automation lanes | Automation captured in multiple lanes. | Automation data per parameter/macro. | Resolution/interpolation absent. |
| Clips | Macro tweaking | Macros can be tweaked while creating clips; support notes macro automation in previews. | Record macro automation into clip. | Automation target list absent. |
| Clips | Key/scale/transpose/mapping | Clip area can set/apply key, scale, transpose, oscillator mapping. | Clip output passes through musical transform/mapping layer. | Scale list absent. |
| Clips | MIDI out | User specifies how plugin outputs MIDI data. | Host MIDI output mode(s). | Mode list absent. |
| Clips | Preset preview clip | Custom clip can become preset preview and remain available when clip module enabled. | Store preview clip with preset metadata; allow reuse/edit. | Storage schema absent. |
| Arp | Activate/show | Arp module can be activated and shown. | Module enable/display state. | UI layout excluded. |
| Arp | Global settings/banks | Module parameters and factory/user arp banks. | Arp preset/bank library. | File schema absent. |
| Arp | 12 slots | Each arp bank offers 12 slots. | Fixed slot count per bank for analogous module. | Slot switching absent. |
| Arp | Pattern/shape | Set arp pattern and related options. | Pattern generator with shape/mode. | Shape list absent. |
| Arp | Pattern editor | Advanced editor available when Shape is Pattern. | Conditional editor for custom patterns. | Pattern data model absent. |
| Arp | Transpose | Shift and range controls. | Pitch transform lane/params. | Units/ranges absent. |
| Arp | Playback | Offset, repeats, gate, chance, and more. | Event generation controls. | Exact probability/repeat/gate ranges absent. |
| Arp | Retrigger | Sets how arp shape/pattern restarts. | Retrigger mode enum. | Mode list absent. |
| Arp | Velocity lane | Raise/lower output velocities over time. | Per-step velocity modulation lane. | Value range/curve absent. |
| Arp | Play slot | Slot can be started with play button. | Audition/trigger control. | Host sync/transport interaction absent. |
| Arp | MIDI out | User specifies how plugin outputs MIDI data. | Host MIDI output mode(s). | Mode list absent. |
| Keyboard | Transpose | Keyboard transpose by semitones within two-octave range. | Global semitone transpose parameter, range -24..+24 implied by two octaves. | Whether inclusive endpoints/default absent. |
| Keyboard | Key/scale | Applied to MIDI input and output of Clip and Arp modules. | Musical quantization/scale transform shared by input and generated MIDI. | Scale list/behavior for out-of-scale notes absent. |
| Keyboard | Swing | Applies to certain notes to add groove. | Timing transform for eligible generated/input notes. | Eligibility and amount range absent. |
| Keyboard | Osc mapping editor | Edit note/velocity ranges of oscillators and arpeggiator to define/limit response. | Per-source key/velocity filters; arp response map. | UI and overlap rules absent. |
| Voicing | Legato | Voicing controls include legato. | Mono/poly voice allocation feature if implemented. | Full semantics absent. |
| Voicing | Portamento | Voicing controls include portamento. | Glide engine. | Time/rate/curve/range absent. |
| Voicing | More | PDF says legato, portamento, and more. | Reserve extensible voicing settings. | Complete list absent. |
| Browser | Content library | Access factory content, artist packs, user patches. | Indexed library with content roots. | Schema and package format absent. |
| Browser | Folders | Tree hierarchy. | Folder tree UI/data model. | Sorting and path rules absent. |
| Browser | Packs | Artists can create a pack from a folder of presets. | Pack metadata generated from folder. | Pack manifest schema absent. |
| Browser | Search | Search presets by name. | Text index on names. | Fuzzy/tokenization details absent. |
| Browser | Preset list | Shows complete list or search results. | List view with filtering. | Sort modes beyond support category not stated. |
| Browser | Menu | Hybridize presets, auto-play, rescan database, and more. | Browser commands. | Hybridize behavior and complete menu absent. |
| Browser | Metadata | Display/edit selected preset metadata. | Metadata editor. | Fields/schema absent. |
| Browser | Categories/tags | Filter presets by categories and tags. | Tag/category index. | Taxonomy absent. |
| Browser | Ratings | Assign ratings and filter by user ratings. | User metadata separate from preset file or overlay. | Rating scale absent. |
| Browser | Previews | Play buttons preview presets; previews can be custom clips; fallback if missing. | Preview clip selection and playback engine. | Fallback pattern/audio details absent. |
| Browser | Macro adjustment while previewing | Macros can be adjusted as presets preview. | Preview audition path accepts live macro changes. | Commit/revert behavior absent. |
| Content | Factory counts | Product page states over 626 presets and 288 wavetables. | Counts are descriptive only; do not copy content. | Actual content excluded. |
| Compatibility | Serum 1 presets | Serum 1 presets compatible and shown in Serum 2 browser. | Geist should not assume loading vendor format without legal review. | Format schema absent. |
| Compatibility | Serum 2 presets in Serum 1 | Serum 2 presets cannot load in Serum 1. | One-way compatibility note only. | N/A for Geist. |
| Compatibility | Ratings transfer | Serum 1 preset ratings transfer on first launch. | User metadata migration pattern. | Storage locations absent. |
| Compatibility | Project update | Existing Serum 1 projects do not auto-update to Serum 2. | Preserve explicit migration behavior for major synth versions. | Migration tooling absent. |
| Authorization | Online account login | Plugin editor login with account credentials authorizes Serum 2. | Not a Geist synth DSP feature. | License protocol details irrelevant. |
| Authorization | Offline activation | Machine ID, account page offline auth, downloaded license file. | Not a Geist synth DSP feature. | License file format not needed. |
| SerumFX | Bonus plugin | Separate SerumFX plugin is available to customers. | Separate FX-only plugin is optional product decision. | Current SerumFX behavior outside Serum 2 docs. |

## 3. Oscillator subsystem detail (S2)

### 3.1 Shared oscillator requirements

- S2: The synth has three primary oscillators, each able to choose Wavetable,
  Multisample, Sample, Granular, or Spectral mode.
- S2: A dedicated sub oscillator and a dedicated noise oscillator exist as
  separate sound sources.
- S2: Primary oscillator controls can be copied and pasted, with or without
  modulation routings.
- S2: Primary oscillator output can be routed to multiple targets.
- S2: The oscillator mapping editor can limit note and velocity ranges for
  oscillators, and it also affects arpeggiator response.
- S2: Product page states dual warps are available for main oscillator types;
  PDF explicitly places dual warp in wavetable enhancements and says granular
  and spectral expose warping options.

Geist implications:

- Use a `HybridOscMode` enum with five documented primary modes.
- Keep sub/noise as separate source modules rather than overloading the primary
  oscillator count.
- Store oscillator copy/paste state as an engine-state object plus optional
  modulation-route subgraph.
- Route oscillator outputs through graph edges with fan-out and per-edge gain or
  mix only if Geist defines such semantics; do not infer Serum send laws.
- Any feature using samples must keep decoded buffers, loop/slice metadata, and
  analysis products out of the real-time allocation path.

### 3.2 Wavetable oscillator

Public behavior to implement or consciously diverge from:

- S2: Wavetable remains a central sound-generation oscillator.
- S2: Smooth interpolation permits near-continuous frame positions and smooth
  frame transitions without requiring morph tables.
- S2: Tuning mode choices include semitone, harmonics, ratio, and step modes for
  octave/semitone control.
- S2: Unison configuration is enhanced relative to the previous product.
- S2: Phase memory controls oscillator phase and phase randomization on new
  notes.
- S2: The oscillator preset menu supports fast mouse-wheel navigation while
  hovered.
- S2: Warp capabilities named publicly include filtering, true FM, phase
  distortion/PD, ring modulation, distortions, and additional/new wavetable
  warps.
- S2: Two warp modes can operate at once.

Parameter semantics publicly available:

| Parameter/control | Public semantics | Gap |
|---|---|---|
| Wavetable position | Scans through wavetable frames; after sample conversion it scans the created frames. | Range, interpolation curve, modulation scaling absent. |
| Smooth interpolation | Smooth table transitions with near-infinite frame positions and no morph table requirement. | Algorithm absent. |
| Tuning mode | Selects semitone, harmonics, ratio, or step/octave-semitone interpretation. | Numeric mappings absent. |
| Unison | More voices/configuration can thicken sound but high counts cost CPU. | Voice count limits, spread, blend, detune absent. |
| Phase memory/randomization | Controls starting phase and randomization for new notes. | Distribution and memory behavior absent. |
| Warp slot A/B | Two warp modes active at once. | Slot order, pre/post unison, modulation interaction absent. |

Data/DSP/realtime implications:

- Continuous frame interpolation and anti-aliasing must be original to Geist.
- Dual warp is an ordered transform pipeline; patch state must serialize both
  slots and amounts.
- True FM, oscillator/filter modulation sources, and 1000 Hz LFO rates imply
  Geist cannot treat all modulation as slow control-rate automation.
- Phase memory requires deterministic note-on initialization and a separate
  randomization source for reproducible rendering if Geist supports offline
  determinism.

### 3.3 Sample oscillator

Public behavior to implement or consciously diverge from:

- S2: Sample oscillator plays back and creatively manipulates samples.
- S2: Start/end and loop start/end points are adjustable and modulatable.
- S2: Loop mode is selectable.
- S2: Crossfade smooths loop transitions.
- S2: Snap loop detection is supported.
- S2: Rate control supports tape-stop-like effects.
- S2: Slicing has automatic and manual modes with advanced options.
- S2: Sample slicing includes realtime score extraction/playback.
- S2: Forward-reverse looping tails are used for real-time stretching workflows.
- S2: Warp modes include FM and PM in the PDF; product page also names
  FM/PD/Distortion types for sample oscillator behavior.
- S2: Sample-mode features are also available in Granular and Spectral modes.

Observable semantics:

| Behavior | Required observable result | Gap |
|---|---|---|
| Region points | Moving start/end changes the active sample playback region; modulation can animate those points. | Boundary behavior absent. |
| Loop points | Moving loop start/end changes loop region independently of overall region. | Invalid ordering behavior absent. |
| Crossfade | Loop transition becomes smoother as crossfade is used. | Curve/range absent. |
| Rate | Playback speed can be changed sufficiently for tape-stop-style effects. | Pitch coupling and min/max absent. |
| Slicing | User may create slices automatically or manually and use extracted playback/score behavior. | Slice detector and score format absent. |
| Tails/stretch | Forward-reverse looping tails support real-time stretching. | Exact stretch model absent. |
| Warp | FM/PM/PD/distortion-style processing can affect sample playback. | Algorithms absent. |

Root-note importer behavior:

- S2: When a sample filename ends in a note name before the extension, the note
  is detected and used as root note.
- S2: Examples use forms such as `F2`, `A#4`, and `A2` at the end of the
  basename.
- S2: Format is letter A-G, optional sharp (`#`), followed by octave number.
- S2: The note must be the last element before the file extension; support warns
  against hidden characters or spaces after it.
- S2: If no note is found, default root is C3.
- S2: Serum 2’s octave convention is MIDI note 69 = A3 = 440 Hz.
- S2: Loading applies pitch adjustment so pressing C3 plays the sample at C3.
- S2: Troubleshooting mentions OCT, SEM, FIN, and CRS pitch controls.

Geist importer pseudocode, original and behavior-only:

```text
basename = filename_without_extension(path)
if basename ends with /(?:^|.*\s)([A-G])(#?)(-?\d+)$/ or an equivalent
   conservative suffix parser:
    root = note_name_to_midi_using_A3_440_convention(match)
else:
    root = C3_using_A3_440_convention
store root in sample metadata
on note playback, transpose sample from root to requested note/key
```

Caution: the public article does not state flats, lowercase, negative octaves,
non-space separators, or international note names. Geist should document any
extensions as Geist behavior, not Serum-derived behavior.

### 3.4 Multisample oscillator

Public behavior to implement or consciously diverge from:

- S2: Multisample mode uses arrays of samples, normally recordings, distributed
  across pitch, intensity, and articulation ranges.
- S2: Product page describes import/creation from multisample recordings through
  SFZ, called open standard and human-readable.
- S2: PDF states support for loading SFZ files.
- S2: Support article describes a workflow from multisample to single sample:
  play the specific note/sample, open the samples menu, choose switch to single
  sample, and isolate the last played sample.

Geist implications:

- Model a multisample instrument as sample zones with dimensions at least for
  key/pitch, velocity/intensity, and articulation.
- Track the last resolved sample zone so editor commands can isolate it.
- Implement only a clearly documented SFZ subset if adding SFZ import; do not
  imply full SFZ coverage from public Serum docs because they do not enumerate
  opcodes.
- Zone selection, voice stealing, crossfades, round robins, and disk streaming
  are independent Geist design decisions because the public docs do not specify
  them.

### 3.5 Granular oscillator

Public behavior to implement or consciously diverge from:

- S2: Granular mode manipulates audio samples by breaking them into small grains
  and recombining them.
- S2: Public capability: play samples with up to 256 grains.
- S2: Parameters/features named: envelope customization, window amount for grain
  amplitude, timbre shifting, advanced/all warp capabilities, and complete
  granular control.
- S2: Granular mode has access to many sample-mode features, including
  start/end points, looping, and slicing.

Geist implications:

- A grain scheduler must be deterministic, bounded, and real-time safe.
- The public 256-grain claim should be treated as a feature target/capability,
  but not as a precise per-voice/global guarantee because the source does not
  say which scope it applies to.
- Window amount and envelope customization require per-grain amplitude shaping.
- Timbre shifting is modulatable; any implementation must use original Geist
  algorithms because the public docs do not define the DSP.

### 3.6 Spectral oscillator

Public behavior to implement or consciously diverge from:

- S2: Spectral mode manipulates a sample’s frequency spectrum and can resynthesize
  samples at the harmonic level in real time.
- S2: Independent control over time and pitch is public behavior.
- S2: Product page names transient-detection processing analogous to advanced
  time-stretching algorithms.
- S2: Controls/features named: high/low frequencies, scan rate for speed and
  direction of sample playback, and access to warping options.
- S2: Spectral mode has access to many sample-mode features, including
  start/end points, looping, and slicing.

Geist implications:

- Perform spectral analysis offline or incrementally outside the audio callback
  and serialize/cache analysis products with patch/project state as needed.
- Separate sample scan time from output pitch.
- Provide signed scan-rate behavior if mirroring the public “speed and direction”
  description.
- Do not infer FFT size, bin mapping, transient algorithm, or phase handling.

### 3.7 Sub oscillator

Public behavior:

- S2: Dedicated sub oscillator, similar to Serum 1.

Geist implications:

- Treat as an independent source with simple low-frequency reinforcement role.
- Provide routing/mix integration like other sources if matching public mixer
  architecture.

Public gap:

- Serum 2 public docs fetched do not state sub waveforms, octave options,
  routing defaults, phase behavior, direct-out behavior, or parameter ranges.

### 3.8 Noise oscillator

Public behavior:

- S2: Dedicated noise oscillator, as in Serum 1.

Geist implications:

- Treat as an independent noise/source playback module, integrated into mixer
  and routing.

Public gap:

- Serum 2 public docs fetched do not state noise sample library behavior,
  one-shot/loop/keytracking, pitch controls, phase/random controls, or ranges.

### 3.9 Sample-to-wavetable conversion

Public workflow:

1. If starting from multisample mode, play the desired note/sample.
2. Open the samples menu in the Sample Oscillator.
3. Select the command that switches to a single sample; this isolates the last
   played sample.
4. With a single sample loaded, open the samples menu.
5. Select the command that switches to wavetable.
6. Choose the public import mode named Frequency Estimation.

Public conversion semantics:

- S2: Any audio sample can be converted to a wavetable with a few clicks.
- S2: Frequency Estimation analyzes frequency content.
- S2: The result is a series of wavetable frames.
- S2: Result preserves character of the original sample, then wavetable position,
  wavetable modifiers, and wavetable-specific modulation options can be used.
- S2: Conversion is non-destructive; the original sample remains unchanged.
- S2: Clear harmonic content and shorter samples are recommended for predictable
  results; drums/complex material may produce unexpected but interesting output.
- S2: Converted wavetable can be saved for future use.

Geist implications:

- Conversion is an offline/editor task, never an audio-callback task.
- Store generated wavetable as a new derived asset or patch-local asset while
  preserving the source sample reference.
- Make failure/quality conditions visible; do not hide analysis errors in real
  time.

## 4. Filters and mixer (S2)

### 4.1 Filters

Public behavior:

- S2: Two filter modules are available.
- S2: Users may use one or both filters.
- S2: Filter routing can be series or parallel.
- S2: Cutoff and resonance can be controlled directly with mouse manipulation.
- S2: New filter types include virtual analog and other filters.
- S2: Drive has a Clean mode.

Observable semantics and implications:

| Feature | Observable behavior | Geist implication | Gap |
|---|---|---|---|
| Dual modules | Patch can engage filter 1, filter 2, or both. | Two filter module states and routing nodes. | Whether per-voice/global absent. |
| Series | Signal passes through filters one after another. | Ordered graph path. | Order controls absent. |
| Parallel | Signal is split between filters and recombined/mixed. | Parallel graph plus mix/gain management. | Mix law absent. |
| Cutoff/resonance direct manipulation | User can adjust cutoff/resonance from graph display. | UI gesture maps to parameters. | Units/ranges absent. |
| Virtual analog and other models | A wide variety of new model types exists. | Extensible filter enum; original DSP models. | Complete model list absent. |
| Clean drive | Drive control has Clean mode. | Drive mode flag/enum. | Transfer curve absent. |

### 4.2 Mixer

Public mixer channel coverage:

| Channel type | Public controls/behavior | Geist implication | Gap |
|---|---|---|---|
| Oscillator channels | Routing, panning, levels. | Channel strip per primary/sub/noise source as applicable. | Which oscillators shown and exact ranges absent. |
| Filter channels | Routing, panning, mix, levels; graphical cutoff/resonance manipulation. | Channel strip plus embedded filter controls. | Filter mix semantics absent. |
| FX bus channels | Levels for FX bus channels. | Return/send channels for two busses. | Send law and pre/post absent. |
| Main/direct outputs | Levels and FX management. | Main out plus direct-out paths. | Number and host mapping of direct outs absent. |
| Routing balance | Balance routing to filters and FX busses. | Graph UI/controls for send balance. | Exact topology absent. |
| FX management | Bypass individual FX modules. | Mixer remote-bypass controls for FX modules. | Full module management scope absent. |

Realtime/project implications:

- Mixer edits are graph mutations; schedule them on the control thread and apply
  atomically to audio graph snapshots.
- Panning/level should be smoothed to avoid clicks, but exact smoothing is a
  Geist decision because public docs do not specify it.
- Direct outputs require project/host output mapping and serialization if
  implemented.

## 5. FX rack (S2)

### 5.1 Rack/bus model

Public behavior:

- S2: FX system has two separate busses.
- S2: Expanded rack view is available.
- S2: 13 effects and 3 splitter modules are available.
- S2: Modules can be dragged to rearrange order.
- S2: Individual modules can be bypassed.
- S2: Multiple instances of a single effect can be added.
- S2: Factory and user presets exist for racks and modules.
- S2: Graphical displays allow direct control manipulation where applicable.

Geist implications:

- Represent each FX bus as an ordered chain of module instances.
- Module instance identity must be separate from effect type so repeated effects
  serialize and automate correctly.
- Splitters are routing modules, not only audio effects; the FX chain must allow
  branch/merge subgraphs or an equivalent internal representation.
- Preset systems need separate scopes for module presets and whole-rack presets.

### 5.2 Publicly named FX effects

Only six of the thirteen public effects are named in the fetched Serum 2 public
PDF/product/support material. The remaining seven are a public-doc gap.

| Effect label | Publicly stated behavior | Geist DSP implication | Missing public detail |
|---|---|---|---|
| Bode / frequency shifter | New frequency shifter effect. | Frequency shifting module. | Shift range, feedback, stereo, mix, modulation details. |
| Convolve / convolution | New convolution effect. | Convolution processor and impulse/response data path if implemented. | IR source, length, latency, normalization, params. |
| Delay | HQ mode exists and is default. | Delay with quality mode defaulting high-quality in analogous design. | Delay time modes, feedback, filters, modulation, sync, range. |
| Distortion | Overdrive mode and DC bias control. | Nonlinear waveshaper with mode and bias parameter. | Transfer functions, bias range, oversampling. |
| Reverb | Three new types: Vintage, Nitrous, Basin. | Reverb module with algorithm/type selector. | All reverb params and algorithms. |
| Utility | New utility effect. | General utility processor. | Parameters entirely absent. |
| Seven unnamed effects | Public docs state 13 total effects but do not name the remaining seven in fetched material. | Do not invent names. | Exact missing public-doc reason: PDF names only Bode, Convolve, Delay, Distortion, Reverb, Utility; product/support pages fetched do not enumerate the rest. |

### 5.3 Public splitter modules

| Splitter | Public behavior | Geist routing implication | Missing public detail |
|---|---|---|---|
| Splitter L/H | Splits lows/highs. | Two-band crossover branch. | Crossover frequency/range/slope/phase. |
| Splitter L/M/H | Splits lows/mids/highs. | Three-band crossover branch. | Crossover frequencies/slopes/phase. |
| Splitter M/S | Splits mid/side signal. | Mid-side encode, separate processing paths, decode or routing merge. | Gain normalization and routing UI. |

## 6. Modulation system (S2)

### 6.1 Modulation source and destination coverage

Public behavior:

- S2: Envelopes, LFOs, velocity/note sources, macros, pitch wheel, mod wheel,
  oscillators, and filters are visible modulation-related surfaces.
- S2: Any oscillator or filter can be a modulation source.
- S2: Envelopes can modulate almost any parameter.
- S2: Macros control multiple parameters simultaneously.
- S2: Each macro can itself be a modulation destination.

Geist implications:

- The modulation graph must support both control-rate and audio-rate sources.
- Source nodes should carry value-rate metadata: event, control, audio, tempo
  synced, or analysis-driven.
- Destinations should expose units and modulation scaling; because public docs
  do not state Serum’s depth math, Geist must define original depth semantics.
- Macro-as-destination creates possible graph ordering/feedback cases; Geist
  should prohibit or explicitly define cycles.

### 6.2 Modulation matrix

Public behavior:

| Matrix feature | Observable behavior | Geist implication | Public gap |
|---|---|---|---|
| Expanded view | Larger matrix view available. | UI mode/state. | Layout excluded. |
| Dynamic visualizations | Matrix displays modulation dynamically. | Safe value metering/visualization. | Visualization style absent. |
| Source scale curves | User can edit source curves. | Per-route source transfer curve. | Curve representation absent. |
| Auxiliary source scale curves | User can edit aux-source curves. | Per-route secondary source transfer curve. | Aux combination formula absent. |
| Remove | User can delete modulation rows. | Graph edge delete operation. | Undo/selection behavior absent. |
| Bypass | User can bypass individual modulations. | Active flag per modulation edge. | Smoothing/retrigger behavior absent. |
| Reorder | Drag handles reorder modulations. | Serialize row order. | Whether order changes result absent. |

### 6.3 Envelopes

Public behavior:

- S2: Four envelopes are available.
- S2: Envelopes have a BPM option to follow host tempo.
- S2: Invert Legato can force an envelope to trigger at note-on even when legato
  is enabled.

Geist implications:

- Envelopes must accept host tempo for time quantization/sync when BPM mode is
  enabled.
- Envelope trigger logic needs access to voicing/legato state.
- Invert Legato is per-envelope behavior unless Geist deliberately makes it
  global; public docs do not specify scope beyond the envelope feature.

Public gaps:

- Segment count/model, curvature, looping, sustain behavior, trigger modes,
  ranges, and defaults are not in fetched public Serum 2 docs.

### 6.4 LFOs

Public behavior:

- S2: Up to ten LFOs are available.
- S2: LFO 7 through LFO 10 appear after LFO 6 is assigned.
- S2: LFOs have enhanced drawing tools and a dedicated editor.
- S2: Graph grid has independent X and Y settings.
- S2: LFOs can follow swing.
- S2: Directional playback is supported.
- S2: Phase can be set and modulated.
- S2: 10x rate behavior reaches rates up to 1000 Hz.
- S2: Presets can be chosen or custom configurations created.
- S2: Chaos LFO modes include Lorenz and Rossler.

Geist implications:

- LFO implementation must not assume slow-only modulation; 1000 Hz requires
  sample-accurate or sufficiently oversampled/control-rate handling depending on
  destination.
- Chaos modes require deterministic state for project recall/offline render.
- Swing-follow needs shared timing information with global swing and clip/arp
  timing.
- Lazy UI reveal of LFO 7-10 is an editor behavior, not an engine limit.

Public gaps:

- Shape list, drawing-tool list, phase units, rate units, retrigger behavior,
  chaos parameters/seeds, preset schema, and modulation smoothing are absent.

### 6.5 Macros

Public behavior:

- S2: Eight macros are available.
- S2: Macros can control multiple parameters simultaneously.
- S2: Each macro can be a modulation destination.
- S2: Public PDF mentions ability to apply and delete a macro.
- S2: Preset previews/clips can automate macros.
- S2: Browser allows macro adjustment while previewing presets.

Geist implications:

- Macro values are both modulation sources and destinations.
- A macro fan-out should reference modulation routes, not duplicate parameter
  automation data.
- Applying/deleting macro operations must be clearly defined in Geist because
  public docs only name them.

Public gaps:

- Macro range, polarity, default labels, apply/delete semantics, modulation
  ordering, and browser preview commit/revert behavior are absent.

### 6.6 MPE / polyphonic expression

- Public fetched Serum 2 docs did not state MPE or polyphonic expression
  behavior.
- Exact missing public-doc reason: the product page, What’s New PDF, Serum 2
  support category, and fetched support articles do not mention MPE or
  polyphonic expression.
- Geist should treat MPE support as an independent product decision, not a
  Serum-derived requirement.

## 7. Clip sequencer, arpeggiator, keyboard, and voicing (S2)

### 7.1 Clip sequencer

Public behavior:

- S2: Clip sequencer is a flexible piano-roll sequencer.
- S2: MIDI data can be input and edited in a full-featured editor.
- S2: Flexible grid is available.
- S2: Loop and offset markers can be set.
- S2: Global/module parameters and factory/user-saved clip banks exist.
- S2: Clip settings and factory/user-saved clips exist.
- S2: Each clip bank has 12 slots.
- S2: Clips can be recorded in Overdub or Extend mode.
- S2: Automation can be captured in multiple lanes.
- S2: Macros can be tweaked while creating clips.
- S2: Key, scale, transpose, and oscillator mapping settings are used in the
  clip area.
- S2: User can specify how Serum outputs MIDI data.
- S2: Clips can serve as custom preset previews and can include macro automation.

Geist implications:

- Treat clips as MIDI/automation data, not audio samples.
- Clip automation should target macro/parameter IDs with stable serialization.
- Preview clips are preset metadata; they should remain editable/reusable when
  the clip module is enabled.
- MIDI output belongs in the host/timeline layer, not oscillator DSP.

Public gaps:

- Clip length limits, grid values, PPQ/resolution, MIDI-out modes, automation
  interpolation, Overdub vs Extend edge rules, marker constraints, and file
  schema are absent.

### 7.2 Preset preview clip workflow

Public workflow:

1. Enable the Clip module.
2. Open/show the Clip module via its label/control.
3. Prepare MIDI that demonstrates the preset, or drag an existing MIDI file into
   the clip module.
4. Right-click the clip thumbnail.
5. Choose the command to set it as the preview clip.
6. Optionally disable the Clip module; the preview remains available in the
   browser.
7. Optionally automate macro knobs inside the clip to demonstrate dynamic
   behavior.

Public browser playback semantics:

- If a preset has a custom preview, browser play/auto-play can audition that
  custom clip.
- If no custom preview exists, a fallback preview plays.
- Custom preview clips help communicate preset character/use in context.
- When the clip module is enabled, the preview clip remains available to use or
  modify directly.

Geist implications:

- Store a preview-clip reference or embedded clip in preset metadata.
- Preserve clip data even if the clip module is disabled.
- Preview playback should occur in an audition context that can also receive
  macro changes.

Public gaps:

- Fallback preview content, preview clip storage format, whether preview state is
  embedded or sidecar, autoplay timing, and macro-change commit behavior are not
  documented publicly.

### 7.3 Arpeggiator

Public behavior:

- S2: Arpeggiator is a sophisticated module with activation/show controls.
- S2: Global settings expose module parameters and factory/user-saved arp banks.
- S2: Each arp bank has 12 slots.
- S2: Slots can be started by a play button.
- S2: Pattern controls set the arp pattern and related options.
- S2: An advanced pattern editor is available when Shape is set to Pattern.
- S2: Transpose controls include shift and range.
- S2: Playback controls include offset, repeats, gate, chance, and more.
- S2: Retrigger controls set how the arp shape/pattern restarts.
- S2: Velocity lane raises or lowers note output velocities over time.
- S2: User can specify how Serum outputs MIDI data.

Geist implications:

- Arp output is generated MIDI/event data before the synth voice layer.
- Pattern editor state, velocity lane, chance, repeats, gate, and transpose
  should be serialized as arp-slot data.
- MIDI-out mode should be host/timeline integration.

Public gaps:

- Shape list, pattern length/resolution, rate/sync modes, chance probability
  semantics, gate range, repeat behavior, retrigger modes, and file schema are
  absent.

### 7.4 Keyboard/global musical controls

Public behavior:

- S2: Standard keyboard supports transposition by semitones within a two-octave
  range.
- S2: Key and scale apply to both MIDI input and output of the Clip and Arp
  modules.
- S2: Swing applies to selected/certain notes to add groove.
- S2: Oscillator mapping editor edits note and velocity ranges of oscillators and
  the arpeggiator, defining and limiting response.

Geist implications:

- Keyboard transpose likely maps to -24..+24 semitones if “within a range of two
  octaves” is interpreted symmetrically; because endpoints/defaults are not
  public, Geist should document its own exact range.
- Key/scale processing must be shared between input handling and generated MIDI.
- Swing timing must be available to LFOs that follow swing.
- Oscillator mapping is a per-source key/velocity filter with UI editor state.

Public gaps:

- Scale list, key/scale quantization rules, swing amount/range, which notes are
  affected by swing, transpose endpoints/default, and mapping overlap behavior
  are absent.

### 7.5 Voicing

Public behavior:

- S2: Voicing controls set legato, portamento, and more.
- S2: Envelope Invert Legato interacts with legato by forcing note-on trigger
  even when legato is enabled.

Geist implications:

- Voice allocation and envelope trigger logic must be coordinated.
- Portamento/glide should be part of pitch generation before oscillator pitch.

Public gaps:

- Exact missing public-doc reason: fetched public sources label only legato,
  portamento, and “more”; they do not specify mono/poly modes, voice limits,
  allocation, glide curves, bend ranges, priority rules, retrigger rules, or MPE.

## 8. Presets, browser, content, import, and compatibility (S2)

### 8.1 Preset browser

Public behavior:

- S2: Browser accesses factory content, artist packs, and user-saved patches.
- S2: Browser has a tree folder hierarchy.
- S2: Artists can create a pack from a folder of presets.
- S2: Search finds presets by name.
- S2: Preset list shows the complete list or search results.
- S2: Browser menu includes hybridize presets, auto-play, rescan database, and
  more.
- S2: Metadata associated with the selected preset can be displayed and edited.
- S2: Presets can be filtered by categories and tags.
- S2: Ratings can be assigned and used as a filter.
- S2: Preview play buttons audition presets.
- S2: Macros can be adjusted while previewing.

Geist implications:

- Separate immutable preset content from user overlays for ratings/metadata edits
  where appropriate.
- Browser database/index should be rebuildable via rescan.
- Search, categories, tags, ratings, folders, and packs should be independent
  index dimensions.
- Hybridize behavior is named but not specified; Geist should not clone or infer
  it without further public docs.

Public gaps:

- Preset file schema, metadata fields, rating scale, tag taxonomy, pack manifest,
  hybridize algorithm, auto-play behavior, preview storage, and macro preview
  commit/revert semantics are absent.

### 8.2 Content counts and exclusions

Public behavior:

- S2: Product page states the product comes with over 626 presets and 288
  wavetables.
- S2: Product page states a factory multisample library exists and includes real
  instruments recorded around the world.
- S2: Upgrade FAQ states Serum 1 wavetables and noise files are provided in
  Serum 2.

Clean-room implications:

- These counts/categories are descriptive and do not license copying any content.
- Geist must create original presets, wavetables, multisamples, samples, noise
  sources, preview clips, and packs.
- Do not copy factory names, artist names, sample names, wavetable names, pack
  artwork, screenshots, or audio content.

### 8.3 Import/rescan behavior

Public behavior from support:

- S2/Serum: User opens menu command to show the Serum presets folder.
- S2/Serum: User places presets in a folder inside the visible `Presets`
  subfolder.
- S2/Serum: User chooses menu command to rescan folders on disk.
- S2/Serum: Presets should then appear in the menu/browser.

Geist implications:

- Provide a visible user preset root and an explicit rescan operation if matching
  this workflow.
- Keep database rescan off the audio thread.
- Do not imply support for Serum preset file formats unless legal review approves
  interoperability work; public docs do not document the schema.

### 8.4 Compatibility and project migration

Public behavior:

- S2: Existing projects using Serum 1 do not automatically update to Serum 2.
- S2: Serum 1 presets are compatible with Serum 2 and automatically shown in the
  Serum 2 browser.
- S2: Serum 2 presets cannot be loaded in Serum 1.
- S2: Ratings applied to Serum 1 presets transfer to Serum 2 the first time Serum
  2 launches.
- S2: Serum 1 and Serum 2 can be installed simultaneously.
- S2: Equivalent patches are stated to use the same or less CPU than Serum 1, but
  new Serum 2 features may require more processing.

Geist implications:

- Major instrument revisions should not silently replace old project devices.
- Backward-compatible preset import should be explicit and tested if Geist ever
  has a v1/v2 synth lineage.
- User metadata migration should be idempotent and visible.

Public gaps:

- Preset schema, rating storage, first-launch migration details, and project
  migration tools are absent.

### 8.5 Authorization/support behavior

Public behavior:

- S2: Authorization occurs by opening the plugin in a DAW, opening the editor,
  and logging in with account credentials.
- S2: Serum 1 serial/license codes do not activate Serum 2.
- S2: Machine authorizations are limited; users can manage activations in their
  account and may need support if they run out.
- S2: Wrong-account errors can occur if logged into an account without the Serum
  2 purchase.
- S2: Offline activation uses a Machine ID from the offline device, an account
  page offline authorization option, and a downloaded license file supplied back
  to Serum 2.

Geist implications:

- Not a synth/DSP requirement.
- If Geist has licensing, design it independently and do not copy protocol or
  user flow beyond general account/offline concepts.

### 8.6 SerumFX support note

Public behavior:

- A separate SerumFX bonus plugin is available to customers through account or
  Splice-related access paths.

Geist implications:

- A standalone FX-only version of the synth effects is an optional product
  decision, not required by the Serum 2 synth behavior spec.

## 9. Performance behavior and authoring guidance (S2)

Public behavior:

- S2: Equivalent Serum 2 patches are described as same-or-less CPU than Serum 1.
- S2: New features may require more processing power.
- S2: Support warns that using more than roughly 3–7 unison voices per oscillator
  is often unnecessary and can indicate inefficient sound design.
- S2: Higher unison counts increase CPU significantly and can introduce phasing
  issues.
- S2: For chorused/thick width, a dedicated FX bus with chorus can be more
  CPU-friendly because the effect runs once instead of for every voice.
- S2: FX-bus processing can be more flexible for tweaking/layering than
  duplicating per-voice unison work.

Geist implications:

- Model per-voice unison as a multiplicative CPU cost.
- Prefer post-voice bus effects for voice-independent widening when musically
  acceptable.
- Surface CPU-cost hints in UI if possible: oscillator count, unison count,
  grain count, spectral/granular engines, convolution, and high-rate modulation.
- Keep heavy analysis/conversion/rescan tasks off the audio callback.

## 10. Data model, DSP, realtime, and project-state implications

### 10.1 Suggested clean-room data model

```text
Patch
  GlobalState
    transpose, key, scale, swing, voicing, mapping references
  Sources
    primary_osc[3]: HybridOscState(mode, tuning, phase, unison, route, warp[2], mode_state)
    sub_osc: SubState
    noise_osc: NoiseState
  Filters
    filter[2]: FilterState(type, cutoff, resonance, drive_mode, routing)
  Mixer
    channels: source/filter/fx/main/direct strips with levels, pan, route
  FX
    bus[2]: ordered list of FxModuleInstance or SplitterGraph
  Modulation
    env[4], lfo[<=10], macro[8], matrix rows with source/aux curves/bypass/order
  Sequencing
    clip banks[bank].slots[12], arp banks[bank].slots[12]
  BrowserMetadata
    categories, tags, ratings, preview clip, user metadata
  Assets
    sample refs, multisample maps/SFZ-derived zones, spectral/granular analysis, wavetables
```

This is a Geist planning model, not an observed Serum file schema.

### 10.2 Audio-thread safety

- No file IO, preset rescan, SFZ parsing, sample-to-wavetable conversion,
  spectral analysis, snap-loop detection, or database indexing in the audio
  callback.
- Use immutable asset snapshots for sample buffers, wavetable data, spectral
  analysis, multisample maps, and convolution data.
- Apply graph edits via lock-free or double-buffered snapshots.
- Smooth level, pan, cutoff, pitch/rate, loop-point, and bypass changes where
  Geist’s own DSP design requires click avoidance; public docs do not provide
  smoothing constants.

### 10.3 Modulation scheduling

- Control/event sources: MIDI note, velocity, wheels, macros, clip/arp events.
- Tempo-synced sources: BPM envelopes, LFOs, clips, arp, swing-follow LFOs.
- High-rate/audio-rate sources: 1000 Hz LFOs, oscillator/filter modulation
  sources, FM/PM/PD/ring-style warps.
- Matrix rows need source curve and aux-source curve evaluation before depth is
  applied, but exact math must be original because public docs do not specify it.

### 10.4 Project and preset serialization

- Serialize module identity and stable IDs, not UI positions.
- Serialize modulation routes by source/destination IDs and parameter IDs.
- Serialize preview clips and clip/arp banks independently of module enable
  state.
- Treat user ratings and editable metadata as user overlays unless Geist chooses
  a patch-embedded model.
- Preserve old-device instances during major version updates; do not silently
  auto-upgrade project devices.

## 11. Deliberate non-goals for Geist

- Do not clone Serum 2’s UI art, screenshots, layout, product naming
  presentation, factory presets, factory wavetables, factory samples, noise
  files, multisamples, artist packs, pack artwork, or preview clips.
- Do not implement Xfer account authorization for a Geist first-party synth.
- Do not copy Serum preset/wavetable/sample/file formats unless a separate legal
  review approves public interoperability work.
- Do not infer undocumented parameter ranges, defaults, smoothing laws, filter
  model algorithms, warp algorithms, effect algorithms, modulation depth math,
  preset schemas, or asset schemas from audio demos, screenshots, or binaries.
- Do not treat product-page content counts as permission to reproduce content.

## 12. Geist mapping notes by subsystem

| Serum 2 public subsystem | Geist original subsystem mapping | Notes |
|---|---|---|
| Three primary oscillators | `hybrid_osc[3]` voice module with mode enum. | Wavetable/sample/multisample/granular/spectral share routing/modulation shell but have separate engines. |
| Sub/noise | Independent `sub_source` and `noise_source`. | Keep inexpensive and independently routable/mixable. |
| Wavetable smooth interpolation | Geist wavetable engine with continuous frame interpolation. | Original anti-aliasing and interpolation. |
| Dual warp | Two-slot transform chain. | Define Geist order and modulation rules explicitly. |
| Sample/granular/spectral assets | Immutable asset-backed engines. | Decode/analyze off audio thread. |
| Multisample SFZ | Scoped SFZ importer. | Document supported subset; no assumption of complete vendor behavior. |
| Sample-to-wavetable | Offline conversion tool. | Frequency-estimation-like behavior can be original; source remains unchanged. |
| Dual filters | Two filter nodes with series/parallel routing. | Original filter models; Clean drive can be represented as a mode. |
| Two FX busses | Two post/source buses with ordered rack graphs. | Splitters require branching representation. |
| Mod matrix curves | Modulation graph edges with source/aux transfer functions. | Original depth math. |
| Osc/filter as mod sources | Audio-rate graph taps. | Requires scheduling/cycle rules. |
| Four envelopes/ten LFOs/eight macros | Fixed source pools. | Lazy UI reveal for LFO 7-10 optional. |
| Clip sequencer | MIDI/automation clip module. | Preview clips stored with presets. |
| Arpeggiator | MIDI event generator. | MIDI out belongs to host/timeline layer. |
| Browser | Indexed preset/content database. | Search/tags/ratings/previews/user metadata. |
| Import/rescan | User content root plus rescan command. | Avoid audio-thread IO. |
| Root-note mapping | Filename parser at sample import/load. | Use A3=440 convention or explicitly convert. |

## 13. Exact public gaps and follow-up questions

These are not implementation blockers; they are boundaries of the public source
material used here.

1. Complete primary-oscillator parameter lists, ranges, defaults, smoothing laws,
   and modulation scaling are absent from the public product page/PDF/support
   pages fetched.
2. Complete wavetable warp list, algorithms, order of dual warp, pre/post-unison
   placement, and interaction with oscillator/filter modulation sources are
   absent.
3. Wavetable editor/import/file format details are absent except for
   sample-to-wavetable Frequency Estimation behavior.
4. Sample oscillator supported audio formats, interpolation quality, loop mode
   list, snap-loop algorithm, slice detector, score extraction representation,
   tails/stretch algorithm, and warp parameters are absent.
5. Multisample SFZ opcode subset, zone-selection rules, round robin, velocity
   crossfades, articulation switching, disk streaming, and mapping editor details
   are absent.
6. Granular exact controls, grain-size/density/jitter/spread/randomization,
   envelope shapes, window functions, timbre-shift algorithm, and whether 256
   grains is per voice/osc/global are absent.
7. Spectral FFT/window/bin/harmonic model, transient detection algorithm,
   latency, analysis cache format, and parameter ranges are absent.
8. Sub oscillator waveforms, octave controls, phase, routing defaults, and ranges
   are absent.
9. Noise oscillator source list, file handling, pitch/keytracking, one-shot/loop
   behavior, phase/randomization, and ranges are absent.
10. Complete filter type list, cutoff/resonance ranges, drive amount/range,
    Clean drive behavior, routing order, and series/parallel mix law are absent.
11. FX: public docs state 13 effects but name only Bode/frequency shifter,
    Convolve/convolution, Delay, Distortion, Reverb, and Utility in the fetched
    sources; seven effect identities and all per-effect parameters are absent.
12. Splitter crossover frequencies, slopes, phase/latency handling, routing UI,
    and mid/side normalization are absent.
13. Modulation matrix depth math, source/aux combination, bipolar/unipolar
    conventions, per-destination units, row-order significance, smoothing, and
    visualization details are absent.
14. Envelope segment model, LFO shape/tool lists, chaos-mode parameters, macro
    range/labels, and macro apply/delete semantics are absent.
15. MPE/polyphonic expression is not mentioned in the fetched public sources.
16. Mixer pan law, level ranges, send laws, direct-output count, pre/post routing,
    and graph-cycle rules are absent.
17. Voicing details such as polyphony limits, mono/poly modes, voice allocation,
    legato priority, portamento curves/ranges, pitch-bend ranges, and retrigger
    rules are absent.
18. Clip sequencer file schema, grid values, PPQ/resolution, automation
    interpolation, MIDI-out mode list, record mode edge behavior, and preview
    storage schema are absent.
19. Arpeggiator shape list, pattern representation, timing/rate modes, chance
    semantics, gate/repeats/range defaults, retrigger modes, and arp-bank schema
    are absent.
20. Keyboard scale list, quantization rules, swing amount/range, affected-note
    rules, transpose default/endpoints, and mapping overlap rules are absent.
21. Preset schema, metadata fields, rating scale/storage, tag taxonomy, pack
    manifest, hybridize algorithm, auto-play behavior, database schema, and
    preview fallback content are absent.
22. Authorization protocol details and license file format are absent and are not
    relevant to Geist synth DSP.

## 14. Warnings for future implementers

- Public docs are feature-highlight material, not a full manual with complete
  parameter tables. Do not fill gaps with assumptions presented as Serum-derived
  facts.
- Screenshots/images on public pages were not used as sources for layout or
  parameter extraction; continue that practice unless legal review permits a
  specific accessibility/compatibility use.
- If new official Serum 2 manuals/support articles become public, update the
  provenance table with URL, fetched date, section names, and newly covered
  behavior before changing feature requirements.
- Keep implementation names, presets, wavetables, samples, multisamples, noise
  sources, preview clips, icons, and UI art original to Geist.
