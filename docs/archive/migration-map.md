<!--
Author: Jeff
Date: 2026-07-11
Description: Non-destructive migration map from legacy Geist documents to the new authority hierarchy
Notes: Records destinations before any move; pre-task modified documents remain untouched
-->

# Documentation Migration Map

- **Status:** accepted
- **Last verified:** 2026-07-11
- **Scope:** every legacy document reviewed during the documentation inventory and its single migration disposition
- **Decision authority:** Jeff
- **Upstream sources:** `docs/README.md`; `docs/audits/documentation-inventory.md`; `docs/audits/reuse-disposition.md`
- **Downstream dependents:** future archive moves, supersession pointers, link repair, root orientation updates
- **Supersedes:** undocumented assumptions about which legacy file remains authoritative
- **Superseded by:** none
- **Open decisions:** exact archive execution slice; whether root legacy files become short pointers or move completely
- **Known gaps:** inbound links have not yet been rewritten because no legacy path moves in this slice

## Disposition vocabulary

- **retain-current:** stays at its current path as active evidence/status.
- **extract-then-archive:** useful content moves into named authority documents, then the original moves unchanged to archive.
- **pointer-after-extraction:** root or high-discovery path becomes a concise pointer after its useful content is extracted and original evidence is archived.
- **archive-direct:** no authoritative content must be extracted before archival.
- **rewrite-in-place:** path remains useful but content is replaced only after equivalent authority exists and history preserves the old version.
- **absent-record-only:** no file exists; do not invent one.

## Root and agent documents

| Current path | Disposition | Authoritative extraction target | Eventual archive/pointer | Guard |
|---|---|---|---|---|
| `CLAUDE.md` | rewrite-in-place | `docs/README.md`, current milestone, local skills index | concise repository instruction | replace stale plan pointers only after hierarchy is populated |
| `CLAUDE(1).md` | absent-record-only | none | none | absent in tree and inspected history |
| `INITIAL_PLAN.md` | pointer-after-extraction | `docs/06-plans/rebuild-roadmap.md`, requirements traceability | `docs/archive/plans/INITIAL_PLAN.md` | preserve completion claims as evidence |
| `PROPOSED_FILE_TREE.md` | extract-then-archive | `docs/03-architecture/crate-and-dependency-boundaries.md` | `docs/archive/plans/PROPOSED_FILE_TREE.md` | proposed paths receive no authority automatically |
| `PRODUCTION_PLAN.md` | pointer-after-extraction | `docs/06-plans/rebuild-roadmap.md`, quality gates | `docs/archive/plans/PRODUCTION_PLAN.md` | preserve admitted runtime gaps |
| `HANDOFF.md` | pointer-after-extraction | `docs/status/STATUS.md`, `docs/status/NEXT.md`, `docs/status/VALIDATION.md` | `docs/archive/handoffs/HANDOFF.md` | pre-task modified; do not touch until changes are preserved |
| `HANDOFF(1).md` | absent-record-only | none | none | absent in tree and inspected history |
| `sol-handoff.md` | archive-direct | continuation content extracted into `docs/status/STATUS.md`, `docs/status/NEXT.md`, `docs/06-plans/current-milestone.md` on 2026-07-11 | `docs/archive/handoffs/sol-handoff.md` (moved 2026-07-11) | ad hoc session handoff; original claims preserved unchanged |
| `AGENTS/changes/modular-rack/PLAN.md` | extract-then-archive | modular-routing spec and rebuild backlog | `docs/archive/agent-plans/modular-rack-PLAN.md` | pre-task modified; preserve user work |
| `AGENTS/changes/modular-rack/SPEC.md` | extract-then-archive | modular reference dossier and original Geist modular spec | `docs/archive/agent-plans/modular-rack-SPEC.md` | separate observations from requirements |
| `AGENTS/changes/sound-design-depth/PLAN.md` | extract-then-archive | flagship-synth backlog | `docs/archive/agent-plans/sound-design-depth-PLAN.md` | no implementation authority |
| `AGENTS/changes/sound-design-depth/SPEC.md` | extract-then-archive | Phase Plant/Serum research and flagship-synth spec | `docs/archive/agent-plans/sound-design-depth-SPEC.md` | vendor limits remain unapproved |

## General architecture and UI documents

| Current path | Disposition | Authoritative extraction target | Eventual archive | Guard |
|---|---|---|---|---|
| `docs/gpt_mega_prompt.md` | archive-direct | none; founding intent already represented by current mandate | `docs/archive/prompts/gpt_mega_prompt.md` | not a current protocol |
| `docs/architecture.md` | extract-then-archive | `docs/03-architecture/system-context.md`, crate boundaries, runtime/threading | `docs/archive/architecture/architecture.md` | declared architecture is not live reachability |
| `docs/realtime_rules.md` | extract-then-archive | `docs/03-architecture/realtime-contract.md`, `docs/05-quality/realtime-verification.md` | `docs/archive/architecture/realtime_rules.md` | do not retain unsupported implemented language |
| `docs/vst_hosting.md` | extract-then-archive | `docs/03-architecture/vst3-hosting.md`, `docs/04-specs/vst3-user-experience.md` | `docs/archive/architecture/vst_hosting.md` | requires current official-source review |
| `docs/clap_hosting.md` | archive-direct | none | `docs/archive/excluded-hosting/clap_hosting.md` | CLAP excluded from active architecture |
| `docs/plugin_sdk.md` | extract-then-archive | native-device spec and contributor documentation | `docs/archive/architecture/plugin_sdk.md` | native devices are not plugin products |
| `docs/architecture/native-vst-internal-devices.md` | extract-then-archive | product principles, device model, new ADR | `docs/archive/architecture/native-vst-internal-devices.md` | preserve boundary principle, reject readiness claims |
| `docs/ui_ux_principles.md` | extract-then-archive | product principles, UI and accessibility specs | `docs/archive/ui/ui_ux_principles.md` | revalidate against workflow research |
| `docs/ui_interaction_model.md` | extract-then-archive | `docs/04-specs/ui-interaction.md`, UI-state architecture | `docs/archive/ui/ui_interaction_model.md` | needs explicit state machines |
| `docs/ui_configuration_model.md` | extract-then-archive | configuration architecture and UI spec | `docs/archive/ui/ui_configuration_model.md` | requires typed command/security boundaries |
| `docs/modular_rack_spec.md` | extract-then-archive | VCV dossier and `docs/04-specs/modular-routing.md` | `docs/archive/specs/modular_rack_spec.md` | split observation, requirement, implementation |

## Clean-room and synth documents

| Current path | Disposition | Authoritative extraction target | Eventual archive | Guard |
|---|---|---|---|---|
| `docs/specs/ableton-live-clean-room-spec.md` | extract-then-archive | `docs/02-reference-research/ableton-live.md` | `docs/archive/reference-specs/ableton-live-clean-room-spec.md` | draft until claim-level provenance passes |
| `docs/specs/bitwig-studio-clean-room-spec.md` | extract-then-archive | `docs/02-reference-research/bitwig-studio.md` | `docs/archive/reference-specs/bitwig-studio-clean-room-spec.md` | separate Grid from Geist architecture |
| `docs/specs/serum-2-clean-room-spec.md` | extract-then-archive | `docs/02-reference-research/serum-2.md` | `docs/archive/reference-specs/serum-2-clean-room-spec.md` | complete guide gap remains explicit |
| `docs/specs/geist-modular-synth-spec.md` | extract-then-archive | Phase Plant dossier and `docs/04-specs/flagship-synth.md` | `docs/archive/specs/geist-modular-synth-spec.md` | no parity or preset compatibility implication |
| `docs/specs/geist-modular-synth-plan.md` | archive-direct | future backlog derives from accepted requirements instead | `docs/archive/plans/geist-modular-synth-plan.md` | premature implementation sequencing |
| `docs/specs/clean-room-spec-completeness-audit.md` | archive-direct | new methodology and source ledger replace it | `docs/archive/audits/clean-room-spec-completeness-audit.md` | metrics are not correctness evidence |
| `docs/specs/clean-room-spec-audit-metrics.json` | archive-direct | none | `docs/archive/audits/clean-room-spec-audit-metrics.json` | preserve unchanged as historical metric |

## ADRs

| Current path | Disposition | Replacement | Eventual archive | Guard |
|---|---|---|---|---|
| `docs/adr/001-clap-over-vst.md` | extract-then-archive | new sourced VST3-only ADR with current license/dependency evidence | `docs/archive/adr/001-clap-over-vst.md` | direction retained; time-sensitive claims unverified |
| `docs/adr/002-arcswap-graph-swap.md` | archive-direct | future graph-state publication ADR | `docs/archive/adr/002-arcswap-graph-swap.md` | ArcSwap is not accepted by filename |
| `docs/adr/003-cbor-project-format.md` | archive-direct | future project-format ADR | `docs/archive/adr/003-cbor-project-format.md` | CBOR is not accepted by filename |
| `docs/adr/004-egui-first-wgpu-later.md` | archive-direct | future UI/rendering/accessibility ADR | `docs/archive/adr/004-egui-first-wgpu-later.md` | egui/wgpu trajectory is undecided |

## New mandate-era documents

| Path | Disposition | Authority |
|---|---|---|
| `docs/README.md` | retain-current | accepted documentation authority map |
| `docs/archive/migration-map.md` | retain-current | accepted migration control |
| `docs/audits/*.md` | retain-current | current attributable audit evidence |
| `docs/status/VALIDATION.md` | retain-current | current command/result evidence |
| `docs/status/subsystems.toml` | retain-current | machine-readable maturity evidence |
| `docs/02-reference-research/external-reference-register.md` | retain-current | draft research classification control |
| `docs/status/STATUS.md` | retain-current | current verified rebuild state (created 2026-07-11) |
| `docs/status/NEXT.md` | retain-current | next slices and blockers (created 2026-07-11) |
| `docs/06-plans/current-milestone.md` | retain-current | single active milestone (created 2026-07-11) |
| `README.md` (root) | retain-current | root orientation; defers to `docs/README.md` for authority (created 2026-07-11) |

## Migration sequence

1. Populate accepted product constraints and readiness vocabulary.
2. Establish requirements ledger, traceability schema, and decision gates.
3. Establish research methodology and source ledger; rebuild substantive dossiers.
4. Establish architecture and quality contracts.
5. Establish detailed specs and rebuild roadmap.
6. Create `docs/status/STATUS.md`, `NEXT.md`, and the current milestone from verified evidence.
7. Check every extraction target for links back to source evidence.
8. Preserve pre-task modified files and attribute their changes.
9. Move or replace legacy paths in coherent groups; repair inbound links in the same change.
10. Verify no active document points to archived material as authority.

No move, deletion, or legacy-file rewrite is authorized by this map alone. Each migration slice requires a fresh status check and link validation.
