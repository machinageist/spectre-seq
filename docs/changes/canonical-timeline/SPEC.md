<!--
Author: Jeff
Date: 2026-07-31
Description: Product contract for canonical timeline ownership and musical time.
Notes: This contract governs the phased replacement of app-owned floating-point arrangement truth.
-->

# Canonical Timeline and Musical Time

## Status

Approved for implementation with 960 PPQ as the canonical arrangement timebase.

## Problem

Geist currently has conflicting arrangement models:

- `spectre-timeline` stores clip and note placement in absolute samples.
- `geist-ui` and `StudioApp` store arrangement placement in floating-point beats.
- Project persistence stores floating-point beats.
- App-level diffing translates UI mutations directly into engine commands.

This prevents one authoritative timeline, deterministic editing, complete undo/redo, and reliable persistence migrations.

## Decisions

### Time domains

- Arrangement positions and edit history use unsigned integer musical ticks.
- One quarter-note beat equals 960 ticks.
- Audio source offsets and source lengths remain sample-based.
- Playback converts musical positions through the tempo map at snapshot or command-publication boundaries.
- Audio begins at its musical anchor and plays at its recorded sample rate.
- Tempo changes do not imply audio time-stretch.
- A per-clip musical/absolute timebase is excluded until explicitly designed.

### Ownership

- `spectre-timeline::Timeline` becomes arrangement truth.
- UI timeline structures become disposable projections.
- UI interactions emit typed edit intents; they do not own durable arrangement mutation.
- The app thread executes reversible commands against the canonical timeline.
- The audio thread consumes bounded commands or immutable snapshots and never accesses mutable timeline state.

### Identity

- Runtime arena handles are not persistent project identity.
- Persistent clip and track IDs require explicit stable newtypes before project migration.
- UI selection refers to stable IDs, never vector positions.

## Conversion policy

- Tick-to-beat conversion is checked through the inclusive `2^52` boundary, preserving adjacent 1/960-beat tick resolution.
- Finite, nonnegative beats quantize to the nearest tick; exact half-tick ties round upward.
- Invalid beat values (`NaN`, infinity, or negative values) are rejected rather than clamped silently.
- Tick-to-sample conversion integrates the tempo map and rounds to the nearest absolute sample.
- Sample-to-tick conversion integrates the tempo map and rounds to the nearest tick.
- Float-mediated sample inputs and outputs above `2^53` are rejected.
- Float-mediated musical tick inputs and outputs above `2^52` are rejected until TempoMap uses integer-aware segment math.
- Conversions that cannot fit their destination integer type are rejected rather than saturated.
- Tempo-map sample rate is never treated as zero.
- Conversion APIs are app-thread utilities; the realtime callback receives already prepared sample-domain state.

## Migration sequence

1. Add and validate canonical musical-time primitives and TempoMap adapters.
2. Extend canonical placements and commands to represent create, delete, move, resize, and cross-track movement in ticks.
3. Introduce stable track and clip IDs distinct from arena handles.
4. Project canonical state into renderer-facing UI models.
5. Route arrangement edits through command execution and undo/redo.
6. Derive engine updates from committed canonical mutations.
7. Migrate project persistence with an explicit schema version and compatibility tests.
8. Remove app-owned arrangement truth only after behavioral parity.

## Invariants

- No floating-point value is authoritative arrangement edit history after migration.
- No migration slice creates duplicate mutable arrangement truth without an explicit mirror boundary and parity test.
- Undo followed by redo restores identical IDs, placements, lengths, track ownership, and media references.
- Tempo edits preserve musical clip positions.
- Existing audio source identity and offline-media behavior remain intact.
- No timeline edit allocates, locks, blocks, logs, or performs I/O on the audio callback.

## First-slice scope

The first slice adds:

- `MusicalTime`, an opaque 960-PPQ tick newtype.
- Checked arithmetic and beat quantization.
- TempoMap adapters between ticks and absolute samples.
- Unit tests for boundaries, quantization, tempo changes, and round trips.

The first slice does not migrate placements, patterns, commands, UI models, project schemas, engine commands, or transport snapshots.

## Acceptance criteria

1. The PPQ constant is public and fixed at 960.
2. Raw ticks round-trip without representation loss.
3. Beat conversion rejects non-finite, negative, and inexact integer-domain inputs.
4. Exact half-tick boundaries round upward.
5. Checked addition and subtraction never wrap.
6. Constant-tempo tick/sample conversion matches known values.
7. Mid-timeline tempo changes integrate correctly in both directions.
8. Existing beat/sample APIs remain behaviorally compatible for nonzero sample rates.
9. `cargo test -p spectre-timeline`, `cargo check --workspace`, and `git diff --check` pass.
