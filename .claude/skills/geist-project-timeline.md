---
name: spectre-project-timeline
description: "Load when implementing or reviewing `geist-timeline`, `geist-automation`, `spectre-project`, transport, tempo maps, tracks, clips, patterns, automation lanes, modulation routes, project schema, migration, asset maps, settings, or autosave."
---

<!--
Author: Jeff
Date: 2026-05-27
Description: Timeline, automation, and persistence guide
Notes: Use for transport, clips, automation, modulation, save/load, schema, migration, assets, autosave
-->

# Geist Project Timeline

## Responsibility

Timeline and automation describe musical time. Project persistence serializes DAW state without hiding version or asset risks.

## Timeline rules

- Transport state is readable from audio thread as an atomic snapshot.
- Clips live in arenas; layout references IDs.
- Mutations are command objects for undo/redo.
- Tempo/time-signature changes are explicit map events.
- Playhead conversion between samples/beats is deterministic.

## Automation/modulation rules

- Automation lanes evaluate timeline curves per block.
- Modulation routes sum into parameter targets with clamp rules.
- Audio-rate modulation travels through graph CV/audio ports.
- Base parameter value and modulation sum remain distinguishable.

## Persistence rules

- Project format has an explicit schema version.
- Unknown fields are tolerated when possible.
- Breaking changes require migration functions.
- Audio files are referenced by relative path plus content hash; not embedded.
- Plugin state is opaque bytes tied to plugin identity.
- Autosave writes temp file then atomically swaps/renames.

## Implementation order

1. Transport and tempo map.
2. Track/clip/pattern/playhead model.
3. Command trait and undo/redo stack.
4. Automation curve segments and evaluator.
5. Modulation route/matrix resolution.
6. Project schema.
7. Serialization/deserialization.
8. Migration table.
9. Asset map.
10. Settings and autosave.

## Validation

- Roundtrip serialization tests.
- Migration tests for every schema bump.
- Tempo/playhead conversion edge cases.
- Modulation sum and clamp tests.
- Autosave crash-safety tests using temp dirs.
