---
name: geist-realtime-rust
description: "Load before implementing or reviewing any Spectre Seq Rust code that can touch the audio thread, process graph, DSP, plugin callbacks, transport snapshots, parameter updates, or lock-free communication."
---

<!--
Author: Jeff
Date: 2026-05-27
Description: Realtime-safe Rust rules for Spectre Seq
Notes: Applies to audio-thread code, shared primitives, callbacks, graph processing, and DSP
-->

# Geist Realtime Rust

## Hard constraints

- Audio callback code must not allocate.
- Audio callback code must not lock.
- Audio callback code must not block on I/O, channels, filesystem, logging, or plugin discovery.
- Audio callback code must not panic across FFI or host boundaries.
- UI/app thread owns mutation; audio thread consumes immutable snapshots and bounded queues.
- `unsafe` is isolated to FFI crates and documented with `// SAFETY:` invariants.

## Comment contract

- Comments are terse and declarative.
- Function comments use Jeff's `// Verb + noun` style.
- Comments state invariants, thread ownership, allocation behavior, and failure modes.
- Remove stale pseudocode as real implementation lands.

## Type rules

- Prefer newtypes for IDs: `NodeId`, `PortId`, `ClipId`, `TrackId`, `ParamId`.
- Prefer enums for domain distinctions: `Signal`, `PortType`, `PortDirection`, transport events.
- Prefer borrowed buffers in process context.
- Make ownership and thread boundary visible in type names.
- Return `SpectreResult<T>` for recoverable app-thread errors.

## Audio-thread checklist

- [ ] No heap allocation in `process` or callback path.
- [ ] No mutex/rwlock in callback path.
- [ ] No unbounded channels in callback path.
- [ ] No formatting/logging in hot path.
- [ ] No filesystem, plugin scanning, DB, serialization, or UI access.
- [ ] Bounded parameter/event queues drain deterministically.
- [ ] Buffer lengths are validated before processing.
- [ ] Denormal handling is explicit where relevant.

## Testing expectations

- Unit tests cover ID/type invariants and edge cases.
- Property-style tests are preferred for routing, ranges, and conversions.
- Debug-only allocation assertions are added for callback/hot paths when feasible.
- `cargo check -p <crate>` is minimum validation for each slice.
