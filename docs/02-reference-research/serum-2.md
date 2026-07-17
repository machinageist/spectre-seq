<!--
Author: Jeff
Date: 2026-07-11
Description: Official-public-source scope and gap dossier for Serum 2 behavioral research
Notes: No complete public user manual was exposed through the inspected official entry points
-->

# Serum 2 Reference Dossier

- **Status:** draft
- **Research state:** blocked-source-gap
- **Last verified:** 2026-07-11
- **Scope:** publicly accessible official Serum 2 behavior relevant to hybrid synthesis and sound-design workflows
- **Decision authority:** Jeff
- **Upstream sources:** `SRC-SERUM2-PRODUCT-PAGE`; `SRC-SERUM2-WHATS-NEW-PDF`; `SRC-SERUM2-SUPPORT-CATEGORY`; `docs/02-reference-research/methodology.md`
- **Downstream dependents:** sound-design workflow study, flagship-synth requirements, native-device specifications
- **Supersedes:** removed prototype-era Serum specification
- **Superseded by:** none
- **Open decisions:** whether a legitimately available installed/customer guide can be reviewed without redistribution
- **Known gaps:** no complete public official guide, build number, dated source scope, full command reference, complete parameter semantics, persistence schema, or exhaustive workflow documentation was exposed

## Completion finding

Serum 2 cannot currently be marked source-complete or accepted for product planning.

The inspected official public entry points exposed:

1. an unversioned product page naming Serum version 2;
2. a public “What's New In Serum 2” PDF endpoint;
3. a mutable official support category with six displayed articles.

They did not expose a complete public Serum 2 user manual. This finding is limited to inspected public official entry points; it does not assert that no authenticated customer download or installed guide exists.

## Source matrix

| Source ID | Source role | What it can support | What it cannot support | State |
|---|---|---|---|---|
| `SRC-SERUM2-PRODUCT-PAGE` | official product page | advertised scope, public specs, system requirements, official outbound links | precise behavior, edge cases, exhaustive command/parameter semantics | inventory-only |
| `SRC-SERUM2-WHATS-NEW-PDF` | official change guide | differences and additions explicitly documented in the PDF once text is inspectable | baseline Serum behavior or exhaustive Serum 2 coverage | inventory-only |
| `SRC-SERUM2-SUPPORT-CATEGORY` | official support knowledge base | bounded operational/support behavior in reviewed articles | complete synthesis, modulation, editing, preset, and shortcut semantics | inventory-only |

## Public support-category inventory

The official category displayed six articles during inspection:

- Machine Authorization Problems
- Serum 2 Upgrade FAQ
- Serum 2 Guidelines for Optimizing CPU Usage
- Serum 2 Preset Previews
- Converting Samples to Wavetables
- Automatic Sample Root Note Mapping

These article titles define a bounded source inventory, not behavioral claims. Individual article contents remain unextracted.

## Legacy-spec disposition rules

The legacy Serum 2 specification relies primarily on the change guide, product material, and support pages while recording extensive gaps. During migration:

- claims traceable to inspected official sections MAY become `OBS-SERUM2-*` records;
- claims based on Serum 1, inference, convention, or missing guide content remain separately typed and MUST NOT be presented as Serum 2 observations;
- product-page language remains advertised capability unless corroborated by stronger documentation;
- undocumented parameter ranges, defaults, algorithms, timing, voice behavior, randomization, preset payloads, file formats, and edge cases remain `SOURCE-GAP`;
- no Serum preset, wavetable, sample, visual, binary, or file compatibility requirement is implied;
- no Serum numeric limit becomes a Geist limit without an independent product rationale.

## Source-gap records

- `GAP-SERUM2-0001`: complete official public Serum 2 user guide not exposed through inspected product/support entry points.
- `GAP-SERUM2-0002`: exact build/version and dated revision scope absent from inspected sources.
- `GAP-SERUM2-0003`: available browser PDF surface did not expose searchable text or metadata for the change guide.
- `GAP-SERUM2-0004`: authenticated customer and installed-documentation availability not assessed.

## Work that can continue safely

The field study MAY use complete, source-attributed patch-design sessions as contextual workflow evidence. Such sessions do not close manual gaps and do not establish undocumented product behavior. Public support articles may be extracted one by one. The change-guide PDF may be processed with an authorized local PDF tool if downloaded from the official endpoint.

## Acceptance blocker

This dossier remains `blocked-source-gap` until either:

- a complete legitimate official guide is available and inventoried; or
- the declared dossier scope is narrowed to the official public sources above, every source is fully mapped, and the resulting limitations are accepted explicitly for product-planning use.
