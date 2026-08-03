<!--
File: docs/adr/001-clap-over-vst.md
Layer: documentation
Purpose: Record the plugin-format decision for the host (VST3 over CLAP)
Status: Accepted 2026-06-11. Supersedes the original CLAP-first intent.
Contract: State current architecture, not aspiration. Record tradeoffs.
-->

# ADR 001 — VST3 over CLAP for the plugin host

- Status: **Accepted** (2026-06-11)
- Supersedes: the original `INITIAL_PLAN.md` Phase 4 intent (CLAP-first hosting)

## Context

Phase 4 originally targeted CLAP hosting (`geist-clap-host` wrapping `clap-sys`),
with VST deferred. The deciding factor at the time was licensing and hosting
ergonomics: the VST3 SDK was dual-licensed GPLv3-or-proprietary, which would have
forced the whole DAW to GPLv3 or required a Steinberg license, while CLAP is MIT
and its C ABI is designed for easy hosting.

Two facts changed the calculus:

1. **VST3 SDK relicensed to MIT (October 2025, SDK 3.8.0).** The copyleft/proprietary
   constraint is gone. The `vst3` Rust crate can now ship bindings generated from
   the MIT headers without a license encumbrance on downstream hosts.
2. **Ecosystem reality.** Nearly every commercial instrument and effect ships VST3.
   CLAP adoption is growing but remains a minority of the installed plugin base a
   user already owns. For a DAW meant to stand beside Ableton and Logic, hosting the
   user's existing plugins is the higher-value capability.

## Decision

- The plugin host targets **VST3 only**. CLAP and LV2 are dropped from the active plan;
  `geist-clap-host` and `spectre-lv2-host` remain as shelved scaffolds, not built.
- The host is built in a new crate **`geist-vst-host`** that **wraps the raw `vst3`
  COM bindings in a safe Rust API** — mirroring the original "wrap `clap-sys`"
  philosophy. The realtime hot path stays first-party IP rather than delegated to a
  higher-level hosting crate.
- Each loaded plugin instance is a `VstPluginNode` implementing the existing
  `spectre-graph::AudioNode`, slotting into the process graph like any other node.

## Consequences

- **Unsafe FFI is required and concentrated here.** `geist-vst-host` is the one
  first-party crate that does not `#![deny(unsafe_code)]`; unsafe is confined to the
  FFI/instance layer behind narrow safe wrappers, per the project's FFI rule.
- **VST3 hosting is harder than CLAP would have been.** VST3 is a bidirectional
  COM-style ABI: the host implements interfaces (`IHostApplication`,
  `IComponentHandler`) the plugin calls back into, on top of driving `IComponent` /
  `IAudioProcessor` / `IEditController`. More surface, more lifecycle invariants.
- **End-to-end behavior cannot be validated headless.** Scanner, bundle resolution,
  and descriptor/cache layers are unit-tested in CI; the FFI/instance/process layers
  are compile-checked here and validated against real `.vst3` binaries on a dev box.
- The project format already stores opaque plugin state blobs (ADR 003), so VST3
  `IComponent` get/setState fits the existing persistence model unchanged.

## Alternatives considered

- **`rack` (multi-format host crate):** unified VST3 + CLAP interface, far less
  unsafe, fastest to working audio. Rejected for now: puts a younger third-party crate
  on the realtime path, against the goal of owning the hot path.
- **`plugin_host` (batteries-included):** scanning, process sandboxing, crash
  auto-restart, presets. Rejected as the foundation for the same control reason; its
  feature set is a useful reference for later (especially crash isolation).
- **VST2 via the `vst` crate:** rejected outright — the VST2 SDK has been
  license-unavailable from Steinberg since 2018 and is not viable for a shippable product.
