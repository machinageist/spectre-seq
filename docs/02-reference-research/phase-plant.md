<!--
Author: Jeff
Date: 2026-07-11
Description: Official-source scope and coverage controls for Kilohearts Phase Plant research
Notes: The official documentation page exposes no product/manual version or revision date
-->

# Phase Plant Reference Dossier

- **Status:** draft
- **Research state:** inventory-only
- **Last verified:** 2026-07-11
- **Scope:** official Phase Plant documentation relevant to modular hybrid-synthesis and patch-design workflows
- **Decision authority:** Jeff
- **Upstream sources:** `SRC-PHASE-PLANT-LIVE-DOCS`; `docs/02-reference-research/methodology.md`
- **Downstream dependents:** sound-design workflow study, flagship-synth requirements, native-device specifications
- **Supersedes:** removed prototype-era Phase Plant material
- **Superseded by:** none
- **Open decisions:** version scope and whether separate Snapins documentation is needed as a bounded reference
- **Known gaps:** official page exposes no product/manual version or revision date; section-level claims, persistence, errors, commands, modulation rates, voice behavior, and cross-feature precedence remain unreviewed

## Boundary

Phase Plant is a substantive sound-design reference, not a compatibility target. Geist does not seek preset, binary, Snapin, project, layout, or terminology compatibility. “Behavioral parity” and “compatibility mode” language in legacy Geist material has no authority.

## Source limitation

The complete rendered official Phase Plant documentation page was inspected through its final “Unison Settings” material. The page title, heading, navigation, article text, footer, metadata, time elements, and image paths exposed no product/manual version or revision date.

Complete inspection of one mutable page is not equivalent to version-complete product coverage. The source remains `inventory-only` until its section structure is captured deterministically and its relationship to the current product release is established or explicitly bounded as unknown.

## Extraction dimensions

Future `OBS-PHASE-PLANT-*` records must keep these dimensions separate when documented:

- generator/source lifecycle and routing;
- modulation source creation, assignment, rate, polarity, scaling, and visualization;
- voice allocation, unison, note handling, and random state;
- per-voice versus global evaluation;
- effects lanes, ordering, routing, and Snapin boundaries;
- wavetable, sample, granular, curve, and editor workflows;
- macro/remote controls and preset state;
- initialization, duplication, randomization, comparison, and save workflow;
- CPU/quality modes and realtime implications;
- errors, unavailable content, migration, and missing-device behavior;
- commands, pointer gestures, focus, and keyboard behavior.

A field absent from official documentation remains unknown. Standard synthesizer convention is not evidence about Phase Plant.

## Legacy-spec correction controls

During extraction from `docs/specs/geist-modular-synth-spec.md` and related plans:

1. External claims require an exact official source section.
2. Geist limits and architecture move only to the requirements/architecture hierarchy after independent review.
3. Terms implying parity or compatibility are removed unless they describe an explicitly accepted target.
4. Device counts, modulation counts, lane counts, polyphony, oversampling, and rate limits are not inherited.
5. Algorithms, preset payloads, distinctive UI composition, factory content, and visual assets are excluded.
6. Product implications remain `GEIST-CANDIDATE` until adopted with stable requirement IDs.

## Source-gap records

- `GAP-PHASE-PLANT-0001`: product/manual version absent from inspected official documentation.
- `GAP-PHASE-PLANT-0002`: revision/publication date absent.
- `GAP-PHASE-PLANT-0003`: current-release alignment cannot be proved from the page alone.
- `GAP-PHASE-PLANT-0004`: broader Kilohearts/Snapins documentation scope has not been inventoried.

## Next extraction slice

Capture the official page's heading hierarchy and direct anchors as a deterministic matrix. Extract atomic observations only for source/generator structure, modulation assignment, effects/routing, and unison where explicit. In parallel, verify current product-version evidence from official release material without treating marketing claims as detailed behavior.
