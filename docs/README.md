<!--
Author: Jeff
Date: 2026-07-11
Description: Authoritative documentation map for the Geist DAW specification-first rebuild
Notes: This index governs document authority; historical files remain evidence until migrated
-->

# Geist Documentation Authority

- **Status:** accepted
- **Last verified:** 2026-07-11
- **Scope:** documentation ownership, authority, status vocabulary, and migration rules
- **Decision authority:** Jeff
- **Upstream sources:** rebuild mandate; `docs/audits/documentation-inventory.md`; `docs/audits/reuse-disposition.md`
- **Downstream dependents:** every active product, requirements, research, architecture, specification, quality, plan, ADR, audit, and status document
- **Supersedes:** implicit authority of root plans, handoffs, legacy architecture summaries, and numbered ADR filenames
- **Superseded by:** none
- **Open decisions:** final archive moves; root README and contributor/security/license posture
- **Known gaps:** most target documents do not exist yet; legacy documents remain in place until authoritative replacements exist

## Authority order

When documents conflict, use this order:

1. Jeff's current explicit decisions and accepted product constraints.
2. Accepted ADRs for their bounded decisions.
3. Accepted requirements in `docs/01-requirements/requirements-ledger.md`.
4. Accepted architecture contracts in `docs/03-architecture/`.
5. Accepted detailed behavior specifications in `docs/04-specs/`.
6. Accepted verification and release gates in `docs/05-quality/`.
7. Current milestone and backlog documents in `docs/06-plans/`.
8. Current audits and status evidence in `docs/audits/` and `docs/status/`.
9. Draft reference research in `docs/02-reference-research/`.
10. Historical plans, handoffs, prompts, legacy specs, and archived documents.

A lower-ranked document MUST NOT override a higher-ranked one. Source code and passing tests prove only the behavior they exercise; they do not create product authority by themselves.

## Canonical document classes

| Class | Canonical location | Owns | Must not own |
|---|---|---|---|
| Product | `docs/00-product/` | audience, identity, principles, scope, terminology, platform intent, readiness meaning | implementation details |
| Requirements | `docs/01-requirements/` | stable normative requirement IDs, provenance, acceptance evidence, decision gates, traceability | vendor observations without adoption |
| Reference research | `docs/02-reference-research/` | source-traceable external observations and workflow evidence | Geist requirements or implementation decisions |
| Architecture | `docs/03-architecture/` | ownership, types, lifecycle, invariants, threading, failure behavior, dependency rules | milestone status or vendor behavior claims |
| Detailed specs | `docs/04-specs/` | user-visible and subsystem behavior linked to requirements | source research or implementation progress |
| Quality | `docs/05-quality/` | verification strategy, budgets, matrices, release gates | optimistic readiness claims |
| Execution plans | `docs/06-plans/` | dependency-ordered work, active milestone, backlog | durable architecture decisions |
| ADRs | `docs/adr/` | one durable decision per accepted record | broad plans or pseudocode checklists |
| Audits | `docs/audits/` | attributable findings about repository evidence | product authority |
| Status | `docs/status/` | latest verified maturity, commands, next slices, blockers | historical narrative |
| Archive | `docs/archive/` | preserved superseded evidence | active instructions or authority |

## Required hierarchy

The target hierarchy is defined now; files are created only when they contain grounded content. Empty placeholder specifications are prohibited.

- `docs/00-product/`: vision, principles, scope/non-goals, terminology, platform support, professional-readiness bar.
- `docs/01-requirements/`: requirements ledger, traceability, decision gates, user workflows, functional and nonfunctional requirements.
- `docs/02-reference-research/`: methodology, source ledger, substantive product dossiers, and workflow field study.
- `docs/03-architecture/`: system, dependency, runtime, realtime, time/event, graph, device, project, VST3, UI, diagnostics, security, and offline-render contracts.
- `docs/04-specs/`: application and musician-facing subsystem behavior.
- `docs/05-quality/`: verification, realtime/DSP evidence, compatibility, performance, and release gates.
- `docs/06-plans/`: rebuild roadmap, current milestone, and backlog.
- `docs/adr/`, `docs/audits/`, `docs/status/`, `docs/archive/`.

## Document metadata contract

Every authoritative Markdown document MUST begin with Jeff's standard HTML header and then declare:

- status;
- last verified date;
- scope;
- owner or decision authority;
- upstream sources or requirements;
- downstream dependents;
- supersedes and superseded-by links;
- open decisions;
- known gaps.

Allowed status values are `draft`, `proposed`, `accepted`, `implemented`, `verified`, `superseded`, and `archived`.

- `accepted` means the bounded decision or contract has authority.
- `implemented` additionally requires linked production code through the intended path.
- `verified` additionally requires linked passing acceptance evidence in a recorded environment.
- A document MUST NOT infer `implemented` or `verified` from its filename, age, compilation, or isolated unit tests.

## Normative and provenance rules

1. Normative requirements MUST use stable IDs and `MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`, or `MAY` deliberately.
2. External observations MUST remain tagged and source-located in reference research.
3. Geist requirements MUST live in the requirements ledger, even when motivated by external behavior.
4. Implementation decisions MUST live in architecture contracts or ADRs and link back to requirements.
5. Plans MUST link to requirement IDs and architecture prerequisites.
6. Status claims MUST link to commands, tests, fixtures, runtime evidence, or named manual QA.
7. Requirement IDs MUST NOT be recycled after removal; retired IDs remain traceable.

## Migration law

1. New authority MUST exist before old authority is archived or replaced by a pointer.
2. Pre-task modified files MUST remain untouched until their changes are attributed and preserved.
3. Historical files MUST retain their original claims as evidence; archive metadata or a pointer may describe supersession without laundering the old content.
4. Each migrated legacy path MUST have exactly one disposition in `docs/archive/migration-map.md`.
5. Moves MUST repair inbound links in the same slice.
6. A stale ADR number grants no authority. ADRs 002–004 remain historical until independently re-decided.
7. No legacy file is moved by this document-architecture slice.

## Current authority state

Currently authoritative evidence:

- `docs/audits/*.md`
- `docs/status/VALIDATION.md`
- `docs/status/subsystems.toml`
- this document
- `docs/archive/migration-map.md`

Current research controls:

- `docs/02-reference-research/external-reference-register.md` is draft and controls classification, not product behavior.

No product requirement, architecture contract, detailed spec, or rebuild milestone is yet accepted through the new hierarchy. The legacy implementation remains active code only because migration has not started; it is not the target architecture.
