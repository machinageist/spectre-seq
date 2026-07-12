<!--
Author: Jeff
Date: 2026-07-11
Description: Official-source inventory controls for FL Studio behavioral and shortcut research
Notes: Version evidence is embedded in mutable official title artwork
-->

# FL Studio Reference Dossier

- **Status:** draft
- **Research state:** inventory-only
- **Last verified:** 2026-07-11
- **Scope:** FL Studio 26 project, pattern, sequencing, recording, mixing, editing, and command workflows documented by official public sources
- **Decision authority:** Jeff
- **Upstream sources:** `SRC-FL-STUDIO26-ONLINE-MANUAL`; `docs/02-reference-research/methodology.md`
- **Downstream dependents:** DAW workflow study, command ontology, arrangement/session requirements
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** none; source extraction remains incomplete
- **Known gaps:** deterministic table of contents, official shortcut inventory, chapter claims, cross-window focus semantics, persistence, undo, errors, and current patch-level alignment

## Version and source boundary

The official online manual entry and title page are unversioned mutable paths. On 2026-07-11, the official title artwork at `html/img_glob/title26.png` read “FL STUDIO 26 — REFERENCE MANUAL.” No labeled revision or publication date was displayed.

This artwork is explicit version evidence for the inspected rendering, but it is not a stable versioned manual identity. All observations must cite a direct official chapter URL and retain the FL Studio 26 association captured on the access date.

## Research role

FL Studio is a substantive DAW workflow reference, especially for:

- pattern-first composition;
- Channel Rack, Piano Roll, Playlist, and Mixer transitions;
- window-oriented command and focus behavior;
- beatmaking, sampling, automation, and rapid variation workflows;
- recording, routing, mixing, export, and project lifecycle behavior.

It is not a compatibility target. Geist does not seek FL Studio project, pattern, preset, plugin, layout, theme, or binding compatibility.

## Required coverage matrix

| Domain | Required official evidence | Current state |
|---|---|---|
| Project and application shell | project lifecycle, settings, templates, recovery, compatibility warnings | unreviewed |
| Pattern and channel model | pattern/channel state, mutation commands, Playlist relationship | unreviewed |
| Playlist and arrangement | clips, tracks, placement, editing, precedence, markers | unreviewed |
| Piano Roll and events | note/event editing, tools, expression, quantization, commands | unreviewed |
| Recording | audio/MIDI capture, monitoring, overdub, compensation, files, failures | unreviewed |
| Mixer and routing | inserts, sends, sidechains, buses, monitoring, latency | unreviewed |
| Automation | creation, targets, clips/events, override, persistence, editing | unreviewed |
| Browser and assets | discovery, preview, locations, plugins, missing media | unreviewed |
| Bounce/export | render modes, stems, formats, quality, dither, metadata | unreviewed |
| Commands and focus | official shortcuts, remapping, window/view scope, text/musical typing conflicts | unreviewed |
| Undo and recovery | transaction scope, destructive actions, autosave, crash recovery | unreviewed |

## Command-study controls

The direct official shortcuts chapter must be inventoried separately. Every binding row must identify platform, context, focused window, remappability, equivalent pointer action, discoverability, destructive consequence, and undo behavior. Function-key and window-switching patterns are workflow evidence; they are not defaults to copy.

## Source-gap records

- `GAP-FLSTUDIO-0001`: manual entry is unversioned and mutable.
- `GAP-FLSTUDIO-0002`: no revision/publication date was displayed.
- `GAP-FLSTUDIO-0003`: deterministic chapter inventory has not been captured.
- `GAP-FLSTUDIO-0004`: official shortcut semantics and context scopes remain unreviewed.

## Next extraction slice

Capture the rendered navigation tree as a deterministic section matrix and inventory the direct official shortcuts chapter. Do not extract product requirements until atomic observations have direct chapter provenance and workflow evidence is independently reviewed.
