<!--
Author: Jeff
Date: 2026-06-08
Description: Concrete UI interaction model for Spectre Seq.
Notes: Translates the VCV Rack, Ableton, Serum, and Phase Plant reference blend into buildable surfaces.
-->

# Spectre Seq UI Interaction Model

## North star

Geist should feel like a modular DAW where sound flow, musical time, and modulation are all visible parts of one instrument.

Reference blend:
- VCV Rack: visible signal relationships.
- Ableton: fast track, clip, browser, and arrangement workflow.
- Serum: readable parameter controls with modulation shown on the destination.
- Phase Plant: modular sound building without forcing users into a pure patch-bay mindset.

Creator workflow ownership is part of the north star. Geist should provide strong default lenses and also let users compose their own workflows through config files: Linux modularity applied to audio production, with safety rails for real-time audio and undoable project commands.

## Default screen layout

The default workspace uses four persistent regions:

1. Transport strip
   - playback, record, loop, tempo, meter, CPU/xrun health.
   - always visible.
   - never overloaded with editing tools.

2. Main canvas
   - arrangement, node graph, piano roll, or focused editor.
   - switches by lens, not by separate application mode.
   - selection remains stable across lenses.

3. Context shelf
   - appears beside or below the selected object.
   - shows immediate actions and key controls.
   - replaces most menu diving.

4. Search/add palette
   - invoked by click, keyboard, or empty-state action.
   - inserts tracks, clips, nodes, effects, modulators, samples, and commands.
   - context-aware: adding from a track suggests instruments/effects; adding from a parameter suggests modulation sources.

## Lens model

Views are lenses over the same project, not isolated modes.

### Arrange lens

Best for time, clips, recording, and automation lanes.

Primary actions:
- add track,
- record clip,
- drag sample into lane,
- draw/edit clip,
- reveal automation for selected parameter,
- jump to graph branch for selected track.

The arrangement should feel Ableton-fast without becoming a spreadsheet grid.

### Build lens

Best for node graph, routing, instruments, effects, and modulation.

Primary actions:
- add source,
- add processor,
- connect ports,
- insert node between cable endpoints,
- collapse node group,
- trace output back to source.

The build lens borrows VCV Rack visibility, but nodes stay semantic and compact instead of hardware-panel-like.

### Shape lens

Best for focused sound design on a selected instrument/effect chain.

Primary actions:
- edit oscillator/filter/envelope/LFO shapes,
- assign modulation by dragging source handles to controls,
- reorder modules,
- bypass modules,
- pin important controls.

The shape lens borrows Serum's destination-visible modulation and Phase Plant's module stacking.

### Mix lens

Best for balance, routing, and monitoring.

Primary actions:
- adjust level/pan,
- arm/mute/solo,
- inspect inserts/sends,
- view meters,
- reveal selected channel graph.

The mix lens should not look like a hardware console. It is a compact routing and monitoring lens.

## Selection behavior

Selection is the anchor for low-menu UI.

When a user selects:
- track: show level, input, output, arm/mute/solo, inserts, sends, graph branch.
- clip: show gain, fades, loop, warp/time, source, automation relation.
- node: show key parameters, ports, bypass, reorder/group, latency/health.
- cable: show source, destination, signal type, latency, remove/replace.
- parameter: show value, unit, default, automation, modulation sources, assign controls.
- modulation route: show source, destination, depth, polarity, curve, mute/remove.

Every context shelf action should be available by command palette too.

## Parameter interaction

Parameter controls must be self-explanatory.

Each dial/fader shows:
- name,
- current value,
- unit,
- range or scale,
- default mark,
- automation status,
- modulation contribution.

Interaction:
- drag adjusts value,
- shift/ctrl drag fine-adjusts,
- double-click resets,
- click value opens direct numeric entry,
- drag modulation source onto control assigns route,
- hover/tap reveals source contributions.

## Modulation interaction

Modulation should be visible where it matters: on the destination.

Assignment flow:
1. User drags a source handle from LFO, envelope, CV node, clip automation, or macro.
2. Valid destination controls glow with typed compatibility.
3. Dropping creates a route and opens a local depth control.
4. The destination shows a modulation ring/overlay.
5. Clicking the overlay shows route details and remove/mute.

The modulation matrix may exist as an overview, but it is not the primary workflow.

## Node graph interaction

Node graph rules:
- Add nodes through search/add palette, empty-state cards, or cable insertion.
- Drag cable from output to compatible input.
- Preview cable explains signal type and destination before commit.
- Invalid route previews show why: type mismatch, cycle policy, unavailable input, duplicate route.
- Dropping on empty canvas opens filtered node search for compatible targets.
- Dropping on an existing cable offers insert-compatible processors.
- Feedback routes show delay marker explicitly.

## Browser interaction

Browser is search-first and context-aware.

Default sections:
- recent,
- favorites,
- instruments,
- effects,
- modulators,
- samples,
- commands.

Insertion behavior:
- drag sample to arrangement creates audio clip.
- drag instrument to empty track creates instrument track.
- drag effect to track creates insert chip.
- drag modulator to parameter creates modulation route.
- drag node to graph inserts node.

## Menu policy

Menus remain shallow and conventional:
- File,
- Edit,
- View,
- Help.

Anything used during normal music creation must also exist in one of:
- context shelf,
- direct manipulation,
- empty-state action,
- search/add palette,
- keyboard shortcut.

## Configurable workflow model

The default interaction model is implemented as a built-in workflow profile, not as hardcoded UI inevitability.

Workflow profiles can configure:
- startup lens,
- lens order and visibility,
- persistent panel placement,
- context shelf action order,
- command aliases,
- shortcuts and controller bindings,
- browser sections and favorites,
- track/rack/graph templates,
- graph and modulation display density.

Configurable workflows must preserve core interaction invariants:
- common actions remain visible or one gesture away,
- hidden lenses remain reachable through command search,
- primary controls keep labels, values, units, and state,
- mutating actions still emit typed `UICommand` values,
- config reload never touches the audio callback,
- invalid config falls back to the last valid/default workflow.

See `docs/ui_configuration_model.md` for config file shape, precedence, safety boundaries, and acceptance checks.

## First UI prototype slice

Build the first prototype around one track with one instrument, one effect, one modulation source, and master output.

Required visible flow:
1. Empty project offers `Add Track`, `Add Instrument`, `Add Sample`, `Open Browser`.
2. Adding an instrument creates a track and graph branch.
3. Track selection shows context shelf with level, mute/solo/arm, output, inserts.
4. Build lens shows instrument → effect → output as semantic nodes.
5. A modulation source can be dragged onto a visible dial.
6. The dial shows modulation overlay and route depth.
7. Mix lens shows level and meter without hardware-console mimicry.

Prototype acceptance:
- no primary action requires nested menu navigation,
- no unlabeled toggle/dial/fader appears,
- selected object explains available actions locally,
- graph and mixer reflect the same selected track,
- modulation assignment is visible on the destination control.
