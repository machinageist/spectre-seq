---
name: geist-daw-working-context
description: "Load first for any Geist DAW task. Establishes repository shape, implementation order, invariants, quality gates, and how agents should use the local skills and agents."
---

<!--
Author: Jeff
Date: 2026-05-27
Description: Project working context for Geist DAW agents
Notes: Load first for every Geist DAW implementation or review task
-->

# Geist DAW Working Context

## Purpose

This repository contains one specification-first Geist DAW implementation workspace.

Current source of truth:
- `docs/README.md` defines documentation authority and conflict precedence.
- `docs/status/STATUS.md` and `docs/status/NEXT.md` define verified state and next slices.
- `docs/06-plans/rebuild-roadmap.md` defines rebuild milestone order.
- `Cargo.toml` and `crates/` are the only implementation workspace.

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

## Skill loading map

- Core IDs, ports, events, params, transport: load `geist-realtime-rust`.
- Graph, audio, DSP, plugin, and transport work: load `geist-realtime-rust`.
- Any review: load `geist-validation-gates`.

## Completion rule

A slice is not complete until:
- code compiles for the touched crate,
- targeted tests pass or a tracked reason exists,
- pseudocode comments reflect the implemented behavior,
- accepted requirements and plans are updated when architecture changes,
- no unrelated files were changed.
