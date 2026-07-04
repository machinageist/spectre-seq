<!--
Author: Jeff
Date: 2026-07-03
Description: Clean-room behavioral spec of a VCV-Rack-class modular environment for Geist planning
Notes: Derived from the public VCV Rack manual only; no VCV source code, assets, presets, or private behavior were used.
-->

# Modular Rack Clean-Room Spec

Behavioral specification for a Eurorack-style virtual modular environment,
derived exclusively from the public VCV Rack manual. This document records
observable behavior, user-facing conventions, and published developer-facing
contracts. It does not copy implementation, source structure, assets, presets,
screenshots, bundled art, or private compatibility behavior. Geist may adopt,
adapt, or reject any item; where Geist has a stronger project rule, Geist wins.

## Provenance, scope, and clean-room rules

Fetched/expanded: 2026-07-03.

Public source pages used, and only these pages:

1. <https://vcvrack.com/manual/GettingStarted>
2. <https://vcvrack.com/manual/MenuBar>
3. <https://vcvrack.com/manual/KeyCommands>
4. <https://vcvrack.com/manual/Core>
5. <https://vcvrack.com/manual/Polyphony>
6. <https://vcvrack.com/manual/RackPro>
7. <https://vcvrack.com/manual/PluginGuide>
8. <https://vcvrack.com/manual/Panel>
9. <https://vcvrack.com/manual/Manifest>
10. <https://vcvrack.com/manual/Presets>
11. <https://vcvrack.com/manual/VoltageStandards>
12. <https://vcvrack.com/manual/DSP>
13. <https://vcvrack.com/manual/Installing>
14. <https://vcvrack.com/manual/FAQ>
15. <https://vcvrack.com/manual/Migrate2>

Explicit exclusions:

- VCV Rack application/source repositories, headers, SDK files, binary packages,
  assets, screenshots, factory presets, and private documentation.
- Third-party module manuals/source except generic public manual examples.
- C++ API shape except where the manual exposes a user-visible behavior or a
  planning contract, such as labels, bypass, state, patch storage, or polyphony.
- Exact `.vcv`, `.vcvm`, `.vcvplugin`, SVG, package, or plugin ABI compatibility
  requirements for Geist. File extensions below are source facts, not Geist
  format requirements.
- Commercial/account/store behavior beyond observable user-facing menu effects.

## Manual page and section inventory covered

| Manual URL | Public sections covered in this spec |
| --- | --- |
| GettingStarted | first-launch template, Audio module, MIDI-to-CV, parameter gestures, voltage categories, patch cables, module movement, context menus, module browser, library recommendation |
| MenuBar | File/New/Open/Save/Save as/Save template/Revert/Quit; Edit/Undo/Redo/Clear cables; View/tooltips/zoom/cable opacity/tension/frame rate/fullscreen/lock cursor/knob mode/scroll-wheel knob control/lock module positions; Engine/performance meters/sample rate/threads; Library/Login/Update all; Help/Tips/User manual/Open user folder |
| KeyCommands | global commands, module commands, parameter commands, expression entry functions/constants |
| Core | Audio; MIDI input drivers; MIDI-CV; MIDI-CC; MIDI-Gate; MIDI-Map; MIDI output modules CV-MIDI/CV-CC/CV-Gate; Blank; Notes |
| Polyphony | cable model, example chain reaction, polyphonic tag, Split/Merge/Sum/Viz/Scope utility roles, zero-channel cables, performance rationale, non-voice uses |
| RackPro | plugin formats, supported/tested DAWs, instrument/effect variants, patch migration by save/open, audio I/O, Ableton and Logic routing notes, MIDI I/O, parameter automation, offline rendering, Pro Modules notice |
| PluginGuide | module component labels/descriptions, parameter display contracts, switches/buttons, bypass, module events, JSON-like custom state, patch storage, plugin settings, context menus, expanders/messages, dark panels, custom widgets/text/light layer/framebuffers, polyphony implementation expectations, SIMD/DSP library capabilities as behavioral guidance |
| Panel | setup dimensions/units, hardware-like panel design, IP warning, SVG limitations, component placeholder semantics, component color/type categories, generated-template workflow as public authoring behavior |
| Manifest | plugin-level metadata fields, module-level metadata fields, complete public tag vocabulary and meanings, hidden/deprecated module contract |
| Presets | factory module presets, stored content, excluded patch storage, preset path convention, reload timing, sort/display rule, partial parameter preset application |
| VoltageStandards | voltage levels, decibel conventions, output saturation, triggers/gates, timing/cable delay/reset guard, pitch/frequency mapping, NaN/infinity policy, polyphony channel rules, mono-module handling |
| DSP | signals, Fourier analysis, sampling, aliasing, IIR/FIR/impulse responses/brick-wall filters/windows/minimum phase/MinBLEP/PolyBLEP, circuit modeling/nodal analysis/ODE methods, profiling/math/compiler/memory/vector optimization |
| Installing | system requirements pointer, Mac/Windows/Linux install, plugin install, offline plugin transfer, third-party plugin warning, major-version compatibility, CLI options, environment variables |
| FAQ | name meaning, user folder paths, touch-screen setting, no multi-touch, plugin definition and plugin load folder |
| Migrate2 | v1-to-v2 compatibility claim, manifest version rule, parameter reset/randomize behavior, runtime font/image lifetime warning, optional v2 labels/buttons/switches/bypass/patch-storage/light-layer features |

## Feature coverage matrix

No row is marked “partial” without a concrete reason. “Speced” means the public
manual contains enough visible behavior for a Geist planning requirement. “Not a
Geist requirement” means the manual feature is recorded for awareness but should
not be cloned as a compatibility promise.

| Area | Source pages | Coverage status | Concrete source-bound notes for Geist |
| --- | --- | --- | --- |
| Getting Started | GettingStarted | Speced | Covers rack metaphor, first-launch template, Audio and MIDI bridge setup, signal categories, universal patching, module browser, and learning/library flow. |
| Menu | MenuBar | Speced, with three source gaps | All menu entries are covered. The public manual says “Documentation coming soon” for lock-cursor dragging, knob mode, and scroll-wheel knob control, so this spec records only their names and implied setting category, not hidden behavior. |
| Key Commands | KeyCommands | Speced | Covers global rack navigation, module hover commands, parameter gestures, reset, context exact-value field, and expression language names. |
| Core modules | Core | Speced | Covers all public Core categories and module behaviors listed in the page: Audio, MIDI input modules, MIDI output modules, Blank, Notes. |
| Polyphony | Polyphony, VoltageStandards, PluginGuide | Speced | Covers 0-16 channel cable semantics, zero-channel behavior, source configuration, chain reaction, poly tag, utility modules, broadcast/index/zero rules, mono fallback, SIMD rationale. |
| Rack Pro | RackPro | Speced for host planning; Pro effects content excluded | Host/plugin behavior is covered. The manual only states that Pro Modules are premium effects, so module-specific Pro effects are intentionally not speced. |
| Plugin/user-visible contracts | PluginGuide, Migrate2 | Speced | Covers labels, tooltips, parameter ranges/units/display transforms, buttons/switches, bypass, lifecycle, serialization, patch storage, settings, context menu, expanders/messages, theming, custom widgets, poly/SIMD/DSP guidance. |
| Panel | Panel, PluginGuide, Migrate2 | Speced for behavior; asset reproduction excluded | Covers dimensions, readability, component categories/placeholders, output visual distinction, SVG unsupported features. Exact VCV art/SVG helper implementation is not a Geist requirement. |
| Manifest | Manifest, Installing, FAQ, Migrate2 | Speced | Covers plugin/module metadata fields, slug/version/license/contact URLs/min host version, tags and hidden/deprecated metadata, major-version compatibility. |
| Presets | Presets, PluginGuide | Speced | Covers stored parameter/custom state, JSON-like preset nature, exclusion of patch-storage files, rescanning, sorting/prefix display, and partial application. |
| Voltage | VoltageStandards, GettingStarted | Speced | Covers audio/CV/pitch/gate/trigger/clock categories and published voltage ranges/timing/safety conventions. |
| DSP | DSP, VoltageStandards, PluginGuide | Speced as design guidance | Covers public algorithmic concepts without copying code. Geist should use independent implementations/tests. |
| Installing | Installing, FAQ | Speced for user data and runtime modes | Covers installs, plugins, user folder, CLI, env overrides, safe/headless/dev/screenshot/version modes. |
| FAQ | FAQ | Speced | Covers only the four FAQ entries present in the public page. |
| Migration | Migrate2 | Speced as compatibility lessons | Covers visible migration contracts useful for Geist: stable major versions, reset/randomize control, resource lifetime, labels, buttons/switches, bypass, large storage, light layer. |

## 1. Rack model and getting-started behavior

### 1.1 Product/rack metaphor

- **Behavior:** The environment is a virtual modular synthesizer platform that
  simulates Eurorack modules and can also include original modules beyond
  hardware. It presents a rack containing modules connected by patch cables.
- **Constraints:** A module may be a sound generator, processor, controller,
  utility, visualizer, bridge to hardware/host systems, text note, or spacer.
  The public manual does not require a fixed set of non-Core modules.
- **Data/project implications:** A project must represent modules, their spatial
  placement, parameter values, internal state, cable connections, and bridge
  configuration sufficient to restore a patch.
- **Realtime implications:** Audio/control signal processing must be separable
  from UI manipulation and file persistence.
- **Geist mapping notes:** Treat the rack metaphor as a device-graph design
  reference, not a mandate to mirror Rack UI, files, art, or plugin ABI.

### 1.2 First launch and template patch

- **Behavior:** On first launch after installation, the user sees a rack with a
  template patch. File > New later clears the current patch and loads the user
  template patch.
- **Constraints:** The source manual names Rack's template path, but Geist should
  not adopt that path as a compatibility promise.
- **Data/project implications:** A default/new-project template concept is useful;
  it should be user-replaceable and versioned separately from normal projects.
- **Realtime implications:** Loading a template is a graph replacement operation
  and should be performed off the realtime path, with an atomic graph swap if
  audio is active.
- **Geist mapping notes:** Provide a factory template plus a user template; keep
  template migration explicit.

### 1.3 Audio bridge setup

- **Behavior:** A Core Audio-style module is the portal between virtual voltages
  and the physical/host audio device. It sends rack signals to speakers/device
  outputs and receives microphone/device inputs. Users set driver, device,
  sample rate, and block size from the module display.
- **Constraints:** The manual's hardware drivers are Core Audio on macOS, WASAPI
  and ASIO on Windows, ALSA and JACK on Linux. Multiple Audio modules are
  described as experimental and potentially unstable.
- **Data/project implications:** Device selections, channel mapping, and sample
  settings are either project data or user/device preferences; Geist should
  decide which are portable.
- **Realtime implications:** Block size controls latency by `blockSize /
  sampleRate`. Sample-rate conversion is needed if device and engine rates
  differ, adding CPU use, latency, and fidelity tradeoffs.
- **Geist mapping notes:** Model hardware/host I/O as explicit bridge nodes;
  prefer one stable device graph unless Geist deliberately supports aggregate I/O.

### 1.4 MIDI-to-CV starter behavior

- **Behavior:** A MIDI-to-CV bridge converts notes from MIDI devices or the
  computer keyboard into rack voltages. The keyboard QWERTY and ZXCVB rows can
  generate notes, converted to 1V/oct pitch and gate.
- **Constraints:** Computer-keyboard MIDI works while the rack window has focus.
  The public Core page says four rows form a two-row virtual MIDI keyboard of
  about 2.5 octaves, shiftable with backtick and `1` keys.
- **Data/project implications:** Input driver selection and octave shift may need
  to be session/user state. Generated voltages enter the normal graph.
- **Realtime implications:** MIDI events must be timestamped/queued into the
  audio engine safely; note gates and retriggers must follow the module contract.
- **Geist mapping notes:** DAW keyboard/MIDI input should be routed through an
  explicit bridge, not hidden globals.

### 1.5 Basic parameter gestures

- **Behavior:** Users drag knobs/sliders vertically to adjust, hold Ctrl/Cmd for
  fine adjustment, right-click to edit/open context, and double-click to
  initialize/reset to default.
- **Constraints:** Shift accelerates adjustment; Ctrl/Cmd+Shift is very fine.
- **Data/project implications:** Parameter defaults, current values, labels,
  units, display transforms, and reset/randomize eligibility are project and
  metadata requirements.
- **Realtime implications:** UI parameter changes must be smoothed or delivered
  sample-accurately according to Geist policy, without unsafe realtime locking.
- **Geist mapping notes:** Implement exact-value entry and reset semantics early;
  they affect automation, tests, and preset application.

## 2. Signals, voltage standards, and timing

### 2.1 Signal categories

- **Behavior:** All cable-carried values are voltages. Categories are conventions:
  audio, CV, 1V/oct pitch, gate, trigger, and clock.
- **Constraints:** Any output may connect to any input regardless of category.
- **Data/project implications:** Store the connection as voltage lanes; optional
  port category metadata is for labels, tooltips, browser search, validation
  warnings, or color/UI, not for hard type safety.
- **Realtime implications:** A single processing representation can serve audio
  and control; modules decide how to interpret values.
- **Geist mapping notes:** Do not enforce typed cables unless Geist explicitly
  chooses a stricter UX; preserve universal patchability for modular workflows.

### 2.2 Levels and decibels

- **Behavior:** Oscillators and CV generators should typically produce 10 Vpp.
  Audio is typically ±5 V before bandlimiting; unipolar CV typically 0-10 V;
  bipolar CV typically ±5 V.
- **Constraints:** Absolute dB measurements should use dBFS with full scale
  defined by the manual as the -10 to 10 V range; 0 dBFS corresponds to 10 V,
  0 dBV to 1 V, and 0 dBVU to -18 dBFS by hardware convention.
- **Data/project implications:** Module specs should declare nominal voltage
  ranges and metering reference.
- **Realtime implications:** DSP must tolerate out-of-nominal values without
  assuming normalized ±1 audio.
- **Geist mapping notes:** Use volts internally at modular graph edges; convert
  to DAW normalized audio only at bridge boundaries.

### 2.3 Output saturation

- **Behavior:** Eurorack power is described as ±12 V, with protection diodes
  often limiting practical range to about ±11.7 V. Modules may model analog
  saturation but should not hard-clip simply for convenience.
- **Constraints:** Allowing voltages outside normal range is preferable to naïve
  hard clipping, because downstream modules may attenuate them and the Audio
  bridge may be the last clipper. Modules capable of >1x gain should consider
  saturating output.
- **Data/project implications:** Saturation should be part of module behavior,
  not a cable/storage constraint.
- **Realtime implications:** Saturation must be deterministic, finite, and cheap.
- **Geist mapping notes:** Avoid global cable clamps; add per-module saturators
  where musically expected.

### 2.4 Gates, triggers, and clocks

- **Behavior:** Gates are active at 10 V. Triggers are short gates, normally 10 V
  for 1 ms. Clocks are steady trigger streams used for patch timing.
- **Constraints:** Trigger inputs should use Schmitt/hysteretic behavior to avoid
  false retriggering from bandlimited ringing: low threshold about 0.1 V and
  high threshold around 1-2 V.
- **Data/project implications:** Trigger/gate input metadata should document
  thresholds and whether velocity/amplitude matters.
- **Realtime implications:** Provide reusable Schmitt trigger and pulse generator
  primitives. Timing uses sample time, not UI frame time.
- **Geist mapping notes:** Standardize edge detectors, pulse duration, and reset
  guards across all Geist modular devices.

### 2.5 Cable timing and reset guards

- **Behavior:** Each cable can induce a one-sample delay from output to input.
  Signals generated simultaneously may arrive one sample apart if their cable
  chain lengths differ.
- **Constraints:** Modules with CLOCK and RESET or similar inputs should ignore
  CLOCK triggers for up to 1 ms after a RESET trigger.
- **Data/project implications:** The graph need not store per-cable delay unless
  Geist exposes it, but tests should account for one-sample path differences.
- **Realtime implications:** Reset suppression must be sample-rate aware.
- **Geist mapping notes:** Make clock/reset arbitration a shared helper to avoid
  sequencer off-by-one behavior.

### 2.6 Pitch/frequency mapping

- **Behavior:** Frequency CV uses 1 V/octave: `f = f0 * 2^V`. Audio oscillators
  use C4/middle C/MIDI note 60/261.6256 Hz at 0 V. LFOs and clock generators use
  120 BPM / 2 Hz at 0 V.
- **Constraints:** Frequency knobs may offset voltage before conversion.
- **Data/project implications:** Store pitch parameters in volts/semitones with
  explicit display transforms.
- **Realtime implications:** Conversion must be stable and efficient; approximate
  exponential functions require audible-error tests.
- **Geist mapping notes:** Define canonical `v_to_hz`/`hz_to_v` helpers and use
  them in expression entry, automation display, and DSP.

### 2.7 Non-finite values

- **Behavior:** Modules that might produce NaN or infinity from finite inputs
  should output 0 V instead.
- **Constraints:** This applies especially to unstable IIR filters, reverbs, and
  feedback algorithms.
- **Data/project implications:** Project files should not persist non-finite
  numeric state.
- **Realtime implications:** Add finite checks at unstable boundaries or module
  outputs where profiling permits.
- **Geist mapping notes:** Test every module for finite output under extreme
  finite input/parameter cases.

## 3. Polyphony model

### 3.1 Cable channel count

- **Behavior:** Rack cables are polyphonic in the general case and carry 0-16
  channels. Mono is the 1-channel special case. Cables with 2-16 channels are
  visually thicker in Rack.
- **Constraints:** Zero-channel cables make modules treat a connection as
  effectively unpatched; modules can automate output channel count to virtually
  patch/unpatch.
- **Data/project implications:** Store channel count at graph edges/output state,
  not as a separate cable type. Preserve 0-channel semantics if virtual
  disconnection is desired.
- **Realtime implications:** Channel count changes are audio-rate/control-rate
  graph facts and must not reallocate in the realtime path.
- **Geist mapping notes:** Use fixed-capacity small arrays or slices up to 16 for
  modular lanes.

### 3.2 Source configuration and chain reaction

- **Behavior:** A user configures a source such as MIDI-CV for N polyphonic
  channels. Poly-aware downstream modules then become polyphonic as poly cables
  are patched through the signal chain.
- **Constraints:** Modules must be explicitly developed to support polyphony and
  are identified with a Polyphonic tag.
- **Data/project implications:** Module metadata must expose poly support for the
  browser and for patch-planning warnings.
- **Realtime implications:** A module should determine active engine count from a
  primary input or configured source without dynamic allocation.
- **Geist mapping notes:** Implement a clear per-module primary-input rule.

### 3.3 Secondary input mapping rules

- **Behavior:** For N active engines and a secondary input with M channels:
  monophonic M=1 broadcasts to all engines; M>=N maps channel i to engine i;
  1<M<N maps existing channels and supplies 0 V to out-of-range engines.
- **Constraints:** The manual recommends supporting up to 16 channels.
- **Data/project implications:** These rules should be part of the module conformance
  contract and automated tests.
- **Realtime implications:** Use branch-light helpers for channel reads.
- **Geist mapping notes:** Name this helper explicitly, e.g. `get_poly_voltage`.

### 3.4 Mono modules receiving poly input

- **Behavior:** Mono-only modules should handle poly input gracefully: audio-only
  inputs sum all channels; CV or hybrid audio/CV inputs read the first channel.
- **Constraints:** Do not crash, ignore connections unpredictably, or emit
  undefined channel counts.
- **Data/project implications:** Port metadata should distinguish audio-only from
  CV/hybrid for fallback behavior.
- **Realtime implications:** Summing up to 16 channels is bounded and cheap.
- **Geist mapping notes:** Provide default mono fallback policies.

### 3.5 Poly utility modules and non-voice uses

- **Behavior:** Public utility roles: Split decomposes one poly signal into mono
  outputs; Merge composes mono inputs into one poly output; Sum unity-mixes all
  channels; Viz/Scope display channels independently.
- **Constraints:** Poly cables are not limited to musical voices. Manual examples
  include stereo, surround, ambisonics up to 16 channels, digital buses,
  oversampled/video-style lanes, and cable-clutter reduction/teleporting with
  Merge/Split.
- **Data/project implications:** Channel labels may be voice numbers, stereo
  names, bus lanes, or custom labels; do not bake in “voice only”.
- **Realtime implications:** Utilities are simple but must preserve channel order.
- **Geist mapping notes:** Build Split/Merge/Sum and channel-aware scopes early;
  they are debugging infrastructure.

### 3.6 Poly performance expectation

- **Behavior:** Poly modules are expected to be more efficient than duplicating N
  mono modules, often through SIMD or compiler vectorization.
- **Constraints:** The manual describes up to 4x float SIMD on x64/SSE-style
  vectors as an example, not a required Geist implementation.
- **Data/project implications:** No project implications except stable behavior
  across scalar/SIMD backends.
- **Realtime implications:** SIMD must be feature-gated and matched against scalar
  tests within tolerance.
- **Geist mapping notes:** Implement scalar first; optimize under conformance tests.

## 4. User interaction, key commands, and menus

### 4.1 Patching gestures

- **Behavior:** Drag port to port to create a cable. Drag a cable plug to move or
  delete an end. An input accepts one cable; an output can fan out to multiple
  cables. Holding Ctrl/Cmd while dragging from an output stacks additional cables.
- **Constraints:** Signal category does not restrict patching.
- **Data/project implications:** Enforce one incoming cable per input and multiple
  outgoing cables per output in the graph model.
- **Realtime implications:** Patch edits require graph rebuild/swap without
  blocking the audio callback.
- **Geist mapping notes:** These are core graph invariants, not merely UI details.

### 4.2 Module browser and module manipulation

- **Behavior:** Enter or right-click empty rack space launches the Module Browser.
  Right-click module panel opens its context menu. Drag panel moves module.
  Ctrl/Cmd-drag force-drags, moving other modules to place the dragged module.
  Backspace/Delete deletes the hovered module; holding the key while moving can
  delete multiple modules. Ctrl/Cmd+V pastes a copied module preset as a new
  module at the cursor.
- **Constraints:** Module commands require the mouse to hover the module.
- **Data/project implications:** Module placement, deletion, paste source state,
  and context actions must be undoable project edits.
- **Realtime implications:** Delete/paste are graph mutations and must be audio-safe.
- **Geist mapping notes:** Keep a searchable device browser with stable metadata;
  spatial rack movement can map to Geist's native UI layout.

### 4.3 Rack navigation

- **Behavior:** Scroll pans vertically; Shift-scroll pans horizontally;
  Ctrl/Cmd-scroll zooms; middle-button drag pans; arrow keys pan with modifiers
  changing speed. Ctrl/Cmd+0 resets zoom to 100%; Ctrl/Cmd+- zooms out;
  Ctrl/Cmd+= zooms in. Menu zoom range is 25-400%. Mouse buttons #4/#5 can zoom
  on multi-button mice. F11 toggles fullscreen.
- **Constraints:** Zoom double-click resets menu value to 100%.
- **Data/project implications:** View state may be persisted separately from
  audible project state.
- **Realtime implications:** None except avoiding UI render work on audio thread.
- **Geist mapping notes:** Adopt native DAW navigation where appropriate; keep
  key equivalents discoverable.

### 4.4 Tooltips and parameter context entry

- **Behavior:** Tooltips can display names, values, descriptions, units, and
  whether a component is an input/output/light. Users can toggle tooltips.
  Right-clicking a parameter opens a context menu with exact-value entry.
- **Constraints:** Expressions are evaluated when Enter is pressed. Supported
  public names include note frequencies (`C`, `A#`, `Gb`, with octaves like
  `C4`), note voltages like `C4v` where C4v=0 V, `log2(x)`, `gaintodb(x)`,
  `dbtogain(x)`, `vtof(x)`, and `ftov(x)`. Function/constant names are
  case-insensitive. Examples include `2+2`, `2*2`, `1/2`, and `2^2`.
- **Data/project implications:** Parameter metadata needs label, unit,
  description, range, default, display base/scale/offset, and enumerated choices
  for switches.
- **Realtime implications:** Expression parsing happens on UI/control thread.
- **Geist mapping notes:** Use a safe expression parser. Do not evaluate a host
  programming language.

### 4.5 View menu settings

- **Behavior:** View menu contains Show tooltips, Zoom, Cable opacity, Cable
  tension, Frame rate, Fullscreen, Lock cursor when dragging params, Knob mode,
  Scroll wheel knob control, and Lock module positions.
- **Constraints:** Cable opacity and cable tension double-click reset to 50%.
  Frame rate redraws on every monitor vsync or every N vsyncs; lowering it
  roughly proportionally lowers GPU use at the cost of choppier motion. Lock
  module positions prevents accidental mouse movement of modules. The public
  manual currently says “Documentation coming soon” for lock-cursor dragging,
  knob mode, and scroll-wheel knob control, so behavior beyond their names is
  not specified here.
- **Data/project implications:** These are mostly user preferences, not portable
  patch audio state, except view state if Geist wants project-specific views.
- **Realtime implications:** Frame-rate and tooltip rendering must stay off audio.
- **Geist mapping notes:** Keep UI preferences separate from project data.

### 4.6 File/Edit/Engine/Library/Help menus

- **Behavior:** File New clears patch and loads template; Open/Save/Save as use a
  `.vcv` patch in Rack; Save template writes the current patch as template;
  Revert restores last saved state; Quit autosaves and closes. Edit Undo/Redo
  rewinds and replays all patch-editing actions; Clear cables removes all
  cables. Engine Performance meters measure per-module sample generation time;
  Sample rate sets engine sample rate; Threads sets multithreaded engine core
  count. Library Login signs into VCV account; Update all downloads/updates all
  account plugins and requires restart. Help Tips opens tips; User manual opens
  manual; Open user folder opens the user data folder.
- **Constraints:** Rack autosaves every 15 seconds. CPU/performance meters consume
  engine CPU themselves and should be disabled when not needed. Engine sample
  rate determines one-sample step duration and CPU use is roughly proportional.
  Recommended thread strategy is start at one, increase until no hiccups, and
  usually not exceed physical cores; SMT/hyperthread overuse may worsen power
  and performance.
- **Data/project implications:** Undo/redo must cover all project edits, including
  patching, module operations, parameter changes if Geist chooses Rack parity,
  and clear cables. Autosave files are recovery data, not authoritative saves.
- **Realtime implications:** Autosave and plugin update must not block audio.
  Performance meters must be optional due to measurement overhead.
- **Geist mapping notes:** Implement autosave as incremental/background snapshot;
  expose engine sample rate/thread controls only if Geist's architecture supports
  them meaningfully.

## 5. Session, installation, folders, and runtime modes

### 5.1 User folder

- **Behavior:** The Rack user folder stores readable/writable data. The manual
  provides defaults: macOS `~/Library/Application Support/Rack2/`, Windows
  `C:\Users\<username>\AppData\Local\Rack2\`, Linux `~/.local/share/Rack2/`.
  Help > Open user folder opens it. In development mode, it is the current
  working directory.
- **Constraints:** Plugins are loaded from a platform/CPU-specific plugin folder
  under the user folder.
- **Data/project implications:** Geist should define its own platform-specific
  user data roots and avoid relying on Rack paths.
- **Realtime implications:** User-folder scans should not occur on audio thread.
- **Geist mapping notes:** Separate settings, plugin packages, caches, templates,
  autosaves, screenshots, and projects.

### 5.2 Installing Rack and plugins

- **Behavior:** Public install flows: macOS installer then Applications folder;
  Windows installer then desktop/start menu, with Rack Pro allowing custom VST
  path; Linux unzip, ensure `zenity`, double-click binary or run `./Rack`.
  Plugins usually install via VCV Library. Offline transfer can copy the
  platform/CPU plugin folder from another computer. Third-party `.vcvplugin`
  packages can be placed in the plugin folder and are extracted/loaded at launch.
- **Constraints:** Third-party plugins from unknown sources are a security risk.
  Plugin major version must match Rack major version.
- **Data/project implications:** Package provenance, trust, signature, version,
  and compatibility metadata matter for any plugin ecosystem.
- **Realtime implications:** Plugin extraction/loading is launch/control-time work.
- **Geist mapping notes:** If Geist supports modular extensions, sandbox/sign or
  gate them; do not allow arbitrary code loading silently.

### 5.3 Command-line/runtime modes

- **Behavior:** CLI can load a patch filename. Options documented publicly:
  `-s/--system` for system folder, `-u/--user` for user folder, `-d/--dev` for
  development mode, `-h/--headless` to launch autosaved patch with no window,
  `-a/--safe` to launch with no plugins/autosave patch, `-t/--screenshot <zoom>`
  to capture module screenshots, and `-v/--version` to print version and exit.
  Environment variables `RACK_SYSTEM_DIR` and `RACK_USER_DIR` can override
  locations.
- **Constraints:** Headless is described as useful for generative installations
  controllable by MIDI. Safe mode is useful for testing. Screenshot zoom factor
  of 1 yields 380 px panel height in Rack.
- **Data/project implications:** Headless, safe, and custom user folder modes are
  strong testing and recovery features.
- **Realtime implications:** Headless must run without UI timing assumptions.
- **Geist mapping notes:** Provide equivalent test/diagnostic modes, but do not
  copy screenshot asset workflows.

### 5.4 Touch screen FAQ

- **Behavior:** Touch screens work if the user disables View > Lock cursor while
  dragging params and optionally sets Knob mode to rotary. Multi-touch gestures
  are not currently supported by Rack.
- **Constraints:** The manual exposes no deeper touch contract.
- **Data/project implications:** Touch preferences are user/UI settings.
- **Realtime implications:** None.
- **Geist mapping notes:** If Geist targets touch, design a native multi-touch
  model rather than inheriting Rack's limitation.

## 6. Core modules and bridge contracts

### 6.1 Core plugin category

- **Behavior:** The built-in Core set provides utilities and interfaces between
  virtual and hardware worlds: Audio, MIDI input modules, MIDI output modules,
  Blank, and Notes.
- **Constraints:** The manual page lists only those categories; other first-party
  non-Core modules are not part of this Core spec.
- **Data/project implications:** Built-in devices should be addressable by stable
  IDs/slugs in projects.
- **Realtime implications:** Bridges must mediate between block/event external
  systems and sample-stepped modular processing.
- **Geist mapping notes:** Treat bridge modules as explicit graph endpoints.

### 6.2 Audio module

- **Behavior:** To Device sends rack signals to hardware/host outputs; From Device
  receives external audio into Rack. Driver is selected first, then a device,
  then sample rate and block size.
- **Constraints:** Manual drivers: Core Audio/macOS, WASAPI/Windows, ASIO/Windows,
  ALSA/Linux, JACK/Linux. Device sample rate differs from engine sample rate;
  mismatches cause SRC with extra CPU, slightly lower fidelity, and latency.
  Multiple Audio modules are experimental and may crash or produce unstable audio.
- **Data/project implications:** Channel count and mapping must be stored or
  recoverably resolved. Hardware device names may not be portable.
- **Realtime implications:** SRC and buffering must be bounded. Low block size
  increases scheduling risk; high block size increases latency.
- **Geist mapping notes:** Prefer one host/device clock source and deterministic
  channel mapping.

### 6.3 MIDI input common drivers

- **Behavior:** MIDI input modules support OS/device drivers: Core MIDI/macOS,
  Windows MIDI/Windows, ALSA/Linux, JACK/Linux, Gamepad, and Computer keyboard.
- **Constraints:** Gamepad maps buttons to MIDI note gates starting at C-1, C#-1,
  D-1, etc., and joystick axes to CC0, CC1, etc. with nonstandard negative CC
  values. Computer keyboard generates notes only while focused.
- **Data/project implications:** Driver/device selections and learned CC/note maps
  need persistence; physical availability may differ across machines.
- **Realtime implications:** MIDI/gamepad/keyboard events must be queued safely.
- **Geist mapping notes:** Normalize all external controllers into timestamped
  musical/control events before voltage conversion.

### 6.4 MIDI-CV

- **Behavior:** V/OCT outputs 1V/oct pitch for the last held note. GATE outputs
  10 V while a key is held and does not retrigger on legato. VEL outputs 0-10 V
  velocity. AFT outputs channel-pressure aftertouch CV, not polyphonic
  aftertouch. PW outputs -5 to +5 V pitch wheel. MW outputs mod wheel CV. CLK
  outputs one trigger for every received 24-PPQN MIDI clock. CLK/N outputs a
  divided clock configured in the panel context menu. RTRG outputs a trigger
  whenever a new note is pressed, including legato. STRT/STOP/CONT output
  triggers for MIDI transport events.
- **Constraints:** Right-click panel enables polyphony and selects channel count
  and allocation mode. Rotate chooses next available channel, or next channel if
  none are available, wrapping. Reuse reuses a channel previously used by the
  same MIDI note, otherwise uses Reset behavior. Reset chooses lowest available
  channel; note release shifts higher channels down. Multiple MIDI interfaces can
  be used with the same driver to combine functions such as MIDI-CV and MIDI-CC.
- **Data/project implications:** Store poly channel count, allocation mode, CLK/N
  division, driver/device, and possibly channel filters if Geist adds them.
- **Realtime implications:** Voice allocation must be deterministic and event-
  ordered; transport and 24-PPQN clock conversion must use sample timestamps.
- **Geist mapping notes:** Implement allocation modes exactly if claiming Rack-like
  behavior; otherwise label Geist modes distinctly.

### 6.5 MIDI-CC

- **Behavior:** Each output maps one MIDI CC number to CV. Standard CC 0-127 maps
  to 0-10 V. Nonstandard negative gamepad values -128 to 127 map to -10 to 10 V.
  Clicking a display enters learn mode; the user types a number or moves a
  controller to assign the CC.
- **Constraints:** 14-bit MIDI CC is not supported in the manual.
- **Data/project implications:** Persist CC assignment per output slot and whether
  a slot is learning as transient UI state.
- **Realtime implications:** CC-to-voltage changes are event/control changes; apply
  smoothing only if module design says so.
- **Geist mapping notes:** Consider 14-bit as an extension, not Rack parity.

### 6.6 MIDI-Gate

- **Behavior:** Each output maps a MIDI note to a 10 V gate while held. Immediate
  note-on/off messages from drum machines or sequencers produce a 1 ms trigger.
  Velocity mode scales output by note velocity instead of fixed 10 V.
- **Constraints:** Velocity mode is enabled from panel context.
- **Data/project implications:** Persist note assignments and velocity-mode flag.
- **Realtime implications:** Immediate on/off detection must still produce a
  sample-accurate 1 ms pulse.
- **Geist mapping notes:** This module is useful for drum rack bridging.

### 6.7 MIDI-Map

- **Behavior:** A hardware MIDI CC can control any on-screen parameter. User
  clicks an unmapped slot, clicks a parameter, and moves a hardware control; the
  parameter and control steps may occur in either order.
- **Constraints:** The module is parameter-control oriented, not arbitrary signal
  routing.
- **Data/project implications:** Mappings require stable parameter identity across
  project save/load and module reorder/move.
- **Realtime implications:** Hardware control changes must update parameters in a
  thread-safe, automatable path.
- **Geist mapping notes:** Prefer a host-wide modulation/MIDI learn system with
  stable IDs; expose bridge modules only if modular self-containment is needed.

### 6.8 MIDI output modules

- **Behavior:** CV-MIDI converts rack CV to MIDI notes/events for external
  hardware. CV-CC converts rack CV to MIDI CC commands. CV-Gate converts rack
  gates to MIDI note on/off commands. The manual frames them as useful for
  hardware synths, parameter control, and drum machines.
- **Constraints:** The Core page does not specify exact voltage-to-message edge
  rules for these outputs beyond these descriptions.
- **Data/project implications:** Persist driver/device/channel/controller/note
  configuration in Geist's own schema.
- **Realtime implications:** Voltage-to-MIDI event generation must debounce gates,
  quantize/control-rate CC as needed, and avoid flooding outputs.
- **Geist mapping notes:** Treat exact conversion rules as Geist design work unless
  another public source specifies them.

### 6.9 Blank and Notes

- **Behavior:** Blank adds space between modules and resizes horizontally with a
  minimum width of 3 HP. Notes stores patch notes, section titles, organization,
  instructions, and author info; text supports copy/paste with Ctrl+C/Ctrl+V.
- **Constraints:** Blank is layout-only; Notes is patch text metadata.
- **Data/project implications:** Blank width and Notes text must persist in the
  patch/project. Notes may affect collaboration/export documentation.
- **Realtime implications:** None; do not involve audio processing.
- **Geist mapping notes:** Provide native annotations/section labels in projects.

## 7. Rack Pro / DAW-host behavior reference

### 7.1 Plugin formats and DAW support

- **Behavior:** Rack Pro can run standalone or as DAW plugins. Public formats are
  VST2, VST3, Audio Unit on macOS, and CLAP. Publicly tested/supported DAWs are
  Ableton Live, Cubase, FL Studio, Reason, Bitwig, Reaper, Mixbus, Studio One,
  Cakewalk, Logic Pro, and GarageBand. Other DAWs may run it but are unsupported.
- **Constraints:** All formats can be instrument or effect plugins. Both audio and
  MIDI can be routed to Rack regardless of type if the DAW allows it.
- **Data/project implications:** A patch can migrate across plugin formats/DAWs by
  saving a `.vcv`, replacing plugin, and opening the saved patch.
- **Realtime implications:** Plugin operation follows host callbacks and block
  sizes; offline rendering may run faster than realtime.
- **Geist mapping notes:** Geist's DAW integration should not couple project data
  to plugin format identity.

### 7.2 Plugin audio I/O

- **Behavior:** Rack plugin formats expose 16 audio inputs and 16 audio outputs,
  organized as 8 stereo pairs. The DAW driver in VCV Audio accesses plugin I/O.
  Channels 1-2 are the main channel-strip pair; 3-16 can route to/from other
  tracks in capable DAWs. All channels are available in instrument and effect
  variants.
- **Constraints:** Ableton routes via Audio To/From track and stereo pair with
  monitoring for returns. Logic multi-output instruments use an 8xStereo variant
  and Aux tracks for outputs 3-4, 5-6, etc. Logic sidechain gives one stereo
  input: channels 1-2 for instrument, 3-4 for effect, and Audio Unit cannot
  receive more than one stereo sidechain input.
- **Data/project implications:** Store audio bridge channel assignments by stable
  pair/channel identifiers, not host-specific track names.
- **Realtime implications:** Host input/output buffers are block based; bridge to
  sample-stepped modular graph deterministically.
- **Geist mapping notes:** Design for at least 16x16 host modular I/O if matching
  this planning reference.

### 7.3 Plugin MIDI I/O

- **Behavior:** DAW MIDI can feed Rack MIDI input modules. Public message classes:
  channel data on MIDI channels 1-16 including notes, pitch wheel, aftertouch,
  and CC; transport start/stop/continue and 24-PPQN clock; song position; SysEx.
  Rack MIDI output modules can send MIDI to DAW tracks, other plugins, clips, or
  external hardware.
- **Constraints:** Ableton routes all MIDI data to channel 1, per the manual note.
- **Data/project implications:** Host MIDI routing and channel assumptions should
  be explicit and inspectable.
- **Realtime implications:** Host MIDI event timestamps must align to audio block
  offsets and transport state.
- **Geist mapping notes:** Do not hide DAW transport; expose it through bridge
  nodes/signals for modular determinism.

### 7.4 Parameter automation

- **Behavior:** A DAW can automate any module parameter. The user records
  automation by enabling DAW automation recording, recording, and moving a Rack
  parameter. Playback moves the parameter as recorded. DAW MIDI assignment can
  map hardware controls/notes to Rack parameters.
- **Constraints:** Deleting a module does not delete DAW automation; automation
  becomes ineffective and may later affect arbitrary parameters if automation
  slots are reused. Users are advised to delete automation clips for deleted
  modules.
- **Data/project implications:** Geist needs stable automation identity that is
  not accidentally reused for unrelated future parameters.
- **Realtime implications:** Automation playback must be sample/block accurate
  according to host capabilities and thread safe.
- **Geist mapping notes:** Use persistent per-parameter automation IDs with tombstone
  handling rather than slot reuse.

### 7.5 Offline rendering

- **Behavior:** DAW render/bounce/bake/mixdown/export can render Rack faster than
  realtime. Manual example: a patch using about 10% CPU can render a 10-minute
  song in about 1 minute.
- **Constraints:** Offline processing must not assume wall-clock realtime.
- **Data/project implications:** Project state must be deterministic under
  non-realtime advancement.
- **Realtime implications:** Timers, animations, random sources, MIDI clock, and
  transport must derive from sample/host time, not UI time.
- **Geist mapping notes:** Offline render conformance is mandatory for DAW export.

## 8. Module lifecycle, state, and plugin/user contracts

### 8.1 Component labels, descriptions, and tooltips

- **Behavior:** Inputs, outputs, and lights can have labels used in tooltips;
  tooltip text appends input/output/light, so labels should not include those
  words. Light tooltips are recommended only when meaning is not obvious.
  Components can also have one-line descriptions.
- **Constraints:** Labels and descriptions are user-facing metadata and should be
  short enough for tooltips.
- **Data/project implications:** Store metadata in module definitions, not per
  project unless user-editable.
- **Realtime implications:** None.
- **Geist mapping notes:** Make every port/parameter/light discoverable and
  searchable with labels/descriptions.

### 8.2 Parameter configuration

- **Behavior:** A parameter has min/max/default, label, optional unit, and display
  transform/scale. Switches can show named choices in right-click menus. Buttons
  are momentary-style controls with range 0-1/default 0 and no arbitrary numeric
  entry in the v2 guidance.
- **Constraints:** Examples in the manual include voltage units, percent scaling,
  1V/oct exponential display to Hz, and logarithmic gain display to dB.
- **Data/project implications:** Parameter metadata must include range, default,
  unit, display transform, choice labels, resetEnabled/randomizeEnabled policy,
  and automation identity.
- **Realtime implications:** Parameter value representation should be normalized
  only at UI/automation boundaries; DSP should receive meaningful units.
- **Geist mapping notes:** Separate raw value, display value, and modulation value.

### 8.3 Reset and randomize

- **Behavior:** Users can initialize/reset and randomize modules. Default reset
  resets parameters; modules can reset custom state. Randomize can randomize all
  parameters or custom behavior. v2 migration guidance allows disabling reset or
  randomization for individual parameters.
- **Constraints:** Custom behavior should be explicit and least surprising.
- **Data/project implications:** Presets and undo must capture pre/post state.
- **Realtime implications:** Reset/randomize are graph/control events; produce a
  safe state snapshot for DSP.
- **Geist mapping notes:** Add per-parameter reset/randomize eligibility.

### 8.4 Sample-rate changes and module events

- **Behavior:** Modules may receive events for reset, randomize, sample-rate
  change, add, save, and other lifecycle moments. Sample-rate changes require
  custom state such as coefficients, timers, and buffers to update.
- **Constraints:** The public plugin guide states Rack does not call module
  methods simultaneously from multiple threads, but Geist should not rely on that
  exact threading model.
- **Data/project implications:** Events define safe times to load/save external
  data and rebuild derived state.
- **Realtime implications:** Reconfiguration must avoid unsafe concurrent mutation
  of active DSP state.
- **Geist mapping notes:** Use immutable DSP snapshots or lock-free handoff.

### 8.5 Serialization and JSON-like state

- **Behavior:** Saving/closing stores module state. Parameter values are stored
  automatically in Rack; additional module instance variables require custom
  serialization. Publicly mentioned JSON value classes: bool, string, 64-bit
  integer, real/number, arrays, and objects with string keys.
- **Constraints:** Integer-looking numbers may need to parse as numeric real values
  when reading custom state. Geist need not use Jansson or JSON internally.
- **Data/project implications:** Distinguish parameter state, custom module state,
  plugin/global settings, patch storage, and large assets.
- **Realtime implications:** Serialization must run outside audio processing.
- **Geist mapping notes:** Use a schema-versioned project format with migration.

### 8.6 Patch storage for large data

- **Behavior:** Large module data above roughly 100 kB should not be serialized
  directly into frequent JSON autosaves because it may lag the main thread.
  Modules can use per-patch storage directories for arbitrarily large files,
  including gigabyte recordings; Rack compresses module patch storage into the
  `.vcv` on save. Modules usually load/save such files on add/save events.
- **Constraints:** File access from the audio process method is possible in Rack
  but warned against because it can block DSP and cause audio hiccups. Patch
  storage methods are unavailable in the constructor because the module is not
  yet added to the engine, per migration guidance.
- **Data/project implications:** Project bundles need sidecar/blob management,
  content addressing or manifest entries, save-as copy semantics, and autosave
  throttling.
- **Realtime implications:** Never perform blocking file I/O on audio thread.
- **Geist mapping notes:** Build a media/blob store separate from parameter JSON.

### 8.7 Plugin/global settings

- **Behavior:** Plugin-scope settings can be saved into the user settings file,
  auto-saved on quit and periodically every 15 seconds by default.
- **Constraints:** Plugin settings are not per-patch module state.
- **Data/project implications:** Separate user preferences from project data.
- **Realtime implications:** Settings autosave should be background/control thread.
- **Geist mapping notes:** Use profile-level settings for UI/device/plugin prefs.

### 8.8 Bypass

- **Behavior:** Users can bypass a module using context menu or key command.
  Bypass disables normal DSP and may route certain inputs directly to certain
  outputs as if jumper cables replace processing.
- **Constraints:** Bypass routes should be least surprising and only connect
  semantically related paths: audio input to filtered audio output, CV input to
  inverted CV output, gate input to delayed gate output. Inappropriate examples:
  pitch CV to sine audio output, mixer channel 1 to mixer output, clock input to
  sequencer trigger output. Complex bypass may implement behavior such as stereo
  normaling.
- **Data/project implications:** Store bypass state and route definitions in the
  module definition/project. Bypass must be undoable and automatable if exposed.
- **Realtime implications:** Bypass switching should be click-safe where audio is
  involved and must preserve channel semantics.
- **Geist mapping notes:** Model bypass at graph level with deterministic pass-
  through or mute policies per device.

### 8.9 Context menus

- **Behavior:** Right-clicking a module panel opens a context menu with common
  actions like Initialize, Randomize, and Delete; modules may append custom menu
  items at the end, normally separated visually. Public examples include labels,
  action items, bool toggles, submenus, and indexed mode choices.
- **Constraints:** Context settings are for options not available on the panel.
- **Data/project implications:** Context-menu settings that affect sound must be
  serialized and undoable; purely UI actions may be transient.
- **Realtime implications:** Menu actions are control-thread operations.
- **Geist mapping notes:** Prefer inspectable parameter/modulation model for sound
  settings; use context menus for less-frequent configuration.

### 8.10 Expanders and messages

- **Behavior:** Adjacent modules can communicate like hardware rear-panel bus
  expanders. Communication is based on touching the left or right side. An
  expander should clear outputs/lights when disconnected.
- **Constraints:** Direct cross-module state access can have ambiguous process
  order under multithreading, with inconsistent 0/1 frame latency or corrupt
  reads. Public guidance uses double-buffered messages to guarantee one engine
  frame of latency.
- **Data/project implications:** Project layout adjacency may affect behavior;
  this is a strong reason to represent expander relationships explicitly.
- **Realtime implications:** Cross-module communication must be deterministic and
  race-free.
- **Geist mapping notes:** Prefer explicit graph edges or declared side-buses over
  physical adjacency magic.

### 8.11 Dark panels, custom widgets, text, light layer, framebuffer

- **Behavior:** Users can request dark panels if available. Modules can contain
  custom displays/waveforms/visualizations; dynamic text can be drawn by widgets;
  custom light-like widgets can draw at full brightness on a self-illuminating
  layer when rack brightness is reduced; expensive custom widgets can cache to a
  framebuffer and mark it dirty when redraw is needed.
- **Constraints:** Migration warns not to store font/image references across
  frames because plugin editor windows can recreate OpenGL contexts; store paths
  and fetch cached resources per draw. Exact graphics APIs/assets are not Geist
  requirements.
- **Data/project implications:** Theme preference is user state; widget content is
  derived from module state unless the widget edits data.
- **Realtime implications:** Rendering/caching must never block DSP.
- **Geist mapping notes:** Use DAW-native theme/render abstractions; preserve the
  behavior of readable dynamic displays and full-bright status indicators.

## 9. Panel/UI conventions

### 9.1 Dimensions and units

- **Behavior:** Public panel setup uses millimeters, 128.5 mm height, and width as
  a multiple of 5.08 mm (1 HP).
- **Constraints:** SVG document units/display units should be mm; px is not
  supported in the Rack panel guide.
- **Data/project implications:** Module width/HP must be known for rack layout and
  spacer behavior.
- **Realtime implications:** None.
- **Geist mapping notes:** Geist can use native UI sizes but should preserve
  module width/layout metadata if rack-style arrangement exists.

### 9.2 Design recommendations

- **Behavior:** Design panels like hardware, leave enough space between knobs and
  ports for fingers/thumbs, use visually distinguishable output ports, keep labels
  succinct, and make text readable at 100% zoom on non-high-DPI monitors.
- **Constraints:** Do not use others' IP without permission. The manual recommends
  community/design help but that is process advice, not product behavior.
- **Data/project implications:** Labels should be short, stable, and translatable
  if Geist localizes.
- **Realtime implications:** None.
- **Geist mapping notes:** Preserve semantic clarity, not Rack visual style.

### 9.3 SVG and component authoring constraints

- **Behavior:** Rack panel SVG renderer does not support all SVG. Text/fonts must
  be converted to paths; simple two-color linear gradients may work; CSS is not
  supported except many fill/stroke inline style properties. Component graphics
  are not included on the panel art; placeholders mark params, inputs, outputs,
  lights, or custom widgets.
- **Constraints:** Placeholder shape/color conventions from the public page: circle
  for centered components, rectangle for top-left/size-defined components; red
  param, green input, blue output, magenta light, yellow custom widget. Components
  may be named to generate code templates.
- **Data/project implications:** Component positions and sizes define hit targets
  and layout; graphics pipeline details should not be project data.
- **Realtime implications:** None.
- **Geist mapping notes:** Do not copy SVG/helper workflow. Use it only as evidence
  that modules need component type, label, position, size, and visual affordance.

## 10. Manifest, catalog, library, and tags

### 10.1 Plugin/package metadata

- **Behavior:** A plugin is a single software unit, typically by one company or
  individual, containing multiple Rack modules. Public manifest fields include
  `slug`, `name`, `version`, `license`, `brand`, `description`, `author`,
  `authorEmail`, `authorUrl`, `pluginUrl`, `manualUrl`, `sourceUrl`,
  `donateUrl`, `changelogUrl`, `minRackVersion`, and `modules`.
- **Constraints:** Slug is immutable, case-sensitive, and uses letters, numbers,
  hyphens, and underscores. Version is `MAJOR.MINOR.REVISION`; the major version
  must match Rack major version for compatibility in the Rack ecosystem. License
  should preferably use SPDX for open-source content. Brand is a prefix shown for
  modules. Description is one line. URLs/emails are optional/support metadata as
  documented by the manifest page.
- **Data/project implications:** Stable plugin slugs and versions are required for
  project references and migration.
- **Realtime implications:** None.
- **Geist mapping notes:** Use immutable IDs and semantic compatibility versions;
  keep catalog metadata independent of binary ABI if Geist has no Rack plugins.

### 10.2 Module metadata

- **Behavior:** Module fields include slug, name, tags, description, keywords,
  manual URL, ModularGrid URL, and hidden flag/deprecation information.
- **Constraints:** Module slug is immutable within the plugin. Name is user-facing.
  Tags come from a fixed vocabulary. Keywords improve search but are hidden from
  users. Hidden modules can represent deprecated/internal modules; deprecation
  text should point users to a successor.
- **Data/project implications:** Project files should reference stable slugs/IDs,
  not display names. Hidden/deprecated modules must still load old projects.
- **Realtime implications:** None.
- **Geist mapping notes:** Browser search should include brand, name, description,
  tags, and hidden keywords; hidden modules remain loadable by projects.

### 10.3 Complete public tag vocabulary and meanings

The public Manifest page lists these tag names. Geist may adopt this vocabulary
or map it to its own taxonomy, but all source-visible categories are covered:

| Tag | Meaning for planning/search |
| --- | --- |
| Arpeggiator | Generates note patterns from held notes/chords. |
| Attenuator | Reduces or scales signal level/CV. |
| Blank | Spacer/empty panel. |
| Chorus | Chorus/modulated-delay effect. |
| Clock generator | Produces timing clocks. |
| Clock modulator | Divides, multiplies, gates, shifts, or otherwise processes clocks. |
| Compressor | Dynamic-range compression. |
| Controller | Human or external control source. |
| Delay | Delay/echo/time-offset effect. |
| Digital | Explicitly digital algorithm/character. |
| Distortion | Nonlinear distortion/saturation. |
| Drum | Percussion/drum sound or trigger-oriented module. |
| Dual | Two related units/channels. |
| Dynamics | Dynamics processing beyond only compression. |
| Effect | General audio/CV processor. |
| Envelope follower | Extracts amplitude/envelope from input. |
| Envelope generator | Produces envelope CV from gate/trigger. |
| Equalizer | Frequency-band gain shaping. |
| Expander | Adjacent module extending another. |
| External | Interfaces external hardware/software. |
| Filter | Frequency-selective processing. |
| Flanger | Flanging/modulated comb effect. |
| Function generator | Generates slopes/functions. |
| Granular | Granular synthesis/processing. |
| Hardware clone | Inspired by or emulates hardware; beware IP/art boundaries. |
| Limiter | Dynamics limiting. |
| Logic | Boolean/gate logic. |
| Low-frequency oscillator | LFO/modulation oscillator. |
| Low-pass gate | Combined low-pass/amplitude gate behavior. |
| MIDI | MIDI input/output/control. |
| Mixer | Mixes signals. |
| Multiple | Signal multiple/fanout utility. |
| Noise | Noise source/processor. |
| Oscillator | Audio or control oscillator. |
| Panning | Stereo/multichannel positioning. |
| Phaser | Phaser/all-pass modulation effect. |
| Physical modeling | Physical-model synthesis/processing. |
| Polyphonic | Explicitly supports polyphonic cables up to the documented limit. |
| Quad | Four related units/channels. |
| Quantizer | Quantizes values/pitches/timing. |
| Random | Random/probabilistic source or processor. |
| Recording | Records audio/CV/MIDI/data. |
| Reverb | Reverberation effect. |
| Ring modulator | Multiplicative/ring modulation. |
| Sample and hold | Samples input and holds value. |
| Sampler | Sample playback/recording. |
| Sequencer | Step/event sequence generator. |
| Slew limiter | Limits rate of change/glide. |
| Speech | Speech/vocal synthesis/processing. |
| Switch | Selects/routes among signals/states. |
| Synth voice | Combined oscillator/filter/envelope/VCA-style voice. |
| Tuner | Pitch/frequency tuning display/utility. |
| Utility | General helper. |
| Visual | Scope/meter/display/visualizer. |
| Vocoder | Vocoder/spectral voice effect. |
| Voltage-controlled amplifier | VCA/amplitude control. |
| Waveshaper | Nonlinear wave shaping. |

### 10.4 Library/account menu

- **Behavior:** Library Login logs into a VCV account. Update all downloads and
  updates all new plugins/plugin versions added to the account; restart is
  required to load updates.
- **Constraints:** Account assistance and store flows are outside this spec.
- **Data/project implications:** Plugin installation/update changes module catalog
  availability and project load resolution.
- **Realtime implications:** Updates/downloads are not realtime operations.
- **Geist mapping notes:** If Geist adds a library, separate catalog sync from
  project load and never mutate plugin binaries while audio is running.

## 11. Presets and project storage

### 11.1 Module presets

- **Behavior:** Factory module presets teach/inspire users and store parameter
  values plus internal data from module custom serialization. They are JSON-like
  files with Rack extension `.vcvm`.
- **Constraints:** Presets cannot store Rack 2 module patch storage files. Factory
  preset path convention is `presets/<module slug>/<preset name>.vcvm` inside a
  plugin package. Presets reload when the user opens the preset context menu, so
  restart is unnecessary.
- **Data/project implications:** Geist presets should specify exactly which
  parameter/custom-state fields they apply and should not silently embed large
  assets unless designed as bundled presets.
- **Realtime implications:** Preset loading is a control event; large load must be
  staged off audio thread.
- **Geist mapping notes:** Use presets as small state overlays, separate from
  project media/blob storage.

### 11.2 Sorting and display names

- **Behavior:** Presets are sorted alphabetically by filename. Since Rack 2, a
  leading numeric prefix followed by underscore, matching `/^\d+_/`, is hidden
  from displayed preset name to allow custom order.
- **Constraints:** This is a Rack UI convention; Geist can choose a richer order
  field instead.
- **Data/project implications:** Preserve stable preset identifiers separately
  from display names.
- **Realtime implications:** None.
- **Geist mapping notes:** Prefer explicit `order` metadata while allowing filename
  import compatibility if needed.

### 11.3 Partial preset application

- **Behavior:** A preset can be edited to include only a subset of parameter
  entries; omitted parameters remain unchanged. Manual example removes parameter
  id 1 while setting ids 0 and 2.
- **Constraints:** This applies to parameter entries; custom state partial merge
  semantics are not fully specified by the public page.
- **Data/project implications:** Preset application should be able to act as a
  patch/merge, not only full replace.
- **Realtime implications:** Applying a preset may change many parameters at once;
  batch updates should be coherent.
- **Geist mapping notes:** Support scoped presets/macros deliberately.

## 12. DSP conventions and independent implementation guidance

### 12.1 Signals, Fourier analysis, and sampling

- **Behavior:** The DSP manual frames digital signals as sampled sequences and
  uses Fourier analysis to reason about frequency content. Sampling imposes a
  sample-rate/Nyquist limit.
- **Constraints:** Frequencies above Nyquist alias unless removed before sampling
  or avoided by bandlimited generation.
- **Data/project implications:** Sample rate affects rendered sound and may need
  project/session capture for reproducibility.
- **Realtime implications:** All time constants and coefficients must derive from
  current sample time.
- **Geist mapping notes:** Tests should render at multiple sample rates.

### 12.2 Aliasing and bandlimited waveforms

- **Behavior:** Discontinuous waveforms such as saw and square need bandlimiting;
  triangle waves also need care. MinBLEP and PolyBLEP are public manual concepts
  for correcting discontinuities. Nonlinear processes such as waveshaping,
  distortion, and saturation generally require antialiasing.
- **Constraints:** Linear processes such as mixing, linear filters, delays, and
  reverbs do not require antialiasing solely because they are linear.
- **Data/project implications:** Oversampling quality may be module state/preset
  state.
- **Realtime implications:** Oversampling adds CPU and latency; decimation filters
  must be stable and bounded.
- **Geist mapping notes:** Implement independent BLEP/oversampling code; do not
  copy Rack code or constants beyond published voltage/frequency standards.

### 12.3 Filters, impulse responses, and windows

- **Behavior:** The public DSP page covers IIR filters, FIR filters, impulse
  responses, brick-wall filters, windows, and minimum phase systems. IIR filters
  are efficient for low-order filtering but require stability. FIR convolution
  can use direct or FFT/overlap approaches depending on length. Windowing trades
  main-lobe/side-lobe characteristics for practical finite filters.
- **Constraints:** Ideal brick-wall filters are theoretical; practical filters
  have transition bands/ringing/phase consequences.
- **Data/project implications:** Filter type/order/quality and convolution assets
  are module state; impulse responses may be large patch storage data.
- **Realtime implications:** Long convolution needs block/FFT scheduling and
  latency management; IIRs need finite/stability guards.
- **Geist mapping notes:** Provide filter design utilities with documented phase,
  latency, and sample-rate behavior.

### 12.4 Circuit modeling and ODEs

- **Behavior:** The DSP manual includes circuit modeling, nodal analysis, and
  numerical methods for ordinary differential equations as virtual-analog design
  approaches.
- **Constraints:** Accuracy, stability, and cost depend on solver/model choices;
  the public manual is guidance, not a required implementation.
- **Data/project implications:** Model quality/oversampling settings may be
  project/preset state.
- **Realtime implications:** Solvers must converge or fail safely within bounded
  CPU; non-finite output must be sanitized.
- **Geist mapping notes:** Use independent numerical implementations and stress
  tests for extreme parameters/feedback.

### 12.5 Optimization

- **Behavior:** Public optimization topics include profiling, mathematical
  approximations, compiler optimization, memory access, and vector instructions.
  The plugin guide also mentions SIMD types/concepts and Rack DSP utilities such
  as trigger detectors, pulse generators, fast math approximations, FFT, sample
  rate converter, ODE solvers, and filters.
- **Constraints:** Approximate expensive functions only when profiling justifies
  it. SIMD element indexing can defeat vectorization benefits. SIMD speedups are
  workload-dependent.
- **Data/project implications:** Optimization must not alter saved project meaning.
- **Realtime implications:** Optimize after correctness; avoid cache misses,
  branch-heavy hot loops, allocations, and locks.
- **Geist mapping notes:** Maintain scalar reference paths and compare SIMD/fast
  math against them in CI.

## 13. Migration lessons from public v1-to-v2 page

### 13.1 Version and compatibility

- **Behavior:** Rack v2 migration says the API is nearly backward-compatible with
  v1 for many plugins, but plugin manifest major version must match Rack major
  version. Developers update to major 2 and rebuild.
- **Constraints:** This is Rack ecosystem compatibility, not a Geist ABI promise.
- **Data/project implications:** Major-version boundaries need migration policy.
- **Realtime implications:** None.
- **Geist mapping notes:** Use explicit project/device schema versions and forward
  migration tools.

### 13.2 Parameter API behavior visible to users

- **Behavior:** v2 guidance replaces old parameter reset/randomize widget behavior
  with module/parameter policies; momentary buttons and multi-state switches
  should present as buttons/switches rather than arbitrary real-valued controls.
- **Constraints:** This affects what users see in context menus and tooltips.
- **Data/project implications:** Control type is metadata, not only DSP value.
- **Realtime implications:** None beyond parameter update path.
- **Geist mapping notes:** Model parameter kind explicitly.

### 13.3 Runtime resource lifetime

- **Behavior:** The DAW plugin version can destroy/recreate graphics contexts when
  editor windows close/open; stored font/image references may become invalid.
  Store paths and retrieve cached resources during draw.
- **Constraints:** Manual mentions fonts/images; broader GPU resource invalidation
  is a reasonable Geist design concern but not a source claim.
- **Data/project implications:** UI resources should be reloadable from stable
  descriptors.
- **Realtime implications:** UI resource reload must not touch audio.
- **Geist mapping notes:** Decouple DSP lifetime from editor lifetime.

### 13.4 Optional v2 usability features

- **Behavior:** Public optional enhancements are port/light labels, buttons,
  switches, bypass routes, patch storage for large data, and full-bright custom
  light widgets.
- **Constraints:** These are optional in Rack migration but valuable Geist baseline
  features.
- **Data/project implications:** Include them in device authoring metadata from
  the start.
- **Realtime implications:** Bypass/large storage/light UI each have the realtime
  constraints listed above.
- **Geist mapping notes:** Treat them as first-class contracts, not plugin extras.

## 14. Deliberate non-goals and remaining source-bound gaps

### 14.1 Deliberate non-goals

- No Rack `.vcv`, `.vcvm`, `.vcvplugin`, SVG, manifest, package, or ABI
  compatibility promise for Geist.
- No copying of VCV module artwork, screenshots, labels beyond generic public
  terms, presets, source code, SDK helper output, or internal APIs.
- No requirement to support Rack plugins.
- No requirement to mirror Rack's exact UI, menu order, theme, account system,
  library/store, or graphics stack.
- No cable-level hard clipping and no typed cable enforcement by signal category.
- No hidden behavior inferred from source, headers, screenshots, or binaries.

### 14.2 Remaining gaps with concrete source reasons

| Gap | Why it remains |
| --- | --- |
| Exact `.vcv` project schema | The public manual names `.vcv` files but does not specify a full schema; source/sample files were excluded. |
| Exact `.vcvm` schema beyond parameter subset example | Presets page gives behavior and a small parameter example, not a complete schema; source/preset files were excluded. |
| Exact CV-MIDI/CV-CC/CV-Gate voltage-to-MIDI edge rules | Core page describes purpose but not detailed conversion thresholds/ranges. |
| Lock cursor when dragging params | MenuBar page says “Documentation coming soon”. |
| Knob mode | MenuBar page says “Documentation coming soon”; FAQ only says rotary can help touch users. |
| Scroll wheel knob control | MenuBar page says “Documentation coming soon”. |
| Exact DAW-specific routing beyond Ableton/Logic notes | RackPro page only provides detailed subsections for Ableton Live and Logic Pro. |
| Pro Modules individual behavior | RackPro page only states Pro Modules are premium effects; no module-specific manual content was included in the requested public pages. |
| Third-party module behavior | Out of scope except generic categories and examples in public manual. |

## 15. Top implementation implications for Geist

1. Model modular connections as voltage lanes with explicit 0-16 channel counts,
   not typed audio/CV-only ports.
2. Enforce one cable per input and fanout from outputs as graph invariants.
3. Add shared realtime-safe primitives for Schmitt triggers, 1 ms pulses,
   reset-clock suppression, 1V/oct conversion, finite-output sanitization,
   channel broadcast/index/zero rules, and mono poly-input fallback.
4. Separate DSP graph state from UI/session/preset serialization; autosave,
   settings save, plugin/library updates, and large file I/O must never block
   audio processing.
5. Treat module lifecycle, bypass, expanders/side-buses, sample-rate changes,
   automation identity, and parameter metadata as first-class host contracts.
6. Preserve stable slugs/IDs and searchable metadata from the start; browser UX,
   project load, automation, hidden/deprecated modules, and migration all depend
   on them.
7. Implement scalar-correct DSP first, then independent SIMD/oversampling/fast
   math optimizations under tests that prove equivalent behavior within defined
   tolerances.
8. Keep Rack-derived facts as clean-room planning references; Geist project
   format, UI, plugin model, security model, and device implementation should be
   native to Geist.
