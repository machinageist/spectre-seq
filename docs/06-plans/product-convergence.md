<!--
Author: Jeff
Date: 2026-09-07
Description: Proposed dependency plan from the implemented sequencer to the full Spectre product
Notes: Requires phase/release ratification; not permission to bypass existing gates or implement proposed contracts
-->

# Product convergence plan

- **Status:** proposed
- **Last verified:** 2026-09-07
- **Scope:** architecture repair prerequisites, final-product coverage and proposed dependency ordering
- **Decision authority:** Jeff
- **Upstream sources:** `../00-product/product-contract.md`, `../00-product/capability-map.md`, `../status/architecture-audit-2026-09-07.md`
- **Downstream dependents:** revised roadmap, subsystem intakes and narrow implementation plans
- **Supersedes:** none until ratified; R0–R12 remain historical/current milestone identifiers
- **Superseded by:** none
- **Open decisions:** release scope, order below, R5 media scope split and domain interview axes
- **Known gaps:** phases are intake boundaries, not fully estimated or build-ready task lists

## Execution authorization — 2026-09-07

Jeff instructed: “continue building according to the blue print, blind checking for AAA quality
as you go”. R-A is the active bounded implementation slice under
`r-a-stale-publications.md`. Continue dependency-ordered delivery with behavior tests, independent
blind review, remediation and parent-run gates. This authorizes execution of grounded slices,
not silent answers to the remaining architectural interview questions or automatic milestone PASS.

## Immediate corrective prerequisites

The earliest work is not a new canvas or a MIDI effect. The audit found existing production
paths that undermine project identity and output. Retain the R4 operator obligation; no green
fixture suite substitutes for hearing and operating the current binary.

Proposed independently reviewable repair slices, with reproductions before implementation:

1. **R-A: reject or identity-bind stale publications.** Reorder/delete/change an instrument or
   insert while the old plan runs, then change a parameter, gain and clip schedule. A command
   must reach the intended existing instance or be visibly refused, never reach an old positional
   neighbor. Pin project/plan generation as well as identity. Do not improvise hot swap.
2. **R-B: authoritative device instances.** Define a track-owned complete device parameter/state
   model, stable instance/parameter identity, command history and schema migration before wider
   routing. Give two tracks different sounds and repeated same-kind inserts; prove switching
   focus, rebuilding, undoing and reloading preserve each sound. Remove the global fixture list
   as a competing production sound authority; preserve necessary fixture tests explicitly.
3. **R-C: lifecycle reconciliation.** Define Open/rebuild/Stop/loop semantics. Replacing a project
   must not leave the previous project sounding; UI and render transport agree after restart.
   Never-saved-project recovery and persisted loop/meter state need explicit policy and tests.
4. **R-D: actual project bounce.** Export an owned authoritative project snapshot, tempo/events,
   tracks, full device states and routing through the common compiled processing path. Test two
   distinct musician-authored projects, mute, changed notes, non-default sound and reload. Compare
   generated WAVE samples with the corresponding live/null path under declared geometry. Wire
   real comparison or remove its promise; logging hashes alone is not comparison.
5. **R-E: current truth and recovery closure.** Reconcile misleading status/ledger claims, test
   bake failures as visible refusals, and reproduce standalone-sidecar recovery behavior. Do not
   call recovery qualified beyond the actual creation/open path exercised.

R-A can start with a narrow refusal policy after approval. R-B is the core model dependency for
R-D and downstream devices/automation. R-C policy and R-E diagnostics can be specified alongside
R-B, but changes to the same lifecycle/state code should remain serial. No DSP algorithm changes
are required merely to establish these facts.

## Ratified sequencing amendment — flagship/modular first

Jeff clarified after this draft: “Flagship and modularity first. They are inseperable and
essential vertical slices”. P6 and P8 below are capability packages, NOT late independent
milestones. Their shared model/DSP/patch/state contracts must be designed together and their
first usable slices delivered together after the necessary P0/P1 and narrowly scoped P3
foundations. Do not require all P4/P5/P7 features before this identity work begins.

I-04 fixes the integration boundary: the flagship is a complete device patchable alongside other
modules, not an editable internal patch with a synth skin. Proposed leading vertical slice:
author and play an original flagship sound, route the device with real native modules and expose
its agreed modulation/parameter boundary, preserve independent instance state through undo and
reload, and export the same project sound. Wavetable depth and module breadth expand in subsequent coupled
slices; “inseparable” is not permission for an unbounded all-at-once implementation.

## Proposed phase map

These are dependency packages, not a mandated waterfall or a replacement release definition.
Infrastructure must be justified by a thin usable vertical slice. Parallelize only independent
contracts/crates after shared state and processing boundaries have been accepted.

| Phase | Outcome and boundary | Dependencies | Demonstrable exit |
|---|---|---|---|
| P0 — Product truth and state repair | R-A through R-E; authoritative instances, real export, lifecycle/recovery safety | Current foundations; required ownership/policy decisions | Different authored projects retain and render their own complete sound across edit/restart/reload/export; no stale misrouting |
| P1 — UI foundation evaluation | Custom surfaces using existing infrastructure; command/focus, text, accessibility, high-DPI and window lifecycle | I-01; retain current shell for qualification | Compare viable stacks with an actual editable clip/device slice, keyboard and accessibility checks, resize and platform evidence; ratify stack before migrating broadly |
| P2 — Linked custom workspace | Shared selection and viewport context, arrangement/device/browser/mixer surface infrastructure | P0/P1; domain contracts before dependent canvases | One real edit survives lens change, undo/reload and reaches live/output; keyboard and accessibility equivalents |
| P3 — Routing, parameter and media foundations | Track/bus/send/return/sidechain/monitoring model; latency accounting; MIDI I/O; typed media references; stable automation/modulation bindings | P0; routing, note, media and parameter contracts | Small real routing/input/media/parameter lifecycles with compensation, persistence, recovery and RT tests; no generic infrastructure-only exit |
| P4 — Complete-track authoring and capture | Piano roll, audio/MIDI editing/recording, sampler baseline, tempo/meter, groove, stretch/fades/comping per source intake | P2/P3; media safety accompanies first media producer | Compose and record a defined complete-track workload, undo/reopen, recover interrupted capture and export correct audio |
| P5 — Session and live performance | Shared clip/slot/scene ownership, quantized launch, precedence, follow actions, controller mappings and arrangement capture | P2/P3; explicit launcher arbitration; sufficient P4 capture | Scripted live performance captures and replays the same musical decisions, including stops/overrides and overload/refusal cases |
| P6 — Flagship instruments and sound design | Wavetable engine/editor, filters, envelopes/LFOs, expressive polyphony, macros, sampler/granular depth, presets/browser | P0/P3 parameter/state; P2 surfaces; P4 media where needed | Independent instrument instances, deep patches and modulations survive undo/reload/migration and render live/offline within stated DSP budgets |
| P7 — Mixing, mastering and effects catalog | Full source native-effects catalog, creative effects, meters, oversampling, mastering/export formats | P3 routing/latency/sidechains; P6 shared DSP where justified | Each processor has real sound/quality/latency/state tests; measured mix/master/export workflows, not labels or a single catalog-wide PASS |
| P8 — Modular environment and SDK | Typed signals, graph nesting, explicit feedback, public/internal module boundary, patch UI and module collection | P3 typed routing/parameters; P6/P7 useful shared DSP; accepted feedback and SDK decisions | Build/save/reload/automate a real modular instrument/effect in the same DAW processing system; extension and RT/resource tests |
| P9 — Hosted plugin ecosystems | VST3 first by current default, CLAP/AU destination, conditional AAX evaluation; scan/isolation/state/editor/latency/missing-plugin recovery | P0/P3 state/routing/latency; fresh license/security decisions | Real fixture/plugin matrix across supported platforms, crash containment, reopen and export; no untested format claims |
| P10 — Release qualification | Workload budgets, soaks, portability, compatibility, installers, documentation and remaining accessibility audit | Release-specific capability set from P0–P9 | Published reproducible gates on release platforms and operator musical workflows; honest outstanding limitations |

Safety, automation identity, undo, serialization, accessibility architecture and profiling begin
in the relevant foundational slice, not at P10. P9 may move earlier if first usable release needs
external instruments/effects. I-03 now places coupled P6/P8 vertical slices early; P5 timing and
the rest of the first-release boundary remain open. No ordering
here silently defers required source capabilities out of the final product.

## R0–R12 reconciliation needed before execution

- Preserve R0–R3 evidence and R4's pending operator protocol; re-run affected product rows.
- R5 missing-media diagnostics depend on unfinished engineering, not external/operator evidence.
  Its carry is not authorized by the literal concurrent-milestone rule. Proposed resolution:
  split project-document safety from media safety, with the latter mandatory at the first real
  media-reference producer (sampling or recording). Jeff must ratify the split/reorder.
- Insert the UI evaluation/migration path rather than hiding it inside R6 or reusing the shell
  as the assumed final UI. Avoid changing the DSP engine solely to select a renderer.
- Bring device/automation identity contracts ahead of the canvases and devices that bind them;
  implementation of full automation can still proceed in bounded later slices.
- Reconcile decision 24 with actual schema types: current audio-effect enums do not establish
  that pre-instrument note-effect state fits schema 3 without migration.
- Expand R11 into explicit catalog/SDK/instrument milestones. A flagship plus modular system,
  deep MIDI and the full effect catalog is not one narrow implementation checkpoint.
- Separate final-product destination, musician beta, 1.0 and professional qualification. The
  source's suggested phases do not automatically overwrite existing release/platform obligations.

## Intake and closure template

For each capability-map row: source section/bullets → accepted behavior → stable ownership and
identity → persistence/migration → command/undo → live/control/offline projection → UI/accessibility
→ diagnostics/recovery → deterministic/RT/quality tests → operator protocol → documentation.

Name exclusions and deferred subfeatures individually. Require production-path tests and negative
controls; a function called only by tests is not a shipped capability. A representative end-to-end
case does not close unrelated catalog requirements. Independent review and parent-run validation
are required before a slice is marked complete.
