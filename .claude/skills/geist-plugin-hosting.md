---
name: geist-plugin-hosting
description: "Load when implementing or reviewing VST3 hosting, plugin scanning, metadata cache, dynamic bundle loading, host FFI, instance lifecycle, plugin params, GUI embedding, or plugin state save/restore."
---

<!--
Author: Jeff
Date: 2026-05-27
Description: External plugin hosting guide
Notes: Use for VST3 scanner, cache, bundle loading, instances, params, GUI, state, and FFI
-->

# Geist Plugin Hosting

## Responsibility

Plugin hosting wraps unsafe third-party VST3 ABI contracts in narrow, testable Rust APIs and exposes plugin instances as internal graph/device nodes.

This skill is for external VST3 hosting only. First-party Geist synths, effects, MIDI tools, modulators, and modular utilities are internal devices and must not gain VST, CLAP, AU, LV2, or standalone plugin-export wrappers.

## Safety posture

- FFI modules may use `unsafe`; safe crates must not.
- Every unsafe block has a `// SAFETY:` comment with the upheld invariant.
- Plugin scanning/loading is app-thread only.
- Plugin processing follows host callback rules and never performs discovery.
- Host must survive plugin metadata errors without corrupting project state.

## VST3 implementation order

1. Define scanner search paths and result model.
2. Add metadata DB/cache abstraction.
3. Load `.vst3` bundle/module and resolve factory/class descriptors.
4. Wrap host COM callback surfaces behind narrow safe Rust APIs.
5. Implement instance lifecycle: init, activate, process, deactivate, destroy.
6. Discover params and map get/set/flush.
7. Implement state save/restore as opaque bytes.
8. Add GUI embedding behind raw-window-handle.
9. Implement the VST wrapper as an internal graph/device node.

## Historical formats

CLAP and LV2 are shelved historical scaffolds. Do not build new CLAP/LV2 features unless Jeff explicitly reverses the VST3-only policy.

## Review checklist

- FFI lifetime ownership is explicit.
- Null pointers and invalid descriptors are handled.
- Plugin state bytes are versioned and associated with plugin identity.
- GUI work never enters audio-thread code.
- Tests use fake plugins or narrow wrapper tests where possible.
