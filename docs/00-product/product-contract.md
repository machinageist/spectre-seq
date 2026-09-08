<!--
Author: Jeff
Date: 2026-09-07
Description: Living final-product contract and ratified interview decisions for Spectre
Notes: Separates accepted destination from proposed architecture and release sequencing
-->

# Spectre final-product contract

- **Status:** accepted (scope and interview decisions below)
- **Last verified:** 2026-09-07
- **Scope:** final-product identity, capability authority, and decision provenance
- **Decision authority:** Jeff
- **Upstream sources:** `vision.md`, `final-product-source.md`, direct 2026-09-07 interview
- **Downstream dependents:** `capability-map.md`, requirements, architecture, roadmap and handoffs
- **Supersedes:** the final-product scope restriction to VST3 alone; the measured-egui-limit condition on custom surfaces; neither change authorizes immediate implementation
- **Superseded by:** none
- **Open decisions:** first usable release scope; revised milestone order; exact UI stack; behavioral decisions enumerated below
- **Known gaps:** the supplied source is a capability specification, not a complete build-ready contract

## Accepted destination

Spectre is one coherent, professional cross-platform DAW with an embedded flagship synthesis
and modular environment. Composition, recording, arrangement, performance, sound design, mixing,
mastering and export operate on one project. Native instruments, processors, modular patches
and hosted plugins participate in compatible processing, parameter and state contracts rather
than separate incompatible engines.

The required capabilities of `final-product-source.md` are the final-product scope. Its optional,
eventual and conditional clauses retain their modality. `capability-map.md` accounts for every
numbered source section; it is an intake index, not a substitute for the detailed source.
Reference products describe capability expectations, not compatibility requirements, copied DSP,
brand names, visual clones or mandatory algorithms. Original code/content and the established
callback-safety contract remain binding.

The existing genre/audience focus and loop-first workflow in `vision.md` remain in force.
Windows/macOS/Linux are destination platforms. The existing platform-order decision remains in
force until explicitly amended; this contract does not newly qualify any platform.

## Ratified interview decisions

### I-01 — Custom musician-facing surfaces

Jeff selected: “Custom DAW surfaces; reuse platform/rendering infrastructure.”

Build original musician-facing editing and device surfaces while reusing windowing, rendering,
text and accessibility infrastructure. This does not select a toolkit, prohibit egui as part of
an evaluated implementation, or commission a bespoke operating-system/UI foundation. Decision 8
records the amendment. Accessibility and keyboard completeness must inform the selection.

### I-02 — Full destination, phased delivery

Jeff selected: “Yes—final-product scope, phased delivery, review technical prescriptions.”

Do not silently trim the supplied specification to the current prototype. Do not interpret its
entire catalog as the first release. Review numerical targets, illustrative APIs, algorithm
choices and suggested phases separately. No response was received to the earlier first-release
question; its scope remains unresolved.

### I-03 — Flagship synthesis and modularity are inseparable

Jeff answered the first-usable sound-generation question: “Flagship and modularity first.
They are inseperable and essential vertical slices”.

The flagship and modular environment must therefore be developed as coupled, essential vertical
slices in the early product, not as a basic-synth release followed by independent late identity
milestones. Their exact shared DSP/patching boundary is the next interview decision. This does
not imply the entire final synth/module catalog must land in one slice, and does not settle
recording, launcher, plugin or release-platform scope.

The existing vision's release table and roadmap's late R11 identity placement are superseded on
this sequencing axis. The current small instruments remain test/prototype foundations, not a
substitute for the accepted early flagship/modular requirement.

### I-04 — Flagship is a complete patchable device

Jeff selected: “Flagship is a complete device patchable alongside other modules”.

The flagship has its own complete instrument identity and focused interface, and participates
as a device in the modular environment alongside other modules. Do not expose its internals as
an editable modular patch by default or make that an acceptance requirement. Shared internal DSP
is an implementation option, not a requirement that the flagship preset be a user-editable graph.
I-03 still binds: instrument and modular integration are developed together in early vertical slices.

Its public audio/note/control/modulation ports, per-voice expression boundary, nesting and
parameter exposure remain to be specified. This choice does not decide those interfaces silently.

### Explicit scope amendments

- CLAP and AU are now final-product scope alongside VST3, subject to platform relevance and
  binding/license/security evaluation. AAX remains conditional on licensing/business need.
- This expands the destination, not current implementation scope. VST3-first staging remains
  the existing default pending a revised roadmap decision.
- `vision.md`'s current-stage plugin exclusions are not permanent final-product exclusions.
- Neither source phase 8 nor its weaker “where practical” language postpones or weakens the
  already accepted project-safety, realtime or before-beta accessibility obligations.

## Product invariants carried into every subsystem

1. One authoritative project state, stable object/parameter identities, and linked view context.
2. Device instances retain independent complete sound state through edits, undo, autosave,
   save/reload and migration; view selection must not determine rendered sound.
3. Live playback and offline export consume equivalent project state and musical events under
   documented rendering conditions. Fixture export is never presented as project export.
4. A user operation is not complete merely because a model helper or a widget exists. Verify the
   production UI/command → model → processing → persistence/reload path, undo and automation
   where applicable, realtime behavior, diagnostics, tests and documentation.
5. Interim slices may defer dependent capabilities explicitly; they must not label a final-product
   feature complete while its stated acceptance conditions remain deferred.
6. No fake controls, fabricated telemetry or hidden destructive fallback. Refusal is visible and
   recoverable; loss of work is a defect, not a performance trade-off.

## Technical prescriptions awaiting review, not newly accepted contracts

- “Everything is a node” means composable processing; aggregate ownership and nested graphs
  need a design rather than forcing tracks, oscillators and projects into one concrete type.
- “Unlimited” needs resource-based capacity and explicit bounded callback work, admission rules,
  resource diagnostics and stress workloads. Existing hard limits cannot disappear by prose.
- Buffer/voice/track/macro/sample-rate targets need independent rationale and qualified workloads.
- Overload behavior needs a deterministic failure policy; glitch-free execution cannot be promised
  for arbitrary excess load.
- Sample-accurate automation, control-rate modulation, audio-rate modulation and timing of
  structural/meter changes need separate contracts.
- Exact restoration must specify version/plugin/media availability, non-deterministic state and
  missing-dependency recovery; cross-machine bit identity is not implied.
- Full undo must define exclusions for irreversible external effects such as recording files,
  external MIDI/hardware output and destructive operations.

## Next interview axes

Resolve one consequential axis at a time, grounded in implementation evidence:

1. First personally usable complete-track release: required instruments, sampling/recording,
   launcher, automation/modulation, plugins and platforms; then map beta/1.0 separately.
2. Device ownership and editing: per-instance state, nested racks, sharing versus copy semantics,
   preset application and stable automation targets.
3. Session/arrangement ownership and arbitration: shared or copied clips, precedence, launch/stop,
   return-to-arrangement and performance capture.
4. MIDI/note-routing expressivity: generated identity/release/order, held state versus feedback,
   pre-instrument processing, MPE/CC and external I/O.
5. Modulation/automation composition: base value, automation, modulation, overrides and restore.
6. Media/recording lifecycle: first media producer, portability, repair, salvage and standalone
   unsaved-project recovery; loop/meter persistence and undo semantics.
7. Modular and plugin extensibility: internal/public SDK stability, nesting, feedback delays,
   resource bounds, supported formats and crash isolation.
8. UI/workflow and qualification: linked selection/zoom, command dispatch, detached windows,
   accessibility, platform gates and measured budgets.

Recommendations in the convergence plan are not accepted answers to these questions.
