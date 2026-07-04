<!--
Author: Jeff
Date: 2026-07-03
Description: Clean-room feature-by-feature behavioral spec for Bitwig Studio-class DAW, modulation, and Grid workflows
Notes: Derived from public Bitwig Studio user guide pages; behavior only, no vendor code, assets, presets, screenshots, private docs, or UI copying
-->

# Bitwig Studio — Clean-Room Behavioral Spec

## Clean-room statement

- Source material is the publicly published Bitwig Studio user guide only.
- No Bitwig source code, assets, presets, project files, device internals, screenshots, controller scripts, binaries, or private docs were used.
- This document records observable workflow semantics and published feature behavior at a clean-room planning level.
- Geist may adopt the product-level ideas only with original naming, UI, DSP, file formats, device designs, presets, and content.
- Built-in device names/categories are used only where they identify public manual sections; they are not Geist product names.

## Provenance

Fetched 2026-07-03 from public user-guide pages:

| Source page | URL | Coverage used |
|---|---|---|
| Welcome / What's New | https://www.bitwig.com/userguide/latest/welcome_to_bitwig_studio | guide scope, version framing |
| Dashboard | https://www.bitwig.com/userguide/latest/the_dashboard | user/settings/packages/help, audio/controllers/sync/shortcuts/behavior |
| Document Conventions | https://www.bitwig.com/userguide/latest/document_conventions | terminology and interaction conventions |
| Concepts | https://www.bitwig.com/userguide/latest/bitwig_studio_concepts | top-level project/clip/track/device concepts |
| Window Anatomy | https://www.bitwig.com/userguide/latest/anatomy_of_the_bitwig_studio_window | window header, project tabs, controller status, view controls |
| Arrange View and Tracks | https://www.bitwig.com/userguide/latest/the_arrange_view_and_tracks | arranger timeline, grid, tracks, editing tools |
| Browsers | https://www.bitwig.com/userguide/latest/browsers | packages, collections, kind, locations, search/filtering |
| Arranger Clips | https://www.bitwig.com/userguide/latest/arranger_clips_and_the_browser_panel | inserting, moving, scaling, slicing, fades, loop, meta clips |
| Clip Launcher | https://www.bitwig.com/userguide/latest/the_clip_launcher | launcher layout, slots, scenes, clip behavior |
| Mix View | https://www.bitwig.com/userguide/latest/the_mix_view | mixer sections, sends, I/O, crossfader, meters |
| Introduction to Devices | https://www.bitwig.com/userguide/latest/introduction_to_devices | device panel, player mode, expanded device view, FX sends |
| Automation | https://www.bitwig.com/userguide/latest/automation | lanes, drawing/editing, parameter follow, recording |
| Audio Events | https://www.bitwig.com/userguide/latest/working_with_audio_events | detail editor, event expressions, stretch/onsets/gain/pan/pitch/formant, comping |
| Note Events | https://www.bitwig.com/userguide/latest/working_with_note_events | note editor, note expressions, micro-pitch, layered editing, comping |
| Operators | https://www.bitwig.com/userguide/latest/operators | chance, repeats, occurrence, recurrence |
| Notes ↔ Audio | https://www.bitwig.com/userguide/latest/going_between_notes_and_audio | loading audio into sampler workflow |
| Projects and Exporting | https://www.bitwig.com/userguide/latest/working_with_projects_and_exporting | project templates, export/bounce family |
| MIDI Controllers | https://www.bitwig.com/userguide/latest/midi_controllers | soft control assignments and remote controls |
| Advanced Device Concepts | https://www.bitwig.com/userguide/latest/advanced_device_concepts | nested chains, mix parameter, containers, common chain types |
| The Grid | https://www.bitwig.com/userguide/latest/welcome_to_the_grid | grid editor, palette, modules, patch cords, scopes, ordering |
| Tablet Computer | https://www.bitwig.com/userguide/latest/working_on_a_tablet_computer | tablet display profile and tablet views |
| Device Descriptions | https://www.bitwig.com/userguide/latest/device_descriptions | public device-description chapter scope; concrete DSP intentionally not cloned |

## Documentation coverage matrix

| Guide feature | Spec status | What is captured | Clean-room limit |
|---|---|---|---|
| Dashboard: User tab | covered | startup/user-document hub behavior | account/license flows not copied |
| Dashboard: Settings tab | covered | behavior, audio, controller, sync, shortcuts, other settings | exact UI layout and setting strings not copied |
| Dashboard: Packages tab | covered | package/content installation as asset-management class | Bitwig package ecosystem not implemented |
| Dashboard: Help tab | covered | help/manual/update entry points | vendor support flow out of Geist scope |
| Concepts | covered | project, tracks, clips, devices, modulation, launcher/arranger posture | product terminology not reused as branding |
| Window header/project tabs | covered | multi-project tabs, controller status, window/view controls | pixel layout out of scope |
| Arrange timeline | covered | timeline, zoom, grid, headers, toggles, editing tools | exact shortcuts/layout out of scope |
| Browser | covered | packages, collections, kind, locations, filters/search, insertion | vendor package catalogs/assets excluded |
| Arranger clips | covered | insert, move, snap, length, scaling, slice, slide, fades, loop, group/meta clips | proprietary stretch algorithms excluded |
| Clip launcher | covered | track/scene slot grid, launcher clips, scenes, stop/empty slots | exact launch UI out of scope |
| Mix view | covered | meter, remotes, devices, sends, I/O, strip, crossfader, comments | exact mixer art/layout out of scope |
| Device panel | covered | chain surface, player mode, expanded view, track headers, FX sends | exact device visual design out of scope |
| Automation | covered | lanes, breakpoints, parameter follow, additional lanes, recording | exact curve-drawing gestures not copied |
| Audio events | covered | event editor, expressions, stretch, onsets, gain/pan/pitch/formant, comping/takes | stretch/formant algorithms excluded |
| Note events | covered | note editor, velocity/chance/gain/pan/timbre/pressure, micro-pitch, layered editing/comping | exact piano-roll UI out of scope |
| Operators | covered | chance, repeats, occurrence, recurrence as per-event playback conditions | proprietary humanization behavior excluded |
| Notes to audio / sampler loading | covered | audio-to-sampler insertion workflow | Sampler internals/assets excluded |
| Projects/exporting | covered | template and export/bounce classes | exact file dialogs/defaults out of scope |
| MIDI controllers/remotes | covered | remote controls, soft assignments, controller status | controller scripts excluded |
| Nested devices/containers | covered | nested chains, mix parameter, containers, layers/selectors | vendor device names not reused |
| Modulators | covered | device-local modulators, mapping/depth, nested targets | exact modulator DSP/curves excluded unless independently designed |
| The Grid | covered | editor, palette, modules, cords, scopes, insert/reorder | module names/DSP/presets not cloned |
| Tablet profile | covered | display profile and tablet views as adaptive UI class | exact touch layout out of scope |
| Device descriptions | source-bound coverage | device categories inform Geist capability map | individual Bitwig device parity is a non-goal |

## 1. Product and project model

### Behavior

- A project is the main document containing tracks, clips, devices, automation, modulation state, routing, mixer state, tempo, meter, controller mappings, and referenced assets.
- A project can be created from templates; templates save reusable startup state.
- Multiple project tabs may be open in one application window; project tab state and unsaved-state signaling are UI/document management concerns, not audio graph state.
- Track and clip content can be edited in different views without changing the underlying project identity.
- The project model treats launcher performance and arranger composition as peer workflows.

### Data/project implications

- Geist needs a project root object with:
  - tracks, scenes, arranger placements, launcher slots;
  - device chains and nested devices;
  - automation lanes and device-local modulator mappings;
  - audio/note event expression data;
  - controller mappings and remote pages;
  - referenced assets/packages/templates.
- Project tabs should be app/session UI state, not stored in the audio engine.
- Templates should serialize the same schema as projects with a template marker and safe defaults.

### Realtime implications

- Opening/saving/templates/packages/browser scanning stay off the audio thread.
- Project tab switching must publish precompiled engine state atomically; it must never mutate callback structures directly.

### UI/command implications

- Project creation/open/save/template actions are undo-independent document commands.
- Edits inside a project remain undoable project commands.
- Window/tab selection is UI state.

### Geist mapping notes

- Extend `geist-project` schema before adding more app-specific fixed parameter slots.
- Keep `app/geist-daw/src/session.rs` as an adapter, not the permanent schema owner.

## 2. Dashboard and global settings

### Behavior

- Dashboard has user, settings, packages, and help concerns.
- Settings cover behavior, audio devices, controller devices, synchronization, keyboard shortcuts, and miscellaneous preferences.
- Audio settings include audio device selection and combined/aggregate audio device concepts.
- Controller settings manage connected control surfaces and MIDI/controller inputs.
- Synchronization settings manage external sync/clock behavior.
- Shortcut settings expose key-command mappings.
- Package settings manage optional installed content.

### Data/project implications

- Separate app preferences from project files:
  - audio backend/device config;
  - controller mappings and enabled devices;
  - sync preferences;
  - shortcut/keymap profile;
  - package/content directories;
  - behavior/UI preferences.
- Project files may reference packages/assets but should not own global install state.

### Realtime implications

- Audio device changes require stream teardown/rebuild outside the callback.
- Sync/clock settings affect transport publication and scheduler inputs; they must enter the callback as snapshots.
- Package scanning and controller discovery must be async/app-thread work.

### UI/command implications

- Settings changes should validate immediately and show non-blocking errors for unavailable devices/controllers.
- Shortcuts and controller maps need conflict detection and reversible editing where project-affecting.

### Geist mapping notes

- `geist-config` should own workflow profiles, shortcuts, controller profiles, and audio/sync preferences.
- `geist-audio-backend` should expose combined-device-like capability at the trait/config layer only if Geist supports it.

## 3. Window anatomy and views

### Behavior

- The main window exposes project tabs, controller status, and window/view controls.
- Views expose the same project through different task surfaces: arrange, mix, edit, device, browser, launcher, and related panels.
- Display/profile selection can adapt to desktop/tablet use.

### Data/project implications

- View selection, panel visibility, zoom, scroll, editor focus, selected track/clip/device, and display profile are UI state.
- Controller status is runtime/controller state.

### Realtime implications

- View switching must not recompile audio unless it changes actual routing/device state.
- Meters/scope/controller status consume lock-free snapshots.

### UI/command implications

- UI views should issue typed commands; they should not mutate engine/project internals directly.
- Tablet profile implies larger hit targets and alternate layout, not a separate feature set.

### Geist mapping notes

- This aligns with `crates/geist-ui` as disposable model + commands and `app/geist-daw/src/studio.rs` as bridge.

## 4. Arrange view and tracks

### Behavior

- Arrange view presents a timeline panel with tracks and clip placements.
- Timeline supports zooming/navigation, beat grid settings, track headers, view toggles, and editing tools.
- Track headers expose track identity and common controls.
- Beat grid controls determine snap and editing resolution.
- Editing tools affect how pointer gestures create/select/move/split/resize/scale content.

### Data/project implications

- Timeline placements should reference clip IDs plus start, length, offset, loop, and lane/track.
- Grid settings are UI/editing state; clips store musical positions independent of current snap.
- Track headers reflect track model fields: name, color, type, mute, solo, arm, monitor, routing, group membership.

### Realtime implications

- Timeline scheduling converts beat placements to sample windows using tempo/meter snapshots.
- Editing clips in the arrangement publishes a new schedule only after app-thread validation.

### UI/command implications

- Move/resize/split/duplicate/delete/select are commands.
- Snap/grid should be a command modifier, not baked into clip data.

### Geist mapping notes

- `crates/geist-timeline` already has clip placements and transport foundations; add clip offset/loop/event-expression fields before complex editing.

## 5. Browser and content insertion

### Behavior

- Browser sources include packages, collections, kind/category filters, and locations.
- Browser supports search/filtering and insertion into tracks, device chains, clips, and projects.
- Package-provided content is discoverable separately from user locations.
- Collections/favorites are user organization metadata, not source assets themselves.

### Data/project implications

- Asset references must preserve source path/package identity and missing/offline state.
- Browser indexes are caches; project files store references and hashes/IDs.
- Collections are user preference data.

### Realtime implications

- Search, preview decoding, waveform creation, and package scanning are not callback work.
- Preview playback uses a prepared audition path with bounded buffers.

### UI/command implications

- Drag/drop or insert actions need validated targets and undoable resulting edits.
- Browser should explain unavailable packages/missing files.

### Geist mapping notes

- Extend `geist-project::asset_map` and `geist-ui::BrowserModel`; keep package/content manager outside audio engine.

## 6. Arranger clips

### Behavior

- Clips can be inserted from browser or created in place.
- Clips can be moved with snap settings, resized by changing boundaries, and content-scaled independently where supported.
- Slicing/quick-slice divides clips into multiple clips/events.
- Sliding changes clip content offset without moving the clip container.
- Audio clips support fades and crossfades.
- Clips can loop, repeating content inside a placement.
- Group/meta clips represent grouped child-track material at a higher hierarchy.

### Data/project implications

- Clip container fields: start, length, content offset, loop enabled, loop region, gain/color/name, track/lane placement.
- Audio event fields: source asset, source offset, event length, fade in/out, crossfade, gain/pan/pitch/formant expression, stretch/onset data.
- Meta/group clip fields reference child track ranges; they should not duplicate all child content.

### Realtime implications

- Sliding/looping affects scheduler read pointers.
- Fades/crossfades are per-sample or per-block gain curves and must be precomputed or cheap.
- Content scaling/stretch metadata should be scheduled; high-quality stretch can render offline or use prepared state.

### UI/command implications

- Slice, slide, fade, loop, resize, and scale are separate commands with distinct undo entries.
- Clip editor should show container vs content operations clearly.

### Geist mapping notes

- Add clip event model before adding proprietary-style stretch processing.

## 7. Clip launcher

### Behavior

- Launcher is a grid of clip slots by tracks and scenes.
- Clips in slots are launchable performance objects.
- Scenes launch a row of slots.
- A track normally has one active launcher clip at a time.
- Empty slots and stop buttons represent track stop behavior distinct from transport stop.
- Launcher clips can coexist with arranger clips and may be recorded/bounced to arrangement.

### Data/project implications

- Add scene table with name, optional tempo/meter metadata if Geist chooses to support it, and slot references.
- Launcher slot fields: clip ref/empty, stop behavior, launch settings, queued/playing state runtime mirror.
- Runtime playing/queued state is not the project source of truth unless captured/recorded.

### Realtime implications

- Launch scheduling needs quantized beat-boundary decisions from the transport snapshot.
- Stop/launch transitions need note-off/audio-tail handling.
- Scene launch must be atomic across tracks at the same musical boundary.

### UI/command implications

- Slot launch/stop are performance commands.
- Editing slot contents is project mutation; triggering slot playback is runtime command.

### Geist mapping notes

- `app/geist-daw/src/engine.rs` already has launcher quantization; add per-slot stop/launch data in `session.rs` and `geist-ui`.

## 8. Mix view and routing

### Behavior

- Mix view exposes track headers, clip launcher, large meters, remote controls, devices, sends, track I/O, channel strip, crossfader, and comments.
- Sends route from tracks to effect/return tracks.
- Track I/O configures input and output routing.
- Channel strip controls level/pan/mute/solo/arm/monitor-like state.
- Crossfader assigns tracks to crossfade sides and blends between them.
- Comments are track/project annotation data.

### Data/project implications

- Track mix state: volume, pan, mute, solo, arm, monitor, send amounts, output route, input route, crossfade assignment, comments.
- Return/effect tracks are routable bus tracks with device chains.
- Meters are runtime snapshots, not saved as project truth.

### Realtime implications

- Sends, crossfade, mute/solo, and route changes affect graph compilation/mixer plan.
- Metering must be lock-free and bounded.

### UI/command implications

- Mix edits are undoable where they change project/mix state.
- Meter and status display are read-only runtime state.

### Geist mapping notes

- Extend `MixerModel` beyond current simplified sends and crossfader absence.

## 9. Device panel and chains

### Behavior

- Device panel shows the selected track's device chain.
- Player mode reduces surface to performance-relevant controls.
- Expanded device view exposes deeper editing for selected devices.
- Device panel includes track headers for context.
- FX tracks and send amounts can be visible from device/mix context.

### Data/project implications

- Device chain is ordered and nested.
- Device state includes parameter values, modulator attachments, nested chains, remote controls, macros, preset identity, and bypass/enable state.
- Player/performance view state is UI profile, not a separate device.

### Realtime implications

- Reordering/inserting/removing devices compiles a new process plan.
- Parameter changes enter via bounded command/automation paths.

### UI/command implications

- Device selection, expanded state, and focus are UI state.
- Insert/delete/reorder/bypass/parameter changes are commands.

### Geist mapping notes

- Move from fixed rack slots in `studio.rs` toward descriptor-driven device chains.

## 10. Automation

### Behavior

- Automation lanes belong to arranger context and target parameters.
- Users can draw/edit automation points/curves.
- Parameter follow selects/displays automation for the touched parameter.
- Additional automation lanes can be shown for multiple parameters.
- Recording automation captures parameter gestures over time.

### Data/project implications

- Automation target identity must survive device reorder and project reload.
- Automation data stores breakpoints/curves over musical or sample time.
- Automation override/re-enable state is runtime/project editing state.

### Realtime implications

- Automation evaluation must produce block/sample parameter values without allocation.
- Recording automation writes from UI/controller gestures outside callback.

### UI/command implications

- Draw/move/delete points are commands.
- Parameter follow is UI state.

### Geist mapping notes

- `geist-automation` already has lanes/evaluator; add durable target identity and coexistence with device modulators.

## 11. Audio event editor

### Behavior

- Audio clip editing occurs in a detail editor panel.
- Audio events inside a clip carry expressions.
- Event expressions include stretch, onsets, gain, pan, pitch, and formant controls.
- Expression spread controls distribute/expose expression edits across selected material.
- Comping supports lanes/takes and a workflow for choosing segments.
- Takes can be added and edited, then assembled into a composite result.

### Data/project implications

- Audio clip contains event list and optional comp/take structure.
- Event expressions are per-event/per-point data, not global track automation.
- Takes need source references, ranges, and chosen segment masks.

### Realtime implications

- Gain/pan/pitch/formant/stretch expressions must be prepared for playback.
- Comp segment selection resolves before scheduler playback.
- Stretch/formant processing must be bounded or rendered/prepared.

### UI/command implications

- Event expression edits are clip-level commands.
- Comp lane edits and take selection are commands with clear undo.

### Geist mapping notes

- Add audio-event expressions to `geist-timeline` separately from `geist-automation` lanes.

## 12. Note event editor

### Behavior

- Note detail editor supports drawing notes, quick draw, note coloring, and layered editing.
- Note expressions include velocity, chance, gain, pan, timbre, and pressure.
- Micro-pitch editing adjusts pitch at note/expression level.
- Layered editing can show/edit by track, clip, channel, and with the audio editor.
- Layered comping extends comp workflow to note material.

### Data/project implications

- Note event fields: pitch, start, duration, velocity, channel, chance, gain, pan, timbre, pressure, micro-pitch/expression curves, mute/enabled state.
- Layered editing is UI selection/view state over multiple clips/tracks.
- Note comping needs take/segment representation analogous to audio comping.

### Realtime implications

- Chance/operators decide event emission deterministically per playback pass/seed policy.
- Expression values must be scheduled with notes and translated to MPE/MIDI/internal expression events.

### UI/command implications

- Drawing, quick draw, expression edit, micro-pitch, and layer selection are distinct commands/state.
- Layer visibility must not mutate notes unless an edit is committed.

### Geist mapping notes

- Extend `geist-core::events` and `geist-timeline::pattern::Note` toward expression/MPE-ready note data.

## 13. Operators

### Behavior

- Operators animate musical sequences at event level.
- Chance controls probabilistic event playback.
- Repeats create repeated triggering behavior.
- Occurrence gates events based on playback occurrence conditions.
- Recurrence controls cyclic repetition conditions.

### Data/project implications

- Operators belong to note/event data, not only clip-level processors.
- Store operator type, amount/condition, and seed/phase policy where needed.

### Realtime implications

- Operator evaluation must be deterministic for render if project seed/transport state is fixed.
- Repeats must schedule additional events without dynamic allocation in the callback.

### UI/command implications

- Operator edit UI should expose conditions without implying destructive note duplication.

### Geist mapping notes

- Implement operators as a MIDI/note event transformation layer before audio rendering.

## 14. Going between notes and audio

### Behavior

- Audio can be loaded into a sampler-style device workflow.
- This is a conversion/insertion path from audio asset to instrument/device state.

### Data/project implications

- Conversion creates device state referencing the audio asset and root/slicing/zone metadata.
- Original audio asset remains referenced; conversion should not require embedding bulk samples in device state.

### Realtime implications

- Analysis, slicing, root detection, and waveform preparation are offline/app-thread tasks.

### UI/command implications

- Browser/drag command should create track/device/asset references atomically.

### Geist mapping notes

- Align with Serum/Phase Plant sample/multisample asset requirements and `geist-project::asset_map`.

## 15. Projects, templates, exporting, and bouncing

### Behavior

- Projects can be saved as templates.
- Export/bounce workflows render project, track, clip, or selection material into audio or other deliverables.
- Bounce commits generated audio into project material.
- Export is a file-output operation, distinct from project save.

### Data/project implications

- Render jobs need source range, target tracks/buses, sample rate/format, normalization/dither choices if Geist supports them, and destination path.
- Bounce creates new audio assets with provenance linking source tracks/clips/devices.
- Templates share project schema with stripped runtime/render state.

### Realtime implications

- Offline render should use deterministic engine path with prepared graph state.
- File writes happen outside realtime callback.

### UI/command implications

- Bounce-to-project is undoable asset/project mutation.
- Export-to-file may be non-undoable external side effect but should be logged/reported.

### Geist mapping notes

- Build render job model before implementing broad export UI.

## 16. MIDI controllers and remote controls

### Behavior

- Controller status is visible in window/header context.
- Soft control assignments map hardware or remote controls to parameters.
- Remote controls pane exposes groups of assignable controls for selected devices/tracks.

### Data/project implications

- Separate global controller profiles from project/device remote mappings.
- Remote pages should reference stable parameter IDs.

### Realtime implications

- Incoming controller data enters via bounded event queues.
- Mapping resolution must avoid locks/allocation in processing path.

### UI/command implications

- Learn/assign/unassign are commands.
- Conflict and missing-controller states must be visible.

### Geist mapping notes

- Add controller mapping to `geist-config`; map to `geist-core` parameter/event IDs.

## 17. Advanced device concepts: nesting, containers, modulators

### Behavior

- Devices can contain nested chains.
- Many devices expose a mix parameter controlling dry/wet or blend behavior.
- Container devices host multiple chains/layers/selectors.
- Drum/instrument/FX layer concepts route notes/audio through nested chains.
- Other common chain types include sidechains, note FX chains, feedback-like or utility chain relationships where allowed.
- Modulators attach to devices and target parameters with depth.

### Data/project implications

- Device state must support child chains and routing roles.
- Container/layer state stores chain list, selection/blend policy, key/velocity/note routing if supported, and macros/remotes.
- Modulator state stores source type, parameters, target mappings, bipolar/unipolar policy, and depth.

### Realtime implications

- Nested chains compile into flat process steps for callback use.
- Container routing must be deterministic and bounded.
- Modulator evaluation must be bounded and rate-aware.

### UI/command implications

- Nested chain editing needs path-based selection and commands.
- Modulator mapping UI should show modulation amount without changing base parameter.

### Geist mapping notes

- Add graph subgraph/container metadata but keep compiled process list flat.
- Rework app rack model toward nested `DeviceState`.

## 18. The Grid

### Behavior

- The Grid is a modular editor inside instrument/effect contexts.
- Grid editor has module palette, module placement, interactive module help, inspector scopes, patch cords, insert-with-cord workflow, and module reordering.
- Modules expose ports and can be patched with cords.
- Patch cords define signal flow inside the Grid device.
- Module scopes/inspector expose signal visualization and editing context.

### Data/project implications

- Grid patch is device state containing modules, ports, cords, parameters, layout, and inspector/scope state where saved.
- Module palette is catalog metadata; module instances are project state.
- Patch cords need validation policy distinct from strict DAW track ports.

### Realtime implications

- Grid compiles to an internal subgraph.
- Patch edits happen on app thread and publish a new compiled graph.
- Scopes/meters use lock-free snapshots.

### UI/command implications

- Add/remove/connect/disconnect/reorder module commands.
- Invalid cords need clear validation messages.

### Geist mapping notes

- Use `geist-graph` subgraph/container work plus `geist-modular` voltage/channel policy.

## 19. Tablet profile and adaptive UI

### Behavior

- Tablet display profile provides alternate view arrangements for touch/tablet use.
- Tablet views adapt controls and layout for direct manipulation.

### Data/project implications

- Display profile is UI preference/session state.
- Project files should not require tablet profile to load safely.

### Realtime implications

- No audio implications beyond avoiding expensive UI work on frame loop.

### UI/command implications

- Touch interactions must still emit typed commands.
- Larger hit targets and simplified views should not create separate project semantics.

### Geist mapping notes

- Keep this as future `geist-ui` profile work, not core engine work.

## 20. Device descriptions and built-in device scope

### Behavior

- Public device descriptions enumerate many built-in instruments, effects, note FX, modulators, Grid modules, and analysis devices.
- Analysis devices such as oscilloscope/spectrum are observable UI/metering tools.
- Device descriptions inform capability classes: instruments, audio effects, note effects, containers, modulators, analysis, Grid modules.

### Data/project implications

- Geist should model device catalogs with category, IO type, parameter descriptors, preset/state support, and nesting/modulation capability.
- Analysis devices store display/config state but process meter/scope data as runtime snapshots.

### Realtime implications

- Analysis devices must be passive/tap-style processors with bounded buffers.
- Native device DSP must be independently designed and tested.

### UI/command implications

- Device browser should filter by device category and IO compatibility.
- Analysis scopes need decimated snapshots, not direct callback UI reads.

### Geist mapping notes

- Do not chase device parity. Use descriptions to define Geist's internal device taxonomy.

## 21. Clean-room gaps and accepted exclusions

- Exact Bitwig project file schema is not specified and must not be inferred.
- Exact UI layout, artwork, colors, icons, screenshots, names as branding, and device visual presentation are excluded.
- Exact time-stretch, pitch, formant, modulation, Grid, and device DSP algorithms are excluded and must be independently designed.
- Vendor packages, presets, controller scripts, Grid patches, content, and device presets are excluded.
- Individual built-in device parity is not a Geist requirement; only broad device-category behavior informs Geist architecture.
- Exact keyboard shortcuts and gestures are not copied; Geist should define its own command map.

## 22. Consolidated Geist implementation implications

| Spec area | Geist target |
|---|---|
| Project/templates/export | `geist-project` schema + app render job model |
| Launcher/arranger | `geist-timeline`, `app/geist-daw/src/engine.rs`, `session.rs` |
| Audio/note events | `geist-timeline::clip`, `pattern`, `geist-core::events` |
| Operators | future MIDI/note transform crate or `geist-timeline` event transform layer |
| Automation/modulation | `geist-automation` plus device-local modulator containers |
| Mixer/routing | `geist-graph`, app mixer, `geist-ui::MixerModel` |
| Nested devices/Grid | `geist-graph` subgraphs + `geist-core::DeviceState` nesting |
| Grid patching | `geist-modular` + modular connection policy |
| Browser/assets/packages | `geist-project::asset_map`, `geist-ui::BrowserModel`, app async scanners |
| Controllers/remotes | `geist-config`, `geist-core` parameter/event identity |
| Tablet/adaptive UI | `geist-ui` workflow profiles/display profiles |

## 23. Implementation warnings

- Keep arranger automation, clip expressions, event expressions, and device-local modulators distinct.
- Keep base parameter value separate from automation and modulation sums.
- Do not let Grid/modular patch permissiveness weaken strict DAW graph routing.
- Do not store runtime queued/playing launcher state as project truth unless explicitly recording/capturing performance.
- Do not implement stretch/formant/Grid/DSP algorithms by copying vendor behavior; specify and test original Geist algorithms.
- Keep package/browser scanning, waveform analysis, audio-to-sampler conversion, controller discovery, and export file I/O off the audio callback.
