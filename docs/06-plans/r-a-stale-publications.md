<!--
Author: Jeff
Date: 2026-09-07
Description: R-A stale-plan publication safety slice contract
Notes: Authorized by Jeff's instruction to continue the blueprint with blind quality checks; no full-product PASS implied
-->

# R-A — Stale publication safety

- **Status:** accepted (slice scope); implementation and review pending
- **Last verified:** 2026-09-07
- **Scope:** app-thread publications calculated against a running track plan
- **Decision authority:** Jeff
- **Upstream sources:** `product-convergence.md` R-A, `../status/architecture-audit-2026-09-07.md` A-03, current graph/realtime contracts
- **Downstream dependents:** R-B device-state ownership, subsequent routing and flagship/modular slices
- **Supersedes:** none; concretizes the next authorized corrective slice
- **Superseded by:** none
- **Open decisions:** none required for conservative stale-publication refusal; hot swap remains excluded
- **Known gaps:** no fresh operator/audio evidence; preexisting device ownership/export/lifecycle issues are not closed by this slice

## Behavior

A publication derived from current model positions must not be applied against a different
project instance or stale compiled target layout. Reordering, removal, instrument choice and
insert-chain edits must not redirect a gain, parameter or schedule to a previous positional
neighbor. Equality of persisted object IDs or coincident structure-revision values across
project adoption is insufficient authority.

Use a conservative generation/identity guard at the app-thread publication boundary. Reject
before sending any portion of a stale publication set. Keep project edits available for save,
undo and later rebuild; give a visible explanation rather than silently claiming live success.
A fresh plan built for the current project/model accepts subsequent valid publications.

The stale-plan warning and the publication guard must agree. Emergency Stop/all-notes-off and
stream close remain available; rejecting stale edits must not trap sounding notes. Do not make
a warning badge the only enforcement or rely on array bounds to detect semantic misrouting.

## Acceptance matrix

- Parameter edits after track reorder cannot change the old positional neighbor.
- Track gain/solo/mute and master publication cannot target the wrong old layout after removal,
  instrument replacement or insert addition/removal; no partial stale set is published.
- Effect edits with repeated effect kinds remain guarded by the correct compiled layout.
- Clip schedule and loop publications refuse stale model/project bindings rather than updating
  a different voice; ordinary same-layout musical edits remain publishable.
- Replacing/reopening a project cannot authenticate an old engine by colliding persisted IDs or
  restarting a structure-revision counter. Cover the actual model adoption path where feasible.
- A newly built/bound engine for the current model resumes valid publication.
- Tests inspect accepted/rejected traffic or rendered state, not merely an error-shaped string.
- A regression/negative control demonstrates that bypassing the central guard is caught.
- Existing fixture-engine tests retain their declared meaning; no test-only production bypass.
- No render-thread allocation/lock/I/O, persisted schema migration or new DSP path.
- All newly introduced refusal conditions reach an observable production diagnostic.

## Exclusions

No glitch-free graph swap, stop/open/rebuild transport-policy redesign, track-owned parameter
migration, real-project bounce rewrite, complete MIDI routing, new renderer, new instrument,
new modular API, automatic roadmap-wide acceptance or hardware qualification.

## Quality gates

1. Run a reproduction against the unfixed path and record the observed failure.
2. Focused null-backend/product-helper behavior tests prove the acceptance matrix.
3. `cargo fmt --all -- --check`.
4. `cargo clippy --locked --workspace --all-targets -- -D warnings`.
5. `cargo test --locked --workspace` and `git diff --check`.
6. Fresh blind review of this contract and actual diff under `../../gauntlet-output/criteria.md`;
   reviewer verifies cited production paths, failure behavior and test sensitivity without the
   implementation author's reasoning or claimed verdict.
7. Remediate concrete findings, rerun gates and review the changed tree. Commit only after PASS.

“AAA” is the repository's quality discipline, not a claim that a narrow safety patch completes
the workstation or that unrun auditory/GUI gates passed. Record exact verified scope and leave
unrelated predecessor/operator obligations open.
