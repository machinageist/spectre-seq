<!--
Author: Jeff
Date: 2026-06-08
Description: Product UI/UX principles for Spectre Seq.
Notes: Establishes interaction direction before Phase 8 implementation.
-->

# Spectre Seq UI/UX Principles

## Product stance

Spectre is a sound-building environment, not a spreadsheet and not a skeuomorphic rack.

The reference blend is VCV Rack's visible patch logic, Ableton's fast arrangement/session flow, Serum's readable sound-design controls, and Phase Plant's modular immediacy. Spectre should borrow their clarity and speed without becoming a clone of any one product.

The interface should feel direct, legible, and instrument-like without copying physical hardware. It should expose the shape of the sound graph, the musical timeline, and the active controls in a way that rewards exploration without forcing menu diving.

Spectre should also bring Linux modularity to audio production. The default UI is a strong opinionated starting point, not a cage. Creators should be able to author workflow profiles in config files so the DAW can match a modular sound-design workflow, songwriting workflow, mixing workflow, performance workflow, or something personal without forking the app.

## Reference grammar

### VCV Rack: visible patch logic

Borrow:
- signal flow that can be followed by eye,
- ports and cables as explicit relationships,
- exploratory drag-to-connect behavior,
- immediate feedback when a route is valid or invalid.

Avoid:
- dense wall-of-modules layouts,
- tiny unlabeled jacks as the only explanation,
- hardware-panel mimicry as the default surface.

### Ableton: fast musical flow

Borrow:
- low-friction track/clip creation,
- fast arrangement editing,
- clear transport and loop state,
- browser-driven insertion,
- command speed for common production actions.

Avoid:
- spreadsheet-like clip grids as the dominant identity,
- deep preference/menu dependence,
- separating sound design so far from arrangement that context is lost.

### Serum: readable sound controls

Borrow:
- visual parameter feedback,
- modulation rings/overlays on destination controls,
- clear envelopes/LFOs/wavetables as editable shapes,
- immediate sense of what changes the sound.

Avoid:
- banks of identical unlabeled knobs,
- tab sprawl for core controls,
- decorative panels that hide signal meaning.

### Phase Plant: modular immediacy

Borrow:
- add-source/add-effect/add-modulator flow,
- modular sound construction without requiring patch-bay expertise,
- inline module reordering and bypass,
- destinations that reveal modulation assignment locally.

Avoid:
- turning every action into a matrix edit,
- hiding routing behind abstract lists,
- making advanced routing feel like a separate mode.

## Non-goals

- Do not present project state as dense tables by default.
- Do not mimic a hardware mixer, modular rack, tape machine, or synth panel as the primary metaphor.
- Do not hide common actions behind deep nested menus.
- Do not require users to remember unlabeled icon-only controls.
- Do not make parameter editing depend on modal dialogs for normal use.

## Core UX principles

### 1. Object first, action nearby

Selecting a track, clip, node, cable, modulation route, or parameter reveals the relevant actions adjacent to that object.

Common actions live in contextual surfaces:
- inline chips,
- small inspectors,
- radial or shelf actions,
- hover/touch affordances,
- command palette search.

Menus are fallback navigation, not the main workflow.

### 2. One visible primary path

Each view should make the next likely action obvious.

Examples:
- Empty arrangement shows clear calls to add an audio track, instrument track, sample, or pattern.
- Empty node graph shows obvious source, processor, modulation, and output entry points.
- Selected parameter shows value, range, modulation depth, automation status, and reset in one local control cluster.

### 3. Controls explain themselves

Toggles, dials, faders, and routing handles must be readable without a manual.

A control should communicate:
- current value,
- unit or state,
- valid range,
- modulation/automation overlay,
- disabled reason when unavailable,
- reset/default affordance.

Prefer short labels plus visual state over mystery icons.

### 4. Progressive disclosure without menu diving

Advanced options should unfold in place from the selected object.

Use:
- expandable inline sections,
- popovers anchored to controls,
- detail drawers,
- keyboard command search,
- persistent favorites/pins for repeated controls.

Avoid:
- nested settings pages,
- multi-step modal workflows,
- large property grids.

### 5. Spatial sound graph, not patch-cable cosplay

The node graph is the signature UI, but it should visualize signal relationships rather than imitate a modular synth panel.

Guidelines:
- Nodes are compact semantic blocks, not device faceplates.
- Port color and shape identify signal type.
- Cable labels and hover traces explain routing.
- Invalid connections fail before commit and explain why.
- Feedback loops are visible as intentional delayed paths.
- Modulation routes are first-class overlays, not hidden matrix rows.

### 6. Timeline and graph stay linked

Arrangement, mixer, rack, modulation, and graph views are different lenses over the same project.

Selecting an object in one view should highlight related objects elsewhere:
- track ↔ graph branch,
- clip ↔ triggering node/pattern,
- parameter ↔ automation lane/modulation routes,
- meter ↔ source/output path.

### 7. Gesture-light, keyboard-friendly

Mouse, trackpad, touch, and keyboard should all feel first-class.

Baseline interactions:
- drag to connect,
- drag to reorder,
- click/tap to select,
- option/alt drag to duplicate where safe,
- type-to-search command palette,
- direct numeric entry on any value control,
- undoable mutating actions.

### 8. Fast feedback beats decorative fidelity

Visual polish should serve timing, confidence, and comprehension.

Prioritize:
- responsive meters,
- clear clipping/xrun warnings,
- visible automation/modulation movement,
- stable layout under playback,
- low frame-loop cost.

Avoid expensive animation that obscures audio state or competes with real-time work.

### 9. Creator-authored workflows

Workflow customization is a core product principle, not a cosmetic preference.

Spectre should let creators define their own working shape through versioned config files:
- startup lens,
- lens order and visibility,
- panel layout,
- command aliases,
- shortcuts and control bindings,
- browser categories and favorites,
- track/rack/graph templates,
- context shelf action order,
- visual density for graph, meters, modulation, and clips.

The default workflow must remain excellent with no config, but advanced users should be able to make Spectre feel like their own modular audio workstation.

Configuration must remain declarative and safe. It can bind and arrange typed commands; it cannot execute arbitrary code, bypass validation, or put work on the audio callback.

See `docs/ui_configuration_model.md` for the detailed configuration model.

## Primary surfaces

### Arrangement

Purpose: compose time.

Default feel: spacious lanes with clips as musical objects, not cells.

Must expose:
- add track,
- add/record clip,
- loop/region controls,
- clip gain/fades,
- automation reveal,
- relation to graph/rack.

### Mixer

Purpose: balance and monitor.

Default feel: compact signal overview, not a hardware console replica.

Must expose:
- level and pan,
- mute/solo/arm,
- meter and peak hold,
- inserts/sends as semantic chips,
- routing target,
- selected channel inspector.

Faders may be vertical where useful, but the strip should not become a skeuomorphic console.

### Node graph

Purpose: build and understand sound flow.

Default feel: spatial map of signal, modulation, and processing.

Must expose:
- typed ports,
- valid connection previews,
- cable tracing,
- node search/add,
- group/collapse,
- inline important parameters,
- route health and latency markers.

### Plugin rack

Purpose: edit a selected chain without losing context.

Default feel: stacked semantic modules with focused controls.

Must expose:
- reorder,
- bypass,
- wet/dry where applicable,
- key parameters,
- open full plugin UI as optional detail,
- pin/favorite controls into the main surface.

### Modulation

Purpose: show what moves what.

Default feel: living overlay attached to parameters and sources.

Must expose:
- source,
- destination,
- depth,
- polarity,
- active value contribution,
- mute/remove,
- conflict/overdrive indication.

Avoid making the modulation matrix the only primary editor.

### Browser

Purpose: find and insert sounds, plugins, and nodes.

Default feel: search-first palette with tags and preview, not filesystem-first navigation.

Must expose:
- type-to-search,
- filter chips,
- preview/audition,
- drag/drop insertion,
- recent/favorites,
- contextual add targets.

## Control language

### Toggles

Use for binary state only.

Requirements:
- explicit label,
- clear on/off state,
- optional short state text when ambiguity exists,
- disabled explanation on hover/tap.

Examples:
- `Bypass On/Off`
- `Loop On/Off`
- `Sync Free/Tempo` only if it is truly binary; otherwise use segmented control.

### Dials

Use for continuous parameters where circular motion maps well to musician expectation.

Requirements:
- label above or beside,
- numeric value and unit,
- visible range arc,
- default marker,
- modulation ring when modulated,
- automation mark when automated,
- drag plus direct numeric entry.

Avoid unlabeled knob banks.

### Faders

Use for level-like continuous controls.

Requirements:
- scale markings where meaningful,
- meter adjacency for gain staging,
- peak/clipping indication,
- reset/default affordance.

### Chips

Use for compact routable or reorderable objects.

Good targets:
- insert plugins,
- sends,
- route destinations,
- modulation sources,
- track tags,
- clip states.

Chips should be draggable and inspectable.

## Menu policy

Top-level menus may exist for platform conventions and rare actions.

Primary workflows should be reachable through:
- visible empty-state actions,
- contextual object actions,
- command palette,
- drag/drop,
- direct manipulation.

If a command is used often, it does not belong only in a menu.

## Empty-state policy

Every empty view must teach the workflow.

An empty state should answer:
- What is this view for?
- What can I add here?
- What is the fastest first action?
- What will happen if I drag something into this area?

## Validation checklist for UI work

Before accepting a UI slice, verify:
- Common action is visible or one gesture away.
- No new common workflow requires nested menu navigation.
- Every toggle/dial/fader has a label and readable state.
- Controls show units/ranges where applicable.
- Disabled controls explain why.
- Mutating actions emit typed commands and remain undoable.
- UI does not own project/audio ground truth.
- Frame-loop work remains bounded and non-blocking.
