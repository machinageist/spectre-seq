<!--
Author: Jeff
Date: 2026-07-12
Description: The single active Geist DAW milestone
Notes: Exactly one milestone is active; the roadmap owns ordering
-->

# Current Milestone — R0/R1 Foundation

- **Status:** accepted
- **Last verified:** 2026-07-16
- **Scope:** R0 completion and R1 musical-kernel exit
- **Decision authority:** Jeff
- **Upstream sources:** `rebuild-roadmap.md`, accepted requirements and decisions
- **Downstream dependents:** `../status/NEXT.md`, implementation slices
- **Supersedes:** all removed prototype plans
- **Superseded by:** none
- **Open decisions:** none that block the next code slice
- **Known gaps:** R1 identity/API disposition plus the R2 editable/compiled graph

## Current evidence

The root workspace contains a deterministic musical kernel plus the first R2/R4 vertical slice. `geist-dsp` defines borrowed planar buffers, bounded note events, immutable layouts, backend parameter descriptors, Pulse, Gain, Saturator, and a deterministic source. `geist-offline` renders the native device chain without a compiled graph. `geist-app` derives its Build and Shape surfaces from those backend contracts. This early vertical slice shortens feedback cycles but does not satisfy the editable/compiled graph or live-engine milestones.

The accepted R1 JSON and 960-PPQ decisions now have checked-in fixture evidence. TIME-003 is verified against signed pre-roll, exact and fractional piecewise boundaries, 24-hour positions, round-once accumulation, and honest nearest-tick quantization bounds. R1 exit still requires requirement-by-requirement identity and atomic-save API disposition.

## Requirements in scope

CORE-001..004, TIME-001..005, RT-001 as workspace policy, and the GRAPH-001 type seam.

## Non-goals

Audio I/O, production UI, devices, DSP, VST3, recording, and broad graph implementation. The launchable interaction prototype is exploratory evidence, not completion of these outcomes.

## Exit evidence

- `cargo fmt --all -- --check` passes.
- `cargo clippy --locked --workspace --all-targets -- -D warnings` passes.
- `cargo test --locked --workspace` passes.
- Tempo, time, transport, event, ID, and persistence properties are covered.
- The deterministic offline harness exists.
- Traceability and status match the implementation.