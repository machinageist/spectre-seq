<!--
Author: Jeff
Date: 2026-07-12
Description: Documentation authority and repository map for Geist DAW
Notes: Defines precedence, ownership, and status vocabulary
-->

# Documentation Authority

- **Status:** accepted
- **Last verified:** 2026-07-12
- **Scope:** all active Geist documentation
- **Decision authority:** Jeff
- **Upstream sources:** product direction and repository implementation
- **Downstream dependents:** every active document and implementation slice
- **Supersedes:** all removed prototype-era plans, handoffs, audits, and specifications
- **Superseded by:** none
- **Open decisions:** none at the authority layer
- **Known gaps:** architecture and quality contracts are created at their roadmap intakes

## Conflict precedence

When active documents disagree, use this order:

1. Accepted product vision in `00-product/`.
2. Accepted requirements and decisions in `01-requirements/`.
3. Accepted architecture contracts in `03-architecture/` when present.
4. Accepted detailed specifications in `04-specs/` when present.
5. Accepted quality contracts in `05-quality/` when present.
6. Roadmap and active milestone in `06-plans/`.
7. Current evidence in `status/`.
8. Draft reference research in `02-reference-research/`.

Implementation proves behavior but does not silently redefine accepted requirements.
Research informs decisions but has no product authority by itself.

## Active document classes

| Class | Location | Owns |
|---|---|---|
| Product | `00-product/` | audience, identity, release bars, non-goals |
| Requirements | `01-requirements/` | normative behavior, decision gates, traceability |
| Research | `02-reference-research/` | external evidence, limitations, observations |
| Architecture | `03-architecture/` | durable component and realtime contracts |
| Specifications | `04-specs/` | build-ready subsystem behavior |
| Quality | `05-quality/` | deterministic validation and release gates |
| Plans | `06-plans/` | dependency order and active execution scope |
| Status | `status/` | latest verified state and immediate next work |

Current architecture entrypoint: `03-architecture/dsp-device-io.md`. Directories for specifications and quality are created only when they contain grounded contracts.

## Status vocabulary

Allowed values are `draft`, `proposed`, `accepted`, `implemented`, `verified`, `superseded`, and `archived`.

- `draft`: incomplete working material.
- `proposed`: complete enough for a decision but not accepted.
- `accepted`: authoritative direction or contract.
- `implemented`: code exists but its full evidence gate may remain open.
- `verified`: stated acceptance evidence passes.
- `superseded`: replaced by named active authority.
- `archived`: retained only as history outside the active tree.

## Required metadata

Every authoritative Markdown document starts with Jeff's header and records status, verification date, scope, decision authority, upstream sources, downstream dependents, supersession, open decisions, and known gaps.

## Working rule

Read `status/STATUS.md`, `status/NEXT.md`, and `06-plans/current-milestone.md` before implementation. Work in one small slice, update traceability and status when claims change, and run targeted validation before completion.