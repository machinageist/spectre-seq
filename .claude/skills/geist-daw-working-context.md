---
name: spectre-seq-working-context
description: "Load first for any Spectre Seq task. Establishes repository shape, implementation order, invariants, quality gates, and how agents should use the local skills and agents."
---

<!--
Author: Jeff
Date: 2026-05-27
Description: Project working context for Spectre Seq agents
Notes: Load first for every Spectre Seq implementation or review task
-->

# Spectre Seq Working Context

## Purpose

This repository is an early scaffold for a modular Rust DAW.

Current source of truth:
- `INITIAL_PLAN.md` defines phase order.
- `PROPOSED_FILE_TREE.md` defines intended architecture.
- Generated Rust/config/docs files currently contain pseudocode comments, not implementation.

## Execution posture

- Work plan-first.
- Execute one fine slice at a time.
- Keep docs and comments synchronized with code in the same slice.
- Preserve Jeff's standard file header block.
- Prefer explicit, boring correctness over cleverness.
- Treat real-time audio safety as a hard product requirement, not an optimization.

## Standard header block

Rust/source files:

```rust
// Author: Jeff
// Date: YYYY-MM-DD
// Description: What this file does
// Notes: Non-obvious context, design decisions
```

Markdown files:

```md
<!--
Author: Jeff
Date: YYYY-MM-DD
Description: What this file does
Notes: Non-obvious context, design decisions
-->
```

## Phase order

1. Workspace and `spectre-core` primitives.
2. `spectre-graph` process graph and atomic graph swap.
3. `spectre-audio-backend` callback boundary.
4. `spectre-dsp` primitives and benchmarks.
5. CLAP host, then LV2 host.
6. First-party plugins: synth, fx, modular utilities.
7. Timeline, automation, modulation.
8. UI shell and views.
9. Project persistence and autosave.
10. xtask, release packaging, docs, ADRs.

## Skill loading map

- Core IDs, ports, events, params, transport: load `geist-realtime-rust`.
- Graph, topology, process list, channels: load `spectre-graph-engine` and `geist-realtime-rust`.
- Audio backends and callbacks: load `spectre-audio-backend` and `geist-realtime-rust`.
- DSP, synth, fx, modular utilities: load `spectre-dsp-and-plugins` and `geist-realtime-rust`.
- CLAP/LV2 hosting: load `geist-plugin-hosting` and `geist-realtime-rust`.
- Timeline, automation, persistence: load `spectre-project-timeline`.
- UI work: load `spectre-ui-workflow`.
- Any review: load `geist-validation-gates` plus the domain skill.

## Completion rule

A slice is not complete until:
- code compiles for the touched crate,
- targeted tests pass or a tracked reason exists,
- pseudocode comments reflect the implemented behavior,
- docs/ADRs are updated when architecture changes,
- no unrelated files were changed.
