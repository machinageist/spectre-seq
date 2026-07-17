<!--
Author: Jeff
Date: 2026-07-11
Description: Geist DAW product North Star — audience, identity, release bars, non-goals
Notes: Accepted by Jeff through delegated decision authority on 2026-07-12
-->

# Geist Product Vision

- **Status:** accepted
- **Last verified:** 2026-07-12
- **Scope:** who Geist is for, what it must be excellent at first, what makes it Geist, release bars, non-goals
- **Decision authority:** Jeff
- **Upstream sources:** Jeff's stated genre priorities and `docs/02-reference-research/`
- **Downstream dependents:** requirements ledger, decision gates, rebuild roadmap, all specs
- **Supersedes:** all removed prototype product framing
- **Superseded by:** none
- **Open decisions:** scoring/video remains outside current scope
- **Known gaps:** workflow research remains below saturation

## Who Geist is for

Geist serves electronic musicians who compose, sound-design, and perform their own material — first concretely: hypnotic techno, forest psytrance, deep dubstep, and modern synthesis-driven arrangement (Jeff's stated priority genres), spanning studio production and live performance with an audio interface, MIDI controllers, and hardware synths. Recording-band and scoring workflows are respected later-stage citizens, not the first target.

## What Geist must be excellent at first

Accepted core loop, subject to evidence-driven refinement:

1. Loop-first composition: sketch a loop, branch variations, audition instantly, grow an arrangement — without losing selection, zoom, or transport context.
2. Sound design in first-party devices: a serious native synth and effect set with visible, assignable modulation.
3. Performance-grade playback: launchable clips/scenes with explicit timeline-vs-performance precedence per track.
4. Trustworthy capture: audio/MIDI recording with correct latency compensation, retrospective MIDI capture, and crash-salvageable media.
5. Project safety: atomic save, autosave, recovery, missing-media repair — losing work is a product-killing defect.

## What makes it distinctly Geist

- One project, linked lenses: timeline, performance grid, mixer, and a modular sound-flow view are views over one model with shared selection and stable identity — not separate apps stapled together.
- Modulation as a first-class visible citizen: every parameter shows base value, automation, and modulation contribution distinctly (research shows both Live and Bitwig converge on override/restore semantics; Geist designs this in from the model outward).
- An explicit, original modular signal contract: Geist defines its own typed signal model (pitch/gate/phase/audio) informed by — not copied from — VCV's voltage standards, Bitwig's typed stereo signals, and Phase Plant's stack routing.
- Rust-native engine with a published realtime contract and enforced allocation/lock discipline on the callback path.
- Keyboard-first, calm UI: context-scoped command resolution (the pattern both Live and Bitwig depend on), searchable and remappable commands, no spreadsheet density, no cable spaghetti by default.
- Open source with original code, DSP, names, content, and formats.

## Release bars

| Stage | Bar |
|---|---|
| Credible alpha | R4 vertical slice verified: one track, MIDI clip, native synth + effect, transport, save/reload, offline bounce — honest telemetry, no fake surfaces |
| Musician beta | Loop-first core loop end-to-end: recording, editing, launcher, automation, project safety (R5–R10), qualified on two platforms |
| 1.0 | Adds VST3 hosting with crash containment, the Geist modular/synth identity layer (R11), accessibility baseline, packaging |
| Professional-ready | Mandate §9 gates: workflow, reliability, performance budgets, recovery drills, compatibility matrix, documentation |

## Non-goals (current stage)

- No CLAP/LV2/AU hosting; no plugin-format authoring of first-party devices.
- No preset/project compatibility with any other DAW or synth.
- No cloud services, collaboration servers, or content stores.
- No video scoring until explicitly scoped in.
- No feature parity race: capability comparisons inform requirements; they do not define completeness.
