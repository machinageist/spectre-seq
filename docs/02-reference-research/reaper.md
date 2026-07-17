<!--
Author: Jeff
Date: 2026-07-11
Description: Official-source inventory controls for REAPER behavioral, customization, and command research
Notes: Application and guide versions are intentionally represented separately
-->

# REAPER Reference Dossier

- **Status:** draft
- **Research state:** inventory-only
- **Last verified:** 2026-07-11
- **Scope:** REAPER recording, editing, routing, mixing, rendering, customization, command, and project workflows documented by official public sources
- **Decision authority:** Jeff
- **Upstream sources:** `SRC-REAPER-USER-GUIDE-775B`; `docs/02-reference-research/methodology.md`
- **Downstream dependents:** DAW workflow study, command ontology, recording/editing/routing requirements
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** how much end-user command composition Geist should support beyond remapping and validated declarative aliases
- **Known gaps:** guide table of contents, official video-series inventory, application/guide delta, chapter claims, Action List semantics, custom actions, mouse modifiers, accessibility, persistence, and failure behavior

## Version and source boundary

The official REAPER guide landing page advertised application version 7.77 dated July 7, 2026 while linking `ReaperUserGuide775b.pdf`, identifying guide version 7.75b. These are distinct facts and MUST NOT be collapsed into a claim that the guide documents application 7.77 completely.

The official landing page and linked PDF identity have been verified. The PDF table of contents and chapter contents have not yet been extracted in this repository. A denied local PDF-processing command was not retried; this dossier therefore records only evidence already available from the official landing page and direct guide identity.

## Research role

REAPER is a substantive DAW workflow reference, especially for:

- linear recording and editing;
- flexible track and routing behavior;
- multichannel audio, buses, sends, sidechains, and hardware I/O;
- comping, takes, item-based editing, and rendering;
- Actions List, custom actions, toolbars, mouse modifiers, templates, and user-defined workflows;
- keyboard-heavy and highly customized operation;
- project portability, backups, recovery, and render management where officially documented.

REAPER is not a project, script, extension, theme, action-ID, toolbar, macro, binding, or layout compatibility target.

## Required source family

| Source family | Verified state | Required next evidence |
|---|---|---|
| Official User Guide landing page | application 7.77 and linked guide 7.75b identity verified | preserve application/guide divergence across later claims |
| Official User Guide PDF | direct official PDF identity verified | deterministic TOC and chapter inventory |
| Official videos | rendered index and direct hash-linked titles inventoried; four end-to-end and eight focused candidates selected | watch and timestamp candidates; capture publication dates and demonstrated versions |
| Official forum/documentation links | not inventoried | bounded use for support behavior and command/customization evidence |

## Required coverage matrix

| Domain | Required official evidence | Current state |
|---|---|---|
| Project and application lifecycle | create/open/save, templates, backups, recovery, portability, compatibility | unreviewed |
| Audio/MIDI devices | setup, latency, routing, monitoring, hardware, reconfiguration | unreviewed |
| Transport and time | tempo/meter, loop, punch, count-in, sync, markers, regions | unreviewed |
| Tracks and routing | track roles, folders, channels, sends, receives, sidechains, master | unreviewed |
| Items, takes, lanes, comping | object state, editing, grouping, takes, fixed lanes, precedence | unreviewed |
| Recording | modes, monitoring, overdub, files, latency correction, failures | unreviewed |
| MIDI and notation | recording, editors, event operations, expression, notation | unreviewed |
| Automation and modulation | envelopes, modes, parameter identity, override, persistence | unreviewed |
| Mixer and effects | plugin chains, latency, bypass, freeze/render behavior | unreviewed |
| Rendering and project collection | mixes, stems, regions, queue, metadata, archive | unreviewed |
| Actions and customization | Action List, custom actions, scripts, toolbars, modifiers, menus | unreviewed |
| Focus and selection | command context, active window/editor, time/item/track selection | unreviewed |
| Undo and recovery | transaction boundaries, action safety, backups, crash recovery | unreviewed |
| Accessibility | keyboard-only operation, labels, screen-reader support, scaling | unreviewed |

## Command-system research controls

REAPER's command system is evidence about extensibility and repeated workflow composition. During extraction:

1. Semantic commands MUST be normalized independently from REAPER action IDs and default bindings.
2. Built-in actions, custom actions, extension actions, and executable scripts MUST remain separate evidence types.
3. Context and focus MUST be recorded; an action available in one editor does not prove global behavior.
4. User customization frequency requires field evidence, not inference from configurability.
5. Geist's current product constraint permits validated typed commands and declarative aliases, not arbitrary code execution by default.
6. REAPER action IDs, scripts, menu structures, toolbar layouts, and bindings MUST NOT be copied.

## Source-gap records

- `GAP-REAPER-0001`: official application version 7.77 and guide version 7.75b diverge.
- `GAP-REAPER-0002`: guide TOC and chapters have not been inventoried.
- `GAP-REAPER-0003`: official video index has initial workflow classification, but candidate contents, publication dates, and demonstrated versions remain unreviewed.
- `GAP-REAPER-0004`: Action List, custom-action, mouse-modifier, and focus semantics remain unreviewed.
- `GAP-REAPER-0005`: accessibility and keyboard-only evidence remains unreviewed.
- `GAP-REAPER-0006`: application changes after guide 7.75b require release-note or later-guide evidence.

## Safe next slices

Work can proceed without inventing behavior by:

- inventorying the official videos landing page and direct video categories;
- reviewing official HTML pages linked from the guide landing page;
- processing a locally supplied or separately authorized official guide copy;
- sampling complete practitioner workflows only as contextual field evidence;
- designing the semantic command schema without choosing Geist defaults.

## Acceptance blocker

This dossier cannot be accepted for product planning until the declared guide scope is inventoried, application/guide divergence is bounded, claim-level observations are sourced, command/customization evidence is separated by type, and field evidence establishes which configurable behaviors matter in repeated professional workflows.
