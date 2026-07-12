<!--
Author: Jeff
Date: 2026-07-11
Description: Qualitative methodology for studying musician workflows, commands, shortcuts, gestures, and friction
Notes: Workflow observations are contextual evidence, not population statistics
-->

# Workflow Field-Study Methodology

- **Status:** accepted
- **Last verified:** 2026-07-11
- **Scope:** musician workflow sampling, observation records, shortcut/gesture evidence, saturation, confidence, and product implication controls
- **Decision authority:** Jeff
- **Upstream sources:** `docs/02-reference-research/methodology.md`; workflow research mandate
- **Downstream dependents:** workflow source index, corpus, archetypes, command ontology, shortcut analysis, friction analysis, and product implications
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** interview recruitment and usability-study protocol after online-source saturation
- **Known gaps:** one FL Studio professional session and three tutorials have passed review; independent friction, measured monitoring latency, recording failure/recovery, and all other product coverage remain absent

## Purpose

This study identifies repeated action chains, muscle-memory commands, transitions, failure recovery, and friction in real music work. It does not estimate market share or prove population frequency.

## Sampling frame

The corpus SHOULD cover electronic composition/sound design, beatmaking, band and singer-songwriter recording, vocal comping/editing, mixing/mastering, live performance, hardware/hybrid work, scoring where relevant, laptop-only work, keyboard-heavy work, pointer-first work, controller/touch work, and varied experience levels.

Jeff's explicit product rationale dated 2026-07-11 makes hypnotic techno, forest psytrance, deep dubstep, modern synthesis/arrangement, and realtime audio-interface input/monitoring priority sampling contexts. This prioritizes evidence collection; it does not pre-authorize feature details, platform backends, device compatibility, latency targets, or implementation choices.

Major DAW references are Ableton Live, Bitwig Studio, FL Studio, REAPER, Logic Pro, and Cubase. Sound-design references are VCV Rack, Phase Plant, and Serum 2. Other tools require a stated Geist-relevant reason.

The initial search floor is 8–12 substantive workflow accounts per major DAW across at least three workflow categories, and 5–8 per synth/modular reference. This is a saturation heuristic, not a statistical sample requirement. Poor evidence MUST be reported rather than padded.

## Evidence preference

1. Visible start-to-finish creation, recording, mixing, or live-performance sequences.
2. Official artist workflows and project walkthroughs.
3. Deep educator demonstrations.
4. Practitioner discussions of repeated actions, pain points, templates, mappings, macros, and workarounds.
5. Verified shortcut references.
6. Release/migration discussions revealing disrupted workflows.

Complete-task evidence receives more interpretive weight than isolated tips. Marketing and community evidence require triangulation for different reasons.

## Observation schema

Each observation MUST contain:

- stable workflow ID and source IDs;
- software/version and evidence class;
- role, experience, genre, and environment when known;
- goal and starting state;
- ordered action sequence;
- semantic commands, bindings, pointer gestures, controller actions, modes, and view transitions;
- object types acted on;
- repeated inner loop and completion condition;
- latency-sensitive moments;
- errors, dead ends, menu hunting, context switches, and recovery;
- templates, presets, macros, custom actions, scripts, or mappings;
- available-but-avoided features;
- separately labeled product inference;
- confidence and corroborating observations.

Unknown fields remain null; they MUST NOT be guessed.

## Command normalization

Bindings are evidence, not command identity. Each action maps to an original semantic command ID such as `edit.duplicate_selection` or `transport.play_stop`. Records include product/platform/version binding, focus scope, remappability, discoverability, equivalent gestures, accidental-activation cost, undo posture, accessibility considerations, and observed workflow use.

The corpus MUST distinguish:

- transferable platform conventions;
- cross-product command convergence;
- product-specific workflow models;
- commands commonly chained or remapped;
- text-entry, musical-typing, plugin-editor, OS, accessibility, and keyboard-layout conflicts;
- destructive or live-critical commands requiring protection.

## Analysis and scoring

The qualitative core-loop score considers observed recurrence, number of workflow archetypes, creative time sensitivity, menu-access cost, error/recovery impact, keyboard/controller need, cross-product convergence, and accessibility importance. Scores rank research priorities only; they are not user-population measurements.

## Saturation and review

A workflow category approaches online-source saturation when additional substantive sources repeat known action chains and friction without adding new command, transition, recovery, or role/context patterns. Saturation is recorded per product and archetype, never globally.

Each accepted observation requires a direct source, relevant timestamp/page/section, extraction review, and confidence rationale. Product implications require corroboration or explicit low-confidence labeling. Final usability targets require Geist prototype testing; external workflows alone cannot set them.
