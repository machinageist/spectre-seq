<!--
Author: Jeff
Date: 2026-07-31
Description: Living product contract for Geist as a modern electronic-music production DAW.
Notes: Amend this contract only through explicit product decisions; implementation plans derive from it.
-->

# Geist Product Vision

## Status

This document is the authoritative product-direction contract. `INITIAL_PLAN.md` remains useful historical sequencing context, but it does not fully define the product.

## Product promise

Geist is a modern, local-first electronic-music production environment in the Ableton Live and Bitwig Studio category. It combines:

- linear arrangement;
- quantized clip and scene launching;
- live audio and MIDI performance;
- non-destructive audio and MIDI editing;
- recording into arrangement and launcher slots;
- a professional first-party instrument, effect, analysis, and MIDI-device suite;
- a deeply modular typed signal system where devices can communicate without collapsing into an unsafe or incomprehensible free-for-all;
- two flagship first-party instruments: a streamlined Serum/Phase Plant-like hybrid synthesizer and a VCV Rack-like modular instrument.

Geist should be fast for ordinary production and progressively disclose its deeper graph. Users should not have to patch a graph to perform routine work, but the normal device chain must not be a separate processing truth from the advanced graph.

## Primary production surfaces

### Arrangement

The arrangement is a linear musical timeline with complete audio and MIDI region editing, automation, transport, loop, punch, recording, and rendering workflows.

### Launcher

The launcher is a second performance surface with one slot at each track/scene intersection. It supports individual clip launch, scene launch, track stop, quantized transitions, and recording launcher performance into the arrangement.

Launcher playback overrides arrangement playback independently per track. The override remains active until the user invokes Back to Arrangement for that track or an explicit grouped scope.

A project-wide launch-quantization default may be overridden by individual clips and scenes. Immediate launch remains available.

Launcher and arrangement are composable tiled workspace panes. Either may be focused independently, and both may be visible simultaneously; they are not mutually exclusive application modes.

Arrangement capture creates independent clip entities. Later edits do not silently propagate back to launcher sources. Explicit linked-content semantics are deferred until propagation, unlinking, persistence, and undo are specified.

### Editors

Audio and MIDI clips are fully editable without destructive source mutation.

Audio regions reference immutable managed assets and own their region-local edit state. Split, trim, fades, gain, reverse state, warp markers, tempo interpretation, and similar edits do not rewrite shared source media. Consolidate, bounce, resample, and render create new managed assets.

Non-destructive warp markers and tempo synchronization are core audio architecture. Initial playback may provide simpler repitch and beat-preserving modes; advanced spectral modes may arrive later without changing region identity.

MIDI clips own structured note entities rather than only opaque raw event streams. The canonical model supports stable note identity, MPE-style per-note pitch, pressure, timbre, and expression curves. The complete expression editor may follow the data model.

Twelve-tone equal temperament is the default, not a permanent schema limitation. Project and device tuning, Scala/MTS interoperability, and per-note pitch are planned from the canonical note and instrument model.

### Conductor, tempo, and groove

One project conductor owns canonical tempo automation, time-signature markers, metronome and count-in behavior, and reusable groove/swing templates. Tracks do not own competing tempo maps. Devices may transform note or event timing explicitly without changing project musical time.

## Track and device model

Tracks are hybrid typed pipelines. Audio, Instrument, Group, Return, and related choices are user-facing templates over one graph-backed track model, not incompatible track classes.

The ordinary track device chain is an ordered projection of the typed processing graph. Users may expand into the graph for advanced valid routing. Chain and graph views must never become competing sources of routing truth.

MIDI devices are first-class note/event processors. They may appear before or between instruments and participate in routing and modulation where their declared ports and parameters permit it.

Stable remote-control pages and macros exist at device, rack, track, and project-performance scopes. MIDI learn and controller mappings target durable parameter identities and survive UI rearrangement.

## Typed communication contract

Geist distinguishes signal domains explicitly:

- audio streams;
- control voltage and modulation streams;
- gates and triggers;
- structured note events;
- raw MIDI where required for interoperability;
- parameter-control streams;
- meters and analysis feedback.

The first production graph supports mono and stereo audio buses as first-class layouts. Upmix and downmix are explicit adapters rather than hidden connection behavior. Bus descriptors remain extensible to declared multichannel layouts for later surround, spatial, and complex plugin I/O support.

Every feedback cycle contains an explicit visible delay or feedback element. The compiler does not silently convert arbitrary cycles into hidden one-block latency. Audio/CV and event/control domains use declared domain-appropriate delay modules so causality, latency, and persisted patch behavior remain inspectable.

Every exposed parameter supports automation and control-rate modulation. Audio-rate modulation is allowed only for destinations that explicitly declare and implement it. Rate conversion, smoothing, latency, and feedback behavior must be deterministic and visible.

Parameter control has three distinct layers: arrangement automation in project musical time, clip-local automation in clip-relative time, and realtime modulation as a live signal layer. They do not share one undifferentiated data model. Durable target identity precedes a dedicated specification of absolute/relative combination, takeover, and evaluation precedence.

Per-voice modulation remains inside its voice domain unless an explicit reduction, broadcast, split, or merge adapter crosses the boundary. Global modulators may target parameters across devices and tracks.

Typed cables in the modular instrument may carry polyphonic lanes. Modules declare whether they process lanes independently, reduce lanes, broadcast values, or alter lane count.

## Live audio and recording

Audio tracks provide explicit input routing and Off, Auto, and In monitor modes. Monitoring includes latency compensation and feedback prevention.

The first complete recording milestone supports:

- recording audio and MIDI into the arrangement;
- recording audio and MIDI into launcher slots;
- overdub behavior defined per clip kind;
- accurate placement against compensated input and transport time;
- project-managed media with exact offline recovery.

Loop take lanes and comping follow in a later milestone rather than blocking the first complete capture workflow.

## Flagship hybrid synthesizer

The streamlined flagship synthesizer is a distinct first-party instrument built from shared Geist DSP, parameter, modulation, preset, and UI primitives.

Its architecture uses bounded add/reorder generator lanes rather than a fixed two-oscillator panel. Planned generator families include:

- wavetable;
- sample and granular;
- virtual analog;
- noise;
- utility and modulation-capable sources.

Generators feed multiple signal lanes. Each lane supports serial processing, lanes mix in parallel, and explicit sends and cross-lane routes are available. The synth remains intentionally bounded and workflow-oriented rather than exposing the entire DAW graph inside its primary interface.

## Flagship modular instrument

The modular flagship is a distinct VCV Rack-like instrument sharing DSP modules, parameter/modulation protocols, presets, and UI primitives with the rest of Geist.

It provides a true patchable instrument environment with typed audio, CV, gate, note, event, and parameter connections; explicit polyphonic-lane behavior; deterministic feedback policy; module discovery; patch persistence; macros; and reusable subpatches.

A rack patch declares a stable typed external port interface. The same rack environment may therefore serve as an instrument, audio effect, note/event device, or modulation source without splitting into incompatible rack products. Reusable subpatches expose deliberate public ports rather than allowing external cables to depend on hidden internals.

It is not equivalent to the current utility-node crate alone. A complete rack also requires durable module identity, a patch model, typed multi-rate execution, a module registry, polyphonic semantics, and an editor.

## First-party production suite

The first production-quality device milestone balances mixing and creative essentials:

- gain, pan, stereo, phase, routing, and metering utilities;
- parametric EQ;
- compressor;
- gate and expander;
- limiter;
- saturation and clipping;
- delay;
- reverb;
- chorus, flanger, and phaser;
- spectrum, oscilloscope, loudness, and level analysis.

Advanced creative, spectral, multiband, pitch, convolution, dynamics, restoration, and mastering devices follow in explicit tiers.

The MIDI-device suite is planned as graph-native note/event processing. Initial candidates include arpeggiation, chord, scale, transpose, velocity, note length, note echo/delay, probability/randomization, expression mapping, filtering, routing, and monitoring.

## External plugins

Geist prioritizes one complete external plugin host before supporting several incomplete formats. VST3 is first under accepted ADR 001, including scanning, state, preset, automation, latency, note-expression/MPE, GUI embedding, missing-plugin preservation, diagnostics, and crash recovery. CLAP and other formats follow only after the VST3 host meets the same project and realtime contracts as first-party devices.

## Project package and recovery

A Geist project is a directory package containing the canonical manifest and managed subdirectories for recordings/media, renders/freezes, autosaves/backups, and disposable cache. External media may remain referenced without being rewritten. Collect All and Save imports external dependencies into managed storage using verified asset identity. Manual saves use atomic replacement; autosave and startup recovery never overwrite the last known-good manual save.

Before the first declared stable project format, prototype schemas may receive intentional breaking cleanup with explicit diagnostics and fixture-tested conversion where practical. Stable project format v1 begins the supported forward-migration contract. Loads never report success after silently discarding unknown or invalid durable state, and Geist never silently writes an older format.

## Product invariants

1. One app-thread `ProjectDocument` owns durable DAW truth. UI, persistence, and compiled realtime state are projections, not peer authorities.
2. Realtime publication is versioned and acknowledged; queue saturation or rejected publication cannot silently advance app/UI mirrors.
3. No file I/O, allocation, deallocation, locking, logging, UI work, or mutable project traversal occurs on the audio callback. Retired graphs, plugins, and media assets return to a non-realtime owner for destruction.
4. Project entities and automatable targets use durable identity; runtime handles do not leak into persistence contracts.
5. Missing media, plugins, devices, modules, and automation targets remain visible, inspectable, relinkable, and saveable where safe.
6. Ordered chains and the advanced graph are two views of one routing truth.
7. Clip launcher and arrangement playback arbitration is explicit per track.
8. Audio editing is non-destructive by default. Native source sample rate and sample-domain source coordinates remain explicit unless a declared render/warp operation transforms them.
9. MIDI note identity survives exact editing, undo, expression editing, and persistence.
10. Modulation-rate and polyphony-domain crossings are explicit.
11. User-facing controls reflect acknowledged canonical or realtime state; unavailable behavior is disabled or explained rather than simulated locally.
12. New functionality lands in small validated slices with independent review and reversible migrations.

## Undo and transaction scope

One project-level history owns every durable edit, including arrangement and launcher edits, routing and device changes, mixer state, automation, mappings, imports, and completed recording commits. Workspace panes do not own competing histories. Transport playback, live audition, launch performance, and an in-progress recording are transient; a successfully stopped recording creates one atomic take transaction.

## Realtime support envelope

The primary performance baseline is 48 kHz with 128-frame blocks on a documented mainstream eight-core reference system. A 64-frame live mode is a supported stress target. Higher sample rates and block sizes receive explicit coverage. Release gates use reproducible project fixtures for graph size, active tracks, voices, devices, modulation density, recording, graph swaps, and plugin teardown rather than unsupported headline track counts.
