<!--
Author: Jeff
Date: 2026-07-11
Description: Official-source inventory controls for Cubase Pro behavioral and command research
Notes: The 15.0 Webhelp branch currently includes 15.0.20 material
-->

# Cubase Pro Reference Dossier

- **Status:** draft
- **Research state:** inventory-only
- **Last verified:** 2026-07-11
- **Scope:** Cubase Pro 15.0-branch production, recording, editing, mixing, scoring, command, and project workflows
- **Decision authority:** Jeff
- **Upstream sources:** `SRC-CUBASE-PRO15.0-WEBHELP`; `docs/02-reference-research/methodology.md`
- **Downstream dependents:** DAW workflow study, command ontology, recording/editing/mixing requirements
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** whether scoring and post-production workflows are first-stage Geist targets
- **Known gaps:** deterministic TOC, key commands/macros/logical editors, patch-pinned snapshot, chapter claims, persistence, error handling, and accessibility evidence

## Version and source boundary

The official Steinberg Webhelp identifies Cubase Pro 15.0 and currently contains “New Features in Version 15.0.20” as well as 15.0.0 material. Its URL pins the 15.0 branch, not a patch-level snapshot. No labeled revision or publication date was shown.

An older `cubase-manuals` URL redirected to an archive page that returned a 404 and is not a current official entry point. Claims must cite the active Webhelp and preserve its mutable-branch caveat.

## Research role

Cubase Pro is a substantive DAW reference for:

- linear recording and editing;
- advanced MIDI and event transformation;
- comping, vocal editing, and multitrack production;
- mixing, Control Room, routing, automation, and export;
- scoring and long-timeline workflows when in scope;
- key commands, macros, workspaces, modifier tools, and logical editors.

It is not a project, preset, plugin, layout, macro, logical-editor, or key-command compatibility target.

## Required coverage matrix

| Domain | Current state |
|---|---|
| Project setup, templates, backups, save/recovery, compatibility | unreviewed |
| Audio/MIDI devices, latency, Control Room, external hardware | unreviewed |
| Transport, tempo/meter, markers, sync, video/scoring time | unreviewed |
| Tracks, folders, groups, FX channels, routing, sends, sidechains | unreviewed |
| Parts/events, arrangement, lanes, takes, comping | unreviewed |
| Audio editing, fades, warp/time/pitch, destructive boundaries | unreviewed |
| MIDI editors, expression, logical editors, transformations | unreviewed |
| Recording, monitoring, punch, retrospective capture, files | unreviewed |
| Automation, parameter identity, override, controller mapping | unreviewed |
| Mixer, latency compensation, freeze/render/export | unreviewed |
| MediaBay/browser, presets, plugins, missing media | unreviewed |
| Key commands, macros, workspaces, modifiers, focus | unreviewed |
| Accessibility and keyboard-only operation | unreviewed |

## Command-study controls

Macros and logical editors are evidence about composable typed operations. They do not authorize arbitrary scripting in Geist. Each extracted command must retain context, selection/focus requirements, remappability, destructive effect, undo behavior, equivalent gesture, and discoverability.

## Source-gap records

- `GAP-CUBASE-0001`: Webhelp URL is branch-pinned but patch-mutable.
- `GAP-CUBASE-0002`: no revision/publication date was shown.
- `GAP-CUBASE-0003`: deterministic section inventory has not been captured.
- `GAP-CUBASE-0004`: command, macro, workspace, modifier, and logical-editor semantics remain unreviewed.
- `GAP-CUBASE-0005`: scoring/post-production relevance to Geist remains a product-scope decision.

## Next extraction slice

Capture the full rendered Webhelp TOC as a deterministic matrix. Prioritize project safety, recording/comping, routing/mixing, MIDI editing, and command-system chapters; keep scoring/post-production sections classified but bounded pending product scope.
