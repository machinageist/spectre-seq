<!--
Author: Jeff
Date: 2026-07-11
Description: Versioned source inventory and coverage matrix for the official Bitwig Studio user guide
Notes: Mutable latest URL currently renders a guide labeled v5.3; this dossier does not claim current-product completeness
-->

# Bitwig Studio Reference Dossier

- **Status:** draft
- **Research state:** inventory-only
- **Last verified:** 2026-07-11
- **Scope:** official Bitwig Studio user-guide coverage relevant to DAW, modulation, Grid, controller, and touch workflows
- **Decision authority:** Jeff
- **Upstream sources:** `SRC-BITWIG-GUIDE-LATEST-WELCOME`; `docs/02-reference-research/methodology.md`
- **Downstream dependents:** workflow field study, Geist requirements ledger, modular and UI specifications
- **Supersedes:** removed prototype-era Bitwig specification
- **Superseded by:** none
- **Open decisions:** current guide/product version alignment; depth of Grid and device-catalog extraction
- **Known gaps:** guide version mismatch is unresolved; chapter contents, subsections, failure behavior, persistence, undo, timing, and cross-feature precedence remain unreviewed

## Boundary

This dossier captures publicly documented behavior. It does not approve Bitwig parity, preset/project compatibility, copied layouts, copied bindings, vendor limits, The Grid compatibility, or a Bitwig-derived Geist architecture.

## Version warning

The official URL is mutable and contains `/userguide/latest/`. On 2026-07-11, its rendered welcome chapter and navigation labeled the guide as Bitwig Studio 5.3, while the official download page advertised application version 6.0.11 the same day. The divergence is therefore resolved as a stale guide: `/latest/` documents 5.3, one major version behind the shipping application. This dossier scopes itself to the rendered v5.3 guide. No claim from this source may be described as current-product behavior; re-verification is required if a version 6 guide is published.

## Coverage matrix

Every row is `inventory-only`. The official rendered navigation showed chapters 0–19 plus a separately listed credits page also labeled 19; this numbering is preserved rather than silently corrected.

| Navigation label | Official title | Geist research domain | State |
|---:|---|---|---|
| 0 | Welcome to Bitwig Studio | guide scope, dashboard, settings, conventions | inventory-only |
| 1 | Bitwig Studio Concepts | project/object model | claims-extracted \|
| 2 | Anatomy of the Bitwig Studio Window | UI surfaces, focus, commands | inventory-only |
| 3 | The Arrange View and Tracks | arrangement and tracks | inventory-only |
| 4 | Browsers in Bitwig Studio | browser, assets, search, preview | inventory-only |
| 5 | Arranger Clips | clips and arrangement editing | inventory-only |
| 6 | The Clip Launcher | launcher/session performance | claims-extracted \|
| 7 | The Mix View | mixer, routing, sends | inventory-only |
| 8 | Introduction to Devices | device chains and lifecycle | inventory-only |
| 9 | Automation | automation semantics | claims-extracted \|
| 10 | Working with Audio Events | audio event editing | inventory-only |
| 11 | Working with Note Events | MIDI/note editing and expression | inventory-only |
| 12 | Operators, for Animating Musical Sequences | generative/event transformations | inventory-only |
| 13 | Going Between Notes and Audio | bounce and conversion workflows | inventory-only |
| 14 | Working with Projects and Exporting | persistence, project lifecycle, export | inventory-only |
| 15 | MIDI Controllers | mappings and external control | inventory-only |
| 16 | Modulators, Device Nesting, and More | modulation, nesting, routing | inventory-only |
| 17 | Welcome to The Grid | bounded modular-system reference | claims-extracted \|
| 18 | Working on a Tablet Computer | touch interaction and responsive UI | inventory-only |
| 19 | Device Descriptions | device capability taxonomy | inventory-only |
| 19 (as rendered) | Credits | provenance only | inventory-only |

## Scope controls

- Chapter 17 is a bounded modular-workflow reference. The Grid is not a compatibility target.
- Chapter 19 may inform capability categories but does not define a mandatory Geist device catalog.
- Chapter 18 can inform touch interaction; it does not decide Geist's supported platforms.
- Operators and modulators may motivate candidates only after workflow corroboration and original Geist requirements.
- Drum-family material visible in the welcome chapter does not create implementation requirements or justify copied device designs.

## Next extraction slice

Guide-version mismatch resolved 2026-07-11 (guide 5.3, application 6.0.11). Chapters 1, 6, 9, and 17 reached claims-extracted; atomic observations live in `bitwig-studio-observations.md` (`OBS-BW53-*`). Next: chapters 3 (Arrange View), 5 (Arranger Clips), 7 (Mix View), 11 (Note Events), 14 (Projects and Exporting), and 16 (Modulators); extract launcher/arranger precedence detail and relative-automation mechanics; spot-verify numeric claims.
