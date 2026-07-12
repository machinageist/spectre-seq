<!--
Author: Jeff
Date: 2026-07-11
Description: Cross-product workflow archetype registry and evidence gates
Notes: Archetypes are research questions until reviewed observations support them
-->

# Workflow Archetypes

- **Status:** draft
- **Last verified:** 2026-07-11
- **Scope:** end-to-end musician task chains used to organize field evidence and later scenario requirements
- **Decision authority:** Jeff
- **Upstream sources:** `docs/02-reference-research/workflow-field-study/methodology.md`; `docs/02-reference-research/workflow-field-study/workflow-corpus.md`; reviewed `workflow-observations.jsonl`
- **Downstream dependents:** command ontology, friction analysis, product implications, user-workflow requirements, usability budgets
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** whether scoring/long-timeline work is an initial target cohort
- **Known gaps:** no reviewed workflow observations; no archetype has reached evidence-supported status; transition, focus, recovery, and accessibility evidence is absent

## Status vocabulary

- `research-question`: required task chain is defined, but no reviewed observation supports it.
- `partially-observed`: at least one reviewed source covers part of the chain.
- `triangulated`: multiple independent reviewed sources and evidence classes cover the chain without material unresolved contradiction.
- `requirement-candidate`: triangulated evidence and product rationale justify requirement review.
- `accepted`: linked Geist scenario requirements have decision authority approval.

No archetype is a requirement merely because it appears here.

## Archetype registry

| Archetype ID | User goal | Required transition chain | Populations to sample | Current state |
|---|---|---|---|---|
| `ARCH-FIRST-SOUND` | move from launch/empty project to audible, controllable material | project start → device/asset/input choice → track/route readiness → first audition/playback → saveable state | beginners; experts; laptop-only; controller users | research-question |
| `ARCH-VARIATIONS` | turn one loop or pattern into contrasting musical variants | select source material → duplicate/branch → edit rhythm/notes/sound → audition/A-B → retain named variants | hypnotic techno; forest psytrance; deep dubstep; hip-hop; keyboard-heavy | research-question |
| `ARCH-LAUNCH-ARRANGE` | turn performance/launcher ideas into a linear song | create/organize clips → launch scenes/combinations → capture or transfer → edit arrangement → resolve precedence | electronic; live performers; controller users | research-question |
| `ARCH-MULTITRACK-COMP` | record linear performances and produce an editable composite take | configure input/monitoring → record takes/lanes → audition → select/comp → edit transitions → preserve alternatives | vocalists; bands; engineers | research-question |
| `ARCH-SAMPLE-MATERIAL` | discover a sample and transform it into playable/arranged material | search/filter → preview in context → import/reference → slice/warp/pitch → map/place → preserve provenance | beatmakers; electronic; laptop-only | research-question |
| `ARCH-MIDI-EXPRESSION` | capture or enter notes and refine timing/expression | choose input → record/step/draw → select/edit → quantize/humanize → edit velocity/expression → audition | keyboard performers; QWERTY users; mouse-first | research-question |
| `ARCH-PATCH-DESIGN` | create an original reusable sound from initialized state | initialize → choose source → shape spectrum/time → route/modulate → performance-test → compare → save preset | hypnotic techno; forest psytrance; deep dubstep; sound designers; controller users | research-question |
| `ARCH-MOD-AUTO` | create evolving parameter behavior through modulation and recorded automation | identify target → assign modulator/controller → set depth/rate → record/edit automation → override/re-enable → inspect result | electronic; mixers; live performers | research-question |
| `ARCH-CREATIVE-PRINT` | commit or resample material to enable further creative work or reduce load | choose scope → configure print/bounce/freeze → render → replace/retain source → edit result → reverse/recover | electronic; mixing; laptop-constrained | research-question |
| `ARCH-ROUTE-MIX` | organize tracks and build groups, sends, sidechains, and gain structure | select sources → create destination → route/send → configure sidechain/monitoring → gain-stage → verify signal flow | mixers; bands; electronic; hardware hybrid | research-question |
| `ARCH-MIX-REVISION` | compare and revise a mix without losing prior state | save/version → import/reference or snapshot → level-match/A-B → revise → compare → export candidate | mixers; mastering; professionals | research-question |
| `ARCH-RECOVER-WORK` | return to productive work after missing media/plugin or crash | detect fault → explain impact → locate/substitute/defer → recover autosave/session → verify integrity → continue | all; large-project professionals | research-question |
| `ARCH-HARDWARE-RECORD` | record external hardware with usable monitoring and timing | configure audio interface/device/input → route through mixer → monitor/effect → measure/compensate → record → verify timing/media → recover from dropout/disconnect | electronic hardware hybrid; bands; engineers; vocal/instrument recording | research-question |
| `ARCH-LIVE-PREP` | prepare and safely perform a set with recoverable navigation | assemble material → map controls → define transitions → rehearse → run performance → recover from fault → archive changes | live performers; controller users | research-question |
| `ARCH-DELIVER-ARCHIVE` | create mixes, stems, alternates, and a portable project archive | choose deliverables → set ranges/names/formats → render → verify → collect media/state → reopen/validate archive | mixers; mastering; collaboration | research-question |
| `ARCH-CROSS-PROJECT` | reuse material between projects without breaking identity or assets | browse/open source → select objects → copy/import → resolve dependencies → place/adapt → save/reopen | composers; template users; professionals | research-question |
| `ARCH-KEYBOARD-EDIT` | perform high-speed editing without pointer dependence | focus intended surface → navigate/select → invoke semantic commands → inspect result → undo/repeat → change surface without losing context | keyboard-heavy; accessibility users | research-question |
| `ARCH-CUSTOM-COMMANDS` | adapt commands and gestures to a recurring workflow safely | discover command → bind/remap → detect conflict → compose validated alias → test → export/version configuration | power users; accessibility; one-handed users | research-question |

## Cross-archetype observation dimensions

Every reviewed archetype synthesis must account for:

- starting state and prerequisite setup;
- authoritative selection and focus;
- active view/lens and view transitions;
- transport, playhead, loop, audition, and monitoring continuity;
- semantic commands independent of bindings;
- pointer, touch, controller, and keyboard alternatives;
- repeated inner loop and its interruption cost;
- completion condition and user-visible confirmation;
- persistence, undo/redo, and versioning behavior;
- latency-sensitive moments;
- failure, recovery, and safe cancellation;
- accessibility and one-handed operation;
- templates, presets, mappings, macros, or scripts used to remove friction;
- features available but deliberately avoided;
- difference between observed behavior and researcher inference.

## Transition matrix

The field study must record transitions, not only feature presence.

| From | To | State that must be checked for preservation |
|---|---|---|
| Browser | arrangement/launcher/device | audition position, selected asset, destination, preview tempo/key, focus |
| Launcher/performance | arrangement | clip identity, launch timing, performance order, automation, precedence |
| Arrangement | clip/note/audio editor | object selection, time selection, playhead, loop, zoom, audition context |
| Track/device | automation/modulation | target identity, base value, automation state, modulation depth, visibility |
| Arrangement/launcher | mixer/routing | track selection, signal path, solo/mute/arm/monitor state, metering context |
| Project edit | render/bounce/freeze | source scope, range, quality, tail, destination, reversibility |
| Recording | comp/editor | take identity, lane state, timing correction, monitoring state, source media |
| Failure dialog/recovery | productive project | recovered revision, unresolved dependencies, muted/bypassed state, integrity warnings |
| Plugin editor | host command surface | keyboard focus, text entry, musical typing, global transport, accessibility |
| Any lens | command search | invocation context, selection, enabled state, shortcut conflict, return focus |

## Evidence sufficiency gate

An archetype may become `triangulated` only when:

1. at least three independent substantive sources have passed review;
2. at least two evidence classes are represented;
3. at least two relevant musician populations or environments are represented;
4. ordered actions and repeated inner loops are timestamped or page-cited;
5. transitions and focus/selection behavior are covered;
6. at least one source provides friction, workaround, or failure evidence;
7. source versions and limitations are explicit;
8. contradictions are recorded rather than averaged away.

This gate is qualitative. Meeting its count floor does not imply population-level representativeness.

## Current conclusion

All archetypes remain `research-question`. The registry establishes consistent extraction targets but provides no evidence for feature priority, command frequency, gesture budgets, or Geist requirements yet.
