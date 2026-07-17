<!--
Author: Jeff
Date: 2026-07-12
Description: Dependency-ordered Geist DAW roadmap R0-R12
Notes: Milestones close on demonstrable capability, not compilation alone
-->

# Rebuild Roadmap

- **Status:** accepted
- **Last verified:** 2026-07-12
- **Scope:** milestone order, outcomes, and exit gates
- **Decision authority:** Jeff
- **Upstream sources:** accepted vision and requirements
- **Downstream dependents:** `current-milestone.md`, implementation slices
- **Supersedes:** all removed prototype roadmaps
- **Superseded by:** none
- **Open decisions:** milestone-gated rows in `../01-requirements/decision-gates.md`
- **Known gaps:** R5+ detail is intentionally deferred to milestone intake

The repository root is the sole implementation workspace.

| Milestone | Outcome | Key requirements | Exit gate |
|---|---|---|---|
| R0 — foundation | Pinned stable toolchain, workspace rules, checks, tests, deterministic offline harness | RT-001 policy, CORE seeds | locked tests, strict Clippy, formatting, docs |
| R1 — musical kernel | IDs, time, tempo/meter, transport, events, parameters, command seed, versioned project round trip | CORE-001..004, TIME-001..005 | property tests and round-trip fixtures |
| R2 — offline graph | Editable graph → validated compiled plan → deterministic offline render; source and gain devices | GRAPH-001..002 | silence, impulse, allocation, and deterministic-hash tests |
| R3 — live shell | Qualified audio backend, callback bridge on the same plan, MIDI timestamps, health telemetry | RT-001..003 | allocation/lock guards and device lifecycle drill |
| R4 — credible alpha | Track→master, MIDI clip, small original synth/effect, minimal UI, save/reload, bounce | product seeds | end-to-end fixture and manual QA protocol |
| R5 — project safety | Atomic save, journaled autosave, recovery, migrations, undo/redo, missing-media diagnostics | CORE-004 full | crash/recovery drills |
| R6 — tracks/routing/mixer | Track types, groups, sends, returns, monitoring, compensation, meters | routing intake | latency matrix |
| R7 — arrangement/recording | Audio/MIDI recording, editing, piano roll, fades, count-in, metronome | recording intake | recording and salvage drills |
| R8 — VST3 host | Isolated scan, fixtures, processing, state, editor, placeholders | VST intake | licensed binding decision and fixture matrix |
| R9 — automation/modulation | Stable bindings, required sample accuracy, override/restore, overlays | PROD-002 | semantics tests |
| R10 — session/live | Slots, scenes, quantized launch, per-track precedence, capture | PROD-001 | performance-capture drills |
| R11 — Geist identity | Original modular surface, flagship synth, deep MIDI, effect catalog | identity intake | identity-layer QA |
| R12 — release qualification | Performance/soak, accessibility, recovery, packaging, documentation | release intake | published release gates |

Reordering requires documented dependency reasoning. R0/R1 is active.