<!--
Author: Jeff
Date: 2026-07-11
Description: Versioned clean-room source inventory and coverage matrix for Ableton Live 12
Notes: Inventory only; no external behavior is adopted as a Geist requirement here
-->

# Ableton Live 12 Reference Dossier

- **Status:** draft
- **Research state:** inventory-only
- **Last verified:** 2026-07-11
- **Scope:** official Ableton Live 12 manual coverage relevant to DAW workflows and interaction research
- **Decision authority:** Jeff
- **Upstream sources:** `SRC-ABLETON-LIVE12-MANUAL-WELCOME`; `docs/02-reference-research/methodology.md`
- **Downstream dependents:** workflow field study, Geist requirements ledger, application and subsystem specifications
- **Supersedes:** none; legacy `docs/specs/ableton-live-clean-room-spec.md` remains historical evidence pending extraction
- **Superseded by:** none
- **Open decisions:** which device-reference chapters merit full behavioral extraction versus capability classification
- **Known gaps:** chapter contents, subsections, cross-feature behavior, errors, persistence, undo, realtime implications, and workflow corroboration remain unreviewed

## Boundary

This dossier inventories publicly documented Live 12 behavior. It does not approve feature parity, file compatibility, copied bindings, vendor limits, Max for Live support, Push support, visual imitation, or any Geist architecture.

## Version evidence

The official page title and navigation identify “Ableton Reference Manual Version 12,” and chapter URLs are under `/live-manual/12/`. The top-level navigation rendered 42 numbered chapters on 2026-07-11.

## Coverage-state vocabulary

- `inventory-only`: title and direct chapter URL captured.
- `section-inventoried`: the complete rendered heading hierarchy was captured from the direct chapter page, but section prose has not yet produced atomic claims.
- `out-of-scope-candidate`: likely outside Geist's active product scope, but exclusion requires review and rationale.
- `claims-extracted`: atomic observations exist with source locations.
- `reviewed-no-relevant-claims`: chapter reviewed and no Geist-relevant observations found.
- `blocked-source-gap`: source cannot establish the needed behavior.

## Top-level manual coverage matrix

Every row is currently `inventory-only`; listing is not claim extraction.

| Chapter | Official title | Geist research domain | State |
|---:|---|---|---|
| 1 | Welcome to Live | source scope and terminology | inventory-only |
| 2 | First Steps | installation, settings, audio/MIDI setup | inventory-only |
| 3 | Live Concepts | project/object model | section-inventoried |
| 4 | Working with the Browser | browser, search, tags, preview | section-inventoried |
| 5 | Managing Files and Sets | project lifecycle, assets, missing media | section-inventoried |
| 6 | Arrangement View | arrangement | claims-extracted \|
| 7 | Session View | performance/session | claims-extracted \|
| 8 | Clip View | clip state and editing | claims-extracted \|
| 9 | Audio Clips, Tempo, and Warping | audio editing, time stretch, tempo | claims-extracted \|
| 10 | Editing MIDI | note/event editing | inventory-only |
| 11 | MIDI Tools | MIDI transformation/generation | inventory-only |
| 12 | Editing MPE | per-note expression | inventory-only |
| 13 | Converting Audio to MIDI | analysis/conversion workflow | inventory-only |
| 14 | Using Grooves | timing/groove | inventory-only |
| 15 | Using Tuning Systems | tuning and pitch semantics | inventory-only |
| 16 | Launching Clips | launch state, quantization, follow behavior | claims-extracted \|
| 17 | Routing and I/O | routing, monitoring, external I/O | claims-extracted \|
| 18 | Mixing | mixer, sends, buses, latency | claims-extracted \|
| 19 | Recording New Clips | audio/MIDI recording | claims-extracted \|
| 20 | Bounce to Audio | bounce/resampling/offline workflow | inventory-only |
| 21 | Comping | takes and comping | inventory-only |
| 22 | Stem Separation | destructive/non-destructive asset transformation | inventory-only |
| 23 | Working with Instruments and Effects | device lifecycle, chains, plugins | inventory-only |
| 24 | Instrument, Drum and Effect Racks | nesting, parallel chains, macros | inventory-only |
| 25 | Automation and Editing Envelopes | automation semantics | claims-extracted \|
| 26 | Clip Envelopes | clip-local automation/modulation | inventory-only |
| 27 | Working with Video | scoring/video; scope undecided | inventory-only |
| 28 | Live Audio Effect Reference | native-effect capability taxonomy | inventory-only |
| 29 | Live MIDI Effect Reference | native MIDI-tool taxonomy | inventory-only |
| 30 | Live Instrument Reference | native-instrument taxonomy | inventory-only |
| 31 | Max for Live | extension workflow; compatibility not intended | inventory-only |
| 32 | Max for Live Devices | extension-device examples; compatibility not intended | inventory-only |
| 33 | MIDI and Key Remote Control | mappings and remote control | inventory-only |
| 34 | Using Push 1 | controller workflow; hardware compatibility not intended | inventory-only |
| 35 | Using Push 2 | controller workflow; hardware compatibility not intended | inventory-only |
| 36 | Synchronizing with Link, Tempo Follower, and MIDI | synchronization and external control | inventory-only |
| 37 | Computer Audio Resources and Strategies | performance and reliability guidance | inventory-only |
| 38 | Audio Fact Sheet | signal, recording, render behavior | inventory-only |
| 39 | MIDI Fact Sheet | MIDI behavior | inventory-only |
| 40 | Accessibility and Keyboard Navigation | accessibility and keyboard operation | inventory-only |
| 41 | Live Keyboard Shortcuts | semantic commands and bindings | claims-extracted \|
| 42 | Credits | provenance only | inventory-only |

## Direct chapter identity

Chapter URLs follow the official versioned form `https://www.ableton.com/en/live-manual/12/<chapter-slug>/`. Exact URLs were captured from rendered navigation during this inventory. Atomic claims MUST cite the exact direct chapter URL and subsection anchor rather than this matrix alone.

## Section-level coverage matrix

This matrix records the complete rendered heading hierarchy for the chapters inspected so far. `section-inventoried` establishes discoverability and source location only. It does not mean the prose, edge cases, or cross-feature behavior have been extracted.

### Chapter 3 — Live Concepts

Source: `https://www.ableton.com/en/live-manual/12/live-concepts/`

| Sections | Geist research domain | State | Extraction gap |
|---|---|---|---|
| 3.1 Control Bar; 3.2 Status Bar | application shell, transport, diagnostics, global state | section-inventoried | command state, failure states, focus, persistence, and accessibility |
| 3.3 Browser; 3.4 Sound Similarity | browser, assets, search and recommendation | section-inventoried | indexing, cache, preview, missing assets, errors, and deterministic behavior |
| 3.5 Live Sets; 3.6 Arrangement and Session | project model and dual performance/timeline surfaces | section-inventoried | state authority, precedence, capture, undo, persistence, and recovery |
| 3.7 Tracks; 3.8 Audio and MIDI | track types, events, signal and event domains | section-inventoried | ownership, routing, timestamping, channel layouts, and limits |
| 3.9 Audio Clips and Samples; 3.10 MIDI Clips and MIDI Files | clip and asset model | section-inventoried | destructive boundaries, file ownership, note-off rules, migration, and portability |
| 3.11 Devices; 3.12 Clip and Device View | device lifecycle and editor surfaces | section-inventoried | lifecycle, stable identity, state, latency, failure containment, and UI authority |
| 3.13 Scale Awareness; 3.14 Mixer | tuning, mixer, signal flow | section-inventoried | tuning propagation, buses, monitoring, compensation, metering, and overload behavior |
| 3.15 Presets and Racks; 3.16 Routing | presets, nesting, parallel chains, routing | section-inventoried | graph semantics, feedback, state envelopes, missing devices, and error diagnostics |
| 3.17 Recording New Clips | audio and MIDI recording | section-inventoried | latency correction, media lifecycle, dropout, disk failure, and crash salvage |
| 3.18 Automation Envelopes; 3.19 Clip Envelopes | automation and clip-local control | section-inventoried | evaluation order, override, smoothing, identity, loop boundaries, and persistence |
| 3.20 MIDI and Key Remote | command mapping and external control | section-inventoried | focus conflicts, remapping, feedback, accessibility, and controller disconnect behavior |
| 3.21 Saving and Exporting | project lifecycle and rendering | section-inventoried | atomicity, migration, collection, offline semantics, interruption, and recovery |

### Chapter 4 — Working with the Browser

Source: `https://www.ableton.com/en/live-manual/12/working-with-the-browser/`

| Sections | Geist research domain | State | Extraction gap |
|---|---|---|---|
| 4.1 Content Pane; 4.2 Search Bar; 4.2.1 Saving Search Results as Custom Labels | asset results, search, saved queries | section-inventoried | query semantics, indexing latency, empty/error states, persistence, and keyboard operation |
| 4.3 Browser History | navigation state | section-inventoried | history scope, invalidation, project boundaries, and undo distinction |
| 4.4 Filters and Tags; 4.4.1 Filter Groups; 4.4.2 Tags; 4.4.3 Tag Editor; 4.4.4 Quick Tags | metadata and faceted search | section-inventoried | metadata authority, user edits, conflicts, migration, and batch operation failure |
| 4.5 Collections; 4.6 Library | favorites and installed content | section-inventoried | stable identity, portability, unavailable items, and content licensing boundaries |
| 4.7 Places | external and user-controlled locations | section-inventoried | permissions, disconnects, path portability, case sensitivity, and rescanning |
| 4.7.1 Downloading and Installing Packs; 4.7.2 Pack Info | downloadable content | bounded-inventory | Geist content delivery is undecided; provenance, licensing, interruption, and integrity remain unreviewed |
| 4.7.3 Splice; 4.7.4 Ableton Cloud; 4.7.5 Push 3 Standalone transfers | vendor/cloud/hardware integrations | out-of-scope-candidate | useful only for workflow/friction evidence; no compatibility target |
| 4.7.6 User Library; 4.7.7 Current Project; 4.7.8 User Folders | user assets and project-local assets | section-inventoried | ownership, collection, missing media, relocation, cache, and backup behavior |
| 4.8 Navigating in the Browser; 4.9 Previewing Files | keyboard/pointer navigation and audition | section-inventoried | focus, transport interaction, preview routing, latency, concurrency, and accessibility |
| 4.10 Adding Content from the Browser to a Live Set | insertion and drag/drop workflow | section-inventoried | target resolution, command/undo semantics, incompatible content, plugin failure, and selection continuity |

### Chapter 5 — Managing Files and Sets

Source: `https://www.ableton.com/en/live-manual/12/managing-files-and-sets/`

| Sections | Geist research domain | State | Extraction gap |
|---|---|---|---|
| 5.1 Sample Files; 5.1.1 Decoding Cache; 5.1.2 Analysis Files | decoding, streaming, cache, waveform/tempo analysis | section-inventoried | format constraints, cache invalidation, concurrency, corruption, disk pressure, deterministic analysis, and disposable-data boundaries |
| 5.1.3 Exporting Audio and Video; 5.1.3.1–5.1.3.5 selection, rendering, encoding, video, and realtime rendering | export and render lifecycle | section-inventoried | signal-path equivalence, interruption, plugin realtime requirements, metadata, dither, SRC, partial-output cleanup, and failure reporting |
| 5.2 MIDI Files; 5.2.1 Exporting MIDI Files | MIDI import/export | section-inventoried | event ordering, tempo/meter representation, unsupported data, round-trip loss, and error behavior |
| 5.3 Live Clips | clip interchange | bounded-inventory | behavioral workflow evidence only; no Live Clip compatibility or file-format target |
| 5.4 Live Sets; 5.4.1 create/open/save; 5.4.2 undo history; 5.4.3 merge; 5.4.4 Session clips as Sets; 5.4.5 templates; 5.4.6 file references | project lifecycle, history, merge, templates, references | section-inventoried | schema/versioning, atomic save, recovery, merge identity/conflicts, unknown-data preservation, and transactional undo |
| 5.5 Live Projects; 5.5.1 Sets; 5.5.2 presets; 5.5.3 project-file management | project bundle and owned content | section-inventoried | ownership, stable IDs, concurrent operations, portability, case sensitivity, and migration |
| 5.6 Locating Missing Files; 5.6.1 manual repair; 5.6.2 automatic repair | missing-media diagnosis and relinking | section-inventoried | matching rules, ambiguity, hashes, user confirmation, batch repair, undo, and non-destructive failure |
| 5.7 Collecting External Files; 5.7.1 collect on export; 5.8 aggregated locate/collect | project collection and archive | section-inventoried | copy atomicity, deduplication, licensing, insufficient space, interruption, and verification |
| 5.9 Finding Unused Files | cleanup and ownership | section-inventoried | reachability definition, shared assets, destructive safety, confirmation, and recovery |
| 5.10 Packing Projects into Packs | distribution bundle | bounded-inventory | Geist archive/content format remains undecided; integrity, provenance, licensing, and compatibility are unreviewed |
| 5.11 File Management FAQs; 5.11.1–5.11.5 project creation, presets, versions, save location, and folder structure | workflow guidance and project organization | section-inventoried | convert only explicit behaviors into atomic observations; recommendations do not become Geist architecture |

## Initial scope notes

- Chapters 31–32 are evidence about an extension ecosystem, not a Geist compatibility target.
- Chapters 34–35 may inform controller workflows, but do not create Push compatibility requirements.
- Chapter 27 remains relevant only if scoring/video enters Geist's accepted scope.
- Device reference chapters may inform capability taxonomy; they MUST NOT be treated as a catalog Geist must clone.
- Chapter 41 is binding evidence only. Workflow frequency requires independent field observations.

## Next extraction slice

Chapters 6, 7, 8, 9, 16, 17, 18, 19, 25, and 41 reached `claims-extracted` on 2026-07-11; atomic observations live in `ableton-live-observations.md` (`OBS-AB12-*`). Next: extract chapters 3–5 (already section-inventoried) plus 10 (Editing MIDI), 21 (Comping), and 36 (synchronization); then spot-verify extracted numeric claims against the rendered pages. No Geist requirement should be created until cross-reference and workflow evidence are reviewed.
