---
name: spectre-ui-workflow
description: "Load when implementing or reviewing `crates/spectre-ui`, renderer abstractions, egui integration, app/UI state, command dispatch, views, widgets, meters, waveform rendering, or node-graph interactions."
---

<!--
Author: Jeff
Date: 2026-05-27
Description: UI implementation guide
Notes: Use for spectre-ui views, widgets, renderer, app state, and command dispatch
-->

# Spectre UI Workflow

## Responsibility

The UI renders project state and emits commands. It does not own DAW truth.

## Rules

- UI state stores selection, zoom, focus, cursor, and transient interaction state.
- Project/audio state remains outside UI widgets.
- Widgets emit commands; app layer validates and applies them.
- Metering consumes lock-free snapshots from audio side.
- Expensive waveform/plugin scans run outside the frame loop.
- Renderer trait keeps egui replaceable by future wgpu renderer.

## Implementation order

1. Define `UIState` and `UICommand`.
2. Define renderer trait.
3. Implement egui renderer shell.
4. Build mixer view first.
5. Build node graph view second.
6. Build piano roll, arrangement, plugin rack, browser, modulation views.
7. Build reusable widgets: knob, fader, meter, cable, waveform, piano.
8. Add screenshot/manual testing notes when automated UI tests are not practical.

## Node graph UX invariants

- Port colors reflect port type.
- Drag-to-connect validates before committing.
- Invalid connections explain why.
- Selection/rubber-band actions are reversible commands where they mutate project state.

## Review checklist

- UI code does not lock audio callback structures.
- Widget state is local and disposable.
- Commands are typed and narrow.
- View files remain thin; shared behavior moves to widgets/app commands.
