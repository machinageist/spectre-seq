<!--
File: docs/clap_hosting.md
Layer: documentation
Purpose: CLAP host implementation notes
Status: Pseudocode scaffold; implementation intentionally pending.
Contract: Keep comments terse, declarative, and synchronized with code.
-->

# Clap Hosting

## Pseudocode plan
- Declare responsibility: CLAP host implementation notes
- Define public types before behavior.
- Separate real-time-safe paths from UI/app paths.
- Prefer explicit errors over implicit panics.
- Add tests beside behavior once implementation begins.
- Isolate unsafe FFI behind narrow wrappers.
- Validate plugin lifecycle transitions.
- Preserve host ABI invariants.
- State current architecture, not aspiration.
- Record tradeoffs and invariants.
- Link decisions to implementation files.
