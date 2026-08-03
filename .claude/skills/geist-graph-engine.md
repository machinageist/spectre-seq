---
name: spectre-graph-engine
description: "Load when implementing or reviewing `crates/spectre-graph`, graph topology, edge validation, process-list compilation, feedback delay insertion, graph swapping, routing nodes, metering, or graph tests."
---

<!--
Author: Jeff
Date: 2026-05-27
Description: Graph engine implementation guide
Notes: Use for spectre-graph topology, routing, process lists, and lock-free swap work
-->

# Geist Graph Engine

## Responsibility

`spectre-graph` converts mutable app-thread graph edits into an immutable process list for the audio thread.

## Architecture rules

- `Graph` owns nodes, edges, and port registry on the app thread.
- `Edge` validation checks direction, type compatibility, and existence before insertion.
- Topology compilation produces a flat `Vec<ProcessStep>`.
- Audio thread receives only compiled process lists.
- Feedback cycles are explicit: reject or insert one-block `DelayNode` per policy.
- Atomic swap is the only graph publication mechanism.

## Implementation order

1. Define `AudioNode` trait and no-op/passthrough test node.
2. Define `Edge` and port validation.
3. Implement graph add/remove/connect/disconnect APIs.
4. Implement topological sort and deterministic ordering.
5. Add cycle detection.
6. Add one-block delay insertion for supported feedback paths.
7. Compile process list.
8. Add swap wrapper.
9. Add ring-buffer command/metering channels.
10. Add routing, cycle, and topology tests.

## Tests to require

- Empty graph compiles.
- Single node processes.
- Linear chain order is deterministic.
- Fan-in and fan-out route correctly.
- Type mismatch returns error.
- Missing node/port returns error.
- Cycle is detected.
- Feedback path inserts delay when policy allows.
- Compiled process list is immutable from audio-thread view.

## Review focus

- No app-thread graph structure leaks into callback path.
- No allocation inside process loop beyond precompiled scratch strategy.
- Stable ordering makes tests reproducible.
- Error messages identify invalid connection without exposing confusing internals.
