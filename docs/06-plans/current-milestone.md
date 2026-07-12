<!--
Author: Jeff
Date: 2026-07-11
Description: The single active rebuild milestone for the Geist DAW specification-first rebuild
Notes: Exactly one milestone is active; the roadmap document will own ordering when it exists
-->

# Current Milestone — Cross-Product Field Research Foundation

- **Status:** proposed
- **Last verified:** 2026-07-11
- **Scope:** the active milestone only
- **Decision authority:** Jeff
- **Upstream sources:** `../status/STATUS.md`, `../02-reference-research/methodology.md`, rebuild mandate Phase C/6.8
- **Downstream dependents:** `../status/NEXT.md`, future `rebuild-roadmap.md`
- **Supersedes:** milestone claims in `PRODUCTION_PLAN.md` and `INITIAL_PLAN.md` for the rebuild lane
- **Superseded by:** none
- **Open decisions:** saturation threshold per product before the milestone closes
- **Known gaps:** long-timeline/scoring and accessibility-focused workflow evidence has no identified sources yet

## Outcome

The workflow field study reaches balanced cross-product coverage: admitted, fully provenanced workflow observations for Ableton Live and Bitwig Studio alongside the existing four FL Studio observations, plus at least one manual promoted to a section-level coverage matrix, so that priority claims about Geist's core loop become defensible.

## Non-goals

- No requirements ledger, architecture contracts, or implementation work in this milestone.
- No completeness claims for any reference spec.
- No new FL Studio sources.
- No changes to the modular-rack or stacksynth code lanes under this milestone.

## Exit evidence

- `workflow-observations.jsonl` contains admitted observations for at least three products, each passing the cross-reference validation (all source IDs in ledger, all corroborating IDs resolve, CSV rows resolve).
- `source-index.md` and `workflow-corpus.md` counts match the machine-readable artifacts.
- One product dossier carries a section-level coverage matrix mapped to its versioned manual inventory.
- `STATUS.md` field-research section updated with the new counts and the validation date.

## Verification

Run the artifact cross-check (JSONL/ledger/CSV parse plus ID resolution) and diff document counts against artifact counts; record the run in `../status/VALIDATION.md` only if commands beyond document checks are executed.
