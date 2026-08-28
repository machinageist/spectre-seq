<!--
Author: Jeff
Date: 2026-08-15
Description: R4-7 spec — implement CORE-004 atomic filesystem save/reload over the accepted persistence contract and land CORE-001's reorder evidence on the first persisted collection
Notes: The persistence API is already accepted in docs/03-architecture/project-persistence.md; this spec implements
  it and does not redesign it. Crash-durability evidence, journaled autosave, recovery, and migrations are R5 and
  are deferred explicitly. Iteration 3 re-baselines every claim onto the committed HEAD 1960a8b, which is
  8b1633d plus four further slices (R4-2 at 20f3056, R4-1 remediation at ab521a9, R4-4 at 41a6be2, R4-6 at
  0d2a03c and 1960a8b). Three iteration-2 statements about the tree are now false and are corrected here rather
  than carried: crates/spectre-app no longer merely declares spectre-project, it uses its track model and graph
  builder in lib.rs, main.rs, and engine.rs; AppModel's own TrackView is gone and its tracks field is now a
  spectre_project::TrackList that already derives Serialize and Deserialize; and AppModel::prototype builds five
  device instances carrying eleven parameter values, not three carrying four. What has not changed is the gap
  this slice closes: no persistence symbol from spectre-project — ProjectEnvelope, ProjectDoc, to_bytes,
  from_bytes, SCHEMA_VERSION, EditHistory — appears anywhere under crates/spectre-app, and no file in
  spectre-project names std::fs or std::path. This spec makes no claim about whether ./spectre produces sound;
  that is R4-1's line to hold, and STATUS holds it as implemented rather than verified. Because two other slices
  are editing crates/spectre-project and crates/spectre-app concurrently, every citation into those two crates is
  by symbol name rather than by line number; line pins are kept only where the file is stable and each one was
  re-opened at 1960a8b.
-->

# Spec: Project Persistence

**Feature ID:** `R4-7` (`project-persistence`)
**Parent feature:** `R4` Credible Alpha (root)
**Spec author agent:** gauntlet spec agent, R4-7 leaf
**Date:** 2026-08-15
**Iteration:** 3

- **Status:** proposed
- **Last verified:** 2026-08-25 at the committed HEAD `1960a8b` (iteration 3; every claim re-opened from source at that commit, none carried over from iteration 2). Iteration 2's baseline was `8b1633d`; `20f3056` (R4-2), `ab521a9` (R4-1 remediation), `41a6be2` (R4-4) and `0d2a03c`/`1960a8b` (R4-6) landed in between, and R4-4 in particular rewrote `crates/spectre-app/src/lib.rs`, `main.rs`, and `engine.rs` and added `crates/spectre-project/src/track.rs` and `routing.rs`. **The baseline is the committed commit, not the working tree:** other slices are editing `crates/spectre-project` and `crates/spectre-app` right now, so anything read from the tree would rot before this document is graded. Every citation into `crates/spectre-app` and into the three `crates/spectre-project` source modules those slices are editing is therefore **by symbol name**, which survives a shift; line pins are kept only for `crates/spectre-core`, `crates/spectre-dsp`, `crates/spectre-offline`, `crates/spectre-project/src/command.rs` and `tests/`, the checked-in fixture, and `docs/`, and each surviving pin was re-opened at `1960a8b`. §7.1 states the whole baseline.
- **Scope:** app-thread `load_project` / `save_project_atomic` on the real filesystem, the reusable semantic validator both call, the first persisted object collection, and the app-layer save/open surface in `./spectre`
- **Decision authority:** Jeff
- **Upstream sources:** `docs/03-architecture/project-persistence.md` (design authority, accepted for R4/R5 implementation), `docs/01-requirements/requirements-ledger.md` (CORE-001 `:46`, CORE-003 `:48`, CORE-004 `:49`, RT-001..003 `:28–30`, PROD-003), `docs/01-requirements/decision-gates.md` (rows 1 `:25`, 3 `:27`, 4 `:28`, 8 `:32`, 13 `:37`, 14 `:38`, 16 `:40`, 17 `:41`, 23 `:49`), `docs/00-product/vision.md:33`, `docs/06-plans/current-milestone.md` §"Inherited debt" items 3–4, `docs/status/NEXT.md` slice 7
- **Downstream dependents:** R4-8 (offline bounce), R4-9 (end-to-end fixture and manual QA), and every R5 project-safety slice (journaled autosave, recovery, migrations, salvage, missing-media). **R4-4 (track model) and R4-6 (`Filament`/`Gloam`) are no longer downstream — they landed before this iteration and are now upstream constraints on it (§7.1, §7.4).** R4-5 (MIDI clips) landed its first part after this spec's baseline and left the `ProjectDoc` clip field to this slice, which §7.4 records as post-baseline drift rather than folding into §7.1
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** §8 Q1–Q14
- **Known gaps:** the accepted benchmark corpus contains **no** citable observation about how any benchmark product writes a project file, recovers from an interrupted save, bounds undo depth, or persists view state. The seven adjacent records that do exist are cited in Appendix A — which is the only place in this spec any `OBS-` ID appears — and they inform §3.6's failure messages and §4.2's non-finite rule rather than being quoted into them. The hole is named there rather than filled in. Crash-durability evidence is R5 by the design authority's own milestone table and is not claimed here.

This spec is subordinate to the conflict precedence in `docs/README.md`. It implements an
accepted architecture contract; it does not amend one. Two places where it touches
accepted material — a schema-version bump and the fate of one `verified` CORE-003
acceptance test — are flagged in §7.2 and routed to §8 (Q1, Q2) rather than asserted.

---

## 1. Purpose

### 1.1 One-sentence job

A musician saves the project they are working on to a file they name, quits, relaunches
`./spectre`, opens that file, and gets back the same tracks in the same order with the
same identities and the same device values — and no crash, cancel, or full disk can ever
leave them with a project file that is half of one project and half of another.

### 1.2 Why it matters

`docs/00-product/vision.md:33` states the pillar in its own words: *"Project safety:
atomic save, autosave, recovery, missing-media repair — losing work is a
product-killing defect."* R4's release bar (`vision.md:48`) names `save/reload` as one
of the six things the credible alpha must actually do.

Right now none of it exists as a product behavior, and iteration 3 has to be careful about
why, because the reason changed under this spec. `crates/spectre-app` no longer merely
*declares* `spectre-project`: R4-4 moved the track model into that crate and R4-1/R4-4
build the render graph from it, so `spectre_project` is named in `src/lib.rs`,
`src/main.rs`, and `src/engine.rs` today. What is still absent is the whole **persistence**
surface. A grep across `crates/spectre-app/src/` and `crates/spectre-app/tests/` for
`ProjectEnvelope`, `ProjectDoc`, `to_bytes`, `from_bytes`, `SCHEMA_VERSION`, `EditHistory`,
`Transaction`, `ProjectCommand`, `std::fs`, and `std::path` returns **nothing**. `main.rs`'s
own header Notes block still says so, and names this slice while doing it: *"Persistence
wiring remains out of scope until R4-7."* There is no menu item, no path field, and no code
path in `./spectre` that writes or reads a file. Everything a musician does in the prototype
is lost when the process exits.

Below that gap the crate is not empty, and this spec must be precise about the
difference. `spectre-project` has a versioned JSON envelope, a schema gate, semantic
validation after decode, atomic in-memory command transactions and bounded undo/redo
(`src/lib.rs`, `src/command.rs`), and — since R4-4 — an ordered `TrackList` and the graph
builder that turns it into a render path (`src/track.rs`, `src/routing.rs`). **None of that
touches a filesystem.** No file in the crate names `std::fs` or `std::path` at `1960a8b`.
The word "atomic" in `command.rs:3` describes a transaction that either applies every
command or reverses the ones it applied (`Transaction::execute`, `command.rs:92–109`); it
has nothing to do with CORE-004's atomic replacement of a file. An in-memory envelope with
a validator is not an atomic save, and this spec's §7.1 does not let those two claims blur.

Two accepted obligations land here and nowhere else:

- **CORE-004** (`requirements-ledger.md:49`) is `accepted` with its API design completed
  at R1; `current-milestone.md` §"Inherited debt" item 3 assigns the filesystem
  implementation to R4 and crash qualification to R5.
- **CORE-001** (`requirements-ledger.md:46`) is `implemented` with reorder evidence
  *"gated on the first persisted object collection (R4 intake)"*. **R4-4 discharged the
  in-memory half of that clause and said so in the source:**
  `crates/spectre-project/tests/track_model.rs:4` reads *"CORE-001's reorder half is
  discharged here; its persistence half waits for R4-7"*, and
  `a_track_keeps_its_identity_and_fields_across_a_reorder` (`:27`) proves identity and every
  field survive `TrackList::reorder`. What R4-4 could **not** discharge is the words the
  ledger actually uses — *persisted* collection, *across save/load*. There is still no
  persisted collection anywhere in the repository. This spec introduces one, so it owns the
  remaining half: identity across reorder **through a real file**, and across undo after a
  reload. `NEXT.md:26` assigns it in the same words — *"persistence of the list (R4-7, which
  also closes CORE-001's other half)"*.

### 1.3 Success signal

One command produces the whole result:
`cargo test --locked -p spectre-project -p spectre-app -p spectre-offline` passes with
(a) a real-filesystem test that saves a project containing three tracks, reorders them,
saves again, reloads, and asserts each track's `ObjectId` is unchanged and the new order
is exact; (b) fault-injection unit tests at **six** of the seven `SaveStage` values —
every stage except `EncodeSnapshot`, which no validated envelope can reach and which §5.1
therefore leaves deliberately untested — each asserting the exact `target_state` and the
ordered `FaultFs` call log for that stage, together with two real-filesystem tests (I4,
I5) asserting that a pre-commit failure leaves an existing destination byte-identical to
its pre-call contents and leaves a previously absent destination absent; and (c) an
offline test asserting that `spectre_offline::render_app_snapshot` returns the **same**
`RenderReport.hash` for a model whose parameters have been **edited away from their
descriptor defaults** before and after a save/reload round trip into a **fresh** model, so
persistence is proved not to change what the project computes and a round trip that
silently dropped the devices would change the hash rather than agreeing vacuously (I16,
§5.2).

Observable in the product: with the shell change in §3, a musician types a path, presses
`Save project`, quits, relaunches, presses `Open project`, and the left panel shows the
same tracks in the same order.

---

## 2. User Stories

> As a musician who has spent an hour arranging tracks, I want to name a file and save
> the project into it, so that quitting the app is not the same as throwing the work
> away.

> As that musician the next morning, I want to open that file and get back exactly what
> I left — same tracks, same order, same names, same device values, same selected track
> — so that reopening a project is resuming work rather than reconstructing it.

> As a musician whose laptop battery dies mid-save, I want the project file on disk to be
> either entirely the old version or entirely the new one, so that a power loss costs me
> one save and never the whole project. *(R4 designs and structures for this; the
> crash evidence itself is R5 — see §4.6 and §7.4.)*

> As a musician whose disk is full, whose project folder is read-only, or whose file was
> written by a newer Spectre, I want the app to keep running, tell me which of those four
> things happened, and leave the project I have open untouched, so that a failed save is
> a message and not a lost session.

> As a musician who reorders tracks and then changes their mind, I want undo to put the
> order back with the same tracks — not copies of them — so that identity survives
> editing and the automation and routing that later attach to these IDs cannot silently
> re-target.

> As a keyboard-only or screen-reader user, I want the save path field, the save action,
> the open action, and the save-result message to be focusable, ordered, and readable as
> text, so that decision 17's beta accessibility bar (`decision-gates.md:41`) is not
> foreclosed by this slice.

> As Jeff running CI in a container, I want every persistence test to run against real
> temporary files with no audio device, no network, and no new third-party dependency,
> so that the gate stays deterministic.

---

## 3. UX Specification

### 3.1 Screen / view inventory

R4-7 introduces **no new window, modal, sheet, popover, or drawer**. A native file dialog
would require a new third-party dependency (`rfd` or equivalent) on both macOS and Linux;
that is deliberately not proposed here and is routed to §8 Q4. It modifies two existing
regions of the single-window shell in `crates/spectre-app/src/main.rs`.

| Region | Navigation path | New or modified | Layout pattern |
|---|---|---|---|
| **PROJECT** block, top of the left panel (`SpectrePrototype::track_list`) | always visible | **new block inside an existing panel** | left `SidePanel::left("tracks")` with `default_width(220.0)` and `min_width(180.0)`, vertical stack above the existing `TRACKS` label |
| Transport bar title area (`SpectrePrototype::transport`) | always visible, 62 px fixed (`.exact_height(62.0)`) | modified — one dirty-state marker beside the `SPECTRE` wordmark | full-width top panel, horizontal centered row |

The PROJECT block deliberately reuses the widget pattern already in the same panel: the
add-track control already in `track_list` is a `TextEdit::singleline` with `hint_text` next
to a small button whose failure path writes into `self.feedback_status`. The save control
is the same shape, so this adds no new interaction vocabulary.

### 3.2 Interaction flows

**Primary flow — save.**

1. The musician types or edits a path in the PROJECT path field. Empty is the launch
   state.
2. They press `Save project`. The button is disabled with a hover reason while the field
   is empty (`"Type a file path to save into."`).
3. The app builds an immutable `ProjectEnvelope` snapshot from `AppModel` on the app
   thread (§4.3, `project_envelope`). It reads only `AppModel`; it never reads engine or
   render state (contract: *"A save consumes an immutable app-thread snapshot. It MUST
   NOT read mutable UI, transport, or engine state during persistence."*).
4. The app calls `save_project_atomic(&path, &snapshot)`. **This blocks the UI thread**
   (§4.7). It does not touch the audio thread (§4.4).
5. On `Ok(SaveReceipt { target_state: ReplacedDurable })` the dirty marker clears and the
   status line reads `Saved to {path}`.
6. On `Err(SaveError::Io { stage: SyncParentDirectory, target_state:
   ReplacedDurabilityUncertain, .. })` the project **stays dirty**, and the status line
   says the new file is in place but its directory entry may not survive a power loss,
   with a `Save again` affordance. The contract requires exactly this and forbids a
   second automatic replacement attempt.
7. On any other `Err` the dirty marker stays, the status line names the stage and the
   guarantee (`"Nothing was written. {path} is unchanged."`), and the model is untouched.

**Primary flow — open.**

1. The musician presses `Open project` with a path in the field.
2. The app calls `load_project(&path)`.
3. On `Ok(envelope)` the app **then** replaces its model — never before. The contract is
   explicit: *"The caller MUST keep the current live project unchanged until that result
   is available and accepted."*
4. Adoption applies `TransportCommand::Stop` (`spectre-core/src/transport.rs:86`) before
   the loaded transport becomes live, so opening a project saved while playing does not
   start playback — and, since R4-1 has landed, cannot start an audio engine as a side
   effect of opening a file.
5. On `Err` the status line names which of the five `LoadError` cases occurred, and the
   open project is **unchanged** — same tracks, same selection, same dirty state.

**Branch — unsaved work.** If the model is dirty, `Open project` first requires a second
press of a `Discard and open` confirm control that replaces the button in place for one
interaction. No modal. Rationale in §3.6.

### 3.3 Layout descriptions

PROJECT block, top → bottom, inside `SidePanel::left("tracks")`:

1. `PROJECT` section label — small, strong, `MUTED`, matching the existing `TRACKS`
   and `BROWSER` labels exactly.
2. Path field — `egui::TextEdit::singleline` sized to the panel width,
   `hint_text("Project file path")`. Data source: a new `project_path: String` field on
   `SpectrePrototype` (the shell struct in `main.rs`), not on `AppModel`. A path is shell state, not
   project state.
3. Action row, leading → trailing: `Save project`, `Open project`. Both `egui::Button`,
   both disabled with `on_disabled_hover_text` when the path field is empty.
4. Status line — one wrapped label. Sources, in priority order: the last
   `SaveError`/`LoadError` message, then the last success message, then the empty-state
   copy.
5. Dirty indicator — the text `Unsaved changes` in `WARM` (one of the shell's palette constants) when dirty,
   `Saved` in `MUTED` when not. Colour is never the only signal (§3.7).

**Empty state.** With no path and no save yet, the status line reads: *"No project file
yet. Type a path and save — Spectre replaces the file atomically, so an interrupted save
leaves the old file intact."* This states a property the code will have, not one it has;
the copy ships in the same slice as the behavior (§6.3).

Transport bar: a single `•` glyph plus the text `unsaved` after the `SPECTRE` wordmark
while dirty. No other transport-bar literal changes. In particular the right-aligned
cluster the two landed slices put in that same row — R4-1's engine state label, its
`blocks` counter, its `ENGINE UNAVAILABLE` copy and its `Retry` control, and R4-4's
`Rebuild engine` button and `PLAN STALE` label — belongs to those slices and must not be
touched, reworded, or displaced here. The dirty marker goes on the **leading** side beside
the wordmark, where nothing else sits, precisely so it cannot collide with either.

### 3.4 Input & gestures

- Pointer: click the two buttons; click and type in the path field.
- Keyboard: the path field and both buttons are ordinary focusable egui widgets reachable
  by `Tab` in declaration order.
- **No default keyboard shortcut is specified for save or open, and no shortcut map is
  proposed.** `criteria.md` AF-5 prohibits asserting a default shortcut map at the
  current evidence level, tracing to
  `docs/02-reference-research/workflow-field-study/product-implications.md`
  §"Prohibited conclusions at current evidence level". The commands are named
  (`project.save`, `project.open`) so that a later remappable, context-scoped command
  resolver can bind them; this spec fixes no binding, and in particular does not claim
  ⌘S. The existing `Space` and `1`–`4` bindings are unchanged.
- Specialized input: none. No stylus, controller, voice, or camera.
- Responsive: the block lives in a resizable panel with `min_width(180.0)`;
  the path field takes the available width, and the two buttons wrap to
  a second row below the panel's minimum comfortable width. Window minimum is
  1060 × 680 (`with_min_inner_size([1060.0, 680.0])`).

### 3.5 Transitions & animation

None. No navigation transition, because no view is entered or left. The dirty marker and
status line change text between frames with no animation. The app repaints on a 250 ms
timer (`request_repaint_after` of 250 ms), so a status change is visible within one repaint.

Reduced motion: nothing to reduce — this is the correct answer here rather than an
omission. A synchronous save blocks the UI thread, so no spinner can animate during it
(§4.7); the honest treatment is that the window is briefly unresponsive, which §5.4
requires be observed and timed by hand rather than masked.

### 3.6 Error states

Presentation is the same inline status line for every case, deliberately: a modal that
steals focus during a failed save is the wrong shape for a tool a musician is playing
into, and the failure is never ambiguous enough to need one. The one place a confirm is
required — discarding unsaved work — is an in-place two-press control, not a dialog.

| Trigger | Presentation | Recovery path | Data-loss risk |
|---|---|---|---|
| `SaveError::InvalidProject { reason }` | inline, `WARM`: *"Not saved — {reason}. {path} is unchanged."* | fix the offending state; the destination was never opened | **No.** Validation precedes every filesystem touch |
| `SaveError::Encode { source }` | inline: *"Not saved — the project could not be encoded. {path} is unchanged."* | report; no user action can currently cause this (§5.1) | **No** |
| `SaveError::Io { stage: CreateTemporary \| WriteTemporary \| SyncTemporary, target_state: Unchanged, source }` | inline, naming the stage in plain words (*"could not create a temporary file next to {path}"*, *"could not write"*, *"could not flush to disk"*) plus the OS message | free space, fix permissions, choose another path, press `Save project` again | **No.** The old file or the prior absence is intact |
| `SaveError::Io { stage: ReplaceTarget, target_state: Unchanged, source }` | inline: *"Not saved — could not replace {path}. The existing file is unchanged."* | retry, or save elsewhere | **No** |
| `SaveError::Io { stage: SyncParentDirectory, target_state: ReplacedDurabilityUncertain, source }` | inline, `WARM`, and **the project stays dirty**: *"Saved, but not confirmed durable. {path} now holds the new project, but a power loss could roll back the directory entry. Save again to retry."* | `Save again` | **Partial, and named.** The new bytes are visible now; a crash before the filesystem flushes its own metadata could revert the directory entry. Never reported as a plain success |
| `LoadError::Open { source }` | inline: *"Could not open {path}: {os message}."* | correct the path | **No.** The open project is untouched |
| `LoadError::Read { source }` | inline: *"Could not read {path}: {os message}."* — also the case for a file above the size bound (§4.2) | correct the path | **No** |
| `LoadError::SchemaTooNew { found, max_readable }` | inline: *"{path} was written by a newer Spectre (schema {found}; this build reads up to {max_readable}). Spectre will not open it, because opening it would show you less than the file contains."* | update Spectre | **No** — and this is the point: failing closed is what prevents an old build from displaying a partial project the user could then overwrite |
| `LoadError::Malformed { source }` | inline: *"{path} is not a readable Spectre project."* | choose another file | **No** |
| `LoadError::InvalidProject { reason }` | inline: *"{path} decoded but is not a valid project: {reason}."* | choose another file; salvage is R5 | **No** |
| Open with unsaved changes | the `Open project` button is replaced in place by `Discard and open` for one interaction | press again to proceed, or press `Save project` first | **Yes if confirmed** — which is why it takes two presses and says "Discard" |

### 3.7 Accessibility

- **Labels.** Every new element is a standard egui widget with visible text: the section
  label `PROJECT`, `TextEdit` with `hint_text("Project file path")`, buttons labelled
  `Save project` / `Open project` / `Discard and open`, and a plain-text status line.
  Nothing is icon-only. Nothing conveys state by colour alone: the dirty state is the
  word `unsaved`, and `WARM` is redundant emphasis.
- **Custom actions:** none needed; there is no compound gesture in this feature.
- **Focus order:** path field → `Save project` → `Open project`, in declaration order,
  ahead of the existing `TRACKS` list so that the block reads top-down.
- **Text scaling:** the status line wraps; the buttons size to their text. The 62 px
  transport bar is unchanged and the dirty marker is short by design so it
  cannot push the transport row out of that fixed height.
- **Screen readers — stated honestly.** `crates/spectre-app/Cargo.toml`'s `eframe`
  dependency reads `{ version = "0.32.3", default-features = false, features =
  ["default_fonts", "glow"] }`, so
  whatever accessibility integration eframe ships behind a feature flag is **off** in
  this build. This spec therefore makes **no screen-reader claim**, and must not be read
  as delivering one. What it does is avoid foreclosing decision 17
  (`decision-gates.md:41`, keyboard-complete operation and screen-reader labels by beta,
  scoped audit at R4): every element here is a standard labelled widget, so enabling the
  feature later is a manifest change plus an audit, not a redesign. Enabling it is
  routed to §8 Q5 and belongs to the R4 accessibility audit, not to this slice. (The
  dependency is quoted rather than line-pinned because R4-1 adds a `[features]` block and a
  `spectre-audio` dependency to the same manifest, which moves every line in it.)

---

## 4. Implementation Specification

### 4.1 Architecture placement

Three crates, in strict dependency order, with no new edges except the one that makes an
already-declared dependency real.

| Crate | What lands here | Why here |
|---|---|---|
| `crates/spectre-core/src/id.rs` | one accessor, `IdGen::state()` | the generator's position must be persisted or reload mints duplicate IDs (§4.2 "Generator state") |
| `crates/spectre-project/src/lib.rs` | schema-2 document types, the reusable public validator, the persisted collections | the document schema is this crate's job (CORE-003, `requirements-ledger.md:48`) |
| `crates/spectre-project/src/fs.rs` **(new)** | `load_project`, `save_project_atomic`, `LoadError`, `SaveError`, `SaveStage`, `TargetState`, `SaveReceipt`, the private `FsOps` seam | the accepted contract names `spectre-project` as the owner of both boundaries |
| `crates/spectre-project/src/command.rs` | `ProjectCommand::reorder_tracks` | decision 13 (`decision-gates.md:37`) puts edits in command transactions; reorder is an edit, so CORE-001's reorder evidence lands on the real edit path rather than on a bare `Vec::swap` |
| `crates/spectre-app/src/project.rs` **(new)** | `project_envelope(&AppModel) -> ProjectEnvelope` and `adopt(&mut AppModel, ProjectEnvelope) -> Result<(), AdoptError>` | mapping the UI model to the document is the app's job; the document crate must never learn about `AppModel` |
| `crates/spectre-app/src/main.rs` | the PROJECT block, path state, dirty state | shell |

**The realtime boundary is structural for four crates and a checked property for the
fifth. The distinction is stated because it is real, and because iteration 2 blurred it.**

The four render-path crates cannot name `spectre-project` at all — the dependency graph
does not permit it, and the edge runs the other way:

- `crates/spectre-audio/Cargo.toml:17–21` — `cpal`, `spectre-core`, `spectre-dsp`,
  `spectre-graph`. No `spectre-project`.
- `crates/spectre-graph/Cargo.toml:12–14` — `spectre-core`, `spectre-dsp`.
- `crates/spectre-dsp/Cargo.toml:12–13` — `spectre-core`.
- `crates/spectre-core/Cargo.toml:12–13` — `serde`, and nothing else. No path
  dependencies. (`serde_json` is present in that manifest only as a **dev**-dependency at
  `:17`, which no library target links.)
- `crates/spectre-project/Cargo.toml:12–17` depends on `spectre-core`, `spectre-dsp`,
  `spectre-graph`, `serde`, and `serde_json` — R4-4 added the two middle edges for the
  track model's descriptors and the graph builder. The direction is what matters: the
  render-path crates still do not depend on `spectre-project`, so the graph stays acyclic
  and the impossibility above is unaffected.

So for `spectre-audio`, `spectre-graph`, `spectre-dsp`, and `spectre-core`, `load_project`
and `save_project_atomic` are not merely "not called" from the render path — they are
**unnameable in the library target**, which is the target the callback runs in. One
qualification, stated so the claim is exact: `crates/spectre-audio/Cargo.toml:23–24`
dev-depends on `spectre-offline`, which depends on `spectre-project`, so `spectre_project`
*is* nameable inside `spectre-audio`'s test targets. No callback executes there.

**`spectre-app` is the exception, and it is not a compile-time impossibility.** R4-1 built
the render closure in `crates/spectre-app/src/engine.rs` —
`Box::new(move |mut block: RenderBlock| bridge.render(&mut block))` — inside `spectre-app`'s
**library** target, which `src/lib.rs`'s `pub mod engine;` puts it in, and R4-4 then made that same module
a genuine `spectre-project` consumer: `engine.rs` names `spectre_project::RoutingError`,
`TrackList`, `TrackPathNodes`, `build_track_graph`, and `track_device_factory`. So
`spectre_project::save_project_atomic` **would be** nameable there. Calling it a
compile-time impossibility would be false, and this spec does not say it.

What is true, checkable, and worth a test is narrower: the render-closure module names the
crate's **track and routing** surface and none of its persistence surface. U12 asserts
exactly that pair of facts — the four manifests, plus the absence of `save_project_atomic`,
`load_project`, `to_bytes`, `from_bytes`, and `ProjectEnvelope` from `engine.rs`. That
assertion could actually fire, which is the whole point of writing it; the manifest half
alone could not. This is the RT-001 argument (`requirements-ledger.md:28`) stated at the
strength the tree supports and no higher.

The secondary argument covers the app thread. `save_project_atomic` blocks in
`write_all`, `sync_all`, `rename`, and a directory `sync_all`. It runs on the egui update
thread (`impl eframe::App for SpectrePrototype`'s `update`), which is not the audio thread: with R4-1 in place the driver owns
its own callback thread, so a blocked UI thread stalls *messages to* the engine, never the
engine itself. §4.4 states what that costs.

### 4.2 Data model

All types below live in `crates/spectre-project`. Jeff's header block and the repository's
comment conventions apply to every new file.

**Schema version.**

```rust
// Current schema version written by this build
pub const SCHEMA_VERSION: u32 = 2;

// Newest schema this build can read
pub const MAX_READABLE_SCHEMA: u32 = 2;
```

Both are `1` today, declared at the top of `crates/spectre-project/src/lib.rs`. The bump is
required because schema 2 adds persisted object collections, and a build that cannot render
them must refuse the file rather than show the musician an empty project it would then let
them overwrite. That refusal already exists and is tested — the version gate inside
`from_bytes`, and `canonical_fixture_with_newer_schema_is_rejected`
(`tests/project_codec.rs:85–98`, which compares against `MAX_READABLE_SCHEMA + 1` and is
therefore version-relative). These two constants are schema identifiers, not numeric
limits, so PROD-003 does not apply to them; the two constants it does apply to are listed
at the end of this section.

**Documents.** `ProjectEnvelope` is unchanged in shape. `ProjectDoc` gains four fields,
all `#[serde(default)]` so a schema-1 file still decodes:

```rust
// Minimal R1 project document, extended at schema 2 with the first persisted collections
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectDoc {
    pub id: ObjectId,
    pub name: String,
    pub tempo_map: TempoMap,
    pub transport: Transport,
    // Splitmix64 generator position; 0 means a schema-1 document that carried none
    #[serde(default)]
    pub id_gen_state: u64,
    // First persisted object collection. The type is R4-4's own TrackList, not a parallel
    // document type: it already derives Serialize/Deserialize and already owns order
    #[serde(default)]
    pub tracks: TrackList,
    // Device instances and their app-thread parameter values
    #[serde(default)]
    pub devices: Vec<DeviceDoc>,
    // Restored working context; never engine or render state
    #[serde(default)]
    pub view: ViewDoc,
    // Unknown document fields from newer writers survive a rewrite
    #[serde(flatten)]
    pub unknown: Map<String, Value>,
}

// One persisted device instance, addressed by stable ID and by canonical key
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeviceDoc {
    pub id: ObjectId,
    pub key: String,
    pub parameters: Vec<ParameterDoc>,
    #[serde(flatten)]
    pub unknown: Map<String, Value>,
}

// One persisted parameter value; range validation belongs to the descriptor owner
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterDoc {
    pub id: ObjectId,
    pub key: String,
    pub value: f32,
    #[serde(flatten)]
    pub unknown: Map<String, Value>,
}

// Working context restored on open; the document schema must not depend on the UI crate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ViewDoc {
    pub lens: LensDoc,
    pub selected_track: Option<ObjectId>,
    pub selected_device: Option<ObjectId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LensDoc {
    #[default]
    Arrange,
    Build,
    Shape,
    Mix,
}
```

**Why `tracks` is a `TrackList` and not a new `TrackDoc` collection — the largest change
iteration 3 makes, and it is forced by the tree rather than chosen.** Iteration 2 specified
a fresh `TrackDoc` struct because `AppModel` then held its own `TrackView` and
`spectre-project` had no track type at all. R4-4 changed both facts. At `1960a8b`
`crates/spectre-project/src/track.rs` defines `Track` and `TrackList`, **both already
`#[derive(Serialize, Deserialize)]`**, both already in the crate that owns the document
schema, and `AppModel.tracks` is now a `TrackList` rather than a `Vec<TrackView>`.
Introducing a parallel `TrackDoc` would mean two track types in one crate, a hand-written
mapping between them, and two places for a future field to be added to and forgotten in.
So "the first persisted object collection" is **partly built already**: the collection, its
ordering, its identity rule (`TrackList::insert` refuses a duplicate `ObjectId`), its
`reorder`, and its serde derives all exist and are tested. What does **not** exist is any
path that writes them to or reads them from a file — no `ProjectDoc` field, no encoder
input, no validator rule, no filesystem. That remaining distance is what this spec closes,
and §7.2's delta is sized against it rather than against a green field.

Three consequences follow and are stated rather than left implicit:

- `TrackList::structure_revision` is `#[serde(skip)]`, and `track.rs`'s own comment already
  says why it must stay that way: *"Session-local, so a reload does not inherit a stale
  revision and R4-7 persists nothing about it."* A decoded `TrackList` therefore starts at
  revision `0`. §4.4 states what the app does about that on open.
- `TrackList::master_level` is real persisted mixer state that iteration 2's `TrackDoc`
  design had no field for. It rides along for free and is covered by the same round-trip
  assertions.
- **Deserialization bypasses every constructor.** `Track`'s fields are private and its
  public mutators clamp (`Track::set_level` through `GAIN_PARAMETERS[0]`,
  `set_instrument_level` through `PULSE_PARAMETERS[0]`), and `Track::new` refuses a blank
  name — but `#[derive(Deserialize)]` calls none of them, and `TrackList`'s derive calls
  neither `insert` nor its duplicate-ID and `MAX_TRACKS` refusals. A hand-edited or corrupt
  file can therefore carry a blank track name, a non-finite level, a level outside the
  descriptor's range, duplicate track IDs, or more than `MAX_TRACKS` tracks. That is
  precisely what the validator below exists to catch, and it is a **sharper** requirement
  than iteration 2 had, not a weaker one. Whether coupling the wire format to an app-thread
  type is the boundary Jeff wants is §8 Q13.

**Adding these fields is a compile-affecting change, and §7.2 inventories it as one.**
`#[serde(default)]` governs deserialization; it does nothing for Rust construction. None
of the four fields is an `Option`, so **every** struct literal of `ProjectDoc` in the
workspace must name them or fail to compile. There are exactly three, re-counted at
`1960a8b`. `grep -rn "ProjectDoc {" crates/` returns six hits: those three literals, plus
the `struct` definition, the `impl` block, and the `-> ProjectDoc {` return type on the
third helper's own signature. The three literals are the in-crate test helper `envelope()`
in `crates/spectre-project/src/lib.rs`, `default_project()` in
`crates/spectre-offline/src/lib.rs:158`, and the `project()` helper in
`crates/spectre-project/tests/command_history.rs:17`. The last two are outside the crate, so
they are ordinary downstream breakage rather than internal churn; §7.2 lists both as
modified files and shows both literals updated. `#[serde(default)]` on `tracks: TrackList`
needs only `TrackList: Default`, which it has — `impl Default for TrackList` delegating to
`TrackList::new`, in `track.rs`.

A `#[derive(Default)]` escape hatch is **not** available and must not be reached for:
`ProjectDoc.id` is an `ObjectId`, whose whole invariant is that it is nonzero
(`ObjectId::from_raw` returns `None` for `0`, and its hand-written `Deserialize` rejects
`0`), so `ObjectId` derives no `Default` and cannot be given one without destroying
CORE-001's guarantee. `..Default::default()` in the literals is therefore not an option
either.

`LensDoc` intentionally duplicates the four variants of `spectre_app::Lens`
(the `Lens` enum in `crates/spectre-app/src/lib.rs`). The document schema is a wire format
with its own compatibility rules; the UI enum is not, and `spectre-project` must not depend
on `spectre-app`. `spectre-app` owns the two conversions (§4.3).

`AppModel` has no zoom state — its eight fields at `1960a8b` are `transport`, `lens`,
`tracks: TrackList`, `devices: Vec<DeviceControl>`, `selected_track`, `selected_device`,
`ids: IdGen`, and `feedback` — so there is no zoom to persist. `feedback` is
prototype-feedback text, read only by `feedback_report`, and is deliberately **not**
persisted; it is instrumentation, not the musician's work.

**There is no arm state to persist, and iteration 2 was wrong to say there was.** It cited
an `armed` field that R4-4 removed along with `TrackView`. At `1960a8b` `Track`'s fields are
`id`, `name`, `instrument`, `instrument_level`, `level`, `muted`, `soloed` — no `armed` —
and `crates/spectre-app/src/main.rs`'s `track_list` states the reason in the shell itself, in
a comment above the row marker: *"Mute and solo are the two states that actually change what
is rendered, so they are what the row shows. There is no arm indicator, because there is no
recording path to arm for."* The
transport bar's Record button is inert with the hover text *"Recording arrives at R7; no
capture path exists yet."* So this spec persists no arm state because none exists, not
because it declined to. Whether arm state becomes project state or session state when
recording arrives is still a real question and stays at §8 Q6, restated to match.

**Generator state — a real hazard this schema closes.** `IdGen` is a deterministic
splitmix64 seeded once (`crates/spectre-core/src/id.rs:45–68`), and `AppModel::prototype`
seeds it with the fixed constant `IdGen::new(0x0047_4549_5354_5549)`. If the generator
position is not persisted, then after a reload the app restarts from that same seed and the
very next `add_track` — which mints `let id = self.ids.next_id();` from the model's own
generator — produces an ID that is already in the loaded project. That is a *guaranteed*
duplicate, not a probabilistic one, and a direct CORE-001 violation. R4-4 made it sharper
rather than softer: `TrackList::insert` now refuses a duplicate `ObjectId` with
`TrackError::DuplicateId`, so the collision surfaces as a **failed add-track** the musician
cannot get past, on the very first track they add after every reload. Persisting
`id_gen_state` closes it, and resuming needs no new constructor: `IdGen::new(seed)` sets
`state` directly (`id.rs:51–53`) and `next_id` advances before use (`id.rs:56–58`), so
`IdGen::new(persisted_state)` resumes the exact sequence. The only addition to
`spectre-core` is one accessor:

```rust
// Expose the generator position for persistence
pub fn state(&self) -> u64 {
    self.state
}
```

For a schema-1 document `id_gen_state` is 0. That case is bounded: schema 1 has no
collections, so the only ID in such a document is `ProjectDoc.id`, which `ObjectId`
guarantees nonzero (`id.rs:16–25`, `:29–35`). The app therefore seeds
`IdGen::new(project.id.raw())` and the project-wide uniqueness validator (below) is the
backstop, refusing any save that would write a duplicate before the destination is
touched. The general question — what to do about a missing or corrupt generator state in
files that will exist after R5 migrations — is routed to §8 Q7.

**The reusable validator.** The contract's "Current codec qualification" section requires
one validator called after decode *and* before encoding. Today `validate` is a private
free function in `crates/spectre-project/src/lib.rs` and only `from_bytes` calls it;
`to_bytes` does not revalidate. This spec makes the validator public with its own error
type and leaves `to_bytes` alone as the low-level encoder, because `save_project_atomic` —
the only path that writes a destination — calls the validator itself:

```rust
// Semantic invalidity, distinct from encoding failure and from I/O failure
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValidationError {
    pub reason: &'static str,
}

// The one semantic validator: called after decode and before encode
pub fn validate_envelope(envelope: &ProjectEnvelope) -> Result<(), ValidationError>;
```

Every rule the private `validate` enforces today is preserved verbatim — nonzero project
ID, tempo-map reconstruction, meter-map decode, enabled-loop-requires-region, and ordered
loop region, in that order, all five inside `validate` in
`crates/spectre-project/src/lib.rs`. Schema 2 adds:

1. **Project-wide unique object IDs.** Every `ObjectId` in the document — `project.id`,
   each `Track::id`, each `DeviceDoc.id`, each `ParameterDoc.id` — must be distinct.
   This is exactly what the contract defers to *"once persisted object collections
   exist"*, and what `crates/spectre-core/src/id.rs:4` records as pending. Nonzero comes
   free from `ObjectId`'s hand-written `Deserialize` (`id.rs:16–25`). Note that
   `TrackList::insert`'s runtime duplicate refusal does **not** cover this: the derived
   `Deserialize` builds the vector directly and calls no method on it.
2. **Referential integrity of the view.** `view.selected_track`, when `Some`, must name a
   `Track` present in `tracks`; `view.selected_device`, when `Some`, must name a
   `DeviceDoc` present in `devices`. A dangling selection is invalid, not silently
   dropped.
3. **The `Track` invariants that deserialization bypasses.** Each `Track::name` must be
   non-blank after trimming, as `Track::new` requires; each `Track::level` and
   `TrackList::master_level` must be finite and inside `GAIN_PARAMETERS[0]`'s range; each
   `Track::instrument_level` must be finite and inside `PULSE_PARAMETERS[0]`'s range; and
   `tracks.len()` must not exceed `MAX_TRACKS`, which `TrackList::insert` enforces at
   runtime and the derive does not. **No new number is introduced by any of this** — every
   bound named is an already-accepted one being re-checked on a path that skipped it.
4. **Finite parameter values, non-empty keys.** Each `ParameterDoc.value` must be finite;
   `DeviceDoc.key` and `ParameterDoc.key` must be non-empty; `(device.key, parameter.key)`
   pairs must be unique within a device.

Rule 4 is a correctness requirement, not hygiene. JSON has no literal for NaN or infinity,
so a non-finite value cannot survive an honest encode: whichever way the encoder resolves
it, the file no longer means what the project meant. Validating **before** encoding turns
that into `SaveError::InvalidProject` at conceptual stage `ValidateSnapshot` with the
destination never opened, instead of a corrupt or unreadable file. This is the same
posture RT-003 (`requirements-ledger.md:30`) takes on the render side — contain the
non-finite value at the boundary rather than let it propagate.

**A persisted `ParameterDoc.value` is the *base* value and nothing else.** R4 has no
automation and no modulation layer, so today there is only one contribution and the
distinction is invisible. It is stated here anyway so the field is not silently overloaded
when R9 delivers PROD-002 (`requirements-ledger.md:95`, automated versus
manually-overridden states with an explicit restore action): automation and modulation
contributions get their **own** document fields under that requirement, and `value` keeps
meaning what it means now. The same applies to `Track::level` and `master_level`.

What the validator explicitly does **not** do: resolve a `DeviceDoc.key` /
`ParameterDoc.key` pair to a DSP descriptor and range-check the value against it. This is
**not** a dependency argument any more, and iteration 2's version of this paragraph is
stale: `crates/spectre-project/Cargo.toml:14` depends on `spectre-dsp` since R4-4, and
`track.rs` already reaches `GAIN_PARAMETERS`/`PULSE_PARAMETERS` through it, which is what
makes rule 3 above cost nothing. The real reason is that `spectre-project` holds **no
device catalogue**: nothing in the crate maps the arbitrary `String` `"saturator"` to
`SATURATOR_PARAMETERS`. That mapping lives where the catalogue lives, and that layer already
does the check: `DeviceValues::from_snapshot` rejects unknown device/parameter identities
(`crates/spectre-offline/src/lib.rs:105–111`) and non-canonical values (`:115–124`), and
`DeviceParameterSnapshot::new` clamps on construction
(`crates/spectre-dsp/src/parameter.rs:91–105`, the `parameter.clamp(value)` at `:103`).
`spectre-app::adopt` owns the mapping and calls that path (§4.3).

**Numeric bounds introduced here.** Two, each with its own rationale, neither copied from
any product. PROD-003 (cited by ID, not by line — the ledger grows during R4 and its line numbers move)
requires the rationale to be recorded **in the ledger**, so both appear in §7.2's modified-file list.

| Constant | Value | Rationale (Spectre-derived) |
|---|---|---|
| `SAVE_TEMP_NAME_ATTEMPTS` | `8` | The temporary name mixes the process ID, the nanosecond field of the wall clock, and a monotonic counter, so a collision needs a leftover temporary whose three components all match. Retrying is required by the contract (*"retries name collisions without truncating another file"*), but the retry must be **bounded**: this runs synchronously on the UI thread, and an unbounded loop over a pathological directory would hang the window instead of failing. Eight attempts is small enough that the worst case is imperceptible and large enough that an accidental collision cannot exhaust it. |
| `MAX_PROJECT_FILE_BYTES` | `64 * 1024 * 1024` | `load_project` reads a file that may be truncated, corrupt, or not a project at all before it can know anything about it. Without a bound, the read allocates whatever the file claims to be. The checked-in R1 fixture is **912 bytes**; a schema-2 project with the collections in this spec is a few kilobytes (§4.7). 64 MiB is four orders of magnitude of headroom for the alpha while keeping the app-thread allocation bounded. It also bounds the validator's uniqueness set, which is sized by the document. It is a starting envelope for R4 and is expected to be revisited when R5 or later persists sample or media references — recorded as such rather than presented as a final number. |

**A third bound was proposed in iteration 2 and is withdrawn here, because the ledger has
since ruled on it.** `TRACK_LEVEL_RANGE = 0.0..=1.0` rested on an inspector slider that read
`0.0..=1.0` and on `0.78`/`0.72` track seeds. R4-4 deleted both: the slider's range is now
`main.rs`'s `GAIN_RANGE` constant, `0.0..=GAIN_MAX` where `GAIN_MAX` is `2.0`, declared under
the comment *"The fader's range IS the accepted gain descriptor's range; no fader law is
invented here"*, and
`Track::new` seeds `level` from `GAIN_PARAMETERS[0].default()`, which is `1.0`. So the
iteration-2 number was wrong on its value, its source, and its seeds. More decisively,
`requirements-ledger.md:69–71` has already refused the row: *"The track fader's range is
likewise not a new row — it **is** `GAIN_PARAMETERS[0]`'s accepted range, and unity **is**
that descriptor's default, which is the whole reason R4-4 introduces no fader law of its
own."* Validator rule 3 therefore checks against the descriptor directly and this spec
proposes **no** ledger row for it. Track count is likewise already a row — ENGINE-005
(`requirements-ledger.md:62`), `MAX_TRACKS` — so rule 3's count check needs none either.

### 4.3 API contracts

**`crates/spectre-project/src/fs.rs` — public.** These signatures are the accepted
contract's, unchanged in name, argument order, and semantics.

```rust
// Read, decode, schema-gate, and semantically validate a complete project
pub fn load_project(path: &Path) -> Result<ProjectEnvelope, LoadError>;

// Validate, encode, and atomically replace the target with a synchronized sibling
pub fn save_project_atomic(
    path: &Path,
    snapshot: &ProjectEnvelope,
) -> Result<SaveReceipt, SaveError>;

pub enum LoadError {
    Open { path: PathBuf, source: io::Error },
    Read { path: PathBuf, source: io::Error },
    SchemaTooNew { found: u32, max_readable: u32 },
    Malformed { source: serde_json::Error },
    InvalidProject { reason: ValidationError },
}

pub struct SaveReceipt {
    pub target_state: TargetState, // always ReplacedDurable on Ok
}

pub enum TargetState { Unchanged, ReplacedDurable, ReplacedDurabilityUncertain }

pub enum SaveStage {
    ValidateSnapshot,
    EncodeSnapshot,
    CreateTemporary,
    WriteTemporary,
    SyncTemporary,
    ReplaceTarget,
    SyncParentDirectory,
}

pub enum SaveError {
    InvalidProject { reason: ValidationError },
    Encode { source: serde_json::Error },
    Io { stage: SaveStage, target_state: TargetState, source: io::Error },
}
```

The contract permits *"richer diagnostics in each variant"*, which is why `Open`/`Read`
retain the path; it requires that the four load distinctions (inaccessible I/O, malformed
encoding, newer schema, semantically invalid) stay separate, and they do. `CodecError`
and `ValidationError` in the contract are design-level names: `CodecError` resolves to
`serde_json::Error`, which is what `ProjectError::Malformed` already carries in
`crates/spectre-project/src/lib.rs`, and `ValidationError` is the new struct in §4.2.

Both calls are synchronous, app-thread-only, allocate freely, and are never
callback-reachable. Neither spawns a thread, owns process-global persistence state, or
hides asynchronous work — the contract forbids all three. No auth, pagination, or rate
limiting applies; this is a local file API.

**`load_project` — ordered steps.**

1. `File::open(path)` → `LoadError::Open`.
2. Read the complete bytes, refusing above `MAX_PROJECT_FILE_BYTES` before allocating for
   the whole file — check `metadata().len()` first, then read with a bounded
   `take(MAX_PROJECT_FILE_BYTES + 1)` so a file that grows between the two steps is still
   refused. Both failures and the refusal are `LoadError::Read`; the refusal carries
   `io::ErrorKind::InvalidData` and a message naming the bound.
   *(A dedicated `TooLarge` variant would extend an accepted enum, so it is routed to §8
   Q8 rather than taken unilaterally.)*
3. Peek `schema_version` and gate it, reusing the existing `VersionPeek` logic inside
   `from_bytes` → `LoadError::SchemaTooNew`.
4. Decode the envelope → `LoadError::Malformed`.
5. `validate_envelope(&envelope)` → `LoadError::InvalidProject`.
6. Return the complete validated envelope. No failure returns a partial project, and no
   failure path writes anything.

**`save_project_atomic` — ordered steps, with what each buys.**

| # | Step | Concrete call | What it buys | Failure → |
|---|---|---|---|---|
| 1 | validate | `validate_envelope(snapshot)?` | a semantically broken project can never reach the disk | `InvalidProject`; no path touched |
| 2 | encode | `to_bytes(snapshot)?` — the existing `serde_json::to_vec_pretty` wrapper, unchanged | the complete bytes exist in memory before anything is created | `Encode`; no path touched |
| 3 | create temp | `OpenOptions::new().write(true).create_new(true).open(&temp)`, retried up to `SAVE_TEMP_NAME_ATTEMPTS` on `ErrorKind::AlreadyExists` | `create_new` is `O_CREAT\|O_EXCL`: the kernel decides, so there is no check-then-create race and an existing file is never truncated | `Io { CreateTemporary, Unchanged }` |
| 4 | write | `file.write_all(&bytes)` | `write_all` loops until every byte is written; a short write is not success | `Io { WriteTemporary, Unchanged }` + best-effort `remove_file(&temp)` |
| 5 | sync file | `file.sync_all()` | the *contents* are handed to the storage device before any name points at them; without this, a rename can publish a file whose data is still only in the page cache | `Io { SyncTemporary, Unchanged }` + cleanup |
| 6 | release | `drop(file)` | releases the handle before replacement, as the contract requires *"as required by the replacement API"* | — |
| 7 | replace | `std::fs::rename(&temp, path)` | POSIX `rename(2)` replaces an existing name atomically with respect to other processes: **no window in which the destination is missing or half-written.** This is the commit point | `Io { ReplaceTarget, Unchanged }` + cleanup |
| 8 | sync dir | `File::open(parent)?.sync_all()` | flushes the *directory entry* so the replacement itself, not just the data, survives a crash | `Io { SyncParentDirectory, ReplacedDurabilityUncertain }`, **no second replacement attempt** |
| 9 | receipt | `Ok(SaveReceipt { target_state: ReplacedDurable })` | only after every required synchronization succeeded | — |

**`save_project_atomic` does not stamp the schema version, and nothing in the algorithm
above does.** Step 2 encodes the snapshot it was handed; the parameter is
`&ProjectEnvelope`, so the function cannot mutate it; and the accepted contract's eight
required steps contain no stamping step, so adding one here would amend accepted material
rather than implement it. The writer therefore reproduces the snapshot's own
`schema_version` byte for byte. The stamp belongs to whoever *builds* the snapshot, which
is `project_envelope` (below).

The temporary lives in the target's **existing parent directory** (`path.parent()`,
falling back to `Path::new(".")` for a bare filename), named
`.{file_name}.{pid:x}-{nanos:x}-{counter:x}.spectre-tmp` — collision-resistant, never a
fixed predictable name, and a sibling so the rename never crosses a filesystem. The
counter is a `static AtomicU64` used only for name entropy: it holds no target, no lock,
and no ordering guarantee, so it is not the *"process-global persistence state"* or the
*"background coordinator"* the contract forbids. Concurrent saves to the same target
remain unsupported and remain the app's responsibility (§4.4).

Cleanup is synchronous, best effort, and never masks the primary error — a failed
`remove_file` is attached as context, never returned. After a successful replacement
there is no temporary path left to clean.

**`crates/spectre-app/src/project.rs` — public.**

```rust
// Build the immutable app-thread snapshot the save boundary consumes
pub fn project_envelope(model: &AppModel, name: &str) -> ProjectEnvelope;

// Replace the live model only after a load has fully succeeded
pub fn adopt(model: &mut AppModel, envelope: ProjectEnvelope) -> Result<(), AdoptError>;

pub enum AdoptError {
    UnknownDevice { key: String },
    UnknownParameter { device_key: String, key: String },
    ValueOutOfRange { device_key: String, key: String, value: f32 },
}
```

`project_envelope` reads `AppModel::track_list()` — R4-4's accessor that borrows the whole
`TrackList`, which is what makes the tracks field a clone rather than a rebuild — plus
`devices()`, `selected_track_id()`, `selected_device_id()`, `lens()`, the transport, and
the generator position via the new `IdGen::state()`. It reads nothing else and mutates
nothing. All six are existing public methods on `AppModel` at `1960a8b`; this spec adds no
accessor to `spectre-app`.

**`project_envelope` is the sole owner of the version stamp.** It is the only component
that constructs a `ProjectEnvelope` from scratch, so it is the only component that can
set `schema_version`, and it sets it to `SCHEMA_VERSION` unconditionally. That is what
makes *"a save always writes the current version"* true of **the product**: `./spectre`
has exactly one save path and it goes through this function. It is deliberately not true
of the crate — a caller that hands `save_project_atomic` a schema-1 envelope gets a
schema-1 file back, which is the property I9 pins and I15 pins from the other side. If a
later slice adds a second snapshot builder, that slice inherits the stamp, and §7.2's
migration rule must be restated rather than assumed.

**What `project_envelope` cannot carry, stated before anyone relies on it.** `AppModel`
has no field for the envelope's or the document's `#[serde(flatten)]` unknown map, and
§4.4 adds none, so an envelope built from `AppModel` carries **empty** unknown maps. An
unknown field read from a file therefore survives `load_project` (I8) and survives a
crate-level rewrite (the existing `canonical_fixture_unknown_fields_survive_rewrite`), but
does **not** survive open → edit → save through `./spectre`. The R4 exposure is narrow —
`load_project` refuses any file whose `schema_version` exceeds `MAX_READABLE_SCHEMA`, so
the only unknown fields this build can read are ones a same-version writer or a hand edit
put there — but it is a real hole in CORE-003's forward-field preservation *at the product
level*, this slice does not close it, and no test, ledger row, or product string may say
otherwise. Routed to §8 Q12.

`adopt` maps persisted device and parameter keys back to the canonical `&'static str`
descriptors — required, because `DeviceParameterSnapshot::device_key` is a `&'static str`
(`crates/spectre-dsp/src/parameter.rs:83`) and a `String` read from a file is not one —
rejecting any key not in the **five** descriptor sets `AppModel` imports at `1960a8b`:
`PULSE_PARAMETERS`, `GAIN_PARAMETERS`, `SATURATOR_PARAMETERS`, `FILAMENT_PARAMETERS`, and
`GLOAM_PARAMETERS`. R4-6 added the last two, and a mapping written against iteration 2's
three would silently refuse every project containing a `filament` or `gloam` value. It also
rejects any value outside its descriptor range, and applies `TransportCommand::Stop`
(`crates/spectre-core/src/transport.rs:86`) before the loaded transport becomes live. It
mutates `model` only after every mapping has succeeded.

`adopt` writes `AppModel`'s private fields directly rather than through new public setters.
That is legal without widening `AppModel`'s surface: `AppModel` is declared in the crate
root (`crates/spectre-app/src/lib.rs`) and `crate::project` is a descendant module of that
root, so the root's private fields are in scope there. This is why §4.4 can say the feature
adds no field **and** no accessor to `AppModel`.

**`crates/spectre-project/src/command.rs` — one added command.**

```rust
// Move one track to an absolute index; identity is addressed, never position
pub fn reorder_tracks(id: ObjectId, to_index: usize) -> Self;
```

**Addressed by `ObjectId`, not by a `from` index, and that is a change from iteration 2
that R4-4 forced and improved.** `TrackList::reorder(id, to_index)` already exists, is
already tested for identity and field preservation, and **returns the index the track came
from**. So the command's `apply` calls it and builds its inverse from the returned index:
`reorder_tracks(id, from)`. That inverse is exact by construction rather than by an argument
about `remove`/`insert` symmetry, and addressing by identity rather than by position is what
CORE-001 is about in the first place — a `from` index can be stale, an `ObjectId` cannot.
It composes with the existing `Transaction`/`EditHistory` machinery
(`Transaction::execute`, `command.rs:92–109`; `EditHistory::apply`, `:134–143`) unchanged.

`TrackList::reorder` already returns `TrackError::UnknownTrack` for an absent id and
`TrackError::IndexOutOfRange { index, len }` for an out-of-range destination, both **before
any mutation**, so the failure vocabulary is not reinvented: `CommandError` gains one
wrapping variant, `Track(TrackError)`, joining the three at `command.rs:11–15`.
`Transaction::execute`'s all-or-nothing property is preserved because the refusal happens
inside `reorder` before it touches the vector.

### 4.4 State management

- **Project truth** stays in `AppModel`. This
  feature adds **no** field and **no** method to it. The single-model, linked-lens invariant
  (`vision.md:37`) is preserved: save serializes the one model, and open replaces the one
  model, so no lens can hold a private copy. The one thing that costs, stated where the
  claim is made rather than buried: because `AppModel` has no field for the decoded
  envelope's `#[serde(flatten)]` unknown maps, an unknown field read from a file does not
  survive a round trip *through the app* (§4.3). That is a deliberate consequence of adding
  no field, not an oversight, and it is routed to §8 Q12.
- **Shell state** — `project_path: String`, `dirty: bool`, `status: String` — goes on
  `SpectrePrototype`, beside the existing `new_track_name` and `feedback_status`. A file path is not project content.
- **Dirty rule.** Set on any model mutation reachable from the shell. R4-4 replaced the
  single `selected_track_mut` borrow iteration 2 named with a set of explicit methods, so
  the list is now: `toggle_play`, `select_lens`, `select_track`, `add_track`,
  `remove_track`, `reorder_track`, `rename_track`, `set_track_level`, `set_track_muted`,
  `set_track_soloed`, `set_track_instrument_level`, `set_master_level`, and
  `set_device_parameter` (through `set_device_parameter_from_ui`). That is a longer list
  than iteration 2's and every entry is a real public method on `AppModel` at `1960a8b`.
  Cleared **only** on `Ok(ReplacedDurable)`. Explicitly **not** cleared on
  `ReplacedDurabilityUncertain` — the contract requires the caller to *"keep the
  in-memory project dirty"*.
- **Structure revision after an open.** `TrackList::structure_revision` is
  `#[serde(skip)]`, so an adopted list starts at `0` while a running engine was built from
  some other revision. `SpectrePrototype::engine_is_stale` compares those two numbers and
  the transport bar already reads `PLAN STALE` with a `Rebuild engine` control when they
  differ. Opening a project therefore lands in R4-4's **existing** stale-plan state rather
  than in a new one, and this spec adds no rebuild trigger of its own: a rebuild restarts
  the stream with an audible gap, and choosing to take that gap stays the musician's.
- **Save serialization is structural.** The contract requires the app layer to serialize
  saves to the same normalized target and forbids the crate from adding a lock or
  coordinator. `save_project_atomic` is synchronous and `./spectre` calls it from the one
  egui update thread, so a second save cannot begin while the first is
  running — the thread is inside the call. No flag, no lock, no queue. If a future slice
  moves saving off the UI thread, that slice owns the serialization, and this spec says
  so rather than pretending the property is intrinsic.
- **Offline / draft persistence:** none. Journaled autosave is decision 14
  (`decision-gates.md:38`) and belongs to R5 by the design authority's milestone table.
  This slice writes only when the musician presses the button.
- **What is never persisted:** engine health, telemetry, `blocks_rendered`, xruns,
  headroom, containment counters, device selection at the driver level, or anything else
  R4-1 introduces. Persisting a measurement would turn it into a claim about a session
  that is not running.

**The audio thread, precisely.** With R4-1 landed, `./spectre` holds the app-thread half
of the RT-002 control transport and the driver owns the callback thread. A save blocks the
app thread for the duration of two `fsync`-class calls. During that window: the render
callback continues to execute the compiled plan with the state it already has; no
parameter, note, or transport message is *sent*; nothing is dropped, because the sender is
simply not called; and the RT-002 lanes are neither drained faster nor slower by anything
this feature does. No allocation, no lock, no I/O, and no logging crosses the callback
boundary, because nothing this feature adds is called from the closure — which §4.1 states
at the strength it actually holds: unnameable in the four render-path crates, and absent
from the app's render-closure module as a property U12 checks. The one real cost is a
stalled UI frame (§4.7), which is a responsiveness issue, not an RT-001 issue, and is
named as such.

### 4.5 Dependencies

- **New third-party packages: none.** `std::fs`, `std::io`, `std::path`, `std::time`,
  `std::sync::atomic`, `std::process` only. In particular no `tempfile` and no `rfd`.
- **New assets or resources:** one checked-in test fixture,
  `crates/spectre-project/tests/fixtures/r4-canonical.json`, generated by the encoder
  itself the same way `r1-canonical.json` was, and read by `include_bytes!` exactly as at
  `tests/project_codec.rs:9`.
- **New internal crate edges:** none. `spectre-app → spectre-project` is declared at
  `crates/spectre-app/Cargo.toml`'s `[dependencies]` block and is already **used** — R4-4 and R4-6 put the track
  model and the graph builder across it. This spec is the first use of that crate's
  *persistence* surface, which is a narrower and more checkable claim than iteration 2's.
- **Test scratch space:** tests create a unique directory under `std::env::temp_dir()`
  named from the process ID and a counter, and remove it at the end of the test. This is a
  ten-line test-local helper, not a dependency. It does not clean up after a panicking
  test; that is a known and accepted cost of not taking `tempfile`, and it is stated here
  rather than discovered later.
- **Infrastructure:** none. No database, no CDN, no service. Nothing leaves the machine.

### 4.6 Platform-specific considerations

macOS and Linux are co-first-class (decision 1, `decision-gates.md:25`), decision 23
(`:49`) leaves Linux **device** qualification undischarged, and the design authority makes
platform behavior an explicit R5 gate. This section therefore separates what is
guaranteed, what is platform-specific, and what this spec is not entitled to claim.

**Guaranteed on both platforms.**

- `rename(2)` replaces an existing name atomically with respect to other processes and
  never leaves the destination absent mid-call. `std::fs::rename` maps to it on both
  targets. Because the temporary is a sibling, the rename is always within one
  filesystem, which is the condition the guarantee requires. This is what buys *"readers
  observe either the complete old file or the complete new file, never an encoded
  prefix"* — a **normal-process** property that holds today, with no crash involved.
- `O_CREAT|O_EXCL` exclusive creation (`create_new`) is atomic on both.
- Opening a directory read-only to obtain a descriptor works on both.

**Platform-specific, and where the two differ.**

- `File::sync_all()` is `fsync(2)` on Linux, which flushes data and metadata to the device
  (subject to the device's own write cache, which modern ext4/xfs/btrfs configurations
  handle with barriers). On Apple platforms plain `fsync(2)` is documented **not** to
  flush the drive's internal cache; `fcntl(F_FULLFSYNC)` is the call that does. Rust's
  standard library is believed to use `F_FULLFSYNC` for `sync_all` on Apple targets.
  **This spec does not treat that as verified.** It is a claim about the pinned
  toolchain's `std`, not about this repository, and no file in this repository can settle
  it. Implementation must confirm it against the toolchain's own source, and R5's
  qualification must confirm it against hardware. Routed to §8 Q9.
- Directory synchronization is where the two platforms genuinely diverge. On Linux,
  `fsync` on a directory descriptor is the defined way to make a rename durable and is
  required. On macOS/APFS the guarantee is not documented by Apple, and a
  `F_FULLFSYNC`-style flush on a *directory* descriptor may be refused with `EINVAL` or
  `ENOTSUP` depending on the filesystem. **The contract already has the right answer for
  this**: step 8's failure is not a failed save, it is
  `ReplacedDurabilityUncertain` — the new file is visible, the caller keeps the project
  dirty, and no second replacement is attempted. So a platform that cannot provide the
  step degrades explicitly rather than silently, which is precisely what the contract
  demands (*"MUST fail explicitly rather than silently weaken the contract"*).
- Filesystems and mounts outside the qualified set — NFS, SMB, FUSE, overlay mounts,
  synchronized cloud folders — may not honor either the rename or the directory-sync
  semantics. The contract states these *"remain unsupported for the durable guarantee"*
  and gives R5 the supported matrix. R4 must not add a row to any qualification table and
  must not claim any of them.

**What R4 proves and what it does not — the `implemented` / `verified` divide.**
`docs/README.md`'s status vocabulary separates `implemented` ("code exists but its full
evidence gate may remain open") from `verified` ("stated acceptance evidence passes").
When this slice lands, CORE-004 moves to **`implemented`, not `verified`**, and STATUS
must say so. R4 can prove, on both platforms, in ordinary tests: correct ordering of the
accepted contract's **eight** required steps — §4.3's table shows nine rows because it
splits the contract's step 5 ("`sync_all` … and close or otherwise release handles") into
a sync row and a release row, which is this spec's decomposition and not the contract's
count; the exact target state at every failure stage; that no partial destination is
ever observable to a concurrent reader in a running process; that the old bytes survive
every pre-commit failure. R4 **cannot** prove that a replaced file survives a power loss,
because that needs crash injection against real qualified filesystems — which is R5's
assignment in the same contract, and which no test in this spec claims to be. Any
sentence in the product or in STATUS implying crash-proof saving before R5 runs its drills
is exactly the optimistic language `docs/status/STATUS.md`'s header prohibits.

**Version compatibility, feature flags, rollout:** none. No OS version floor is
introduced; every call used has been in POSIX and in Rust's `std` for the life of both
targets. No feature flag, because a half-enabled save path is worse than no save path.

### 4.7 Performance budget

- **Storage.** The R1 fixture is **912 bytes** on disk, measured. A launch-state schema-2
  project carries what `AppModel::prototype()` actually builds at `1960a8b`: **one** track
  and **five** device instances — `pulse`, `gain`, `saturator`, `filament`, `gloam` —
  holding **eleven** parameter values in total (1 + 1 + 2 + 4 + 3, from the five descriptor
  arrays). Iteration 2 said three devices and four parameters, which was true before R4-6.
  Each parameter is an id, a key, and a float in pretty-printed JSON; each track is seven
  small fields. That puts a launch-state document at roughly 3 KB and a three-track project
  at roughly 3.5 KB. Peak disk usage during a save is the old file plus the temporary —
  under 8 KB for an alpha project. **These are arithmetic from the object counts, not
  measurements**; the real number lands when the R4 fixture is generated (§7.2), and it
  grows again with clips (R4-5), so it is not a standing estimate for later milestones.
- **Memory.** One encoded `Vec<u8>` the size of the document, one decoded envelope, and
  one `HashSet<u64>` for the uniqueness check sized by the object count. All bounded by
  `MAX_PROJECT_FILE_BYTES` on the load side. Steady-state memory is unchanged: no cache,
  no retained buffer, no background structure.
- **CPU / render time.** Zero effect on render time. The render path does not link this
  code (§4.1). Encoding a few kilobytes of JSON is microseconds.
- **UI responsiveness — the real cost, stated plainly.** Save blocks the egui update
  thread for the duration of a file `sync_all` plus a directory `sync_all`. A full flush
  is not free even for a small file: it is commonly single-digit milliseconds on an SSD
  and can be tens of milliseconds when it forces the drive's own cache, which is exactly
  what `F_FULLFSYNC`-class behavior does on Apple hardware. At the app's 250 ms repaint
  cadence that is at most a visible hitch, not a freeze. These figures are
  **estimates from the nature of the calls, not measurements**; §5.4 requires the manual
  protocol to record the real elapsed time on both platforms, and §8 Q10 asks whether the
  measured number should force saving off the UI thread in R5.
- **Network payload:** none. Nothing here uses the network.
- **Startup time:** unchanged. `./spectre` does not open a project on launch in this
  slice; the path field starts empty.

---

## 5. Test Specification

Every command below is real. The workspace gate is unchanged and is the outer loop:

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

Targeted commands used while iterating:

```sh
cargo test --locked -p spectre-project
cargo test --locked -p spectre-app
cargo test --locked -p spectre-offline
```

All of them run today. Three test files are new (`crates/spectre-project/tests/project_save.rs`,
`crates/spectre-project/tests/render_path_isolation.rs`, `crates/spectre-app/tests/project_io.rs`)
and are created by this slice (§7.2).

**Where each test lives, and why it has to live there.** The contract requires the
fault-injection seam to be private — *"only under test configuration or behind a private
generic seam; it MUST NOT become application-global state or a public pluggable filesystem
API."* An integration test in `tests/` can only see the public API, so the injection tests
**must** be unit tests in `#[cfg(test)] mod tests` inside `crates/spectre-project/src/fs.rs`,
where `FsOps`, `FaultFs`, and the private `save_with` are visible. Real-filesystem tests
are integration tests, because they need nothing private.

### 5.1 Unit tests

*Group A — fault injection through the private seam (`crates/spectre-project/src/fs.rs`).*
`FaultFs` implements the private `FsOps` trait, records an ordered call log, and fails at a
chosen `SaveStage`. Setup for all of A: a valid schema-2 envelope from a module-local
`envelope()` helper modeled on the existing `envelope()` in `src/lib.rs`'s
`#[cfg(test)] mod tests`.

Group A covers **six** of the seven `SaveStage` values — `ValidateSnapshot` (U1),
`CreateTemporary` (U2, U4), `WriteTemporary` (U5, U9), `SyncTemporary` (U6),
`ReplaceTarget` (U7), and `SyncParentDirectory` (U8). `EncodeSnapshot` is the seventh and
is deliberately uncovered, for the reason stated after the table. Byte-identity of a real
destination is **not** a Group A property — `FaultFs` has no real destination — and is
proved instead by I4 and I5 on a real filesystem.

| # | Name | Setup | Assertion | Edge covered |
|---|---|---|---|---|
| U1 | `validation_failure_touches_no_path` | envelope whose `TrackList` holds two `Track`s sharing one `ObjectId` — constructed by decoding hand-built JSON, because `TrackList::insert` refuses the duplicate at runtime and only `Deserialize` can produce it | `Err(SaveError::InvalidProject { .. })` **and** `FaultFs`'s call log is **empty** | contract: validation precedes every filesystem touch |
| U2 | `temporary_creation_failure_leaves_target_unchanged` | `FaultFs` fails `CreateTemporary` | `Err(Io { stage: CreateTemporary, target_state: Unchanged, .. })`; log contains no `write_all`, no `replace` | pre-commit failure |
| U3 | `temporary_name_collision_retries_without_truncating` | `FaultFs` returns `AlreadyExists` for the first two names, succeeds on the third | `Ok`; log shows exactly three `create_new` calls with **three distinct paths**, and no `open`-for-write of an existing path | contract: exclusive creation, retry, never truncate another file |
| U4 | `temporary_name_retries_are_bounded` | `FaultFs` always returns `AlreadyExists` | `Err(Io { stage: CreateTemporary, .. })` after exactly `SAVE_TEMP_NAME_ATTEMPTS` `create_new` calls | the bound in §4.2 is real; no unbounded UI-thread loop |
| U5 | `write_failure_cleans_up_and_preserves_target` | `FaultFs` fails `write_all` | `Err(Io { stage: WriteTemporary, target_state: Unchanged, .. })`; log ends with `remove` of the temporary; no `replace` | cleanup is attempted |
| U6 | `temporary_sync_failure_preserves_target` | `FaultFs` fails `sync_file` | `Err(Io { stage: SyncTemporary, target_state: Unchanged, .. })`; log contains `remove`, no `replace` | data must be durable before it is published |
| U7 | `replacement_failure_never_deletes_first` | `FaultFs` fails `replace` | `Err(Io { stage: ReplaceTarget, target_state: Unchanged, .. })`; the log contains **no** `remove` of the *destination* at any point | contract: "Never delete, truncate, or move aside the destination first" |
| U8 | `parent_sync_failure_reports_durability_uncertain` | `FaultFs` fails `sync_parent_dir` | `Err(Io { stage: SyncParentDirectory, target_state: ReplacedDurabilityUncertain, .. })`; the log shows exactly **one** `replace` and no `remove` after it | the one non-`Unchanged` failure; no second replacement attempt |
| U9 | `cleanup_failure_does_not_mask_primary_error` | `FaultFs` fails `write_all` **and** fails `remove` | the returned error is still `Io { stage: WriteTemporary, .. }`, not a cleanup error | contract: cleanup never masks the primary failure |
| U10 | `bounded_read_refuses_oversize` | private `read_bounded(path, max)` called with `max = 16` against a 64-byte temporary file | `Err` with `ErrorKind::InvalidData` and fewer than `max + 1` bytes buffered | makes `MAX_PROJECT_FILE_BYTES` testable without writing 64 MiB |
| U11 | `ordering_is_validate_encode_create` | `FaultFs` that never fails | the first three log entries are `create_new`, `write_all`, `sync_file` in that order, and `replace` follows `sync_file` | the contract's required step order is pinned, not incidental |

There is deliberately **no** test for `SaveError::Encode`. With the current encoder a
validated envelope cannot fail to encode, so any such test would be a test that cannot
fail, which `criteria.md` 1G scores 0. The variant exists because the contract requires the
distinction; that it is currently unreachable is stated here rather than papered over with
a green check. Routed to §8 Q11.

There is also deliberately **no** test for the last item on the contract's fault-injection
checklist — *"simultaneous same-target operations are prevented by the app-level owner
rather than serialized by hidden project-crate state."* §4.4 discharges it structurally
(the crate holds no lock, no queue, and no target-keyed state, and `./spectre` calls the
synchronous function from its one update thread, so a second save cannot begin while the
first is running), and a test that spawns two threads to prove a single-threaded caller is
single-threaded would assert the test harness, not the product. This is named here, the
same way `EncodeSnapshot` is named, rather than left silently uncovered. It stops being
structural the moment saving leaves the UI thread, which is §8 Q10.

*Group B — structural guard (`crates/spectre-project/tests/render_path_isolation.rs`).*
Two assertions, not one, because §4.1's boundary has two halves and only the second one can
actually fire today.

| # | Name | Setup | Assertion | Edge covered |
|---|---|---|---|---|
| U12 | `render_path_crates_do_not_depend_on_the_project_crate` | read `../spectre-audio/Cargo.toml`, `../spectre-graph/Cargo.toml`, `../spectre-dsp/Cargo.toml`, `../spectre-core/Cargo.toml` relative to `env!("CARGO_MANIFEST_DIR")` | none of the four contains the string `spectre-project` | the compile-time half of RT-001's boundary. Same structural technique the RT module scan already uses in `crates/spectre-audio/tests/rt_guard.rs` |
| U12b | `the_apps_render_closure_module_names_no_persistence_api` | read `../spectre-app/src/engine.rs`, the module that builds the render closure | it contains **none** of `save_project_atomic`, `load_project`, `to_bytes`, `from_bytes`, `ProjectEnvelope`, `ProjectDoc`, `std::fs`; and — as a positive control that the file was actually read and the scan is looking at the right module — it **does** contain `build_track_graph` | the half that can fire. `spectre-app` depends on `spectre-project` and R4-4 made `engine.rs` a real consumer of it, so the four-manifest scan alone guards nothing here (§4.1). This fails the moment anyone reaches the filesystem from the crate that owns the callback closure |

*Group C — command and codec unit tests (`crates/spectre-project/src/command.rs`,
`src/lib.rs`).*

| # | Name | Setup | Assertion | Edge covered |
|---|---|---|---|---|
| U13 | `reorder_is_exactly_reversible` | `ProjectDoc` whose `TrackList` holds three tracks `[a, b, c]` built through `TrackList::push`; `EditHistory::new(8).unwrap()` (`command.rs:122`); apply `Transaction::single(ProjectCommand::reorder_tracks(a_id, 2))` (`command.rs:86`) | order becomes `[b, c, a]`; `undo` restores `[a, b, c]`; `redo` restores `[b, c, a]`; every `Track::id`, `name`, `level`, `muted`, and `soloed` is unchanged at every step | CORE-001 across reorder **and** undo, on the real command path. Distinct from R4-4's `a_track_keeps_its_identity_and_fields_across_a_reorder`, which calls `TrackList::reorder` directly and never enters a transaction or an undo stack |
| U14 | `reorder_out_of_range_is_rejected_before_mutation` | three tracks; `reorder_tracks(a_id, 7)`, then `reorder_tracks(unknown_id, 1)` | `Err(CommandError::Track(TrackError::IndexOutOfRange { index: 7, len: 3 }))` and `Err(CommandError::Track(TrackError::UnknownTrack(..)))`; the list is byte-identical to before in both cases, and `EditHistory`'s undo stack is still empty | `Transaction::execute`'s all-or-nothing property (`command.rs:92–109`) survives the new command, and a refused edit does not enter history |
| U15 | `duplicate_object_ids_are_rejected` | envelope where a `ParameterDoc.id` equals `project.id`; and separately one where two `Track`s share an id | `validate_envelope` returns `Err` for both, and so does `from_bytes` on the encoded form | the project-wide uniqueness rule the contract defers to "once persisted object collections exist" |
| U16 | `dangling_selection_is_rejected` | `view.selected_track = Some(id)` for an id in no `Track` | `validate_envelope` returns `Err` | referential integrity of restored view state |
| U17 | `non_finite_parameter_value_is_rejected_before_encoding` | `ParameterDoc.value = f32::NAN`, then `f32::INFINITY` | `validate_envelope` returns `Err` for both | JSON cannot represent them; catching them here is what keeps a corrupt file from existing |
| U18 | `signed_zero_and_subnormal_values_round_trip_bit_exactly` | `ParameterDoc.value` set to `-0.0` and then to `f32::from_bits(1)` | `to_bytes` → `from_bytes` yields values whose `to_bits()` are equal to the originals | the exact-value discipline the existing snapshot tests already hold the DSP boundary to. **This is an assumption the test settles, not a property this spec has evidence for**: it is a claim about the pinned `serde_json` float path, and no file in this repository proves it. `snapshot_constructor_preserves_signed_zero_and_subnormal_bits` (`crates/spectre-offline/tests/harness.rs:319`) proves the analogous property for the DSP boundary, which is why the discipline is worth extending here — but if the JSON path turns out not to hold it, that is a finding, and §8 Q11's honesty applies |
| U19 | `deserialization_cannot_smuggle_a_track_past_its_own_constructors` | hand-built JSON decoding into a `ProjectDoc` whose `TrackList` carries, in four separate cases: a blank `name`; a `level` of `f32::NAN`; a `level` above `GAIN_PARAMETERS[0].maximum()`; and `MAX_TRACKS + 1` tracks | `validate_envelope` returns `Err` in all four, and `from_bytes` does too | §4.2 rule 3. Each of the four is refused at runtime by `Track::new`, `Track::set_level`, or `TrackList::insert` and **not** by the derived `Deserialize`, so without this rule a hand-edited file reaches the model through a door the constructors close |

### 5.2 Integration tests

*`crates/spectre-project/tests/project_save.rs` — real filesystem, real temporary
directories.*

| # | Name | Assertion |
|---|---|---|
| I1 | `save_then_load_round_trips_a_populated_project` | `save_project_atomic` returns `Ok(SaveReceipt { target_state: TargetState::ReplacedDurable })`; `load_project` returns an envelope `==` the snapshot (`ProjectEnvelope`'s derive list in `src/lib.rs` includes `PartialEq`, and `TrackList`'s does too, so the whole document compares) |
| I2 | `successful_save_leaves_no_temporary_behind` | after a save into a directory containing only the target, `read_dir` yields exactly one entry, and its name is the target's | the contract's "no temporary path remains after successful replacement" |
| I3 | `save_over_an_existing_project_replaces_it_completely` | write project A, save project B over it, load: the result is B, and the file contains no fragment of A (byte comparison against `to_bytes(&b)`) | replacement, not merge |
| I4 | `invalid_snapshot_leaves_an_existing_target_byte_identical` | save A; attempt to save an invalid snapshot; the target's bytes are `==` the bytes read immediately after saving A, and `read_dir` still yields exactly one entry | pre-commit failure on a real filesystem |
| I5 | `save_into_a_missing_directory_fails_and_creates_nothing` | `Err(SaveError::Io { stage: SaveStage::CreateTemporary, target_state: TargetState::Unchanged, .. })`, and the target path still does not exist | "a previously absent target remains absent" |
| I6 | `reorder_preserves_identity_across_save_and_reload` | build three tracks; save; apply `reorder_tracks(first_id, 2)` through `EditHistory`; save; load — the loaded `tracks()` are in the reordered order and the `ObjectId`s are the same three values as before, in the new positions, with `name`/`level`/`muted`/`soloed` intact on each | **CORE-001's remaining R4 evidence: the persisted half.** R4-4 proved identity survives `TrackList::reorder` in memory; this proves it survives the reorder *and the file*. Fails if identity is ever derived from index, and fails if the encoder writes position where it should write identity |
| I7 | `undone_reorder_reloads_in_the_original_order` | continue I6: `undo`, save, load — original order, same IDs | reorder + undo + save/load in one sequence, which is the full clause CORE-001 states |
| I8 | `schema_one_fixture_loads_with_empty_collections` | `load_project` on a copy of `tests/fixtures/r1-canonical.json` succeeds; `tracks` is an empty `TrackList` with `master_level == GAIN_PARAMETERS[0].default()` and `structure_revision == 0`; `devices` is empty; `id_gen_state` is 0; `project.id.raw() == 1_311_768_467_463_790_320`; the `r1_extension` and `future_session` unknown fields are still present | tolerant read of the older schema; CORE-003's preservation guarantee still holds. The `master_level` assertion is what proves `TrackList`'s `Default` — not `Vec::default` plus a zero float — is what `#[serde(default)]` reaches |
| I9 | `resaving_a_loaded_schema_one_project_keeps_its_version_and_unknown_fields` | load a copy of the v1 fixture, hand the returned envelope straight to `save_project_atomic`, reload: `schema_version` is still **1**, `tracks` and `devices` are still empty, `id_gen_state` is still `0`, and `r1_extension` / `future_session` are intact | the writer's half of the version rule — `save_project_atomic` reproduces the snapshot's own `schema_version` and never rewrites it (§4.3) — plus CORE-003's unknown-field preservation across a real filesystem round trip. Fails if anyone puts a stamp in the writer |
| I10 | `a_newer_schema_file_is_refused_and_not_rewritten` | a file whose `schema_version` is `MAX_READABLE_SCHEMA + 1`: `Err(LoadError::SchemaTooNew { .. })`, and the file's bytes are unchanged afterward | failing closed is what protects the file |

*`crates/spectre-app/tests/project_io.rs` — model mapping. This test file is the first
code in `spectre-app` to use the `spectre-project` dependency its manifest already declares.*

| # | Name | Assertion |
|---|---|---|
| I11 | `model_envelope_round_trip_preserves_identity_order_and_edited_values` | `AppModel::prototype()` + two `add_track` calls + `set_device_parameter("saturator", "drive", 6.0)` + `set_track_level(first_id, 0.5)`; `project_envelope(&model, "T")` → `to_bytes` → `from_bytes` → `adopt` into a fresh `AppModel::prototype()`. Assertions: `tracks()` match by id, name, and order — the fresh target holds **one** track before the call and **three** after; the first track's `level()` reads `0.5` and `0.5 != GAIN_PARAMETERS[0].default()` (`1.0`) is asserted in the same test; the target's `saturator` `drive` reads `6.0`; and `6.0 != SATURATOR_PARAMETERS[0].default()` (`1.0`) is asserted in the same test so the discriminating property is explicit | CORE-001 across save/load for tracks, plus two assertions that can actually fail: an `adopt` that ignored the persisted devices would leave `drive` at its descriptor default, and one that rebuilt tracks from `Track::new` rather than adopting them would leave `level` at unity. `6.0` lies inside `SATURATOR_PARAMETERS[0]`'s `1.0..=24.0` and `0.5` inside `GAIN_PARAMETERS[0]`'s `0.0..=2.0`, so neither is clamped away before it can discriminate |
| I12 | `adopting_a_project_stops_the_transport` | envelope whose transport state is `Playing`; after `adopt`, `model.is_playing()` is `false` | opening a file never starts playback — and, post R4-1, never starts audio |
| I13 | `a_failed_adopt_leaves_the_model_untouched` | envelope with `DeviceDoc.key = "not-a-device"`: `Err(AdoptError::UnknownDevice { .. })`, and `tracks()`, `devices()`, `selected_track_id()`, `selected_device_id()`, `lens()`, and `track_list().master_level()` are all identical to before the call | the contract's "keep the current live project unchanged until that result is available and accepted" |
| I14 | `out_of_range_parameter_values_are_refused_not_clamped` | a `ParameterDoc.value` above its descriptor maximum: `Err(AdoptError::ValueOutOfRange { .. })` | a file must not be able to smuggle a value the UI cannot produce; refusing beats silently clamping on the load path |
| I15 | `an_app_built_snapshot_carries_the_current_schema_version` | `project_envelope(&AppModel::prototype(), "T")` returns an envelope whose `schema_version` is `SCHEMA_VERSION`; saving it with `save_project_atomic` and reloading returns `SCHEMA_VERSION` again | the version stamp has exactly one owner — the snapshot builder (§4.3). With I9 this pins both halves of the rule: the builder stamps, the writer does not. Fails if the stamp is moved, dropped, or duplicated |

**Why I11 mutates a parameter and a track level before the round trip.**
`AppModel::prototype()` seeds `IdGen` with a fixed constant and mints every device and
parameter `ObjectId` **before** any `add_track`; `DeviceControl::from_descriptors` sets
`value: descriptor.default()`; and `Track::new` sets `level` to
`GAIN_PARAMETERS[0].default()`. A source model and a fresh target model therefore agree on
device `instance_id`s, on every parameter value, and on every track level *by construction*.
Asserting that they match would pass even if `adopt` ignored the persisted devices
entirely — a test that cannot fail, which `criteria.md` 1G scores 0. The edited `drive` and
`level` values are what make those halves of I11 discriminating. The `instance_id` equality
is deliberately **not** asserted as evidence: it would be a consistency check that cannot
fail, and making it discriminate would need a seeded `AppModel` constructor this slice does
not add and does not need.

*`crates/spectre-offline/tests/harness.rs` — determinism. This file already has
`spectre-app` as a dev-dependency (`crates/spectre-offline/Cargo.toml:20–21`), so no new
dependency is introduced. It is also the file whose `first.schema_version` assertion the
schema bump breaks — see §7.2.*

| # | Name | Assertion |
|---|---|---|
| I16 | `a_save_reload_round_trip_does_not_change_what_the_project_renders` | build `AppModel::prototype()` and **edit it away from its defaults first** — `set_device_parameter("saturator", "drive", 6.0)` and `set_device_parameter("gain", "gain", 0.5)`; take `model.device_parameter_snapshot()`; render with `spectre_offline::render_app_snapshot(48_000.0, 256, &snapshot)` (`crates/spectre-offline/src/lib.rs:295`); round-trip the model through `project_envelope` → `to_bytes` → `from_bytes` → `adopt` into a **fresh** `AppModel::prototype()`; take that model's snapshot; render again — `RenderReport.hash` is **equal**, `peak` is nonzero, and the same test asserts the edited render's hash **differs** from an unedited `AppModel::prototype()`'s | Determinism, using the **existing** FNV-1a hash walk in `spectre-offline` rather than a new comparison method. Three separate guards against a vacuous pass: the edit means an `adopt` that dropped the devices would render descriptor defaults and change the hash; the fresh target means the round trip is actually exercised rather than compared against itself; and the nonzero peak plus the differs-from-default assertion together prove the two renders are not agreeing because both are silent or both are the default chain. Iteration 2's version had none of the three and could not fail |

### 5.3 UI / E2E tests

No automated UI test exists or is proposed. `./spectre` has no GUI harness: the only
process-level test is the headless `--smoke-test` path (`smoke_test()`, invoked from `main`
before any window opens), which builds a whole `SpectrePrototype` and prints one line
carrying `lens=`, `tracks=`, `transport=`, `selected_device=`, and `engine=`. This spec
extends that line with `project=none` so `crates/spectre-app/tests/smoke_cli.rs` asserts the
shell starts with no project loaded — a real assertion that fails if launch state changes,
and the honest limit of what can be automated here. **It must be derived, not a literal.**
R4-1's remediation (`ab521a9`) found and fixed exactly that defect on the `engine=` field —
it was a hard-coded string, so the assertion guarding the headless path could not fire, and
`smoke_test` now records the rule in a comment above its own `println!`: *"The engine field
is DERIVED from the shell's own engine, not written as a literal, so the assertion in
tests/smoke_cli.rs actually fails if startup is wired into the headless path."* `project=` must be read from
the shell's own project state for the same reason, and the review that lands this slice must
check that it is.

Everything in §3 that lives in `main.rs` — the button enablement, the two-press discard,
the status-line text, the dirty marker — is therefore **manual-only at R4**, exactly as
`main.rs` code is today. That is a stated limit, not a hidden one; R4-9 owns the
end-to-end fixture and the written QA protocol, and building a UI harness belongs there.

### 5.4 Visual / manual verification

Run `./spectre` (`spectre:9` → `cargo run --locked --quiet -p spectre-app`) on macOS and
on Linux, and record both.

| Configuration | What to confirm |
|---|---|
| Empty state | The PROJECT block shows the empty-state copy, both buttons are disabled, and their hover text explains why |
| Happy path | Type a path in an existing writable directory, add two tracks, save, quit, relaunch, open — same tracks, same order, same names, same selected track, same lens |
| Dirty marker | The transport bar reads `unsaved` after any edit and stops after a successful save; the word, not just the colour, changes |
| Durability-uncertain copy | If the message is ever produced on real hardware, confirm it does **not** read as an ordinary success and that the project stays dirty |
| Read-only directory | `chmod a-w` the parent, save, confirm the message names the stage and confirm the old file is byte-identical afterward |
| Nonexistent directory | Save into `/nope/x.spectre`, confirm the message and that nothing is created |
| Newer schema | Hand-edit `schema_version` to 3, open, confirm the refusal message names both numbers and the file is unchanged on disk |
| Discard guard | Edit, press `Open project`, confirm the button becomes `Discard and open` and that one press does not discard |
| **Timing** | Time the save on both platforms with a warm and a cold page cache; record the numbers in the R4-9 QA protocol. §4.7's figures are estimates until this runs |
| Text size / window extremes | At the 1060 × 680 minimum and with a large system text scale, the PROJECT block still fits the 180 px minimum panel width and the status line wraps rather than clipping |
| Theme variants | **N/A** — the shell defines one dark palette as seven `Color32` constants in `main.rs` and no light theme exists. Nothing in this slice introduces one |
| **Honesty check** | Confirm no copy anywhere claims the save is crash-proof, recoverable, or autosaved. None of those is true at R4 |

---

## 6. Compliance & Safety Gate

### 6.1 Sensitive data classification

- [ ] No sensitive data involvement
- [x] **Handles sensitive data — describe protection measures**
- [ ] Uses synthetic/test data only until compliance gate clears

Not "sensitive" in a regulatory sense — no personal, health, financial, or credential
data. But the data this feature handles is **the user's creative work and their filesystem
paths**, which is the most valuable thing the product touches, and the vision calls losing
it *"a product-killing defect"* (`vision.md:33`). Protections: the project file is written
only where the musician typed; nothing is copied, uploaded, or cached elsewhere; no
network call exists anywhere in this feature; paths and OS error strings appear only in the
in-app status line and are never written to any log, telemetry sink, or crash report,
because no such sink exists in this workspace and this spec adds none. Existing file
permissions are not implicitly preserved through the replacement — the design authority
states this outright and defers preservation to a later accepted contract — so a save
creates the new file with the process default. That is a deliberate, documented limitation
and is repeated in §8 Q3 so it is a decision rather than an accident.

### 6.2 Asset provenance

- [x] **No third-party assets.** No models, images, fonts, samples, or data files. The one
  new checked-in file is `tests/fixtures/r4-canonical.json`, generated by Spectre's own
  encoder from Spectre's own types, in the same manner as the existing
  `r1-canonical.json`. No format, schema, field name, or file layout is taken from any
  other product; the envelope is Spectre's own design under decision 3
  (`decision-gates.md:27`).

### 6.3 Language / claims audit

- [ ] Makes claims not supported by evidence
- [ ] Promises capabilities not yet built
- [ ] Uses language restricted by domain regulations

Audited line by line. Three user-visible strings were written specifically to avoid
overclaiming:

1. The empty-state copy says the replacement is atomic — true of the code this slice
   ships, and it ships in the same slice, so it is never a promise about a future build.
   It says nothing about crashes.
2. The `ReplacedDurabilityUncertain` message deliberately does **not** say "Saved." It
   says saved-but-not-confirmed-durable and keeps the project dirty. Reporting that state
   as a plain success is forbidden by the contract and would be a fake surface.
3. No string anywhere says "autosave", "recovered", "backup", or "safe". None of those
   exists at R4.

This spec also states three negatives about the current build that the product copy must
respect. First, this slice adds no audio path, so no copy here may borrow R4-1's or R4-2's
engine as evidence for persistence — and, symmetrically, no copy here may claim or deny
that `./spectre` produces sound, because that is R4-1's line and R4-1 holds it as
`implemented`, not `verified` (`docs/status/STATUS.md:39`: *"no one has confirmed by ear
that sound leaves the speakers"*). Second, R4-6's `Filament` and `Gloam` are not reachable
from a track at `1960a8b` (`STATUS.md:59`), so no copy may imply that saving a project
captures a voice the engine can play. Third, no screen-reader claim is authorized while
`eframe`'s features are as declared in `crates/spectre-app/Cargo.toml`.

### 6.4 Regulatory alignment

Lens 3 of `gauntlet-output/criteria.md`, criterion by criterion:

- **3A Milestone fit.** `save/reload` is one of the six R4 scope items
  (`current-milestone.md` scope line; `vision.md:48`) and this is `NEXT.md` slice 7
  verbatim. Everything beyond it is deferred by name in §7.4, not smuggled: journaled
  autosave, recovery selection, migrations, salvage, missing-media diagnostics, and crash
  qualification are all R5 by the design authority's own milestone table and by
  `rebuild-roadmap.md:30`.
- **3B Non-goal respect.** No plugin hosting of any format, no first-party device
  published as a plugin, **no cross-DAW project or preset compatibility of any kind**, no
  cloud, no content store, no video. `vision.md:56` states the prohibition and this spec
  proposes nothing in that direction: the file format is Spectre's own, it imports nothing,
  it exports nothing, and no import/export affordance appears anywhere in §3.
- **3C Deliberately small first devices.** No device is added, grown, or given a preset
  system. What is persisted is the **eleven** existing parameter values across the **five**
  existing device instances that `AppModel::prototype()` builds at `1960a8b` — `pulse`,
  `gain`, `saturator`, and R4-6's `filament` and `gloam`. Iteration 2 said four values
  across three instances, which was true before R4-6 landed and is not true now; the number
  changed because R4-6 shipped, not because this spec grew anything. Decision 15
  (`decision-gates.md:39`) is untouched, and `TrackInstrument` still has one variant, which
  this spec does not add to (§7.4, §8 Q14).
- **3D Originality.** Schema, field names, error vocabulary, and algorithm are Spectre's.
  The atomic write-temp-fsync-rename-fsync-dir sequence is a POSIX idiom, not a product's
  intellectual property, and it is specified here from the accepted contract rather than
  transcribed from any tool. The two numeric bounds have Spectre-derived rationales
  (§4.2) and proposed ledger rows (§7.2); neither is copied. A third bound iteration 2
  proposed is withdrawn rather than justified, because `requirements-ledger.md:69–71`
  already ruled that the track fader's range is not a new number.
- **3E Platform commitment.** §4.6 treats macOS and Linux as co-first-class, names the
  concrete divergence (`fsync` versus `F_FULLFSYNC`; directory-sync guarantees), refuses
  to assume the macOS behavior on Linux or the reverse, and marks the one external claim
  it cannot verify from this repository as unverified. §5.4 requires both platforms.
  Decision 23's device debt is untouched by this slice and is not claimed as discharged.
- **3F Accessibility trajectory.** Every element is a standard labelled widget with a
  defined focus order and no colour-only state (§3.7). No screen-reader support is
  claimed, because the manifest disables the feature that would provide it; enabling it is
  routed to the R4 audit under decision 17.

---

## 7. Gap Analysis vs. Current State

### 7.1 What exists today

**Baseline, stated exactly, because the tree moved substantially between iterations 2 and
3.** Iteration 1 read `2e005e5`; iteration 2 read `8b1633d`. Iteration 3 re-opened **every**
claim below at the committed HEAD **`1960a8b`**, which is `8b1633d` plus five commits:
`20f3056` (R4-2, runtime parameter seam), `ab521a9` (R4-1 remediation), `41a6be2` (R4-4,
track model), and `0d2a03c` + `1960a8b` (R4-6, `Filament` and `Gloam`).

**Three statements iteration 2 made about the tree are now false. They are corrected here
rather than carried forward, and each correction is named so a reviewer can check that the
correction, not the original, is what this section relies on:**

1. Iteration 2 said `crates/spectre-app` "declares `spectre-project` and never uses it",
   and pinned `crates/spectre-app/Cargo.toml:16` for the dependency. Both halves are wrong
   at `1960a8b`. The dependency is **used** — see below — and the line pin was never
   right at `8b1633d` either: at both `8b1633d` and `1960a8b` line 16 of that manifest is
   `live-audio = ["spectre-audio/cpal-backend"]`, inside the `[features]` block R4-1 added.
   The dependency line is `:24`. Those two numbers appear here only to *state the
   correction*; no claim anywhere else in this document pins a line in that manifest.
   Every reference to it quotes the literal instead, which is what §3.7 already does
   correctly for `eframe`, and for the same reason: R4-1 inserted a `[features]` block
   above the dependencies, so every line below it moved and will move again.
2. Iteration 2 pinned six line ranges into `crates/spectre-app/src/lib.rs` and asserted
   they were `8b1633d` values. They were `2e005e5` values, off by the `+3` the same section
   computed. R4-4 has since rewritten that file wholesale — `TrackView` is **deleted** —
   so re-pinning would only reset the same clock. **Every citation into
   `crates/spectre-app` and into `crates/spectre-project/src/lib.rs`, `track.rs`, and
   `routing.rs` — the modules two other slices are actively editing — is now by symbol
   name.** Symbols survive a shift; line numbers do not. Pins are retained only in
   `crates/spectre-project/src/command.rs` and `crates/spectre-project/tests/`, which have
   not changed since `6c397d9`, and each of those was re-opened at `1960a8b`.
3. Iteration 2's §7.2 said `default_project()` has one consumer and told STATUS to preserve
   a "`./spectre` makes no sound" line. Both are corrected in §7.2 itself.

Line pins are kept **only** where the file is genuinely stable, and each surviving pin was
re-opened at `1960a8b`: `crates/spectre-core/src/id.rs` and `transport.rs` (unchanged since
`6c397d9`), `crates/spectre-dsp/src/parameter.rs` (same), `crates/spectre-offline/src/lib.rs`
and `tests/harness.rs`, `crates/spectre-project/tests/` and its fixture (unchanged since
`6c397d9`), and `docs/`.

**Absent — this is the gap R4-7 closes.**

- **No persistence code exists anywhere in `spectre-project`.** At `1960a8b` the crate is
  four source modules — `src/lib.rs` (226 lines), `src/command.rs` (182),
  `src/routing.rs` (226), `src/track.rs` (292) — and four test files plus one fixture.
  There is no `fs` module, no `load_project`, no `save_project_atomic`, no `SaveStage`, no
  `TargetState`, no `SaveReceipt`, and **not one occurrence of `std::fs` or `std::path` in
  any of the eight `.rs` files**, which is the durable form of the claim and is what a
  single grep settles. The only file I/O in the crate is `include_bytes!` of a checked-in
  fixture at `tests/project_codec.rs:9`, which is compile-time.
- **No persistence code exists anywhere in `spectre-app` either, and this is now the
  precise statement rather than iteration 2's cruder one.** `crates/spectre-app/Cargo.toml`
  declares `spectre-project = { path = "../spectre-project" }` in `[dependencies]` — quoted,
  not line-pinned, because R4-1 restructured that manifest and later slices may again — and
  the crate **does** use it: `src/lib.rs` imports `Track`, `TrackError`, `TrackInstrument`,
  and `TrackList`; `src/engine.rs` names `RoutingError`, `TrackList`, `TrackPathNodes`,
  `build_track_graph`, and `track_device_factory`; `src/main.rs` names `TrackList`. What is
  absent is the entire persistence surface: a grep across `crates/spectre-app/src/` and
  `crates/spectre-app/tests/` for `ProjectEnvelope`, `ProjectDoc`, `to_bytes`, `from_bytes`,
  `SCHEMA_VERSION`, `EditHistory`, `Transaction`, `ProjectCommand`, `std::fs`, and
  `std::path` returns **nothing**. The app reaches this crate for its track model and its
  graph builder, and for nothing that touches a file.
- **`main.rs`'s own header says so, and says it about this slice by name.** Its Notes block
  reads *"Persistence wiring remains out of scope until R4-7."* There is no menu bar, no
  file dialog, no path field, and no save or open button in the shell.
- **No persisted object collection exists.** `ProjectDoc` has exactly four typed fields —
  `id`, `name`, `tempo_map`, `transport` — plus the flattened unknown map. There is no
  `tracks`, no `devices`, no `view`, and no `id_gen_state`.
  `crates/spectre-core/src/id.rs:4` records the consequence in the source itself:
  *"project-wide duplicate validation arrives with persisted object collections."*
- **R4-1, R4-2, R4-4, and R4-6 have all landed since iteration 1, and they narrow the gap
  without closing any of it.** The app owns a live engine (`src/engine.rs`), a consumed
  RT-002 parameter lane, an ordered `TrackList`, and five device instances. None of that
  reaches a file. This spec makes **no claim in either direction about whether `./spectre`
  produces sound**: that sentence belongs to R4-1, and `docs/status/STATUS.md:39` holds it
  as `implemented`, not `verified` — *"no one has confirmed by ear that sound leaves the
  speakers."*

**Implemented — `spectre-project`, all of it in memory.** Cited by symbol; every item below
is in `crates/spectre-project/`.

| Element | Where | What it actually does |
|---|---|---|
| `SCHEMA_VERSION = 1`, `MAX_READABLE_SCHEMA = 1` | `src/lib.rs` | schema identifiers |
| `ProjectEnvelope`, `ProjectDoc`, each with a `#[serde(flatten)]` unknown map | `src/lib.rs` | CORE-003's forward-field preservation |
| `ProjectDoc::meter_map` | `src/lib.rs` | decodes optional meter state out of the unknown map |
| `ProjectError { SchemaTooNew, InvalidProject(&'static str), Malformed(serde_json::Error) }` | `src/lib.rs` | three of the five distinctions the load contract needs; it has **no** I/O variants, because there is no I/O |
| `to_bytes` | `src/lib.rs` | `serde_json::to_vec_pretty`. **Does not revalidate** — the design authority's "Current codec qualification" section says so explicitly |
| `from_bytes` | `src/lib.rs` | peeks the version through a local `VersionPeek`, gates it against `MAX_READABLE_SCHEMA`, decodes, then calls `validate` |
| `validate` — **private free function** | `src/lib.rs` | nonzero project ID, tempo-map reconstruction, meter-map decode, loop-enabled-requires-region, ordered loop region. Not callable from outside the crate |
| `ProjectCommand::rename`, `Transaction`, `EditHistory` | `src/command.rs:43`, `:72–110`, `:114–182` | in-memory command transactions with generated inverses (`Transaction::execute`, `:92–109`) and bounded undo/redo with oldest-edit eviction (`push_bounded`, `:176–181`). `command.rs:4` calls it the *"App-thread project mutation seam; never callback-reachable"*. `command.rs` is unchanged since `6c397d9`, which is why it keeps its pins |
| **`Track`, `TrackList`, `MAX_TRACKS`, `TrackError`, `TrackInstrument`** — R4-4 | `src/track.rs` | the ordered collection this feature persists. Both types **already derive `Serialize` and `Deserialize`**. `TrackList` owns order, `insert` refuses a duplicate `ObjectId` and a count past `MAX_TRACKS`, `reorder(id, to_index)` returns the origin index, `effective_gain` composes level × mute × solo, and `structure_revision` is `#[serde(skip)]` with a comment naming this slice: *"a reload does not inherit a stale revision and R4-7 persists nothing about it"* |
| **`build_track_graph`, `track_device_factory`, `TrackPathNodes`, `RoutingError`** — R4-4 | `src/routing.rs` | app-thread construction of the instrument → track gain → sum → master graph. Not persisted and not touched here; listed so §7.2's "deliberately not modified" is complete |

**Verified — the R1 codec fixtures.** `crates/spectre-project/tests/fixtures/r1-canonical.json`
(52 lines, 912 bytes, `schema_version: 1` at `:2`, project id `1311768467463790320` at
`:4`, a `future_session` unknown block at `:43–46` and an `r1_extension` unknown block at
`:48–51` — all re-measured at `1960a8b`), driven by **eight** tests in
`crates/spectre-project/tests/project_codec.rs` — the file is 134 lines and `#[test]` appears
at `:21, :52, :60, :69, :85, :100, :112, :124`: decode with representative state (`:21–50`),
byte-stable rewrite (`:52–58`), decode-encode-decode round trip (`:60–67`), unknown-field
survival (`:69–83`), newer-schema rejection (`:85–98`), and invalid tempo/meter/loop
rejection (`:100–110`, `:112–122`, `:124–134`). Five more codec unit tests live in the
crate's own `#[cfg(test)] mod tests` in `src/lib.rs`, and **five** command tests live in
`tests/command_history.rs` (121 lines; `#[test]` at `:26, :50, :69, :91, :111`). R4-4 added
two further test files — `tests/track_model.rs` and `tests/track_routing.rs`, **five**
`#[test]`s each — bringing the crate to **28** tests at `1960a8b`. CORE-003 is `verified` in
the ledger (`:48`) on the codec evidence. **None of these tests opens a file at runtime**;
`cargo test --locked -p spectre-project` was run for this iteration and all 28 pass.

**Implemented — what the app has that would need to persist. This block is the one R4-4
rewrote hardest, and iteration 2's version of it no longer describes any file in the tree.**

- `AppModel` still has eight fields, but two of them changed type or meaning: `transport`,
  `lens`, **`tracks: spectre_project::TrackList`**, `devices: Vec<DeviceControl>`,
  `selected_track: Option<ObjectId>`, `selected_device: Option<ObjectId>`, `ids: IdGen`,
  `feedback: String`.
- **`TrackView` no longer exists.** R4-4 deleted the app's own track type and replaced it
  with `spectre_project::Track`. This is the single most consequential change for R4-7:
  the collection it set out to invent a document type for is **already** in the document
  crate and **already** serde-derived (§4.2). What remains missing is every link between
  that collection and a file.
- `AppModel` exposes `track_list()` (borrows the whole `TrackList`), `tracks()`,
  `selected_track()`, `add_track`, `remove_track`, `reorder_track`, `rename_track`,
  `set_track_level`, `set_track_muted`, `set_track_soloed`, `set_track_instrument_level`,
  and `set_master_level`. Iteration 2's `selected_track_mut` is gone.
- `DeviceControl` and `ParameterControl` each still carry an `instance_id: ObjectId` minted
  from the same generator. `DeviceControl::from_descriptors` seeds every parameter with
  `descriptor.default()`, which is why I11 and I16 must edit before they compare.
- **`AppModel::prototype()` now builds five device instances, not three.** R4-6 appended
  `filament` and `gloam` after `pulse`, `gain`, and `saturator` — and the source says why
  the position matters: *"Appended rather than inserted: `prototype()` allocates every
  identity from one seeded `IdGen` in declaration order, so a new device in the middle would
  renumber every device after it and break the offline fixture's stable IDs."* Across the
  five, `PULSE_PARAMETERS` (1), `GAIN_PARAMETERS` (1), `SATURATOR_PARAMETERS` (2),
  `FILAMENT_PARAMETERS` (4), and `GLOAM_PARAMETERS` (3) give **eleven** parameter values to
  persist. It also builds **one** track, `"Pulse"`, which `smoke_cli.rs:18` pins as
  `tracks=1`.
- `device_parameter_snapshot` still publishes exactly **four** validated
  `DeviceParameterSnapshot` values — `pulse[0]`, `gain[0]`, `saturator[0]`, `saturator[1]`.
  That number did **not** change with R4-6, and the reason is recorded in
  `crates/spectre-offline/src/lib.rs:382–384`: the four values are the offline contract
  `spectre-app` publishes, and appending to it would change that contract. The two counts
  are therefore different on purpose: **eleven** parameter values exist and are persisted;
  **four** are the offline render fixture. Conflating them would misdescribe both.
- `IdGen` (`crates/spectre-core/src/id.rs:45–68`) is deterministic splitmix64 seeded by
  `AppModel::prototype()` with the fixed constant `0x0047_4549_5354_5549`; it exposes
  `next_id` (`:56–67`) but **no accessor for its state**, so its position cannot currently
  be saved. That one-method gap is the whole of this spec's `spectre-core` delta.

**Implemented — CORE-001's reorder evidence, half of it.** `TrackList::reorder` preserves
identity and every field, and `crates/spectre-project/tests/track_model.rs` proves it in
five tests, one of which is `a_track_keeps_its_identity_and_fields_across_a_reorder` (`:27`).
That file's own header (`:4`) draws the line this spec has to respect: *"CORE-001's reorder
half is discharged here; its persistence half waits for R4-7."* The ledger's wording is
*"preserved across save/load, undo, reorder, and migration"* and its R1 disposition gates
the reorder evidence on *"the first persisted object collection"* — so what remains
undischarged is exactly the word *persisted*, and I6/I7 are what discharge it.

**Accepted but not implemented — the design this spec implements.**
`docs/03-architecture/project-persistence.md` is `accepted for R4/R5 implementation` and
specifies the two signatures, the load ordering, the save algorithm's **eight** ordered
steps (`project-persistence.md:120–127`; §4.3's table has nine rows because it splits the
contract's step 5 into a sync row and a handle-release row — that ninth row is this spec's
decomposition, not the contract's), the failure vocabulary, the per-stage target-state table, the platform-qualification split,
and the private test seam. Its own §"Milestone ownership" says: *"**R1:** accept this API,
state machine, and failure contract. No filesystem API is implemented in this slice."*
Nothing in the repository implements any of it.

**Gated — deliberately not here.**

- **R5 owns** crash injection, parent-directory durability evidence, journaled autosave,
  recovery selection, migrations, salvage policy, missing-media diagnostics, and the
  supported filesystem matrix — the design authority's milestone table and
  `rebuild-roadmap.md:30`.
- **Decision 14** (`decision-gates.md:38`) adopts journaled autosave to a sidecar with the
  recovery drill required at R5 exit. Not this slice.
- **Decision 13** (`decision-gates.md:37`) is `SD adopted` and gates at R5 exit; undo
  exists in the crate but is **not wired into `./spectre`** — no `EditHistory` is
  constructed anywhere outside tests.
- **Decision 23** (`:49`): Linux device qualification is undischarged. Unaffected by this
  slice and not claimed.

### 7.2 Delta to spec

**New files**

- `crates/spectre-project/src/fs.rs` — the two boundaries, the five error/state types, the
  private `FsOps` seam with `RealFs`, `save_with`, `read_bounded`, the two constants, and
  Group A + U10 unit tests.
- `crates/spectre-project/tests/project_save.rs` — I1–I10.
- `crates/spectre-project/tests/render_path_isolation.rs` — U12 and U12b.
- `crates/spectre-project/tests/fixtures/r4-canonical.json` — schema-2 canonical fixture
  with three tracks, the **five** device instances `AppModel::prototype()` builds at
  `1960a8b`, their **eleven** parameter values, a non-default `master_level`, and a view
  block. Generated by Spectre's own encoder, as `r1-canonical.json` was.
- `crates/spectre-app/src/project.rs` — `project_envelope`, `adopt`, `AdoptError`, the two
  `Lens ↔ LensDoc` conversions.
- `crates/spectre-app/tests/project_io.rs` — I11–I15.

**Modified files**

- `crates/spectre-project/src/lib.rs` — `pub mod fs;`; `SCHEMA_VERSION` and
  `MAX_READABLE_SCHEMA` to 2; the four new `ProjectDoc` fields, one of which is
  `tracks: TrackList` reusing the type `track.rs` already exports rather than a new
  document type (§4.2); `DeviceDoc`, `ParameterDoc`, `ViewDoc`, `LensDoc`;
  `ValidationError`; the private `validate` becomes `pub fn validate_envelope` returning
  `ValidationError`, with the existing five rules preserved verbatim and the four schema-2
  rules added; `from_bytes`'s call site adapts. `to_bytes` and `from_bytes` keep their
  **signatures**. The in-crate test helper `envelope()` in `#[cfg(test)] mod tests` contains
  a `ProjectDoc` literal and gains the four fields. U15–U19 added to the module's test block.
  *(No line pins: this file gained `pub mod routing;`/`pub mod track;` and two re-export
  lines in R4-4 and is being edited again by a concurrent slice.)*
- `crates/spectre-project/src/command.rs` — `CommandKind::ReorderTracks { id, to_index }`;
  `ProjectCommand::reorder_tracks(id, to_index)`, applied through the existing
  `TrackList::reorder` and inverted from the index that call returns;
  `CommandError::Track(TrackError)` joining the three variants at `:11–15`; U13–U14.
  `command.rs` is unchanged since `6c397d9`, so its pins are kept and were re-checked.
- `crates/spectre-core/src/id.rs` — `IdGen::state()` accessor. One method, no behavior
  change; `next_id` (`:56–67`) and `IdGen::new` (`:51–53`) untouched. This file is unchanged
  since `6c397d9`.
- `crates/spectre-project/tests/project_codec.rs` — **two of its eight tests change; six
  are unchanged.** (1) `canonical_fixture_decodes_with_representative_r1_state` asserts
  `envelope.schema_version == SCHEMA_VERSION` at `:25`; the fixture is a v1 document, so
  that assertion is false the moment the constant is 2 and becomes an assertion against the
  literal `1`. (2) `canonical_fixture_rewrite_is_byte_stable` (`:52–58`) **cannot survive
  this change by construction** — and the reason is *not* a version stamp, which no
  component performs (§4.3). It is that `id_gen_state`, `tracks`, `devices`, and `view` are
  plain `#[serde(default)]` fields with no `skip_serializing_if`, so `to_bytes` emits all
  four on **every** encode, including the rewrite of a v1 document whose `schema_version`
  the writer leaves at `1`. The encoded bytes differ from the checked-in golden regardless
  of what `SCHEMA_VERSION` holds. It is retargeted to the new `r4-canonical.json`, and its
  v1 role is taken by I8 and I9. The other six — decode-encode-decode round trip
  (`:60–67`), unknown-field survival (`:69–83`), newer-schema rejection (`:85–98`, which
  compares against `MAX_READABLE_SCHEMA + 1` and so is version-relative), and the three
  invalid tempo/meter/loop rejections (`:100–110`, `:112–122`, `:124–134`) — pass unchanged
  under the bump and under the four new validator rules, because the v1 fixture has one
  `ObjectId`, no collections, and no view. **This touches the acceptance evidence of a
  `verified` requirement (CORE-003, `requirements-ledger.md:48`), so it is flagged here and
  routed to §8 Q2 rather than done quietly.**
- `crates/spectre-app/src/lib.rs` — `pub mod project;`. No `AppModel` field, no new
  accessor, and no method-semantics change: `project.rs` is a descendant of the crate root
  and reaches `AppModel`'s private fields directly (§4.3).
- `crates/spectre-app/src/main.rs` — the PROJECT block inside `track_list`;
  `project_path`, `dirty`, and `status` on `SpectrePrototype` and on its `Default` impl,
  beside the seven fields it already carries; the dirty marker inside `transport`, placed on
  the leading side beside the `SPECTRE` wordmark so it displaces neither R4-1's engine
  status cluster nor R4-4's `Rebuild engine` / `PLAN STALE` pair; dirty-setting at the
  existing mutation call sites (§4.4 lists them); `project=` added to `smoke_test`'s single
  println — **derived from the shell's own state, never a literal** (§5.3) — and the header
  Notes line, which currently reads *"Persistence wiring remains out of scope until R4-7"*
  and stops being true when this slice lands. **No line in this file is pinned anywhere in
  this spec**: R4-1 restructured it by 216 lines, R4-2 and R4-4 moved it again, and it is
  870 lines at `1960a8b`.
- `crates/spectre-project/tests/command_history.rs` — **a compile break, not a behavior
  change.** Its `project()` helper (`:13–24`) builds a `ProjectDoc` literal (`:17–23`), which
  stops compiling the moment the four non-`Option` fields exist. It is an integration-test
  target, so it sees only the public API, exactly like an out-of-workspace consumer would.
  The import list gains `TrackList` and `ViewDoc` and the literal gains four lines:

  ```rust
  fn project() -> ProjectDoc {
      let mut ids = IdGen::new(41);
      let mut unknown = Map::new();
      unknown.insert("future".into(), json!({"kept": true}));
      ProjectDoc {
          id: ids.next_id(),
          name: "Original".into(),
          tempo_map: TempoMap::constant(120.0).unwrap(),
          transport: Transport::new(),
          id_gen_state: ids.state(),
          tracks: TrackList::new(),
          devices: Vec::new(),
          view: ViewDoc::default(),
          unknown,
      }
  }
  ```

  None of its five tests changes an assertion: they exercise `rename`, `Transaction`, and
  `EditHistory`, none of which reads the new fields. This file is unchanged since `6c397d9`,
  so its pins hold.
- `crates/spectre-offline/src/lib.rs` — **also a compile break, and this crate is not
  unaffected.** `default_project()` (`:153–167`) holds the workspace's other out-of-crate
  `ProjectDoc` literal (`:158–164`). Its `use` block (`:15–18`) already imports
  `SCHEMA_VERSION`, `from_bytes`, `ProjectDoc`, `ProjectEnvelope`, **and `TrackList`** —
  R4-4 added the last one for `render_track_list`, so the tracks field needs no new import.
  `to_bytes`/`from_bytes` keep their signatures, but the **value** of `SCHEMA_VERSION`
  changes and `:157` stamps it into every `default_project()` envelope, so this file is
  touched by both halves of the change:

  ```rust
  // Build the deterministic empty project used by smoke tests and future render fixtures
  pub fn default_project() -> ProjectEnvelope {
      let mut ids = IdGen::new(0x0047_4549_5354);
      ProjectEnvelope {
          schema_version: SCHEMA_VERSION,
          project: ProjectDoc {
              id: ids.next_id(),
              name: "Untitled".into(),
              tempo_map: TempoMap::constant(120.0).expect("constant default tempo is valid"),
              transport: Transport::new(),
              id_gen_state: ids.state(),
              tracks: TrackList::new(),
              devices: Vec::new(),
              view: ViewDoc::default(),
              unknown: Map::new(),
          },
          unknown: Map::new(),
      }
  }
  ```

  `id_gen_state` is initialized from the new `IdGen::state()` accessor rather than from `0`,
  because struct-literal fields evaluate in source order and `id: ids.next_id()` has already
  advanced the generator — which is the whole point of persisting the position (§4.2).

  **`default_project()` has two consumers, not one. Iteration 2 said one, and that was
  false.** They are (a) `crates/spectre-offline/src/main.rs:6`, which imports it, and `:14`,
  which calls it for the no-argument and `--self-test` paths, and (b)
  `default_project_report_is_deterministic` in `crates/spectre-offline/tests/harness.rs:19–20`.
  Neither is harmed by the added fields, and the reason is different for each, which is why
  both have to be named: the **binary** feeds the bytes to `inspect_project` and pretty-prints
  the resulting `OfflineReport`, asserting nothing and byte-comparing nothing, so a larger
  document only prints a larger report; the **test** compares two `inspect_project` reports of
  the same bytes to each other and likewise never compares against a golden. The
  `schema_version` literal inside that test is a separate break, below.
- `crates/spectre-offline/tests/harness.rs` — I16, **plus one existing assertion that the
  bump breaks.** `default_project_report_is_deterministic` asserts
  `assert_eq!(first.schema_version, 1)` at `:25`, and `first` comes from
  `inspect_project(to_bytes(default_project()))`, so it reads whatever `SCHEMA_VERSION`
  holds. It becomes `assert_eq!(first.schema_version, SCHEMA_VERSION)`, which is the
  assertion it meant. This is a **second crate** and a **second existing passing test**
  disturbed by the bump; §8 Q2 states the blast radius accordingly.
- `crates/spectre-app/tests/smoke_cli.rs` — assert `project=none`, joining the five
  assertions `smoke_mode_reports_launchable_prototype` already makes — including `tracks=1`,
  `selected_device=Pulse(pulse)`, and `engine=not-started`.
- **`docs/01-requirements/requirements-ledger.md`** — **two** rationale rows, one per
  numeric bound: `SAVE_TEMP_NAME_ATTEMPTS` and `MAX_PROJECT_FILE_BYTES`, each carrying the
  §4.2 text. PROD-003 requires the rationale to be recorded **in that ledger**, and
  decision 16 (`decision-gates.md:40`) makes it a standing rule, so §4.2's table does not
  discharge it on its own. The rows are proposed in the CORE family (`:44–49`) as CORE-005
  and CORE-006, since both are project-envelope bounds. **A third row iteration 2 proposed
  is withdrawn**: the track level range is not a new number, and `:69–71` of this same
  ledger already says so in its own words. Track count is likewise covered by the existing
  ENGINE-005 (`:62`). The same edit updates CORE-001's evidence column (`:46`) to cite I6/I7
  as the *persisted* reorder evidence — noting that R4-4's
  `a_track_keeps_its_identity_and_fields_across_a_reorder` supplied the in-memory half — and
  CORE-004's (`:49`) to record the implementation, moving it to `implemented`.
- `docs/03-architecture/project-persistence.md` — no content change, but its "Open
  decisions" and "Known gaps" lines shorten when the filesystem implementation lands;
  R5's items stay.
- `docs/status/STATUS.md`, `docs/status/NEXT.md`, `docs/06-plans/current-milestone.md` —
  update when the slice lands, per `docs/README.md`'s working rule. STATUS must move
  CORE-004 to **`implemented`, not `verified`** (§4.6) and must not claim crash durability.
  **What it must not weaken is `:39`'s paragraph, not the sentence iteration 2 named.**
  There is no "`./spectre` makes no sound" line in `docs/status/STATUS.md` at `8b1633d` or
  at `1960a8b`; R4-1 replaced it with *"Status is `implemented`, not `verified`: the manual
  protocol in the R4-1 spec §5.4 has not run, and no one has confirmed by ear that sound
  leaves the speakers."* That paragraph is R4-1's to hold or release, this slice neither
  strengthens nor weakens it, and nothing this slice adds may be cited as evidence for or
  against it. STATUS's `:19` "Known gaps" entry — that no landed slice has run its manual
  protocol — gains this slice unless §5.4 is run and recorded.

**Deliberately not modified:** all of `crates/spectre-audio`, `crates/spectre-graph`,
`crates/spectre-dsp`; every RT module scanned by `crates/spectre-audio/tests/rt_guard.rs`
(`bridge.rs`, `control.rs`, `spsc.rs`, `null.rs`, and R4-2's `route.rs`); and
`crates/spectre-project/src/routing.rs`, which builds the render graph and which this slice
neither reads nor writes. No render code, no DSP, no callback-reachable path, and no
`Cargo.toml` in the render-path crates is touched — U12 and U12b exist to keep that true.
`crates/spectre-offline/src/lib.rs` is **not** on this list: an earlier draft of this spec
put it there, and that was wrong. Adding non-`Option` fields to a public struct breaks every
downstream literal of it, and that file holds one.

**The schema change's full blast radius, in one place, counted rather than characterised.**
Two compile sites outside `spectre-project` (`spectre-offline`'s `default_project`,
`spectre-project`'s own `tests/command_history.rs` helper — an integration-test target, so it
too sees only the public API) and **three** existing passing tests across **two** crates:
`canonical_fixture_decodes_with_representative_r1_state`, whose `SCHEMA_VERSION` assertion at
`project_codec.rs:25` flips to the literal `1`; `canonical_fixture_rewrite_is_byte_stable`
(`:52–58`), which cannot survive the change by construction; and
`default_project_report_is_deterministic` (`spectre-offline/tests/harness.rs:25`). Iteration 2
wrote "two existing passing tests" and then listed three; the count is three. Four files, four
repairs, none of them optional and none of them cosmetic. Anything that describes this change
as touching one test in one file is understating it.

**Migrations / schema changes:** the version moves 1 → 2 with a **tolerant read plus a
version stamp in the snapshot builder**: absent fields default on decode, and every
envelope `project_envelope` constructs carries `SCHEMA_VERSION`, so every save `./spectre`
performs writes the current version. The stamp is `project_envelope`'s and nothing else's —
`save_project_atomic` writes the `schema_version` it is handed (§4.3), which is why I9 and
I15 assert opposite-looking things and are both correct. This is deliberately *not* the migration framework R5 owns — there is no
transformation of existing data, no down-conversion, no salvage, and no per-version code
path. Whether that boundary is drawn where Jeff wants it is §8 Q1.

**New dependencies:** none.

### 7.3 Estimated scope

**L.** Roughly 400–500 new lines across `spectre-project` (the `fs` module dominates),
about 120 in `spectre-app`, and a test surface larger than the implementation: **20** unit
tests (U1–U19 plus U12b), 16 integration tests (I1–I16), one new fixture. It is above **M**
for three reasons that are each independently costly — a schema version bump that touches an
existing `verified` requirement's acceptance tests, breaks a third existing test in a second
crate, and breaks two out-of-crate struct literals (§7.2); a private generic seam that has
to be designed so it stays private while still reaching the six stages §5.1 injects; and a
correctness surface where the tests are the deliverable, since "the old file is intact" is
only true if something checks it at every stage. It is below **XL** because it writes no
DSP, adds no dependency, touches no callback-reachable code, and implements an API that is
already designed line by line. **R4-4 reduced the estimate rather than raising it:** the
track collection, its ordering, its identity refusals, its `reorder`, and its serde derives
already exist and are tested, so this slice writes no track type and no track mapping — a
saving iteration 2 could not have counted, because none of it existed then.

### 7.4 Blocking dependencies

- **Nothing blocks authorship or implementation.** Every piece this composes — the
  envelope, the validator, the command transactions, the ID generator, the app model's
  track collection — exists today and is tested.
- **R4-1 (`live-audio-wiring`) and R4-2 (`runtime parameter seam`) landed at `8b1633d` and
  `20f3056`, so both coordination points are this spec's to absorb rather than open
  questions.** The transport bar it edits no longer holds `ENGINE OFFLINE` and `CPU —`; it
  holds R4-1's engine status cluster, and the dirty marker in §3.3 goes on the leading side
  without replacing any of it. §3.2 step 4's transport-stop-on-open stops being theoretical:
  an engine now exists, so opening a project saved while playing must not start it, which is
  what I12 asserts. R4-2 changes nothing here directly, but it is why a Shape edit is now
  audible, which is what makes persisting parameter values worth doing rather than
  bookkeeping. Neither point changes a signature or a file this spec creates.
- **R4-4 (`track-model`) landed at `41a6be2`, and it is the largest single change to this
  spec's premises.** It did **not** merely "extend `TrackDoc`" as iteration 2 predicted — it
  deleted `AppModel`'s `TrackView`, moved the track model into `spectre-project` as `Track`
  and `TrackList` with serde derives already on them, added the routing that turns the list
  into a render graph, discharged CORE-001's in-memory reorder half, and added `MAX_TRACKS`
  with its own accepted ledger row (ENGINE-005). §4.2 persists **that** type rather than a
  parallel one, and §7.3 says why this shrinks the slice. What R4-4 explicitly left for here
  is stated in `NEXT.md:26` — *"persistence of the list (R4-7, which also closes CORE-001's
  other half)"*.
- **R4-6 (`Filament` and `Gloam`) landed at `0d2a03c`/`1960a8b` and changed two counts this
  spec depends on.** `AppModel::prototype()` now builds five device instances holding eleven
  parameter values, not three holding four (§7.1). Every count in this document has been
  re-derived from that. The offline fixture's four values are deliberately unchanged and are
  a different number for a different purpose.
- **R4-5 (`midi-clips`) landed its first part as `487bc49`, *after* this spec's baseline,
  and it hands this slice one concrete obligation.** Disclosed as post-baseline drift
  rather than folded into §7.1, because §7.1 describes `1960a8b` and must keep doing so.
  What `487bc49` did: `crates/spectre-project/src/clip.rs` adds `ClipNote`, `MidiClip`,
  `ClipPlacement`, and a per-track `TrackClips`, and `crates/spectre-audio/src/clip.rs` adds
  the baked schedule and the player. What it deliberately did **not** do: `ProjectDoc` is
  byte-for-byte unchanged at `487bc49` — still `id`, `name`, `tempo_map`, `transport`, and
  the unknown map — and `Track` holds no clips. R4-5's own queue entry says why and to whom
  it falls: *"Absent by design: the spec's U-9 undo test, because `EditHistory` mutates
  `ProjectDoc` and `ProjectDoc` has no clip field until slice 7."* So the clip field on
  `ProjectDoc` is **this slice's**, on the same schema-2 step as the other four, and the two
  slices must not each bump the version independently — §8 Q1's additive-versus-bump rule is
  what settles that. This spec does not restate R4-5's data model, because it was written
  against a tree that did not contain it; a fifth `ProjectDoc` field and its validator rules
  are a bounded addition to §4.2 that the implementing session should make against whatever
  `clip.rs` reads at that point, and §5.2 gains the corresponding round-trip assertion. R4's
  exit evidence requires the round trip to include clips, and this is where that lands.
- **One R4 obligation is assigned to "slice 5 or 7" and this spec does not take it.**
  `NEXT.md:19` records that *"slice 6's two devices are not reachable from a track until
  slice 5 or 7 adds a `TrackInstrument` variant for them"*, and `STATUS.md:59` says the same.
  `TrackInstrument` has one variant, `Pulse`, and it is a **persisted** enum, so adding
  variants is a wire-format change that interacts with this slice's schema step. Taking it
  here would be scope this spec was not asked for; ignoring it would leave a real assignment
  unclaimed. It is routed to §8 Q14 rather than decided.
- **R4-8 (`offline-bounce`) depends on this**, in that a bounce of a *loaded* project is
  only meaningful once loading exists. I16 is deliberately the shape R4-8 will extend.
- **R5 blocks the durability claim, not the code.** Crash injection, the qualified
  filesystem matrix, autosave, and recovery are R5's by the design authority's own
  milestone table. Until they run, this feature is `implemented`, not `verified`, and no
  product copy may say otherwise.
- **External gates:** a writable filesystem for the tests (any temp directory), and access
  to both a macOS and a Linux machine for §5.4. No Linux **audio** device is needed — this
  feature does not touch decision 23's debt in either direction.

---

## 8. Open Questions

- **Q1 — Is the schema-1 → schema-2 step a tolerant read (R4) or a migration (R5)?**
  This spec treats "absent fields default on decode, and the snapshot builder stamps the
  current version" as tolerant reading, not migration, because nothing is transformed and
  there is no per-version code path. The alternative is to keep `SCHEMA_VERSION = 1` and
  make the four fields additive with `skip_serializing_if` — at the cost that an older build
  silently opens a project with tracks and shows the musician an empty one. §4.2 argues the
  bump is safer for exactly that reason. **One correction to that alternative, because it is
  easy to state too generously:** `skip_serializing_if` preserves
  `canonical_fixture_rewrite_is_byte_stable` **only if it skips all four fields when the
  document carries none** — the two empty `Vec`s, the `0` generator state, and the default
  `ViewDoc` alike. Skipping only `tracks` and `devices` still changes the v1 fixture's
  encoded bytes and still breaks that test, so "the byte-stability test stays untouched" is
  not free with the approach: it costs a predicate on every one of the four fields, and the
  default-valued `ViewDoc` is the awkward one. Note also that the byte-stability test breaks
  under *this spec's* choice for the same reason — the always-serialized new fields — and
  **not** because of a version stamp, since the writer performs none (§4.3).
  — *blocks: §4.2, §7.2*
- **Q2 — The schema-2 change disturbs a `verified` requirement's evidence and reaches a
  second crate; the whole radius is stated here so the decision is made on it.**
  `canonical_fixture_rewrite_is_byte_stable` is acceptance evidence for CORE-003
  (`requirements-ledger.md:48`, status `verified`) and cannot survive the four added fields
  by construction; this spec retargets it to the new v2 fixture and replaces its v1 role
  with the I8/I9 pair. That is the headline and it is not the whole change. The full list,
  from §7.2: `project_codec.rs:25`'s `SCHEMA_VERSION` assertion flips to the literal `1`;
  `crates/spectre-offline/tests/harness.rs:25`'s `assert_eq!(first.schema_version, 1)` — an
  existing **passing** test in a **second crate** — fails on the bump and becomes an
  assertion against `SCHEMA_VERSION`; and two out-of-crate `ProjectDoc` literals stop
  compiling until they name the four new fields. So what is being approved is "amend a
  `verified` requirement's acceptance evidence, repair one further existing test in another
  crate, and repair two compile sites," not "retarget one test." Confirm the retarget is a
  preserved guarantee under a new fixture rather than a weakened one, and confirm the radius
  is acceptable. — *blocks: §5.2, §7.2*
- **Q3 — Should a save preserve the destination's existing permissions and ownership?**
  The design authority says they are *"not implicitly preserved unless a later accepted
  contract explicitly requires and tests them."* Preserving them costs a `metadata` read
  and a `set_permissions` before the rename and affects anyone saving into a shared or
  group-writable folder. R4 does not do it. — *blocks: §4.3, §6.1*
- **Q4 — Native file dialog, or a path field?** §3 specifies a path field because a native
  dialog means a new third-party dependency (`rfd` or equivalent) on both platforms, and
  this spec adds none. A path field is a poor experience for a credible alpha. If a dialog
  is wanted, it is a separate dependency decision with its own license audit and a Linux
  portal question. — *blocks: §3.1, §4.5*
- **Q5 — Enable `eframe`'s accessibility feature at R4?** `crates/spectre-app/Cargo.toml`
  sets `default-features = false` on `eframe`, so no screen-reader integration is compiled in and none
  is claimed. Decision 17 (`decision-gates.md:41`) scopes an audit at R4; turning the
  feature on is one manifest line plus whatever the audit finds. This spec does not do it
  unilaterally. — *blocks: §3.7*
- **Q6 — When recording arrives, is track arm state project state or session state?**
  Nothing is persisted here because **no arm state exists to persist**: `Track`'s seven
  fields at `1960a8b` are `id`, `name`, `instrument`, `instrument_level`, `level`, `muted`,
  `soloed`, and `crates/spectre-app/src/main.rs`'s `track_list` records the reason in the shell —
  *"There is no arm indicator, because there is no recording path to arm for."* Iteration 2
  described this as a deliberate exclusion of an existing `armed` field; that field was
  R4-4's to delete and it is gone. The question survives the correction because it still has
  to be answered before R7 adds recording, and restoring an armed track into a live engine
  is a recording decision rather than a file-format one. There is **no citable benchmark
  evidence either way** in the accepted corpus (Appendix A). — *blocks: §4.2*
- **Q7 — Recovery rule when `id_gen_state` is missing or implausible.** Bounded today
  (schema-1 documents have no collections, so seeding from the project ID is sound and the
  uniqueness validator is the backstop). It stops being bounded once R5 migrations produce
  documents from arbitrary older states. — *blocks: §4.2*
- **Q8 — Should `LoadError` gain a `TooLarge { bytes, max }` variant?** This spec maps the
  size refusal onto `Read` with `ErrorKind::InvalidData` rather than extending an accepted
  enum, which preserves the four required distinctions but gives the user a less specific
  message. Adding the variant is a contract amendment and belongs to Jeff. — *blocks:
  §4.3, §3.6*
- **Q9 — Confirm `sync_all`'s mapping on Apple targets.** §4.6 states the `F_FULLFSYNC`
  belief as **unverified**, because no file in this repository can settle it. Whether R4
  confirms it against the pinned toolchain's `std` source, or waits for R5's hardware
  qualification, changes what the R4 status line may say. — *blocks: §4.6, §7.2*
- **Q10 — Does saving stay on the UI thread?** §4.7's stall figures are estimates. If
  §5.4's measured save time on either platform is bad enough to be felt while playing,
  moving the save off the UI thread becomes an R5 question — and it brings the
  serialization obligation with it, since §4.4's "one save at a time" is currently
  structural rather than enforced. — *blocks: §4.4, §4.7*
- **Q11 — Leave `SaveError::Encode` untested?** It is currently unreachable for a
  validated envelope, so no honest test can fail on it. The alternative is to extend the
  private seam to force an encoder failure, which tests the plumbing rather than a real
  condition. §5.1 chooses to say so instead. — *blocks: §5.1*
- **Q12 — Unknown fields do not survive an `AppModel` round trip. Is that acceptable at R4?**
  `project_envelope` builds a fresh document from `AppModel`, which has nowhere to hold the
  envelope's or the document's `#[serde(flatten)]` unknown map (§4.3, §4.4), so
  open → edit → save through `./spectre` drops any field this build does not know. The
  crate-level guarantee CORE-003 is `verified` on is untouched — `to_bytes`/`from_bytes`
  still preserve unknown fields, and I8/I9 prove it across a real filesystem — but the
  product-level guarantee is narrower than the crate-level one, and this is the first slice
  where that difference is observable at all. The R4 exposure is small: `load_project`
  refuses any `schema_version` above `MAX_READABLE_SCHEMA`, so only a same-version writer or
  a hand edit can put an unknown field where this build will read it and then drop it.
  Closing it costs one opaque preservation field on `AppModel` plus a write-back in
  `project_envelope`. This spec does not take that unilaterally, because §4.4's "this feature
  adds **no** field to `AppModel`" is load-bearing for the single-model invariant, and
  because the question is really "is unknown-field preservation a crate property or a product
  property" — which is Jeff's to answer. Surfaced by giving the version stamp an explicit
  owner; it was invisible while the stamp had none. — *blocks: §4.3, §4.4, §7.2*
- **Q13 — Should the wire format reuse `TrackList`, or should it own a parallel document
  type? New in iteration 3, and forced by R4-4 rather than raised for its own sake.**
  §4.2 makes `ProjectDoc.tracks` a `spectre_project::TrackList` because that type already
  exists in the same crate, already derives `Serialize`/`Deserialize`, is already the type
  `AppModel` holds, and already owns order and identity. The cost is a coupling that this
  spec must name rather than let a later slice discover: **the persisted shape becomes
  whatever `Track` and `TrackList` are**, so any future field added to `Track` for an
  in-memory reason silently enters the file format, and any field removed from it silently
  leaves. Today that coupling is a benefit — one type, one place, no drift, no mapping to
  forget — and R4-4 already used `#[serde(skip)]` deliberately on `structure_revision` to
  keep session state out, which is evidence the crate can hold the line. The alternative is
  a parallel `TrackDoc` plus a hand-written mapping, which decouples the two but adds a
  second place for every future field and a mapping that can be wrong. This spec takes the
  reuse and states the cost; whether the wire format should be allowed to track an
  app-thread type at all is a schema-governance question that belongs with decision 3
  (`decision-gates.md:27`) and is Jeff's. If the answer is "decouple", the change is
  contained: one document type, one mapping function, and §5.2's assertions unchanged.
  — *blocks: §4.2, §7.2*
- **Q14 — Who adds the `TrackInstrument` variants for `Filament` and `Gloam`, slice 5 or
  slice 7?** `NEXT.md:19` and `STATUS.md:59` both record the obligation and both leave it
  to "slice 5 or 7". `TrackInstrument` has one variant, `Pulse`, so R4-6's two devices are
  built and rendered offline but are not reachable from a track. This matters to R4-7
  specifically because `TrackInstrument` is a **persisted enum** on `Track`: adding variants
  is a wire-format change, and doing it in the same slice as the schema-2 step is either
  efficient or a scope violation depending on how the two slices are meant to divide. This
  spec **does not take it**. `NEXT.md:29` scopes slice 7 to atomic save/reload and CORE-001's
  reorder evidence, and neither requires new instrument variants; taking it would be
  unrequested scope, and taking it silently would be worse. Naming it here is what keeps an
  assigned obligation from falling between two slices. — *blocks: §7.4*

---

## Appendix A — Benchmark evidence used, and where it does not exist

Jeff's benchmark set is Ableton Live, Logic Pro, Serum 2, Phase Plant, and VCV Rack 2
(`criteria.md` Lens 2). Persistence is a thin surface in the accepted corpus, and the
honest answer is mostly a gap.

**The gap, stated exactly.** There is **no citable observation in the accepted corpus
describing how any benchmark product writes a project file, what it does when a save is
interrupted, whether it uses a temporary-and-rename strategy, how deep its undo stack is,
or whether it restores view state on open.** The reasons are checkable:

- **Ableton Live 12** has 85 records, but
  `docs/02-reference-research/ableton-live-observations.md:12` scopes them to manual
  chapters 6, 7, 8, 9, 16, 17, 18, 19, 25, and 41. The chapters covering file and set
  management are not among them, and `:19` lists what remains unextracted. So the corpus
  has depth on arrangement, clips, warping, routing, mixing, recording, and automation —
  and nothing on saving a Live Set.
- **Logic Pro** has **zero** behavioral records; its dossier is inventory-only.
- **Serum 2** has exactly two citable records, `OBS-SR2-CPU-001` and `OBS-SR2-KB-001`
  (`docs/02-reference-research/synth-modular-observations.md:54–55`), and neither concerns
  persistence. `OBS-SR2-KB-001` explains why: the public knowledge base is six articles and
  *"no complete public user manual is exposed"*. A larger `serum-2-observations.md` file
  exists in the tree but carries a **QUARANTINED — NOT ACCEPTED RESEARCH** banner at
  `docs/02-reference-research/serum-2-observations.md:10–12` instructing that it not be
  cited until a provenance question is resolved. Nothing from it is used here.
- **Phase Plant** (11 records) and **VCV Rack 2** (6) are generator/routing and
  signal-convention dossiers; neither touches file writing.

Recorded as a research need: chapter 5, "Managing Files and Sets", is the highest-value
extraction for any future persistence work. Its status is **`section-inventoried`, not
unextracted**, and the distinction matters — the headings and source locations are already
captured, so the work is extraction rather than discovery.
`docs/02-reference-research/ableton-live.md:49` carries its matrix row and `:132–148` the
section breakdown, including 5.4, whose recorded open questions are *"schema/versioning,
atomic save, recovery, merge identity/conflicts, unknown-data preservation, and
transactional undo"* — the exact list this feature is working without. `ableton-live.md:160`
names chapters 3–5 as the next extraction target for that reason. Chapter 5 is **not** among
the chapters listed as unextracted at `ableton-live-observations.md:19` (2, 10–15, 20–24,
26, 33, 36–40); an earlier draft of this appendix cited that line for it, and that citation
did not hold.

**What the corpus *does* support, and how it is used.**

- `OBS-PP-UNI-003` (Phase Plant): bend range *"is saved with the project, not the preset."*
- `OBS-AB12-WARP-005` (Ableton 9.2.3.2): warp markers save with the set, and optionally
  into the sample's own analysis data.
- `OBS-AB12-CLIP-002` (Ableton 8.1.1): default clip settings can be saved into the
  sample's `.asd` analysis file.

Two researched products therefore **converge** on drawing an explicit, documented line
between what belongs to the project, what belongs to a preset, and what belongs to a
sidecar (criterion 2E). Spectre's R4 position is a deliberate simplification of that
pattern, not a rejection of it: there is no preset layer and no sidecar yet, so everything
persisted is project state, and §4.2 states the line anyway by naming what is *excluded* —
prototype feedback text, arm state, and every engine measurement. That line is what R5's
autosave sidecar (decision 14) and any later preset system have to respect.

- `OBS-AB12-REC-009` (Ableton 19.8) and `OBS-AB12-ARR-008` (6.12–6.13): recorded and
  consolidated material lives in the project folder once the set is saved and in a
  temporary folder before that, with the temporary folder's disk space named as the
  documented exhaustion risk.

This is the closest citable evidence to a real product's unsaved-state failure mode, and
it is the reason §3.6 gives disk-space and permission failures their own explicit messages
with an explicit "nothing was written" guarantee rather than a generic error.

- `OBS-BW53-CON-001` (Bitwig — **not** in Jeff's benchmark set; valid corroborating
  evidence per `criteria.md` Lens 2): multiple projects may be open at once, but audio is
  active for only one.

Corroborating, not load-bearing: it is the closest external precedent for the
one-owner-per-project posture §4.4 takes when it makes save serialization structural
rather than locked. Spectre at R4 has one window and one project, which is a strictly
narrower position, and §4.4 says what would have to change if that stops being true.

- `OBS-VCV-VOLT-006` (VCV Rack 2): modules should output 0 on NaN/infinity detection —
  already adopted as RT-003's provenance (`requirements-ledger.md:30`).

Used only by analogy, and labelled as such: §4.2's rule 4 applies the same
contain-at-the-boundary posture on the persistence side, where the failure mode is an
unencodable value rather than audible noise.

**Two dossiers outside the benchmark set are in the tree and in-review, and are not relied
on.** `docs/02-reference-research/ozone-observations.md` is status `draft` /
`research-state: in-review` (`:10–11`). Iteration 2 also described it as *untracked in git*;
that was a `2e005e5` fact and is corrected here — the file has been tracked since `b5af060`
and is tracked at `1960a8b`. Immaterial to the argument, since nothing here rests on it, but
it was the same un-rebaselined-fact pattern §7.1 exists to prevent. Its `OBS-OZ-PRESET-009`
happens to name *"what exactly is in a device preset versus what is session state"* as a
persistence-contract question a product should answer explicitly, which agrees with the
line §4.2 draws. iZotope Ozone is not one of Jeff's five benchmarks, the file is not
accepted research, and nothing in this spec rests on it; it is mentioned so the reviewer
knows it was read and deliberately not leaned on.

**Differentiation (criterion 2F).** Spectre's stated difference here is not feature parity
— it is that the failure vocabulary is part of the product surface rather than hidden. The
corpus contains no benchmark record of any product distinguishing "saved and durable" from
"saved but the directory entry may not survive a power loss" to the musician, and this
spec does not claim any of them fails to; it claims only that Spectre does it, that
`TargetState` makes the distinction machine-checkable rather than editorial, and that the
project stays dirty until the strong case is reached. That is a small, honest difference,
and it is the one this slice can actually stand behind. Parity is not claimed as
completeness: R4 has no autosave, no recovery, no migration, and no crash evidence, all of
which the benchmark products have had for years, and §7.4 says so.

---

**End of spec.**
