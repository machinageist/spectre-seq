---
name: geist-plugin-hosting
description: "Load when implementing or reviewing CLAP or LV2 hosting, plugin scanning, metadata cache, dynamic bundle loading, host FFI, instance lifecycle, plugin params, GUI embedding, or plugin state save/restore."
---

<!--
Author: Jeff
Date: 2026-05-27
Description: External plugin hosting guide
Notes: Use for CLAP/LV2 scanner, DB, bundle loading, instances, params, GUI, state, and FFI
-->

# Geist Plugin Hosting

## Responsibility

Plugin hosting wraps unsafe external ABI contracts in narrow, testable Rust APIs and exposes plugin instances as graph nodes.

## Safety posture

- FFI modules may use `unsafe`; safe crates must not.
- Every unsafe block has a `// SAFETY:` comment with the upheld invariant.
- Plugin scanning/loading is app-thread only.
- Plugin processing follows host callback rules and never performs discovery.
- Host must survive plugin metadata errors without corrupting project state.

## CLAP implementation order

1. Define scanner search paths and result model.
2. Add metadata DB/cache abstraction.
3. Load bundle and resolve entry point.
4. Wrap host vtable in `ffi/host_impl.rs`.
5. Implement instance lifecycle: init, activate, process, deactivate, destroy.
6. Discover params and map get/set/flush.
7. Implement state save/restore as opaque bytes.
8. Add GUI embedding behind raw-window-handle.
9. Implement `ClapPluginNode`.

## LV2 priority

LV2 is lower priority. Keep the same `AudioNode` interface. Do not let LV2 complexity shape core graph APIs prematurely.

## Review checklist

- FFI lifetime ownership is explicit.
- Null pointers and invalid descriptors are handled.
- Plugin state bytes are versioned and associated with plugin identity.
- GUI work never enters audio-thread code.
- Tests use fake plugins or narrow wrapper tests where possible.
