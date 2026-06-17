<!--
File: docs/adr/004-egui-first-wgpu-later.md
Layer: documentation
Purpose: Define 004 egui first wgpu later responsibilities and integration boundaries.
Status: Pseudocode scaffold; implementation intentionally pending.
Contract: Keep comments terse, declarative, and synchronized with code.
-->

# 004 Egui First Wgpu Later

## Pseudocode plan
- Declare responsibility: Define 004 egui first wgpu later responsibilities and integration boundaries.
- Define public types before behavior.
- Separate real-time-safe paths from UI/app paths.
- Prefer explicit errors over implicit panics.
- Add tests beside behavior once implementation begins.
- Read state; emit commands; own no ground truth.
- Keep rendering deterministic from UI state.
- Throttle expensive work outside frame loop.
- State current architecture, not aspiration.
- Record tradeoffs and invariants.
- Link decisions to implementation files.
