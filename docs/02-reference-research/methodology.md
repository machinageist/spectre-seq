<!--
Author: Jeff
Date: 2026-07-11
Description: Clean-room reference research and acceptance methodology for Geist DAW
Notes: Governs research provenance; does not adopt external behavior as a Geist requirement
-->

# Clean-Room Reference Research Methodology

- **Status:** accepted
- **Last verified:** 2026-07-11
- **Scope:** external product research, claim provenance, coverage, review, and promotion into Geist requirements
- **Decision authority:** Jeff
- **Upstream sources:** `docs/README.md`; clean-room and original-design constraints in the rebuild mandate
- **Downstream dependents:** all files under `docs/02-reference-research/`; requirements ledger; traceability ledger
- **Supersedes:** methodology implied by legacy clean-room specs and line/heading/URL metrics
- **Superseded by:** none
- **Open decisions:** long-term source snapshot storage and redistribution policy
- **Known gaps:** complete official source inventories and second-pass reviews remain pending for every substantive reference

## Research boundary

External products are evidence about publicly documented behavior and observed workflows. They are not Geist's architecture, visual design, limits, file formats, compatibility promises, or product identity.

Researchers MUST use publicly authorized sources. They MUST NOT use leaked/private manuals, proprietary project files, decompiled code, private formats, copied screenshots, factory assets, presets, samples, wavetables, or distinctive expression.

## Required claim layers

Every research record uses exactly one layer:

- `OBSERVED`: behavior explicitly supported by a cited public source.
- `SOURCE-GAP`: a relevant fact unavailable or ambiguous in the inspected source.
- `GEIST-CANDIDATE`: an inference or possible product implication; no authority.
- `GEIST-REQ`: an adopted requirement with a stable ID in the requirements ledger.
- `IMPL-DECISION`: an original Geist decision linked to an architecture contract or ADR.

A dossier MUST NOT combine these layers in one untyped bullet. Only the requirements ledger can grant `GEIST-REQ` authority.

## Source hierarchy

1. Versioned official manuals and user guides.
2. Official developer documentation and public standards for interoperability.
3. Official release notes and support articles for version-specific behavior.
4. Official artist/trainer walkthroughs showing end-to-end practice.
5. Experienced independent practitioner walkthroughs and discussions.
6. Secondary discovery material, clearly labeled and never substituted for available primary documentation.

Marketing pages MAY establish advertised scope but MUST NOT establish precise semantics when a manual exists. Search-result pages are not sources.

## Source record

Each source MUST record:

- stable source ID;
- product and product version or `unknown`;
- title, publisher/author, and direct canonical URL or document identity;
- publication/revision date when available;
- access date;
- source and evidence class;
- sections/pages/timestamps inspected;
- mutable-URL warning and captured version evidence;
- access limitation;
- notes on claims supported and not supported.

The machine-readable ledger is `source-ledger.json`. A source being listed does not mean it has been exhaustively inspected.

## Coverage process

For each substantive reference:

1. Fix a declared product/manual version.
2. Inventory the complete top-level table of contents for that source scope.
3. Assign every relevant section one state: `unreviewed`, `reviewed-no-relevant-claims`, `claims-extracted`, `out-of-scope-with-rationale`, or `blocked-source-gap`.
4. Decompose relevant text into atomic observations.
5. For each observation, record preconditions, action, state transition, result, persistence, undo/redo, error behavior, realtime implications, offline implications, and unknown fields when those dimensions apply.
6. Record cross-feature interactions and precedence.
7. Keep inferred Geist implications separate.
8. Link adopted implications to accepted requirement IDs.
9. Add scenario acceptance criteria only in Geist requirement/spec documents.
10. Perform a second-pass contradiction and terminology review.

Unknown behavior MUST remain unknown. Researchers MUST NOT infer numeric limits, defaults, algorithms, timing, schemas, or edge cases from silence.

## Coverage status

A dossier may use:

- `inventory-only`: source set or TOC captured; claims not systematically extracted.
- `in-review`: extraction underway and coverage matrix incomplete.
- `source-complete-for-declared-scope`: every declared source section accounted for.
- `accepted-for-product-planning`: source-complete, claim-traceable, layer-separated, requirement-linked where adopted, gap-explicit, legally bounded, and consistency-reviewed.
- `superseded` or `archived`.

`Complete`, `exhaustive`, or `ready` MUST NOT derive from word count, line count, headings, URLs, TODO absence, or an agent assertion.

## Claim identifiers

Atomic observations use stable IDs scoped by product, such as `OBS-ABLETON-12-0001`. Source gaps use `GAP-ABLETON-12-0001`. IDs MUST NOT be recycled. If retracted, retain the record with status and rationale.

## Review gates

A dossier cannot become `accepted-for-product-planning` until:

- its declared scope and source versions are explicit;
- the source ledger contains direct records;
- its coverage matrix has no unexplained sections;
- every observation cites source ID plus section/page/timestamp;
- observations, gaps, candidates, requirements, and implementation decisions are distinguishable;
- adopted behaviors link to accepted requirement IDs;
- exclusions and original-design constraints are explicit;
- a second reviewer or separate review pass checks contradiction, paraphrase, and terminology;
- mutable sources have version evidence or a declared limitation.

## Workflow research separation

Manual coverage and field-workflow research are related but independent. Manual records establish documented capability. Workflow observations establish how a particular musician used or discussed a system in context. Workflow evidence MUST record source type, role/context when known, ordered actions, shortcuts/gestures, friction, workarounds, confidence, and corroboration. It MUST NOT be presented as population statistics.
