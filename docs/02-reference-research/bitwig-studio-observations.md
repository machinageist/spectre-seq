<!--
Author: Jeff
Date: 2026-07-11
Description: Atomic clean-room behavioral observations from the Bitwig Studio 5.3 user guide
Notes: Observed public behavior only; guide is one major version behind the shipping application
-->

# Bitwig Studio 5.3 Guide — Atomic Observations

- **Status:** draft
- **Last verified:** 2026-07-11
- **Scope:** source-anchored behavioral observations from guide chapters 1, 6, 9, 17 (including the On Grid Signals sub-page)
- **Decision authority:** Jeff
- **Upstream sources:** `SRC-BITWIG-GUIDE-53-CHAPTERS`; `docs/02-reference-research/bitwig-studio.md`
- **Downstream dependents:** requirements ledger, modular-routing and automation specs, command ontology
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** whether to re-verify claims against a version 6 guide when one is published
- **Known gaps:** extraction used assisted summarization of rendered pages; the guide documents Bitwig Studio 5.3 while the shipping application is 6.0.11 (verified on the official download page 2026-07-11), so no claim here describes current-product behavior

Every observation is `observed public behavior` from the rendered 5.3 guide. Adoption happens only in the requirements ledger.

## Chapter 1 — Concepts

- `OBS-BW53-CON-001`: Multiple projects may be open simultaneously, but audio is active for only one project at a time.
- `OBS-BW53-CON-002`: Clips are containers for notes or audio plus control and automation data; the documented model is "one DAW, two sequencers" (Arranger timeline and Clip Launcher) over the same tracks.

## Chapter 6 — The Clip Launcher

- `OBS-BW53-LAUNCH-001`: Arranger clips play at their designated timeline positions; Launcher clips are launched at will. Each track has a Stop Clips button and a "Switch Playback to Arranger" button restoring the Arranger as that track's active sequencer; global equivalents exist.
- `OBS-BW53-LAUNCH-002`: Launcher recording options include Automation Write, Overdub (merge vs. overwrite incoming notes), Record as Comping Takes with a defined take length, and record-on-scene-launch.
- `OBS-BW53-LAUNCH-003`: Empty slots on record-enabled tracks show record buttons; otherwise they alias stop buttons.

## Chapter 9 — Automation

- `OBS-BW53-AUTO-001`: Three automation recording modes: Latch (starts on first change, records until transport stops), Touch (records only while the control is held), Write (records from transport start to stop, overwriting existing points).
- `OBS-BW53-AUTO-002`: Manual override of an automated parameter arms a "Restore Automation Control" button; the automation indicator changes color (blue = automated, green = overridden) until restored.
- `OBS-BW53-AUTO-003`: "Automation Follow" (default on) moves track automation together with Arranger clip move/copy/duplicate; disabling it leaves automation stationary.
- `OBS-BW53-AUTO-004`: The primary automation lane is a focus-following "joker lane" (last-clicked parameter), with a Pin Parameter lock; additional named lanes are available; parameter choosers list items in signal-flow order (MIDI, devices, then mixer).
- `OBS-BW53-AUTO-005`: Point editing: Shift bypasses grid, Alt+drag curves a transition, Alt+double-click resets it to linear, Alt+drag on a selection boundary time-scales the enclosed automation. Recorded curves are automatically simplified when transport stops.
- `OBS-BW53-AUTO-006`: The guide distinguishes absolute automation from relative automation, where multiple control layers cooperate on one parameter (detailed mechanics beyond this chapter's opening remain unextracted).

## Chapter 17 — The Grid

- `OBS-BW53-GRID-001`: The Grid ships 180+ modules in 16 categories across three host devices: Poly Grid (instrument), FX Grid (audio effect), Note Grid (note processor/generator).
- `OBS-BW53-GRID-002`: Patching rules: an out port fans out to unlimited in ports; an in port accepts exactly one cable; unconnected in ports read zero; add/delete of modules does not interrupt sound.
- `OBS-BW53-GRID-003`: Signal types are color-coded with defined semantics: Logic (yellow) treats ≥ +0.5 as high, emits +1/0, and responds to transitions; Phase (purple) is unipolar 0 to just-below-1 with wraparound, used as a lookup index for phase-driven sequencing; Pitch (orange) is bipolar with 0 = middle C and ±0.1 per octave (−1..+1 spans twenty octaves); untyped generic signals carry no fixed range.
- `OBS-BW53-GRID-004`: Every Grid signal — including triggers and control signals — is a stereo pair, and the Grid runs internally at four times the configured sample rate; modulators, by contrast, are mono at standard rate.
- `OBS-BW53-GRID-005`: Voice lifetime is governed by "Affect Voice Lifetime" flags on envelope modules (AR/AD/ADSR/Pluck), Note In, Gate In, and Audio Out — a voice stays alive until all participating conditions complete.
- `OBS-BW53-GRID-006`: FX Grid voicing modes are True Mono (default), Polyphony, and Digi Mono with optional auto-gate envelope triggering; Note Grid defaults to Polyphony.
- `OBS-BW53-GRID-007`: Inline patching accelerators: dragging a module onto a port, module edge, or existing cord inserts it into the flow; drawing a cord while holding Shift inserts a processor (e.g., attenuator); drawing into an occupied in port with a modifier inserts a mixer; a padlock "performance mode" allows parameter changes but blocks structural edits; F1 opens interactive module help.

## Cross-cutting patterns worth carrying into requirements work

1. Bitwig types its modular signals semantically (logic/phase/pitch) rather than only by rate, and gives each a documented numeric contract — the phase-as-lookup-index concept is the foundation of its sequencing modules.
2. Uniform stereo signal paths and a fixed internal oversampling factor trade CPU for the elimination of mono/stereo special cases.
3. Voice lifetime is a distributed predicate (any module can hold a voice open) rather than a single envelope's gate.
4. Automation override/restore is a first-class visible state with a dedicated restore control, same conceptual shape as Live's Re-Enable Automation.
5. The two-sequencer precedence model is per-track, with explicit "return authority to timeline" controls at track and global scope — matching Live's Back to Arrangement semantics from the other direction.
