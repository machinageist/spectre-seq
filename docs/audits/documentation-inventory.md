<!--
Author: Jeff
Date: 2026-07-11
Description: Forensic inventory and authority classification of Geist DAW documentation
Notes: Presence and history were checked against the working tree and all Git history
-->

# Documentation Inventory

- **Status:** draft
- **Last verified:** 2026-07-11
- **Scope:** current Markdown/JSON project documents, mandated legacy evidence set, authority and migration candidates
- **Decision authority:** Jeff
- **Upstream sources:** working-tree file inventory, `git log --all --name-only`, legacy-document audit, Phase A audits
- **Downstream dependents:** documentation authority map, archive migration map, reference register
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** final archive movement and root-pointer strategy
- **Known gaps:** non-Markdown source headers and all historical revisions have not been individually catalogued

## Inventory result

The working tree contains 37 Markdown files, including the six new Phase A audit documents and `docs/status/VALIDATION.md`. Every document explicitly required by the rebuild mandate exists except:

- `CLAUDE(1).md` — absent from the working tree and all Git history;
- `HANDOFF(1).md` — absent from the working tree and all Git history.

No evidence supports treating either duplicate name as a lost authoritative version. If copies exist outside the repository, they were not supplied to this environment and cannot be merged or assigned authority.

The required JSON artifact `docs/specs/clean-room-spec-audit-metrics.json` exists. It is historical corpus metadata, not correctness evidence.

## Authority classes

- **forensic evidence:** preserve unchanged until archived; useful for reconstructing intent or status drift, not authoritative now;
- **candidate source material:** useful content must be revalidated and migrated into the new hierarchy;
- **current audit:** attributable Phase A evidence created under this mandate;
- **historical metric:** machine-generated counts without correctness authority;
- **repository instruction:** currently applies to agent behavior, but its stale project-plan pointers must be replaced only after new authority files exist.

## Mandated evidence set

| Document | Presence | Current class | Principal finding | Planned destination |
|---|---|---|---|---|
| `CLAUDE.md` | present | repository instruction + forensic evidence | valid local workflow guardrails; stale authority over old phase order/tree | concise root instruction pointing to new hierarchy |
| `CLAUDE(1).md` | absent now/history | absent | no evidence it existed in repository | record absence only |
| `INITIAL_PLAN.md` | present | forensic evidence | phase completion claims conflate isolated and integrated work | `docs/archive/plans/` or supersession pointer |
| `PROPOSED_FILE_TREE.md` | present | forensic evidence | mixes present, proposed, and missing paths | `docs/archive/plans/` |
| `PRODUCTION_PLAN.md` | present | forensic evidence | admits core disconnections while using readiness framing | `docs/archive/plans/` |
| `HANDOFF.md` | present, pre-task modified | forensic evidence | append-only contradictory clean/test/commit status | archive after preserving user changes |
| `HANDOFF(1).md` | absent now/history | absent | no evidence it existed in repository | record absence only |
| `docs/gpt_mega_prompt.md` | present | forensic evidence | broad founding intent; not a clean-room protocol or current plan | `docs/archive/prompts/` |
| `docs/architecture.md` | present | candidate source material | labels intended crate architecture as current runtime | split into `docs/03-architecture/` contracts |
| `docs/realtime_rules.md` | present | candidate source material | useful callback law; enforcement coverage overstated | `docs/03-architecture/realtime-contract.md` + quality plan |
| `docs/vst_hosting.md` | present | candidate source material | responsibility list lacks maturity/evidence | `docs/03-architecture/vst3-hosting.md` after official-source review |
| `docs/clap_hosting.md` | present | forensic evidence | stale architecture excluded by current VST3-only constraint | archive |
| `docs/plugin_sdk.md` | present | candidate source material | historical title and stale device-abstraction status | native-device contributor docs after device contract |
| `docs/architecture/native-vst-internal-devices.md` | present | forensic evidence + candidate principle | useful VST/native boundary pivot; unsupported vertical-slice language | product constraint + rewritten ADR |
| `docs/ui_ux_principles.md` | present | candidate source material | high-level product intent mixed with external references | product principles + UI requirements |
| `docs/ui_interaction_model.md` | present | candidate source material | interaction ideas need workflow evidence and state machines | `docs/04-specs/ui-interaction.md` |
| `docs/ui_configuration_model.md` | present | candidate source material | declarative configuration concept needs command/security contracts | architecture/configuration + UI spec |
| `docs/modular_rack_spec.md` | present | candidate reference dossier | mixes VCV observations, inferred architecture, and Geist commitments | `docs/02-reference-research/vcv-rack.md` plus original Geist spec |
| Ableton clean-room spec | present | candidate reference dossier | flat URL provenance; observed behavior and mapping not claim-tagged | `docs/02-reference-research/ableton-live.md` |
| Bitwig clean-room spec | present | candidate reference dossier | strongest separation, still page-level provenance | `docs/02-reference-research/bitwig-studio.md` |
| Serum 2 clean-room spec | present | candidate reference dossier | relies on public “What’s New” and support material, not full guide | `docs/02-reference-research/serum-2.md` |
| Geist modular synth spec | present | candidate reference dossier | Phase Plant behavior separated better, but vendor limits became commitments | Phase Plant research + original flagship synth spec |
| Geist modular synth plan | present | forensic evidence | implementation plan precedes accepted product/device contracts | archive after extraction |
| clean-room completeness audit | present | forensic evidence | declares readiness without claim-level traceability | archive |
| clean-room metrics JSON | present | historical metric | counts lines/headings/URLs/markers only | archive unchanged |
| ADR 001 | present | candidate decision source | correct VST3-only direction; current licensing and validation unsupported | rewrite as current sourced ADR |
| ADRs 002–004 | present | forensic evidence | pseudocode checklists, not decisions | archive; re-decide subjects independently |

## Other current Markdown

| Path | Class | Treatment |
|---|---|---|
| `AGENTS/changes/modular-rack/PLAN.md` | forensic work plan; pre-task modified | preserve user change; archive after migration decision |
| `AGENTS/changes/modular-rack/SPEC.md` | candidate source material | extract original Geist requirements only after reference separation |
| `AGENTS/changes/sound-design-depth/PLAN.md` | forensic work plan | archive after extraction |
| `AGENTS/changes/sound-design-depth/SPEC.md` | candidate source material | reassess against Phase Plant/Serum source and product decisions |
| `docs/audits/*.md` | current audit | remain authoritative evidence, initially draft |
| `docs/status/VALIDATION.md` | current audit/status | remain live; update only from actual command results |

## Contradiction index

1. `HANDOFF.md` says the worktree is clean while it is itself among five pre-existing modified files.
2. `INITIAL_PLAN.md` says graph implementation is complete; `PRODUCTION_PLAN.md` says the compiled graph is unused by the live path.
3. `docs/architecture.md` calls itself current architecture while describing disconnected crates as runtime layers.
4. ADR 002 is a pending pseudocode scaffold while plans cite an “ADR 002 pattern.”
5. ADR 003 is a pending pseudocode scaffold while ADR 001 and production plans treat it as accepted opaque-state/project-format authority.
6. The old mega prompt specifies JSON bundle language while current code serializes CBOR; no accepted migration/format decision resolves the conflict.
7. ADR 004 is a pending pseudocode scaffold while egui has already become implementation fact.
8. `CLAUDE.md` requires old phase order while `PRODUCTION_PLAN.md` claims successor sequencing.
9. `docs/realtime_rules.md` calls the contract implemented while broad enforcement and even callback-side deallocation remain unresolved.
10. Clean-room completion language conflicts with extensive declared source gaps and missing claim-level locators.

## Migration constraints

1. Historical documents MUST NOT be rewritten into apparently current evidence.
2. Pre-task modified documents MUST NOT be moved or replaced until their user changes are safely preserved and attributed.
3. Each old path MUST map to exactly one archive destination, supersession pointer, or authoritative extraction target.
4. Broken links MUST be repaired as part of the same migration slice.
5. New authority documents MUST exist before old North Stars are archived.
6. Reference dossiers MUST remain draft until the clean-room acceptance gate passes.
7. Audit documents MAY be amended when new evidence appears, but changes must retain conservative status language.

## Inventory completion statement

The mandated document presence/history inventory is complete for the current tree and Git history. Content review is complete enough to classify authority and migration direction, but reference factual completeness remains a separate Phase C research task.
