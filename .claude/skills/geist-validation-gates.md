---
name: geist-validation-gates
description: "Load before reviewing or declaring any Geist DAW slice complete. Defines preflight, implementation, review, and completion gates for faithful execution of the plan."
---

<!--
Author: Jeff
Date: 2026-05-27
Description: Validation gates for Geist DAW slices
Notes: Use for reviews, task handoffs, CI expectations, and stop/go decisions
-->

# Geist Validation Gates

## Gate model

Every slice has four gates:

1. Preflight gate: correct files, phase, skill, and dependency direction are known.
2. Build gate: touched crates compile.
3. Behavior gate: targeted tests prove intended behavior.
4. Alignment gate: comments, docs, and plan state match implementation.

## Preflight gate

Check before editing:
- Which phase from `INITIAL_PLAN.md` applies.
- Which files from `PROPOSED_FILE_TREE.md` apply.
- Which local skills apply.
- Which agent should own implementation.
- Whether the crate dependency direction stays legal.

## Build gate

Minimum commands:
- `cargo check -p <crate>` for touched crate.
- `cargo check --workspace` after cross-crate changes.

## Behavior gate

Prefer focused tests:
- `cargo test -p spectre-core` for primitives.
- `cargo test -p spectre-graph` for routing/topology.
- `cargo test -p spectre-dsp` for DSP correctness.
- `cargo test -p spectre-project` for persistence/migration.
- `cargo test --workspace` before broad handoff.

## Alignment gate

Confirm:
- Jeff header block is present in new files.
- Pseudocode comments were replaced or tightened as code landed.
- `docs/architecture.md` or ADRs changed when architecture changed.
- No stale plan comments claim behavior that does not exist.
- No unrelated file churn remains.

## Reviewer verdicts

Use only:
- `APPROVED`: all gates pass.
- `REQUEST_CHANGES`: fixable issues exist.
- `BLOCKED`: missing decision, missing dependency, or impossible requirement.

Each non-approved verdict lists exact file paths and required changes.
