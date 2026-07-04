<!--
Author: Jeff
Date: 2026-07-03
Description: Clean-room behavioral spec from the public Ableton Live 12 online manual for Geist DAW planning
Notes: Behavior-only; no vendor code, assets, screenshots, presets, reverse engineering, or private material referenced
-->

# Ableton Live — Clean-Room Behavioral Spec for Geist Planning

This document records observable DAW behavior described in the public Ableton
Live 12 online manual. It is a clean-room planning artifact for Geist: it
captures user-visible concepts, editing semantics, routing models, automation
rules, and export behavior without copying vendor implementation, code, assets,
UI graphics, presets, DSP internals, or private material.

Geist may adopt the product-level patterns where useful, but must implement them
with original names, data structures, UI, DSP, device designs, and content.

## Provenance

Fetched: 2026-07-03.

Primary public sources:

- https://www.ableton.com/en/live-manual/12/welcome-to-live/
- https://www.ableton.com/en/live-manual/12/live-concepts/
- https://www.ableton.com/en/live-manual/12/managing-files-and-sets/
- https://www.ableton.com/en/live-manual/12/arrangement-view/
- https://www.ableton.com/en/live-manual/12/session-view/
- https://www.ableton.com/en/live-manual/12/clip-view/
- https://www.ableton.com/en/live-manual/12/audio-clips-tempo-and-warping/
- https://www.ableton.com/en/live-manual/12/editing-midi/
- https://www.ableton.com/en/live-manual/12/launching-clips/
- https://www.ableton.com/en/live-manual/12/routing-and-i-o/
- https://www.ableton.com/en/live-manual/12/mixing/
- https://www.ableton.com/en/live-manual/12/recording-new-clips/
- https://www.ableton.com/en/live-manual/12/bounce-to-audio/
- https://www.ableton.com/en/live-manual/12/working-with-instruments-and-effects/
- https://www.ableton.com/en/live-manual/12/instrument-drum-and-effect-racks/
- https://www.ableton.com/en/live-manual/12/automation-and-editing-envelopes/
- https://www.ableton.com/en/live-manual/12/clip-envelopes/
- https://www.ableton.com/en/live-manual/12/audio-fact-sheet/
- https://www.ableton.com/en/live-manual/12/midi-fact-sheet/

## Coverage tracker

| Manual area | Status | Captured here |
|---|---:|---|
| Live concepts / tracks / clips | done | set, clip, track, view, mixer, device-chain model |
| Session View | done | clip slots, scenes, launch precedence, stop buttons, recording to Arrangement |
| Arrangement View | done | linear timeline, locators, loop, clip editing, split/consolidate, fades |
| Clip View | done | clip regions, loop, start/end, groove, scale, audio/MIDI panels |
| Clip launch | done | launch modes, quantization, velocity, legato, follow actions |
| Audio clips / warping | done | warp markers, tempo independence, warp modes, clip gain/pitch |
| MIDI editing | done | note editor, draw/edit/select/stretch/quantize, velocity/probability, scales |
| Recording | done | arming, monitoring, Session/Arrangement recording, overdub, capture MIDI |
| Routing and I/O | done | track I/O, monitoring, resampling, internal routings, external devices |
| Mixing | done | volume/pan/sends, groups, returns, main, solo/cue, crossfader, track delay |
| Devices and racks | source-bound covered | chain ordering, presets, plugins, racks/chains/macros/zones; individual device DSP is a deliberate non-goal |
| Automation | done | Arrangement/Session automation, override/re-enable, breakpoints, tempo automation |
| Clip envelopes | done | clip-local modulation/automation, unlinking, audio and MIDI controller envelopes |
| Export / bounce | source-bound covered | file export, MIDI export, bounce-to-audio, collect referenced files; exact dialog defaults are source-limited |
| Audio/MIDI fact sheets | source-bound covered | neutral/non-neutral operations, timing constraints, quality notes; algorithms are not public |
| Push/control surfaces/video/Max | out of scope | defer unless Geist targets hardware/video/M4L-class extensibility |

## 1. Top-level DAW model

- A project/set is the user-facing document containing tracks, clips, devices,
  mixer state, automation, routing, tempo, meter, and file references.
- The core musical unit is a clip: audio clips reference sample files plus
  playback metadata; MIDI clips contain note/controller data embedded in the set.
- Two primary composition surfaces coexist over the same tracks:
  - **Linear timeline**: clips placed against absolute musical time.
  - **Session grid**: clips launched in real time from per-track slots and scene
    rows.
- Switching views changes the UI, not transport playback state. The two views
  share tracks, devices, routing, and mixer state.
- A track can play either its timeline material or a launched session clip at a
  given time, not both. Launched session clips take precedence until the user
  returns the track or the whole set to timeline playback.
- Tracks are the signal-flow containers. They host clips, optional devices, mix
  controls, routing, recording inputs, and automation lanes.
- Track classes exposed by behavior:
  - Audio track: hosts audio clips and audio effects.
  - MIDI track: hosts MIDI clips, MIDI effects, instruments, and downstream
    audio effects; without an instrument it can output MIDI rather than audio.
  - Return track: receives sends and hosts audio effects, not clips.
  - Group track: submix container for child tracks; groups may be nested.
  - Main/master track: final mix target and global controls.
- Device chains are ordered left-to-right signal processors inside a track.
  MIDI tracks process MIDI effects first, then an instrument, then audio effects.
  Audio/return/group tracks process audio effects only.
- The mixer is shared by both views and exposes level, pan, sends, routing,
  mute/solo/arm/monitor-style state, and final summing to the main output.

### Geist mapping notes

- Model Session and Arrangement as two clip-source layers over one track graph.
  The track's active source selector decides timeline vs launched clip.
- Store clip identity independently from placement. A MIDI/audio clip instance
  can appear in slots or timeline ranges with clip-local playback metadata.
- Keep mixer/device/routing state track-owned, not view-owned.

## 2. Transport, time, tempo, and meter

- Transport provides play/stop, recording, current song position, tempo, global
  quantization, metronome/count-in, loop, punch-in/out, and view-independent
  playback.
- Timeline position is represented in musical time. Arrangement loop defines a
  start and length and can be enabled independently of clip loops.
- Tempo can be typed, tapped, nudged temporarily, automated, or influenced by
  tempo-following/leader behavior.
- Meter/time-signature changes can exist at scene or arrangement locations and
  affect musical grid interpretation.
- Global launch quantization determines when launched clips/scenes begin or
  stop, unless a clip overrides launch quantization.
- Locators/markers in the timeline can label positions and provide jump targets.
- The editing grid can be fixed or adaptive; snapping applies to clip movement,
  resizing, note editing, automation, and time selections.

### Geist mapping notes

- Use a single transport clock that exposes sample time, beat time, bar/beat,
  tempo map, meter map, loop region, and quantization scheduler.
- Quantized launches should be scheduled events, not UI callbacks, so Session
  recording and external control remain sample-accurate enough for the engine.

## 3. Session grid behavior

- The Session surface is a grid: tracks are columns; scenes are rows; each
  track/scene intersection is a clip slot.
- A track can play one Session clip at a time. Launching a new clip on the same
  track stops/replaces the prior clip according to launch quantization.
- Launching a scene launches the clips in that row across tracks. Empty slots may
  either leave the track alone or stop it, depending on whether a stop button is
  present/active for that slot.
- Clip slots expose state: empty, stopped, triggered-to-start, playing,
  triggered-to-stop, recording, or recorded-and-playing.
- Scenes can carry names and may encode scene-specific tempo and meter values.
- Session recording can create clips in armed tracks without stopping playback.
  Pressing Session Record again ends recording and launches the newly recorded
  clips subject to quantization.
- Session performance can be recorded into the timeline, creating arrangement
  clips and automation from launched clips and control changes.
- Per-track status fields show whether a track is playing a clip, waiting to
  launch/stop, or recording.

### Geist mapping notes

- Represent scenes as launch command groups, not as audio containers.
- Stop buttons are per-slot launch commands. This avoids special-casing empty
  cells in the scheduler.
- Preserve the precedence rule: session source overrides timeline source until a
  per-track or global return-to-arrangement command resets it.

## 4. Arrangement/timeline behavior

- The Arrangement surface is a linear timeline. Tracks are stacked vertically;
  time moves left-to-right.
- Clips can be moved, copied, duplicated, resized from edges, split at a time
  selection/cursor, consolidated into a new clip, cropped, deactivated, or
  deleted.
- Time selections support insert/delete/cut/copy/paste/duplicate time commands
  that affect clips and automation over selected tracks.
- Audio clips have fades at clip boundaries and can crossfade where adjacent or
  overlapping clips meet. Fades are non-destructive playback metadata.
- Linked-track editing lets operations on one selected/linked track apply to
  corresponding ranges on multiple tracks, useful for multi-mic edits.
- Arrangement recording writes incoming audio/MIDI into armed tracks and can
  create take lanes/comp material where multiple passes overlap.
- Per-track return-to-arrangement controls resume timeline playback only for
  that track; a global command resumes timeline playback for all tracks.

### Geist mapping notes

- Use immutable audio sources plus non-destructive clip regions/fades for edits.
- Treat consolidate/bounce as explicit render-to-new-media operations.
- Implement linked editing as selection grouping metadata over timeline ranges,
  not as duplicated edit code.

## 5. Clip model

### 5.1 Shared clip properties

- A clip has a name/color, start/end markers, loop start/loop length, current
  playback position behavior, time signature, groove assignment, scale settings,
  and launch behavior.
- Clip start/end define the playable region. Loop region defines repeated
  playback inside the clip. Start can be outside or inside the loop depending on
  launch/scrub/edit state.
- Multiple selected clips can have compatible properties edited together.
- Clip defaults influence newly created clips but do not retroactively rewrite
  existing clips.
- Clips can be deactivated/non-playing without being deleted.
- Clip View exposes the selected clip's properties plus an editor specialized
  for audio or MIDI content.

### 5.2 Audio clips

- Audio clips reference audio files. The clip stores what part to play and how to
  play it; normal clip edits do not destructively modify the referenced file.
- Audio clip playback metadata includes warp on/off, warp markers, warp mode,
  clip gain, transposition, detune, reverse, fades, high-quality interpolation,
  RAM/disk playback preference, and sample start/end details.
- Reversing, cropping, consolidating, and destructive external sample editing are
  exposed as explicit commands with clear new-file or external-edit behavior.
- Missing referenced files make affected clips/sampler slots offline and silent
  until repaired or replaced.

### 5.3 MIDI clips

- MIDI clips contain note events and controller/envelope data in the set rather
  than referencing the source MIDI file after import.
- Notes have pitch, start time, duration, velocity, probability/chance-style
  playback metadata, and enabled/disabled state.
- The MIDI editor supports drawing notes, previewing notes, selecting notes and
  time spans, moving, transposing, changing length, stretching, duplicating,
  quantizing, editing velocities, editing probabilities, and cropping/looping.
- Scale mode can highlight or fold the note grid to the selected root/scale.
- Multi-clip MIDI editing can show notes from multiple clips while focusing edits
  on a selected target clip.

### Geist mapping notes

- Store clip playback metadata separately from source media identity.
- Model MIDI note probability and disabled state as first-class note fields.
- Keep clip loop region and arrangement placement independent: the same clip
  content can loop inside a longer timeline/session duration.

## 6. Clip launch behavior

- Launch controls are clip-local and determine how a clip responds to launch and
  stop commands.
- Launch modes include trigger-style start, gate-style play-while-held,
  toggle-style start/stop, and repeat/retrigger-style behavior.
- Launch quantization can be inherited from global quantization or overridden per
  clip.
- Legato launch starts a newly launched clip at a playback position related to
  the previously playing clip on that track rather than from its own start.
- Launch velocity can affect clip playback level or mapped behavior.
- Clip offset/nudge commands move the playback start position during performance.
- Follow actions can automatically trigger another clip, stop, or perform a
  configured action after a clip-defined time, with probability/choice behavior.
- Follow actions enable cycles, temporary loops, variations, and non-repeating
  structures without timeline automation.

### Geist mapping notes

- Implement clip launch as a deterministic state machine per track with
  scheduled transitions on the transport clock.
- Keep follow actions in the same scheduler as user launches so they obey the
  same quantization and precedence rules.

## 7. Audio warping and timing

- Warping changes audio playback speed independently of pitch so clips can match
  song tempo or intentional timing edits.
- Warp markers map sample time to musical time. Moving markers changes timing
  non-destructively.
- Short samples and long samples have different default auto-warp/import
  behavior; auto-analysis can infer tempo and place initial markers.
- Warp modes are selected per clip and optimize for different material:
  beats/transients, tonal material, texture/granular material, repitch-style
  varispeed, and complex polyphonic material.
- Quantizing audio moves warp markers or timing references to the grid.
- Groove can impose timing/velocity/feel changes non-destructively and can be
  committed if the user wants explicit edits.
- Some operations are neutral under constrained conditions, while complex warp
  modes, sample-rate conversion, transposition, dithering, fades, pan-law, groove
  commitment, and consolidation can change rendered audio.

### Geist mapping notes

- Separate a clip's media clock from the project beat clock via a warp map.
- Mark warp algorithms as original Geist DSP; this spec only requires the user
  behavior of tempo/pitch independence and mode choice.
- Expose neutral-vs-render-changing operations in docs/tests so users can reason
  about null tests.

## 8. Recording behavior

- Tracks must be armed/record-enabled to record incoming audio or MIDI. Multiple
  selected tracks can be armed together; exclusive arm vs multi-arm is a user
  preference/gesture behavior.
- Input selection chooses external hardware, another track, a device output, or
  an internal/resampling source, depending on track type.
- Monitoring states determine when input is heard: automatic, always-on, or off.
- Arrangement recording writes clips into armed tracks along the timeline.
- Session recording writes clips into selected/targeted empty slots in armed
  tracks and can immediately launch recorded material when recording ends.
- MIDI overdub adds notes/controllers to an existing looping MIDI clip rather
  than replacing the clip.
- Record quantization can quantize newly recorded MIDI notes.
- Count-in and metronome settings affect performer timing before recording
  starts.
- Capture MIDI can recover recently played MIDI into a new or existing clip even
  when formal recording was not armed, subject to available buffered material.
- Recorded audio files are stored in the project recording area and referenced by
  the created clips.

### Geist mapping notes

- Maintain pre-record MIDI buffers per armed/monitored MIDI input for capture.
- Recording should produce normal clips plus source files/events; avoid a special
  recorded-clip type.
- Monitoring latency compensation must be explicit, especially when recording
  internally routed or externally monitored signals.

## 9. Routing and I/O

- Track routing is a patchbay. Each track has input source choosers, output
  destination choosers, channel/subsource selectors, and monitor controls.
- Audio tracks can receive mono/stereo hardware inputs, other tracks, return/main
  signals where allowed, or resampling sources. Mono/stereo conversions are
  handled automatically at routing boundaries.
- MIDI tracks can receive MIDI hardware ports/channels, computer-keyboard MIDI,
  other MIDI tracks, or device MIDI outputs. They can send MIDI to instruments,
  external hardware ports, or other routable targets.
- Internal routing points can target whole tracks or specific device-chain
  points where a device exposes a tap/output.
- Resampling records the DAW's own output or an internal mix as audio into an
  armed audio track.
- External Audio Effect and External Instrument-style devices insert hardware
  send/return paths inside a device chain, with latency/level considerations.
- Routing choices interact with mixer visibility: a MIDI track with no
  audio-producing instrument may not expose audio mix/sends until it produces or
  receives audio.

### Geist mapping notes

- Treat routing as graph edges with stable endpoints: hardware ports, tracks,
  device pins, buses, sends, and resample buses.
- Use typed endpoints for safety, but permit explicit conversion nodes for
  mono/stereo and MIDI-to-instrument boundaries.
- Device-chain tap points need stable identifiers for automation and saved sets.

## 10. Mixer behavior

- Mixer controls include volume, pan, sends, mute/activator, solo, arm, monitor,
  track delay, crossfader assignment, and routing I/O.
- Sends feed return tracks so multiple tracks can share effect chains.
- Return tracks can be included in crossfade and solo/cue behavior but do not
  host clips.
- Group tracks sum child tracks and can host devices/mix controls; nested groups
  behave as submixes.
- Main/master receives track, group, and return outputs unless routed elsewhere.
- Soloing isolates selected tracks. Cueing/prelisten can route audition signals
  to a separate cue output path.
- Track delay offsets a track relative to the global timeline to compensate or
  create timing shifts.
- Performance/load indicators can mark tracks/devices that contribute heavily to
  CPU or disk load.

### Geist mapping notes

- Sends and groups should compile into explicit bus nodes in `geist-graph`.
- Track delay belongs in the scheduler/graph boundary, not hidden inside clips.
- Cue/prelisten needs a separate monitor bus from the main render/export path.

## 11. Devices, presets, plugins, and racks

- Devices are loaded from a browser into a track's device chain by drag/drop or
  keyboard command.
- Device order determines signal flow. Reordering a device changes processing
  order immediately.
- Devices have title-bar controls for activation/bypass, hot-swap/preset access,
  configuration, and A/B or variation-style comparison where supported.
- Presets save parameter values independently of a set so they can be reused in
  other projects. Defaults can be saved for future device creation.
- Third-party plugins appear as devices and participate in the same chain,
  automation, preset/bank, sidechain, and delay-compensation behavior where the
  plugin format supports it.
- Device delay compensation accounts for latency introduced by devices and
  routing so tracks remain time-aligned where possible.
- Racks wrap one or more device chains into a single container preset:
  - Multiple chains can run in parallel.
  - Chain zones can select by key, velocity, or chain selector range.
  - Macro controls map one visible control to one or more internal parameters.
  - Macro ranges can invert or scale mapped parameter response.
  - Chains can be mixed, muted/soloed, extracted, or auto-selected based on the
    active zone.
  - Drum-style racks map pads/notes to chains and can expose per-pad devices.

### Geist mapping notes

- Implement racks as subgraphs with macro-parameter fanout and zone predicates.
- Delay compensation must be graph-wide because plugin, hardware insert, and
  routing latencies cross track boundaries.
- Preset storage should reference Geist-owned device IDs and schema versions, not
  vendor names or assets.

## 12. Automation and envelopes

- Automation records or draws parameter value changes over time.
- Arrangement automation is stored on timeline lanes and represented by
  breakpoint envelopes.
- Session automation is stored inside Session clips when enabled/recorded into
  clips.
- Most mixer and device controls, including tempo, can be automated.
- Automation Arm plus recording determines whether live parameter movements are
  written as automation.
- Changing an automated parameter manually while not recording overrides that
  automation for playback. A re-enable command returns overridden parameters to
  stored automation.
- Automation can be deleted independently of the parameter's current value.
- Envelope editing supports drawing, adding/removing/moving breakpoints, curved
  segments, stretching/skewing ranges, simplifying dense automation, inserting
  shapes, locking envelopes against clip edits, and edit-menu time operations.
- Tempo automation is a specialized envelope over the tempo map and affects the
  transport's beat-to-time conversion.

### Geist mapping notes

- Automation playback should resolve `base parameter + active automation source +
  clip modulation` deterministically at sample/block boundaries.
- Store override state separately from automation data so edits do not destroy
  envelopes.
- Tempo automation requires transport-map invalidation and deterministic render
  behavior.

## 13. Clip envelopes and modulation

- Clip envelopes are clip-local envelopes that automate or modulate mixer,
  device, clip, and MIDI controller parameters during clip playback.
- Audio clips expose clip envelopes for clip-specific properties such as pitch,
  volume/gain, and other audio playback transforms.
- MIDI clips expose controller envelopes that emit MIDI CC-style data or control
  device/mixer parameters.
- Clip envelopes are non-destructive: they alter playback/control output without
  modifying the referenced sample or base parameter.
- Clip envelopes can be linked to the clip loop or unlinked with their own loop
  length, allowing long modulation over short repeating audio/MIDI clips or LFO-
  like behavior.
- Clip envelopes can act as modulation rather than absolute automation for some
  targets, combining with a base value instead of replacing it.

### Geist mapping notes

- Represent clip envelopes as clip-owned modulation lanes with their own loop
  clock and target binding.
- Target binding must survive device reordering by using stable parameter IDs.
- Define conflict order early: arrangement automation, session automation, clip
  envelope modulation, MIDI CC, and manual override must compose predictably.

## 14. Export, bounce, files, and project management

- Saving a set stores clips, tracks, devices, mixer/routing state, automation,
  and references to external media.
- A project is a folder collecting related sets and media. File-management tools
  can show referenced files, locate missing files, collect external files into
  the project, and identify unused files.
- Imported MIDI file data becomes embedded in the set; audio files remain
  referenced by clips unless collected/copied by project management.
- Export/render produces audio files from the set or a selected range, with user
  choices such as rendered track/source, range, sample rate, bit depth,
  normalization, dither, and file type where available.
- MIDI clips can be exported as MIDI files.
- Session clips can be exported/saved as reusable clip/set material by dragging
  or saving to the browser/project.
- Bounce-to-audio renders selected track/group material into new audio that can
  be pasted or placed back into the set.
- Freeze/flatten-style workflows render device-heavy tracks to audio to reduce
  CPU while preserving a path back to editable state where supported.
- Collect-on-export policy determines whether samples referenced by saved clips,
  presets, or tracks are copied next to the exported item.

### Geist mapping notes

- Distinguish set save, project collect, render/export, and bounce as separate
  operations with separate side effects.
- Renders must be offline-deterministic and use the same graph semantics as
  real-time playback except for explicit high-quality/offline options.
- Missing-file handling should degrade to silence plus clear diagnostics, never
  crash or silently relink to unrelated media.

## 15. Audio and MIDI quality constraints

- Internal recording of routed signals at high precision can be neutral when no
  non-neutral processing is inserted.
- Bypassed effects, simple routing, splitting clips, and summing at a single mix
  point can be neutral operations under documented conditions.
- Non-neutral operations include complex time-stretching, sample-rate conversion,
  transposition, dithering, volume automation, clip fades, panning, groove
  commitment, and consolidation/rendering choices.
- MIDI timing quality depends on input device timing, OS/hardware drivers,
  buffer sizes, plugin/device latency, monitoring paths, and synchronization.
- MIDI recording/playback needs latency compensation and clear distinction
  between what the performer hears, what is recorded, and when notes are played
  back.

### Geist mapping notes

- Add null-test fixtures for neutral operations and golden behavioral tests for
  non-neutral operation boundaries.
- Keep MIDI event timestamps in high-resolution transport time at input capture;
  quantize only by explicit user command or record-quantize setting.

## 16. Deliberate non-goals from this source pass

- Do not clone Live's UI layout, icons, screenshots, browser taxonomy, device
  names, presets, sample content, lesson content, or Max/Push integrations.
- Do not infer private file formats, internal algorithms, DSP code, or plugin
  implementation details.
- Do not implement branded warp algorithms or factory devices from this spec;
  only the observable DAW behavior is in scope.
- Do not treat this as a legal compatibility target. It is planning input for an
  original Geist workflow.

## 16A. Additional manual feature inventory and Geist implications

This section records feature families from the broader Live 12 manual table of
contents that are not central to the first Geist implementation slice but still
need clean-room accounting.

### First steps, settings, and learning surfaces

- Installation/authorization, Learn View, Info View, language/theme/display,
  audio-device, Link, Tempo/MIDI, file/folder, library, plug-in, record/warp/
  launch, license/update settings are app-level configuration and onboarding
  surfaces.
- Geist should map these to `geist-config`, `geist-audio-backend`, async plug-in
  scanning, workflow profiles, and contextual help. None of this belongs in the
  audio callback.

### Browser, tags, places, packs, cloud-like locations

- Browser features include content pane, search, saved searches/custom labels,
  history, filters, tags, tag editing, quick tags, collections, library, places,
  packs, current project, user folders, previewing, navigation, and insertion.
- Geist should implement an asset index/cache, tag metadata, current-project
  references, user-folder roots, audition/preview routing, and validated
  drag/drop insertion commands. Vendor cloud, pack-store, Splice, and Push file
  transfer behavior are non-goals unless Geist later ships equivalent services.

### File management, projects, sets, missing files, collect/export

- File behavior covers sample decoding cache, analysis sidecars, MIDI file
  import/export, clip/set/project organization, templates, set merge, session
  clip export, references, missing-file repair, collecting external files,
  finding unused files, and packing projects.
- Geist should store relative paths plus content hashes, keep analysis caches
  disposable, support missing/offline diagnostics, and distinguish save,
  collect, export, render, and pack/archive side effects.

### MPE and per-note expression

- Public manual features include viewing/editing MPE data, drawing MPE envelopes,
  device/plugin MPE support, and MPE/multi-channel settings.
- Geist should extend note events with per-note pitch, pressure, timbre, channel,
  and expression-curve data; external plugin MPE routing belongs behind the VST
  host boundary.

### Audio-to-MIDI and stem separation

- Audio-to-MIDI features include slicing to MIDI, resequencing slices, applying
  effects to slices, converting harmony/melody/drums, and conversion-quality
  guidance.
- Stem separation features include separating audio files/clips and speed-vs-
  quality tradeoffs.
- Geist should treat these as offline analysis/render jobs producing new clips,
  sampler/device state, or audio assets. Exact ML/transcription/separation
  algorithms are not public and must be original or omitted.

### Grooves and tuning systems

- Groove features include a groove pool, groove parameters, commit behavior,
  groove editing/extraction, single-voice grooving, non-destructive quantization,
  and randomization texture.
- Tuning-system features include loading tunings, a tuning section, per-track
  MIDI tuning options, bypass tuning, and controller-layout concerns.
- Geist should model groove as non-destructive timing/velocity transformation
  metadata and tuning as a project/track/device pitch-map layer.

### Comping and take lanes

- Comping features include take lanes, inserting/managing lanes, recording takes,
  inserting samples, auditioning lanes, creating comps, and source highlights.
- Geist should represent takes as alternate source/event lanes and comps as a
  segment-selection map that resolves to normal clip playback without duplicating
  media.

### Device/rack feature details

- Device behavior includes device view, title-bar controls, activation/bypass,
  A/B comparison, presets, hot-swap, saving defaults, plug-in device wrapping,
  sidechain parameters, VST folders/presets/banks, Audio Unit support in Live,
  and device delay compensation.
- Rack behavior includes parallel chains, macro controls, rack creation/viewing,
  chain list, auto-select, key/velocity/chain-select zones, drum-pad mapping,
  macro map mode, macro randomization, macro variations, rack mixing, and chain
  extraction.
- Geist should implement the architecture, not Live's devices: nested device
  graphs, macro fanout, zone predicates, sidechain endpoints, plugin latency, and
  preset/default state are required; first-party device DSP remains original.

## 17. Open coverage gaps

- Exact export dialog defaults are not copied. Geist should define its own
  defaults once render jobs exist.
- Individual Live instruments/effects and Max for Live devices are not specified
  because cloning device designs/content is a clean-room non-goal.
- Push/control-surface, video, cloud/pack-store, Splice, and accessibility flows
  are product/ecosystem surfaces; Geist may design original equivalents later.
- The public manual describes behavior at user level; it does not provide
  internal scheduler, warp-DSP, file-format, or latency-compensation algorithms.
- Version drift: this pass targets the Live 12 online manual fetched on
  2026-07-03. Re-check source URLs before using this as a long-term parity list.

## 18. Top implementation implications for Geist

- Build Session and Arrangement as peer source layers over one track graph, with
  explicit per-track source precedence and return-to-arrangement state.
- Make the quantized launch scheduler a core engine service shared by Session
  clips, scene launches, recording boundaries, follow actions, and remote input.
- Treat clips as non-destructive playback programs over embedded MIDI or
  referenced audio media; edits mostly mutate metadata, not source files.
- Implement routing as a typed graph/patchbay with stable endpoint IDs for
  hardware, tracks, buses, device pins, sends, sidechains, and resampling.
- Design automation/modulation composition early, including manual override and
  re-enable behavior, clip-local envelopes, tempo automation, and parameter ID
  stability through device-chain edits.
- Plan offline render/bounce/freeze around the same graph and transport semantics
  as playback, with explicit differences for dithering, sample-rate conversion,
  and high-quality modes.
- Track file provenance in project metadata: referenced media, collected media,
  missing/offline status, recorded files, and rendered derivatives.
