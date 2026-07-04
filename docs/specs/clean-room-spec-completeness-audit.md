<!--
Author: Jeff
Date: 2026-07-03
Description: Completion audit for clean-room manual specs before implementation planning
Notes: Tracks whether the manual-derived specs are exhaustive enough to drive code changes
-->

# Clean-Room Spec Completeness Audit

## Purpose

This audit prevents implementation work from starting from underspecified manual summaries. Jeff's requirement is that every feature in every public manual/source family be specified thoroughly enough to drive clean-room Geist design decisions.

## Completion standard

Each spec must include:

1. Public-source provenance with every manual/source page used.
2. A feature-by-feature coverage matrix tied to the manual/source sections.
3. Observable behavior for each feature.
4. Data model implications for Geist project/session/preset/device state.
5. Realtime implications for graph scheduling, callback safety, bounded buffers, and offline render.
6. UI/command implications for undoable edits and view/project separation.
7. Geist mapping notes naming likely crates/files where known.
8. Explicit clean-room non-goals: vendor UI, code, assets, presets, algorithms, screenshots, private formats.
9. Concrete gap notes. Source-bound gaps are acceptable only when the public docs do not specify private algorithms, schemas, exact defaults, or vendor content.

## Final audit state

| Spec | State | Reason |
|---|---|---|
| `docs/specs/ableton-live-clean-room-spec.md` | ready with source-bound exclusions | Covers core Live manual behavior plus an added feature inventory for settings/browser/file management/MPE/audio-to-MIDI/stems/grooves/tuning/comping/devices/racks. Remaining items are accepted non-goals or public-doc algorithm/default limits. |
| `docs/modular_rack_spec.md` | ready with source-bound exclusions | Expanded to feature-by-feature VCV manual coverage across all requested public pages. Remaining gaps are exact `.vcv`/`.vcvm` schema, undocumented MenuBar settings, DAW-specific edge cases, Pro module specifics, and third-party behavior. |
| `docs/specs/serum-2-clean-room-spec.md` | ready with source-bound exclusions | Expanded to exhaustive public Serum 2 product/PDF/support coverage. Remaining gaps are exact parameter ranges, full hidden lists, DSP algorithms, schemas, MPE/voicing details, and content files not present in public docs. |
| `docs/specs/geist-modular-synth-spec.md` | ready with source-bound exclusions | Expanded to Phase Plant/Kilohearts feature coverage across generators, lanes, unison, modulation, curves/remaps, assets, presets, and warnings. Remaining gaps are exact numeric ranges/algorithms and factory content. |
| `docs/specs/bitwig-studio-clean-room-spec.md` | ready with source-bound exclusions | Rewritten after two Bitwig subagent timeouts. Covers the public user-guide TOC, dashboard/settings, arranger, launcher, mixer, devices, automation, audio/note events, operators, export, controllers, nesting, Grid, tablet profile, and device-description scope. |

## Deterministic audit metrics

Generated 2026-07-03 with `docs/specs/clean-room-spec-audit-metrics.json` as the machine-readable snapshot.

| Spec | Lines | Headings | Source URLs | Needs-attention markers |
|---|---:|---:|---:|---|
| `ableton-live-clean-room-spec.md` | 626 | 48 | 19 | `gap`: 1; accepted source-bound gap section; no `partial` markers |
| `modular_rack_spec.md` | 1263 | 93 | 15 | `partial`: 5 and `not fully`: 1 occur in explanatory text or exact source-bound gap reasons; no vague partial rows |
| `serum-2-clean-room-spec.md` | 1228 | 55 | 11 | `gap`: 24; exact public-doc limitations, no `partial` markers |
| `geist-modular-synth-spec.md` | 733 | 68 | 8 | `partial`: 8 in public feature wording/examples or exact gaps; needs no follow-up before planning |
| `bitwig-studio-clean-room-spec.md` | 713 | 127 | 22 | `gap`: 1; exact clean-room/source limitation section; no `partial` markers |

## Code implementation gate

The clean-room documentation gate is satisfied for planning work. Code implementation may proceed from the improvement plan, provided each implementation task cites the relevant spec section and does not attempt vendor UI/content/algorithm/file-format cloning.
