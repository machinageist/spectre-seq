<!--
Author: Jeff
Date: 2026-07-11
Description: Semantic command ontology for cross-product workflow research and future Geist command design
Notes: Command identifiers are provisional research normalization, not accepted Geist API
-->

# Command Ontology

- **Status:** draft
- **Last verified:** 2026-07-11
- **Scope:** product-independent semantic command normalization for workflow, shortcut, gesture, focus, safety, and accessibility research
- **Decision authority:** Jeff
- **Upstream sources:** `docs/02-reference-research/workflow-field-study/methodology.md`; `workflow-corpus.md`; `workflow-archetypes.md`; reviewed `workflow-observations.jsonl`; `shortcut-action-map.csv`
- **Downstream dependents:** shortcut analysis, friction analysis, product implications, UI command architecture, command requirements
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** final Geist command namespaces, typed payload model, alias semantics, and view-state command boundary
- **Known gaps:** no reviewed command observations, no verified cross-product binding corpus, no frequency evidence, no accepted Geist command IDs

## Authority warning

Identifiers in this document are provisional research-normalization IDs. They allow actions described by different products to be compared without copying product action IDs or bindings. They are not accepted Geist API names, requirements, default shortcuts, or implementation decisions.

## Normalization rules

A semantic command ID:

1. names user intent rather than a key, menu path, action number, or widget;
2. distinguishes materially different state transitions;
3. omits a surface name when semantics are genuinely shared;
4. includes context when the same phrase has different state or safety consequences;
5. separates selection mutation from object mutation;
6. separates project-model commands from transient view state;
7. identifies payload shape when evidence requires one;
8. does not encode another product's terminology solely for familiarity;
9. does not imply support merely by existing in this registry;
10. receives a final Geist identifier only through requirements and architecture review.

## Command record model

Each normalized command requires these research fields before it can be evaluated:

| Field | Meaning |
|---|---|
| Semantic ID | product-independent action identity |
| Intent | concise user goal |
| Object classes | project, range, track, clip, note, device, route, parameter, asset, take, marker, view, etc. |
| Preconditions | required selection, focus, transport, mode, and object state |
| Typed payload | values or targets required to execute the action |
| State transition | authoritative model mutation or transient view mutation |
| Context scope | global, project, lens, editor, plugin-window, text-entry, musical-typing, modal |
| Selection effect | preserve, replace, extend, clear, derive, or unknown |
| Focus effect | preserve, transfer, return, or unknown |
| Transport/audition effect | preserve, start, stop, seek, retrigger, or unknown |
| Persistence | saved project, saved configuration, session-only, transient, or unknown |
| Undo class | project undo, view undo/history, non-undoable, confirmation-gated, or unknown |
| Safety class | routine, disruptive, destructive, irreversible, external-side-effect |
| Realtime interaction | none, schedule at boundary, sample-timed, control-only, prohibited in callback |
| Discoverability | menu, command search, tooltip, editor, context menu, undocumented, unknown |
| Remappability | fixed, remappable, context-remappable, scriptable, unknown |
| Alternate inputs | pointer, keyboard, touch, controller, accessibility action |
| Evidence | source IDs, timestamps/pages, product/platform/version |
| Confidence | direct, corroborated, inferred, conflicting |

## Provisional namespace registry

| Namespace | Object or intent family | Examples | Authority state |
|---|---|---|---|
| `app.*` | application lifecycle and global surfaces | `app.open_preferences`, `app.quit` | research-only |
| `project.*` | project creation, persistence, recovery, versions | `project.save`, `project.save_new_version`, `project.recover_autosave` | research-only |
| `edit.*` | generic model editing and transaction control | `edit.undo`, `edit.redo`, `edit.duplicate_selection`, `edit.delete_selection` | research-only |
| `selection.*` | selection ownership and mutation | `selection.select_all_context`, `selection.clear`, `selection.invert` | research-only |
| `focus.*` | keyboard/accessibility focus transitions | `focus.next_region`, `focus.return_to_host` | research-only |
| `transport.*` | play state and transport commands | `transport.play_stop`, `transport.record`, `transport.seek` | research-only |
| `time.*` | time ranges, loop, markers, tempo/meter edits | `time.set_loop_from_selection`, `time.duplicate_range`, `time.insert_marker` | research-only |
| `track.*` | track creation, state, ordering, grouping | `track.insert_audio`, `track.arm`, `track.group_selection` | research-only |
| `route.*` | sends, receives, buses, sidechains, monitoring | `route.create_send`, `route.set_sidechain_source` | research-only |
| `clip.*` | arrangement/launcher clip lifecycle | `clip.insert_midi`, `clip.launch`, `clip.consolidate` | research-only |
| `scene.*` | scene/performance-row lifecycle | `scene.launch`, `scene.capture`, `scene.duplicate` | research-only |
| `take.*` | takes, lanes, alternatives, comp selection | `take.next`, `take.promote_range`, `take.keep_alternate` | research-only |
| `notes.*` | note/event editing and expression | `notes.quantize`, `notes.transpose`, `notes.set_velocity` | research-only |
| `audio.*` | audio-event editing and transforms | `audio.split`, `audio.reverse`, `audio.set_fade` | research-only |
| `automation.*` | automation creation, editing, modes | `automation.toggle_view`, `automation.capture`, `automation.reenable` | research-only |
| `modulation.*` | modulation assignment and depth | `modulation.assign`, `modulation.set_depth`, `modulation.remove` | research-only |
| `device.*` | native/VST wrapper chain and lifecycle actions | `device.insert`, `device.bypass`, `device.open_editor` | research-only |
| `parameter.*` | explicit parameter value operations | `parameter.set_value`, `parameter.reset_default`, `parameter.enter_value` | research-only |
| `browser.*` | search, preview, favorites, insertion | `browser.focus_search`, `browser.preview`, `browser.insert_selected` | research-only |
| `asset.*` | media reference, collection, repair | `asset.relink`, `asset.collect`, `asset.reveal_source` | research-only |
| `recording.*` | recording setup and captured-media lifecycle | `recording.set_input`, `recording.monitor`, `recording.punch` | research-only |
| `render.*` | export, bounce, freeze, stems, archive | `render.mix`, `render.stems`, `render.freeze_selection` | research-only |
| `view.*` | transient lens, panel, zoom, scroll state | `view.zoom_to_selection`, `view.restore_zoom`, `view.toggle_mixer` | research-only |
| `workspace.*` | saved layouts/workflow profiles | `workspace.recall`, `workspace.save` | research-only |
| `command.*` | command discovery, remapping, aliases | `command.search`, `command.bind`, `command.run_alias` | research-only |
| `accessibility.*` | explicit accessibility actions/settings | `accessibility.announce_selection`, `accessibility.toggle_reduced_motion` | research-only |
| `controller.*` | controller mappings and takeover behavior | `controller.map_parameter`, `controller.clear_mapping` | research-only |

## Initial semantic registry

These entries seed normalization for the declared archetypes. Their presence does not claim observed frequency or product convergence.

| Semantic ID | Intent | Primary objects | Mutation class | Safety/undo expectation | Evidence state |
|---|---|---|---|---|---|
| `project.new` | create an empty project | project | project model | routine; project-history boundary | unobserved |
| `project.open` | open a selected project | project/file | application/project | disruptive; confirmation if dirty | unobserved |
| `project.save` | persist current project revision | project | external side effect | non-undoable; failure must surface | unobserved |
| `project.save_new_version` | persist a distinct revision | project | external side effect | non-undoable; preserve current revision | observed in one professional FL session; no frequency claim |
| `project.recover_autosave` | restore a recoverable revision | recovery record | project replacement | disruptive; preview/confirmation expected | unobserved |
| `edit.undo` | reverse latest eligible transaction | transaction | project model | routine; itself redoable | unobserved |
| `edit.redo` | reapply latest reverted transaction | transaction | project model | routine; undoable | unobserved |
| `edit.duplicate_selection` | create a copy of selected objects | contextual selection | project model | routine; undoable | corroborated by official and independent FL tutorials; no frequency claim |
| `edit.delete_selection` | remove selected objects | contextual selection | project model | destructive; undoable | unobserved |
| `transport.play_stop` | toggle playback state | transport | engine/control | routine; not project undo | observed in one official recording tutorial; no frequency claim |
| `transport.record` | enter or leave record capture | transport/armed targets | engine + media side effect | disruptive; captured media retained/recoverable | unobserved |
| `transport.seek` | move transport position | timeline position | engine/control | routine; not project undo | unobserved |
| `time.set_loop_from_selection` | derive loop range from time selection | time range | project or session control | routine; undo policy unresolved | corroborated by official and independent FL tutorials with different gestures; no frequency claim |
| `time.duplicate_range` | copy material and time structure across a range | time range + intersecting objects | project model | routine; undoable | unobserved |
| `track.insert_audio` | create an audio-capable track | project/track list | project model | routine; undoable | unobserved |
| `track.insert_instrument` | create a note/instrument track path | project/track list | project model | routine; undoable | unobserved |
| `track.arm` | enable target for recording | track | project/session control | disruptive; visible and safe | unobserved |
| `route.create_send` | connect source to destination with send semantics | tracks/buses | graph/project | potentially disruptive; undoable | unobserved |
| `route.set_sidechain_source` | route a source into a device sidechain | source/device bus | graph/project | potentially disruptive; undoable | unobserved |
| `clip.insert_midi` | create a note clip at a target location | track/time/slot | project model | routine; undoable | unobserved |
| `clip.launch` | request launcher playback | clip/slot | scheduled engine control | performance-sensitive; not ordinary undo | unobserved |
| `clip.consolidate` | replace selected material with a consolidated object | clips/time range | project + possible media | destructive-looking; undo/recovery required | unobserved |
| `take.promote_range` | choose a take segment for the active comp | take lane/time range | project model | routine; undoable | unobserved |
| `notes.quantize` | move selected note timing toward a grid/rule | notes | project model | routine; undoable; amount explicit | unobserved |
| `notes.transpose` | move selected notes by interval | notes | project model | routine; undoable | unobserved |
| `audio.split` | divide an audio event at explicit boundaries | audio event/time | project model | routine; undoable | unobserved |
| `automation.capture` | record parameter motion as automation | parameter/time | scheduled project capture | disruptive; undoable capture transaction | unobserved |
| `automation.reenable` | return parameter control to automation | parameter | engine/project control | routine; state visible | unobserved |
| `modulation.assign` | connect a modulation source to a destination | modulator/parameter | project model | routine; undoable | unobserved |
| `device.insert` | add a device to a chain/graph location | device descriptor/target | project/graph | routine; undoable; failures contained | unobserved |
| `device.bypass` | bypass processing while retaining state | device | project or control | routine; automation semantics required | unobserved |
| `device.open_editor` | show a native or hosted editor | device/window | transient UI | no project undo; focus rules required | unobserved |
| `parameter.reset_default` | restore a parameter's declared default | parameter | project model | routine; undoable when persistent | unobserved |
| `browser.focus_search` | move focus to asset/device search | browser | transient UI | no project undo; return-focus required | unobserved |
| `browser.preview` | audition selected content | asset | transient audio/UI | no project undo; audition context explicit | unobserved |
| `browser.insert_selected` | insert selected browser item at destination | asset/device + destination | project model | routine; undoable | unobserved |
| `asset.relink` | repair a missing-media reference | asset reference/file | project model + external read | consequential; undoable mapping | unobserved |
| `render.mix` | produce a mix deliverable | project/range/settings | external side effect | non-undoable; cancel/failure/report required | unobserved |
| `render.freeze_selection` | create reversible rendered substitutes | tracks/devices | project + media | consequential; reversible/undoable | unobserved |
| `view.zoom_to_selection` | frame selected objects or time | view/selection | transient UI | view history, not project undo | unobserved |
| `view.restore_zoom` | return to prior zoom state | view history | transient UI | reversible view action | unobserved |
| `command.search` | find an available command by intent/name | command registry | transient UI | no project undo; context filtering | unobserved |
| `command.bind` | assign an input gesture to a command | command/configuration | configuration | conflict validation; reversible | unobserved |
| `command.run_alias` | execute a validated sequence of typed commands | command alias | depends on members | atomicity/rollback must be explicit | unobserved |

## Selection and focus invariants under study

The corpus must test, rather than assume, whether successful workflows require:

- one authoritative contextual selection shared across lenses;
- independent time and object selections with explicit interaction;
- selection preservation when changing lenses;
- deterministic focus return after command search, dialogs, browser insertion, and plugin editors;
- global transport commands that remain available during non-destructive text/plugin focus;
- suppression of editing commands during text entry;
- explicit musical-typing mode with visible keyboard capture;
- commands disabled with a reason when their preconditions are unmet;
- command search filtered by context without hiding discoverability.

These remain `GEIST-CANDIDATE` questions, not requirements.

## Alias and macro research boundary

Cross-product research distinguishes:

- remapping one gesture to one typed command;
- declarative aliases composed only from registered typed commands;
- parameterized aliases with validated values and object selectors;
- product macros with unclear transaction boundaries;
- arbitrary executable scripts or extensions.

Geist's standing direction allows validated declarative aliases but does not authorize arbitrary code execution. Final alias semantics require architecture for validation, preconditions, atomicity, undo grouping, failure, realtime scheduling, versioning, and configuration portability.

## Core-loop scoring gate

No command has a core-loop score. Scoring begins only after reviewed observations exist. Each command will be assessed qualitatively across:

- observed repetition;
- number of supported archetypes;
- time sensitivity;
- menu-access cost;
- error and recovery impact;
- keyboard/controller need;
- cross-product convergence;
- accessibility importance.

Scores MUST retain source links and confidence and MUST NOT be represented as population statistics.
