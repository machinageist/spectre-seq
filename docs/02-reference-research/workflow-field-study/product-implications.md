<!--
Author: Jeff
Date: 2026-07-11
Description: Controlled translation of workflow evidence and explicit product rationale into provisional Geist implications
Notes: Research implications are not requirements, architecture decisions, priorities, or usability targets
-->

# Workflow Product Implications

- **Status:** draft
- **Last verified:** 2026-07-11
- **Scope:** evidence-backed and product-rationale-backed questions for Geist workflow design
- **Decision authority:** Jeff
- **Upstream sources:** `workflow-observations.jsonl`; `workflow-archetypes.md`; `command-ontology.md`; explicit product direction from Jeff dated 2026-07-11
- **Downstream dependents:** future requirements ledger, usability scenarios, command priorities, architecture contracts, and rebuild roadmap
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** first release workflow boundary; native synthesis catalog; audio-interface backend/platform order; monitoring and latency target methodology
- **Known gaps:** only three FL Studio observations have passed review; no genre-specific session, live-input recording session, interface setup session, independent friction discussion, or Geist prototype measurement has passed review

## Interpretation boundary

This document separates explicit product rationale from external workflow evidence. Product rationale can establish what Geist is intended to serve; it cannot establish interaction details, performance budgets, default bindings, architecture, or acceptance targets without further specification and verification.

Nothing here is a stable Geist requirement ID. No observation count is a population-frequency claim.

## Explicit product rationale

Jeff identified the primary intended creative use as modern electronic music, specifically:

- hypnotic techno;
- forest psytrance;
- deep dubstep.

Geist must ultimately provide sufficiently modern synthesis and arrangement capability to author those styles rather than merely play back imported material. It must also support recording audio from common USB audio interfaces, including M-Audio- and Focusrite-class devices, and route that live input through the mixer in realtime.

Brand examples identify the device class and workflow expectation, not a vendor-specific compatibility promise. Supported devices ultimately depend on the declared operating-system audio backend matrix and class-compliant or vendor-driver behavior.

## Immediate research priorities

The field study MUST now seek direct, authorized evidence for these bounded workflows:

1. **Hypnotic techno construction**
   - evolving percussion and polymetric/polyrhythmic sequencing;
   - long-form tension and variation without destructive duplication sprawl;
   - parameter modulation, automation capture, resampling, and dub-style sends;
   - arrangement navigation over repetitive material where small changes matter.
2. **Forest psytrance construction**
   - high-density bass and percussion sequencing;
   - modulation-heavy synthesis, rapid sound iteration, audio-rate/control-rate distinctions, and layered resampling;
   - CPU/latency management under dense device chains;
   - detailed automation and event editing without losing musical context.
3. **Deep dubstep construction**
   - sub/bass synthesis and controlled low-frequency monitoring;
   - groove, swing, sparse arrangement, negative space, and variation;
   - resampling, destructive-looking edits with recoverability, sends, sidechains, and spatial effects;
   - mix translation and reference/A-B workflows.
4. **Live audio-interface path**
   - select device, sample rate, buffer size, and channel layout;
   - choose physical input, create/route a track, arm, monitor, meter, and hear the live signal through the mixer;
   - insert native and VST3 effects while monitoring;
   - expose input, output, plugin, and total monitoring latency honestly;
   - record, stop, audition, preserve media, and recover from dropout, disconnect, permission failure, or disk failure;
   - distinguish direct hardware monitoring from software monitoring and prevent accidental feedback.

## Current evidence-backed implications

The three admitted FL Studio observations support only provisional questions:

- `WF-FL-ARRANGE-001` and `WF-FL-PLAYLIST-002` show that rapid duplication, bounded audition loops, variation branching, and orientation-preserving organization deserve cross-product study.
- `WF-FL-NICK-MIRA-003` shows a professional loop of sound selection, note entry, audition, transformation, arrangement, version branching, revision, and bounce. It also shows that a usable loop may be deliberately rejected in favor of further transformation.
- The professional session moves repeatedly among host, plugin editor, note editor, Playlist, mixer, slicer, save flow, and render flow. Selection, focus, audition position, and undo confidence across comparable Geist lenses therefore require direct study.
- Saving a new project version before substantial layering is observed once as revision-risk management. It is not yet a frequency or default-workflow claim.

## Candidate first-class scenarios

These scenarios are candidates for later requirements, not accepted requirements:

- create an evolving synthesized loop, branch a variation, audition it in context, and preserve both states;
- transform rendered material through slicing, pitching, effects, and resampling without losing its source state;
- arrange a long repetitive electronic track while retaining orientation and making subtle changes visible;
- connect an audio interface, route a named input through a mixer channel and effect chain, monitor it at bounded latency, record it, and recover the captured media;
- move between arrangement, device/synthesis, mixer, browser, and modulation views without losing selection, playhead context, or keyboard control.

## Prohibited conclusions at current evidence level

The current corpus does not justify:

- a final native-device list;
- a synthesis architecture or modulation limit;
- a default shortcut map;
- a gesture-count or time budget;
- a supported interface model list;
- a monitoring-latency threshold;
- platform/backend order;
- command-frequency or feature-priority scores;
- promotion of any workflow archetype.

Those decisions require broader workflow evidence, explicit architecture and platform decisions, and measured Geist prototypes.
