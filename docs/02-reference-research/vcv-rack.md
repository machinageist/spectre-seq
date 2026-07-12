<!--
Author: Jeff
Date: 2026-07-11
Description: Official-source inventory and coverage matrix for VCV Rack 2 behavioral research
Notes: User behavior and plugin-development documentation are deliberately separated
-->

# VCV Rack 2 Reference Dossier

- **Status:** draft
- **Research state:** inventory-only
- **Last verified:** 2026-07-11
- **Scope:** official VCV Rack user-manual behavior relevant to modular patching, polyphony, commands, module lifecycle, and performance workflows
- **Decision authority:** Jeff
- **Upstream sources:** `SRC-VCV-RACK2-MANUAL-INDEX`; `docs/02-reference-research/methodology.md`
- **Downstream dependents:** modular workflow field study, modular-routing requirements, graph/device architecture
- **Supersedes:** none; legacy `docs/modular_rack_spec.md` remains historical evidence pending extraction
- **Superseded by:** none
- **Open decisions:** whether selected public plugin-development contracts are useful architecture comparisons
- **Known gaps:** no point-version or manual-revision marker was exposed; chapter contents, errors, persistence, undo, feedback/cycle behavior, and real workflow evidence remain unreviewed

## Boundary

VCV Rack is a substantive modular-workflow reference, not a compatibility target. Geist will not copy Rack source code, panel art, module layouts, patch files, voltage conventions, library content, or interaction composition. Publicly documented modular behavior may motivate original Geist candidates only after provenance and product review.

## Version evidence and limitation

The official manual index was accessed on 2026-07-11. Official site navigation identifies “Rack 2,” but the manual index does not expose a point version, revision date, or immutable versioned URL. All records therefore carry a mutable-source limitation.

## User-manual coverage matrix

| Official chapter | Geist research domain | State |
|---|---|---|
| Installing & Running | platform startup, filesystem locations, lifecycle | inventory-only |
| Getting Started | empty patch to sound, add/connect/configure workflow | inventory-only |
| Menu Bar | project/patch lifecycle, settings, commands | inventory-only |
| Key Commands | semantic command and binding evidence | inventory-only |
| Core Modules | minimum modular capability taxonomy | inventory-only |
| Polyphony | signal representation, channel behavior, routing | inventory-only |
| Rack Pro Features | product-tier and plugin/host workflows requiring scope review | inventory-only |

## Separately scoped official documentation

The same index exposes plugin-development chapters for tutorial, API guide, panels, manifest, presets, voltage standards, DSP, migration, and licensing, plus Rack-development chapters for building and versioning. These are not part of the user-behavior dossier.

They MAY later serve as implementation comparisons or licensing evidence when a specific Geist architecture question requires them. They MUST NOT become Geist contracts by proximity, and they MUST NOT be used to copy implementation details.

## Required atomic observation dimensions

Future `OBS-VCV-RACK2-*` records must distinguish:

- patch document state and lifecycle;
- module creation, deletion, duplication, replacement, bypass, and reset;
- cable endpoint rules and visible connection state;
- signal/channel/polyphony behavior;
- command, pointer, and keyboard gestures;
- persistence and undo behavior;
- invalid connection and missing-module behavior;
- CPU/overload and realtime implications when publicly documented;
- performance/live interaction and recovery behavior;
- source gaps around feedback, cycles, scheduling, and internal execution.

Undocumented engine behavior remains a `SOURCE-GAP`; it MUST NOT be inferred from visual patching behavior or public source code.

## Legacy-spec correction controls

The legacy modular-rack specification mixes VCV observations, Geist limits, UI choices, and implementation details. During extraction:

1. VCV claims move only into atomic sourced observations.
2. Geist product choices move only into the requirements ledger after review.
3. Graph execution, cycle policy, port types, and realtime behavior move only into original architecture contracts.
4. Numeric limits are discarded unless independently justified as Geist requirements.
5. Terms implying VCV compatibility are removed unless Jeff explicitly creates such a target.

## Next extraction slice

Inspect “Getting Started,” “Menu Bar,” “Key Commands,” and “Polyphony” first. They cover the core patch lifecycle, command language, and signal model needed to evaluate the legacy rack claims. Add direct chapter URLs and subsection anchors to the source ledger before accepting any observation.
