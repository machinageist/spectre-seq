<!--
Author: Jeff
Date: 2026-08-15
Description: R4-7 spec — implement CORE-004 atomic filesystem save/reload over the accepted persistence contract and land CORE-001's reorder evidence on the first persisted collection
Notes: The persistence API is already accepted in docs/03-architecture/project-persistence.md; this spec implements
  it and does not redesign it. Crash-durability evidence, journaled autosave, recovery, and migrations are R5 and
  are deferred explicitly. Today crates/spectre-app declares spectre-project as a dependency and never uses it;
  ./spectre can neither save nor open a project file, and it still produces no sound.
-->

# Spec: Project Persistence

**Feature ID:** `R4-7` (`project-persistence`)
**Parent feature:** `R4` Credible Alpha (root)
**Spec author agent:** gauntlet spec agent, R4-7 leaf
**Date:** 2026-08-15
**Iteration:** 1

- **Status:** proposed
- **Last verified:** 2026-08-15 (source read at branch `rename/geist-to-spectre`, HEAD `2e005e5`)
- **Scope:** app-thread `load_project` / `save_project_atomic` on the real filesystem, the reusable semantic validator both call, the first persisted object collection, and the app-layer save/open surface in `./spectre`
- **Decision authority:** Jeff
- **Upstream sources:** `docs/03-architecture/project-persistence.md` (design authority, accepted for R4/R5 implementation), `docs/01-requirements/requirements-ledger.md` (CORE-001 `:46`, CORE-003 `:48`, CORE-004 `:49`, RT-001..003 `:28–30`, PROD-003 `:64`), `docs/01-requirements/decision-gates.md` (rows 1 `:25`, 3 `:27`, 4 `:28`, 8 `:32`, 13 `:37`, 14 `:38`, 16 `:40`, 17 `:41`, 23 `:49`), `docs/00-product/vision.md:33`, `docs/06-plans/current-milestone.md` §"Inherited debt" items 3–4, `docs/status/NEXT.md` slice 7
- **Downstream dependents:** R4-4 (track model), R4-5 (MIDI clips), R4-6 (small synth/effect), R4-8 (offline bounce), R4-9 (end-to-end fixture and manual QA), and every R5 project-safety slice (journaled autosave, recovery, migrations, salvage, missing-media)
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** §8 Q1–Q11
- **Known gaps:** the accepted benchmark corpus contains **no** citable observation about how any benchmark product writes a project file, recovers from an interrupted save, bounds undo depth, or persists view state. The four adjacent records that do exist are used in §2/§3 and the hole is named in Appendix A rather than filled in. Crash-durability evidence is R5 by the design authority's own milestone table and is not claimed here.

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

Right now none of it exists as a product behavior. `crates/spectre-app/Cargo.toml:16`
declares `spectre-project = { path = "../spectre-project" }`, and **no file under
`crates/spectre-app/src/` or `crates/spectre-app/tests/` mentions `spectre_project` at
all** — the app declares the persistence crate and never uses it. `main.rs`'s own header
says so in line 4: *"Interaction prototype only; audio and persistence wiring remain out
of scope."* There is no menu item, no path field, and no code path in `./spectre` that
writes or reads a file. Everything a musician does in the prototype is lost when the
process exits.

Below that gap the crate is not empty, and this spec must be precise about the
difference. `spectre-project` has a versioned JSON envelope, a schema gate, semantic
validation after decode, atomic in-memory command transactions and bounded undo/redo
(`src/lib.rs`, `src/command.rs`). **None of that touches a filesystem.** The word
"atomic" in `command.rs:3` describes a transaction that either applies every command or
reverses the ones it applied (`command.rs:92–112`); it has nothing to do with CORE-004's
atomic replacement of a file. An in-memory envelope with a validator is not an atomic
save, and this spec's §7.1 does not let those two claims blur.

Two accepted obligations land here and nowhere else:

- **CORE-004** (`requirements-ledger.md:49`) is `accepted` with its API design completed
  at R1; `current-milestone.md` §"Inherited debt" item 3 assigns the filesystem
  implementation to R4 and crash qualification to R5.
- **CORE-001** (`requirements-ledger.md:46`) is `implemented` with reorder evidence
  *"gated on the first persisted object collection (R4 intake)"*. R4 has no persisted
  collection yet. This spec introduces one, so it owns that evidence.

### 1.3 Success signal

One command produces the whole result:
`cargo test --locked -p spectre-project -p spectre-app -p spectre-offline` passes with
(a) a real-filesystem test that saves a project containing three tracks, reorders them,
saves again, reloads, and asserts each track's `ObjectId` is unchanged and the new order
is exact; (b) a fault-injection test at every one of the seven `SaveStage` values that
asserts the destination is byte-identical to its pre-call contents whenever the call
failed before the replacement; and (c) an offline test asserting
`spectre_offline::render_app_snapshot` returns the **same** `RenderReport.hash` for a
project before and after a save/reload round trip, so persistence is proved not to change
what the project computes.

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
| **PROJECT** block, top of the left panel (`track_list()`, `main.rs:115–158`) | always visible | **new block inside an existing panel** | left `SidePanel` (`main.rs:116`, `default_width(220.0)`, `min_width(180.0)`), vertical stack above the existing `TRACKS` label at `main.rs:126` |
| Transport bar title area (`transport()`, `main.rs:50–84`) | always visible, 62 px fixed (`main.rs:52`) | modified — one dirty-state marker beside the `SPECTRE` wordmark (`main.rs:60`) | full-width top panel, horizontal centered row |

The PROJECT block deliberately reuses the widget pattern already in the same panel: the
add-track control at `main.rs:141–152` is a `TextEdit::singleline` with `hint_text` next
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
   start playback — and, once R4-1 lands, cannot start an audio engine as a side effect
   of opening a file.
5. On `Err` the status line names which of the five `LoadError` cases occurred, and the
   open project is **unchanged** — same tracks, same selection, same dirty state.

**Branch — unsaved work.** If the model is dirty, `Open project` first requires a second
press of a `Discard and open` confirm control that replaces the button in place for one
interaction. No modal. Rationale in §3.6.

### 3.3 Layout descriptions

PROJECT block, top → bottom, inside `SidePanel::left("tracks")`:

1. `PROJECT` section label — small, strong, `MUTED`, matching the existing `TRACKS`
   (`main.rs:126`) and `BROWSER` (`main.rs:153`) labels exactly.
2. Path field — `egui::TextEdit::singleline` sized to the panel width,
   `hint_text("Project file path")`. Data source: a new `project_path: String` field on
   `SpectrePrototype` (`main.rs:17–21`), not on `AppModel`. A path is shell state, not
   project state.
3. Action row, leading → trailing: `Save project`, `Open project`. Both `egui::Button`,
   both disabled with `on_disabled_hover_text` when the path field is empty.
4. Status line — one wrapped label. Sources, in priority order: the last
   `SaveError`/`LoadError` message, then the last success message, then the empty-state
   copy.
5. Dirty indicator — the text `Unsaved changes` in `WARM` (`main.rs:13`) when dirty,
   `Saved` in `MUTED` when not. Colour is never the only signal (§3.7).

**Empty state.** With no path and no save yet, the status line reads: *"No project file
yet. Type a path and save — Spectre replaces the file atomically, so an interrupted save
leaves the old file intact."* This states a property the code will have, not one it has;
the copy ships in the same slice as the behavior (§6.3).

Transport bar: a single `•` glyph plus the text `unsaved` after the `SPECTRE` wordmark
(`main.rs:60`) while dirty. No other transport-bar literal changes; the `ENGINE OFFLINE`
string at `main.rs:79` is R4-1's, not this spec's, and must not be touched here.

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
  ⌘S. The existing `Space` and `1`–`4` bindings (`main.rs:425–437`) are unchanged.
- Specialized input: none. No stylus, controller, voice, or camera.
- Responsive: the block lives in a resizable panel with `min_width(180.0)`
  (`main.rs:119`); the path field takes the available width, and the two buttons wrap to
  a second row below the panel's minimum comfortable width. Window minimum is
  1060 × 680 (`main.rs:508`).

### 3.5 Transitions & animation

None. No navigation transition, because no view is entered or left. The dirty marker and
status line change text between frames with no animation. The app repaints on a 250 ms
timer (`main.rs:443`), so a status change is visible within one repaint.

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
  transport bar (`main.rs:52`) is unchanged and the dirty marker is short by design so it
  cannot push the transport row out of that fixed height.
- **Screen readers — stated honestly.** `crates/spectre-app/Cargo.toml:13` sets
  `default-features = false` on `eframe` with only `default_fonts` and `glow` enabled, so
  whatever accessibility integration eframe ships behind a feature flag is **off** in
  this build. This spec therefore makes **no screen-reader claim**, and must not be read
  as delivering one. What it does is avoid foreclosing decision 17
  (`decision-gates.md:41`, keyboard-complete operation and screen-reader labels by beta,
  scoped audit at R4): every element here is a standard labelled widget, so enabling the
  feature later is a manifest change plus an audit, not a redesign. Enabling it is
  routed to §8 Q5 and belongs to the R4 accessibility audit, not to this slice.

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

**The realtime boundary is structural, not a convention.** Nothing on the callback path
can name `spectre-project`, because the dependency graph does not permit it:

- `crates/spectre-audio/Cargo.toml:17–21` — `cpal`, `spectre-core`, `spectre-dsp`,
  `spectre-graph`. No `spectre-project`.
- `crates/spectre-graph/Cargo.toml:12–14` — `spectre-core`, `spectre-dsp`.
- `crates/spectre-dsp/Cargo.toml:12–13` — `spectre-core`.
- `crates/spectre-core/Cargo.toml:12–14` — `serde`, `serde_json`. No path dependencies.

So `load_project` and `save_project_atomic` are not merely "not called" from the render
path — they are **unnameable** there. §5.1 test U12 turns that into a test that fails if
anyone adds the edge. This is the primary RT-001 argument (`requirements-ledger.md:28`):
a compile-time impossibility, not a code-review promise.

The secondary argument covers the app thread. `save_project_atomic` blocks in
`write_all`, `sync_all`, `rename`, and a directory `sync_all`. It runs on the egui update
thread (`main.rs:424`), which is not the audio thread: with R4-1 in place the driver owns
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

Both are `1` today (`src/lib.rs:13`, `:16`). The bump is required because schema 2 adds
persisted object collections, and a build that cannot render them must refuse the file
rather than show the musician an empty project it would then let them overwrite. That
refusal already exists and is tested — the gate at `src/lib.rs:92` and the test at
`tests/project_codec.rs:85–98`. These two constants are schema identifiers, not numeric
limits, so PROD-003 does not apply to them; the three constants it does apply to are
listed at the end of this section.

**Documents.** `ProjectEnvelope` (`src/lib.rs:20–27`) is unchanged in shape.
`ProjectDoc` (`src/lib.rs:30–38`) gains four fields, all `#[serde(default)]` so a
schema-1 file still decodes:

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
    // First persisted object collection; presentation order is the vector order
    #[serde(default)]
    pub tracks: Vec<TrackDoc>,
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

// One persisted track; identity is the ObjectId, never the index
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackDoc {
    pub id: ObjectId,
    pub name: String,
    pub muted: bool,
    pub solo: bool,
    pub level: f32,
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

`LensDoc` intentionally duplicates the four variants of `spectre_app::Lens`
(`crates/spectre-app/src/lib.rs:12–18`). The document schema is a wire format with its
own compatibility rules; the UI enum is not, and `spectre-project` must not depend on
`spectre-app`. `spectre-app` owns the two conversions (§4.3).

`AppModel` has no zoom state — its fields are `transport`, `lens`, `tracks`, `devices`,
`selected_track`, `selected_device`, `ids`, `feedback`
(`crates/spectre-app/src/lib.rs:205–214`) — so there is no zoom to persist. `feedback` is
prototype-feedback text (`lib.rs:432–445`) and is deliberately **not** persisted;
it is instrumentation, not the musician's work.

`armed` (`crates/spectre-app/src/lib.rs:43`) is deliberately **not** persisted. Recording
does not exist — the Record button is inert with the hover text *"Recording arrives after
the live audio shell."* (`main.rs:72–73`) — and restoring an armed track into a live
engine is a decision that belongs with recording, not with a file format. Routed to §8 Q6.

**Generator state — a real hazard this schema closes.** `IdGen` is a deterministic
splitmix64 seeded once (`crates/spectre-core/src/id.rs:45–67`), and `AppModel::prototype`
seeds it with the fixed constant `IdGen::new(0x0047_4549_5354_5549)`
(`crates/spectre-app/src/lib.rs:219`). If the generator position is not persisted, then
after a reload the app restarts from that same seed and the very next `add_track`
(`lib.rs:405–422`) mints an ID that is already in the loaded project — a *guaranteed*
duplicate, not a probabilistic one, and a direct CORE-001 violation. Persisting
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
one validator called after decode *and* before encoding. Today `validate` is private
(`src/lib.rs:105–128`) and only `from_bytes` calls it (`src/lib.rs:101`); `to_bytes`
(`src/lib.rs:80–82`) does not revalidate. This spec makes the validator public with its
own error type and leaves `to_bytes` alone as the low-level encoder, because
`save_project_atomic` — the only path that writes a destination — calls the validator
itself:

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
ID (`:106–108`), tempo-map reconstruction (`:109–110`), meter-map decode (`:111`),
enabled-loop-requires-region (`:112–118`), and ordered loop region (`:119–123`). Schema 2
adds:

1. **Project-wide unique object IDs.** Every `ObjectId` in the document — `project.id`,
   each `TrackDoc.id`, each `DeviceDoc.id`, each `ParameterDoc.id` — must be distinct.
   This is exactly what the contract defers to *"once persisted object collections
   exist"*, and what `crates/spectre-core/src/id.rs:4` records as pending. Nonzero comes
   free from `ObjectId`'s `Deserialize` (`id.rs:16–25`).
2. **Referential integrity of the view.** `view.selected_track`, when `Some`, must name a
   `TrackDoc` present in `tracks`; `view.selected_device`, when `Some`, must name a
   `DeviceDoc` present in `devices`. A dangling selection is invalid, not silently
   dropped.
3. **Finite, in-range track level.** `level` must be finite and within `0.0..=1.0`.
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

What the validator explicitly does **not** do: check parameter values against their DSP
descriptor ranges. `spectre-project` depends only on `spectre-core`, `serde`, and
`serde_json` (`crates/spectre-project/Cargo.toml:12–15`), and it must not gain a
`spectre-dsp` dependency to reach `DspParameter::minimum`/`maximum`
(`crates/spectre-dsp/src/parameter.rs:66–72`). Descriptor-range validation belongs to the
layer that owns descriptors, and that layer already does it: `DeviceValues::from_snapshot`
rejects unknown device/parameter identities (`crates/spectre-offline/src/lib.rs:87–108`)
and non-canonical values (`:110–120`), and `DeviceParameterSnapshot::new` clamps on
construction (`crates/spectre-dsp/src/parameter.rs:89–103`). `spectre-app::adopt` calls
that path (§4.3).

**Numeric bounds introduced here.** Three, each with its own rationale, none copied from
any product. PROD-003 (`requirements-ledger.md:64`) requires the rationale to be recorded
**in the ledger**, so all three appear in §7.2's modified-file list.

| Constant | Value | Rationale (Spectre-derived) |
|---|---|---|
| `SAVE_TEMP_NAME_ATTEMPTS` | `8` | The temporary name mixes the process ID, the nanosecond field of the wall clock, and a monotonic counter, so a collision needs a leftover temporary whose three components all match. Retrying is required by the contract (*"retries name collisions without truncating another file"*), but the retry must be **bounded**: this runs synchronously on the UI thread, and an unbounded loop over a pathological directory would hang the window instead of failing. Eight attempts is small enough that the worst case is imperceptible and large enough that an accidental collision cannot exhaust it. |
| `MAX_PROJECT_FILE_BYTES` | `64 * 1024 * 1024` | `load_project` reads a file that may be truncated, corrupt, or not a project at all before it can know anything about it. Without a bound, the read allocates whatever the file claims to be. The checked-in R1 fixture is **912 bytes**; a schema-2 project with the collections in this spec is a few kilobytes. 64 MiB is four orders of magnitude of headroom for the alpha while keeping the app-thread allocation bounded. It also bounds the validator's uniqueness set, which is sized by the document. It is a starting envelope for R4 and is expected to be revisited when R5 or later persists sample or media references — recorded as such rather than presented as a final number. |
| `TRACK_LEVEL_RANGE` | `0.0..=1.0` | Not a new number: the shell already constrains it at `crates/spectre-app/src/main.rs:180` (`Slider::new(&mut track.level, 0.0..=1.0)`) and seeds `0.78` / `0.72` (`crates/spectre-app/src/lib.rs:226`, `:417`). The validator has to enforce *something*, and enforcing a different bound than the UI would let a file exist that the UI cannot represent. The rationale row makes an existing undocumented UI bound explicit, which PROD-003 currently has no row for. |

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
`serde_json::Error`, which is what `ProjectError::Malformed` already carries
(`src/lib.rs:57`), and `ValidationError` is the new struct in §4.2.

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
3. Peek `schema_version` and gate it, reusing the existing logic at `src/lib.rs:86–97` →
   `LoadError::SchemaTooNew`.
4. Decode the envelope → `LoadError::Malformed`.
5. `validate_envelope(&envelope)` → `LoadError::InvalidProject`.
6. Return the complete validated envelope. No failure returns a partial project, and no
   failure path writes anything.

**`save_project_atomic` — ordered steps, with what each buys.**

| # | Step | Concrete call | What it buys | Failure → |
|---|---|---|---|---|
| 1 | validate | `validate_envelope(snapshot)?` | a semantically broken project can never reach the disk | `InvalidProject`; no path touched |
| 2 | encode | `to_bytes(snapshot)?` (`src/lib.rs:80`) | the complete bytes exist in memory before anything is created | `Encode`; no path touched |
| 3 | create temp | `OpenOptions::new().write(true).create_new(true).open(&temp)`, retried up to `SAVE_TEMP_NAME_ATTEMPTS` on `ErrorKind::AlreadyExists` | `create_new` is `O_CREAT\|O_EXCL`: the kernel decides, so there is no check-then-create race and an existing file is never truncated | `Io { CreateTemporary, Unchanged }` |
| 4 | write | `file.write_all(&bytes)` | `write_all` loops until every byte is written; a short write is not success | `Io { WriteTemporary, Unchanged }` + best-effort `remove_file(&temp)` |
| 5 | sync file | `file.sync_all()` | the *contents* are handed to the storage device before any name points at them; without this, a rename can publish a file whose data is still only in the page cache | `Io { SyncTemporary, Unchanged }` + cleanup |
| 6 | release | `drop(file)` | releases the handle before replacement, as the contract requires *"as required by the replacement API"* | — |
| 7 | replace | `std::fs::rename(&temp, path)` | POSIX `rename(2)` replaces an existing name atomically with respect to other processes: **no window in which the destination is missing or half-written.** This is the commit point | `Io { ReplaceTarget, Unchanged }` + cleanup |
| 8 | sync dir | `File::open(parent)?.sync_all()` | flushes the *directory entry* so the replacement itself, not just the data, survives a crash | `Io { SyncParentDirectory, ReplacedDurabilityUncertain }`, **no second replacement attempt** |
| 9 | receipt | `Ok(SaveReceipt { target_state: ReplacedDurable })` | only after every required synchronization succeeded | — |

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

`project_envelope` reads `AppModel::tracks()` (`lib.rs:289`), `devices()` (`:313`),
`selected_track_id()` (`:293`), `selected_device_id()` (`:324`), `lens()` (`:281`), the
transport, and the generator position via the new `IdGen::state()`. It reads nothing
else and mutates nothing.

`adopt` maps persisted device and parameter keys back to the canonical `&'static str`
descriptors — required, because `DeviceParameterSnapshot::device_key` is a `&'static str`
(`crates/spectre-dsp/src/parameter.rs:83`) and a `String` read from a file is not one —
rejecting any key not in `PULSE_PARAMETERS` / `GAIN_PARAMETERS` / `SATURATOR_PARAMETERS`
(`crates/spectre-app/src/lib.rs:8`), and any value outside its descriptor range. It
applies `TransportCommand::Stop` (`spectre-core/src/transport.rs:86`) before the loaded
transport becomes live. It mutates `model` only after every mapping has succeeded.

**`crates/spectre-project/src/command.rs` — one added command.**

```rust
// Move one track within the persisted collection; identity never moves with the index
pub fn reorder_tracks(from: usize, to: usize) -> Self;
```

Applied as `remove(from)` then `insert(to, track)`, so its exact inverse is
`reorder_tracks(to, from)` — which is what makes it composable with the existing
`Transaction`/`EditHistory` machinery (`command.rs:92–112`, `:134–144`) unchanged. Out-of-range
indices return a new `CommandError::TrackIndexOutOfRange` variant, joining the three at
`command.rs:11–15`, and are rejected before any mutation, preserving
`Transaction::execute`'s all-or-nothing property.

### 4.4 State management

- **Project truth** stays in `AppModel` (`crates/spectre-app/src/lib.rs:205–214`). This
  feature adds **no** field to it. The single-model, linked-lens invariant
  (`vision.md:37`) is preserved: save serializes the one model, and open replaces the one
  model, so no lens can hold a private copy.
- **Shell state** — `project_path: String`, `dirty: bool`, `status: String` — goes on
  `SpectrePrototype` (`main.rs:17–21`), beside the existing `new_track_name` and
  `feedback_status`. A file path is not project content.
- **Dirty rule.** Set on any model mutation reachable from the shell (`toggle_play`,
  `select_lens`, `select_track`, `add_track`, `set_device_parameter`, the inspector's
  direct mutations through `selected_track_mut` at `lib.rs:308`). Cleared **only** on
  `Ok(ReplacedDurable)`. Explicitly **not** cleared on
  `ReplacedDurabilityUncertain` — the contract requires the caller to *"keep the
  in-memory project dirty"*.
- **Save serialization is structural.** The contract requires the app layer to serialize
  saves to the same normalized target and forbids the crate from adding a lock or
  coordinator. `save_project_atomic` is synchronous and `./spectre` calls it from the one
  egui update thread (`main.rs:424`), so a second save cannot begin while the first is
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
boundary, because — per §4.1 — the callback cannot name this code. The one real cost is a
stalled UI frame (§4.7), which is a responsiveness issue, not an RT-001 issue, and is
named as such.

### 4.5 Dependencies

- **New third-party packages: none.** `std::fs`, `std::io`, `std::path`, `std::time`,
  `std::sync::atomic`, `std::process` only. In particular no `tempfile` and no `rfd`.
- **New assets or resources:** one checked-in test fixture,
  `crates/spectre-project/tests/fixtures/r4-canonical.json`, generated by the encoder
  itself the same way `r1-canonical.json` was, and read by `include_bytes!` exactly as at
  `tests/project_codec.rs:9`.
- **New internal crate edges:** none. `spectre-app → spectre-project` already exists at
  `crates/spectre-app/Cargo.toml:16`; this spec is what finally uses it.
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
nine steps; the exact target state at every failure stage; that no partial destination is
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

- **Storage.** The R1 fixture is **912 bytes** on disk. A schema-2 project with one
  track, three device instances, four parameters, and the view block is roughly 2 KB;
  three tracks roughly 2.5 KB. Peak disk usage during a save is the old file plus the
  temporary — under 5 KB for an alpha project. This grows with clips (R4-5) and is not a
  standing estimate for later milestones.
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
  cadence (`main.rs:443`) that is at most a visible hitch, not a freeze. These figures are
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
`envelope()` helper modeled on the one at `src/lib.rs:133–147`.

| # | Name | Setup | Assertion | Edge covered |
|---|---|---|---|---|
| U1 | `validation_failure_touches_no_path` | envelope with two `TrackDoc`s sharing one `ObjectId` | `Err(SaveError::InvalidProject { .. })` **and** `FaultFs`'s call log is **empty** | contract: validation precedes every filesystem touch |
| U2 | `temporary_creation_failure_leaves_target_unchanged` | `FaultFs` fails `CreateTemporary` | `Err(Io { stage: CreateTemporary, target_state: Unchanged, .. })`; log contains no `write_all`, no `replace` | pre-commit failure |
| U3 | `temporary_name_collision_retries_without_truncating` | `FaultFs` returns `AlreadyExists` for the first two names, succeeds on the third | `Ok`; log shows exactly three `create_new` calls with **three distinct paths**, and no `open`-for-write of an existing path | contract: exclusive creation, retry, never truncate another file |
| U4 | `temporary_name_retries_are_bounded` | `FaultFs` always returns `AlreadyExists` | `Err(Io { stage: CreateTemporary, .. })` after exactly `SAVE_TEMP_NAME_ATTEMPTS` `create_new` calls | the bound in §4.2 is real; no unbounded UI-thread loop |
| U5 | `write_failure_cleans_up_and_preserves_target` | `FaultFs` fails `write_all` | `Err(Io { stage: WriteTemporary, target_state: Unchanged, .. })`; log ends with `remove` of the temporary; no `replace` | cleanup is attempted |
| U6 | `temporary_sync_failure_preserves_target` | `FaultFs` fails `sync_file` | `Err(Io { stage: SyncTemporary, target_state: Unchanged, .. })`; log contains `remove`, no `replace` | data must be durable before it is published |
| U7 | `replacement_failure_never_deletes_first` | `FaultFs` fails `replace` | `Err(Io { stage: ReplaceTarget, target_state: Unchanged, .. })`; the log contains **no** `remove` of the *destination* at any point | contract: "Never delete, truncate, or move aside the destination first" |
| U8 | `parent_sync_failure_reports_durability_uncertain` | `FaultFs` fails `sync_parent_dir` | `Err(Io { stage: SyncParentDirectory, target_state: ReplacedDurabilityUncertain, .. })`; the log shows exactly **one** `replace` and no `remove` after it | the one non-`Unchanged` failure; no second replacement attempt |
| U9 | `cleanup_failure_does_not_mask_primary_error` | `FaultFs` fails `write_all` **and** fails `remove` | the returned error is still `Io { stage: WriteTemporary, .. }`, not a cleanup error | contract: cleanup never masks the primary failure |
| U10 | `bounded_read_refuses_oversize` | private `read_bounded(path, max)` called with `max = 16` against a 64-byte temporary file | `Err` with `ErrorKind::InvalidData` and fewer than `max + 1` bytes buffered | makes `MAX_PROJECT_FILE_BYTES` testable without writing 64 MiB |
| U11 | `ordering_is_validate_encode_create` | `FaultFs` that never fails | the first three log entries are `create_new`, `write_all`, `sync_file` in that order, and `replace` follows `sync_file` | the nine-step order is pinned, not incidental |

There is deliberately **no** test for `SaveError::Encode`. With the current encoder a
validated envelope cannot fail to encode, so any such test would be a test that cannot
fail, which `criteria.md` 1G scores 0. The variant exists because the contract requires the
distinction; that it is currently unreachable is stated here rather than papered over with
a green check. Routed to §8 Q11.

*Group B — structural guard (`crates/spectre-project/tests/render_path_isolation.rs`).*

| # | Name | Setup | Assertion | Edge covered |
|---|---|---|---|---|
| U12 | `render_path_crates_do_not_depend_on_the_project_crate` | read `../spectre-audio/Cargo.toml`, `../spectre-graph/Cargo.toml`, `../spectre-dsp/Cargo.toml`, `../spectre-core/Cargo.toml` relative to `env!("CARGO_MANIFEST_DIR")` | none of the four contains the string `spectre-project` | AF-3 / RT-001: fails the moment anyone makes persistence reachable from the callback. Same structural technique the RT module scan already uses in `crates/spectre-audio/tests/rt_guard.rs` |

*Group C — command and codec unit tests (`crates/spectre-project/src/command.rs`,
`src/lib.rs`).*

| # | Name | Setup | Assertion | Edge covered |
|---|---|---|---|---|
| U13 | `reorder_is_exactly_reversible` | `ProjectDoc` with three tracks; `EditHistory::new(8).unwrap()` (`command.rs:122`); apply `Transaction::single(ProjectCommand::reorder_tracks(0, 2))` (`command.rs:86`) | order becomes `[b, c, a]`; `undo` restores `[a, b, c]`; `redo` restores `[b, c, a]`; every `TrackDoc.id` is unchanged at every step | CORE-001 across reorder **and** undo, on the real command path |
| U14 | `reorder_out_of_range_is_rejected_before_mutation` | three tracks; `reorder_tracks(0, 7)` | `Err(CommandError::TrackIndexOutOfRange { .. })` and the vector is untouched | `Transaction::execute`'s all-or-nothing property (`command.rs:92–112`) survives the new command |
| U15 | `duplicate_object_ids_are_rejected` | envelope where a `ParameterDoc.id` equals `project.id` | `validate_envelope` returns `Err`, and so does `from_bytes` on the encoded form | the project-wide uniqueness rule the contract defers to "once persisted object collections exist" |
| U16 | `dangling_selection_is_rejected` | `view.selected_track = Some(id)` for an id in no `TrackDoc` | `validate_envelope` returns `Err` | referential integrity of restored view state |
| U17 | `non_finite_parameter_value_is_rejected_before_encoding` | `ParameterDoc.value = f32::NAN`, then `f32::INFINITY` | `validate_envelope` returns `Err` for both | JSON cannot represent them; catching them here is what keeps a corrupt file from existing |
| U18 | `signed_zero_and_subnormal_values_round_trip_bit_exactly` | `ParameterDoc.value` set to `-0.0` and then to `f32::from_bits(1)` | `to_bytes` → `from_bytes` yields values whose `to_bits()` are equal to the originals | the exact-value discipline the existing snapshot tests already hold the DSP boundary to |

### 5.2 Integration tests

*`crates/spectre-project/tests/project_save.rs` — real filesystem, real temporary
directories.*

| # | Name | Assertion |
|---|---|---|
| I1 | `save_then_load_round_trips_a_populated_project` | `save_project_atomic` returns `Ok(SaveReceipt { target_state: TargetState::ReplacedDurable })`; `load_project` returns an envelope `==` the snapshot (`ProjectEnvelope` derives `PartialEq`, `src/lib.rs:19`) |
| I2 | `successful_save_leaves_no_temporary_behind` | after a save into a directory containing only the target, `read_dir` yields exactly one entry, and its name is the target's | the contract's "no temporary path remains after successful replacement" |
| I3 | `save_over_an_existing_project_replaces_it_completely` | write project A, save project B over it, load: the result is B, and the file contains no fragment of A (byte comparison against `to_bytes(&b)`) | replacement, not merge |
| I4 | `invalid_snapshot_leaves_an_existing_target_byte_identical` | save A; attempt to save an invalid snapshot; the target's bytes are `==` the bytes read immediately after saving A, and `read_dir` still yields exactly one entry | pre-commit failure on a real filesystem |
| I5 | `save_into_a_missing_directory_fails_and_creates_nothing` | `Err(SaveError::Io { stage: SaveStage::CreateTemporary, target_state: TargetState::Unchanged, .. })`, and the target path still does not exist | "a previously absent target remains absent" |
| I6 | `reorder_preserves_identity_across_save_and_reload` | build three tracks; save; apply `reorder_tracks(0, 2)` through `EditHistory`; save; load — the loaded `tracks` are in the reordered order and the `ObjectId`s are the same three values as before, in the new positions | **CORE-001's R4 reorder evidence.** Fails if identity is ever derived from index |
| I7 | `undone_reorder_reloads_in_the_original_order` | continue I6: `undo`, save, load — original order, same IDs | reorder + undo + save/load in one sequence, which is the full clause CORE-001 states |
| I8 | `schema_one_fixture_loads_with_empty_collections` | `load_project` on a copy of `tests/fixtures/r1-canonical.json` succeeds; `tracks`, `devices` are empty; `id_gen_state` is 0; `project.id.raw() == 1_311_768_467_463_790_320`; the `r1_extension` and `future_session` unknown fields are still present | tolerant read of the older schema; CORE-003's preservation guarantee still holds |
| I9 | `resaving_a_schema_one_project_writes_schema_two_and_keeps_unknown_fields` | load the v1 fixture, save it, reload: `schema_version == SCHEMA_VERSION` (2), collections empty, `r1_extension` and `future_session` intact | the version-stamp-on-save rule in §7.2, and CORE-003's unknown-field preservation across it |
| I10 | `a_newer_schema_file_is_refused_and_not_rewritten` | a file whose `schema_version` is `MAX_READABLE_SCHEMA + 1`: `Err(LoadError::SchemaTooNew { .. })`, and the file's bytes are unchanged afterward | failing closed is what protects the file |

*`crates/spectre-app/tests/project_io.rs` — model mapping. This test file is the first
code in `spectre-app` to use the dependency declared at `crates/spectre-app/Cargo.toml:16`.*

| # | Name | Assertion |
|---|---|---|
| I11 | `model_envelope_round_trip_preserves_identity_and_order` | `AppModel::prototype()` (`lib.rs:218`) + two `add_track` calls (`lib.rs:405`); `project_envelope(&model, "T")` → `to_bytes` → `from_bytes` → `adopt` into a fresh `AppModel::prototype()`: `tracks()` match by id, name, and order; `devices()` match by `instance_id`; every `ParameterControl.instance_id` and `value` matches | CORE-001 across save/load for tracks, devices, **and** parameters |
| I12 | `adopting_a_project_stops_the_transport` | envelope whose transport state is `Playing`; after `adopt`, `model.is_playing()` (`lib.rs:264`) is `false` | opening a file never starts playback — and, post R4-1, never starts audio |
| I13 | `a_failed_adopt_leaves_the_model_untouched` | envelope with `DeviceDoc.key = "not-a-device"`: `Err(AdoptError::UnknownDevice { .. })`, and `tracks()`, `devices()`, `selected_track_id()`, `lens()` are all identical to before the call | the contract's "keep the current live project unchanged until that result is available and accepted" |
| I14 | `out_of_range_parameter_values_are_refused_not_clamped` | a `ParameterDoc.value` above its descriptor maximum: `Err(AdoptError::ValueOutOfRange { .. })` | a file must not be able to smuggle a value the UI cannot produce; refusing beats silently clamping on the load path |

*`crates/spectre-offline/tests/harness.rs` — determinism. This file already has
`spectre-app` as a dev-dependency (`crates/spectre-offline/Cargo.toml:23–24`), so no new
dependency is introduced.*

| # | Name | Assertion |
|---|---|---|
| I15 | `a_save_reload_round_trip_does_not_change_what_the_project_renders` | take `model.device_parameter_snapshot()` (`crates/spectre-app/src/lib.rs:348`); render with `spectre_offline::render_app_snapshot(48_000.0, 256, &snapshot)` (`crates/spectre-offline/src/lib.rs:306`); round-trip the model through `project_envelope` → `to_bytes` → `from_bytes` → `adopt`; take the snapshot again; render again — `RenderReport.hash` is **equal** and `peak` is nonzero | Determinism, using the **existing** FNV-1a hash walk in `spectre-offline` rather than a new comparison method. The nonzero peak is what stops two silent renders from agreeing vacuously |

### 5.3 UI / E2E tests

No automated UI test exists or is proposed. `./spectre` has no GUI harness: the only
process-level test is the headless `--smoke-test` path (`main.rs:480–497`, invoked at
`main.rs:500`), which constructs an `AppModel` and prints one line. This spec extends that
line with `project=none` so `crates/spectre-app/tests/smoke_cli.rs` asserts the shell
starts with no project loaded — a real assertion that fails if launch state changes, and
the honest limit of what can be automated here.

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
| Text size / window extremes | At the 1060 × 680 minimum (`main.rs:508`) and with a large system text scale, the PROJECT block still fits the 180 px minimum panel width (`main.rs:119`) and the status line wraps rather than clipping |
| Theme variants | **N/A** — the shell defines one dark palette as constants (`main.rs:9–15`) and no light theme exists. Nothing in this slice introduces one |
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
respect: `./spectre` produces no sound; R4-1 is specified but not implemented; and no
screen-reader claim is authorized while `eframe`'s features are as declared at
`crates/spectre-app/Cargo.toml:13`.

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
  system. What is persisted is the four existing parameter values across the three
  existing device instances (`crates/spectre-app/src/lib.rs:348–383`). Decision 15
  (`decision-gates.md:39`) is untouched.
- **3D Originality.** Schema, field names, error vocabulary, and algorithm are Spectre's.
  The atomic write-temp-fsync-rename-fsync-dir sequence is a POSIX idiom, not a product's
  intellectual property, and it is specified here from the accepted contract rather than
  transcribed from any tool. The three numeric bounds have Spectre-derived rationales
  (§4.2) and ledger rows (§7.2); none is copied.
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

Read at branch `rename/geist-to-spectre`, HEAD `2e005e5`. Every row is checkable in one
`Read`.

**Absent — this is the gap R4-7 closes.**

- **`crates/spectre-app` declares `spectre-project` and never uses it.**
  `crates/spectre-app/Cargo.toml:16` reads
  `spectre-project = { path = "../spectre-project" }`, and **grep for `spectre_project`
  across `crates/spectre-app/src/` and `crates/spectre-app/tests/` returns nothing.** The
  app crate is two source files (`lib.rs`, `main.rs`) and two test files (`app_model.rs`,
  `smoke_cli.rs`); none of them names the persistence crate. This is the cleanest possible
  statement of how far this feature is from existing: the dependency edge is declared and
  entirely unused.
- **No filesystem code exists anywhere in `spectre-project`.** The crate is
  `src/lib.rs` (221 lines), `src/command.rs` (182 lines), and two test files. There is no
  `fs` module, no `load_project`, no `save_project_atomic`, no `SaveStage`, no
  `TargetState`, no `SaveReceipt`, and no use of `std::fs` or `std::path`. The only file
  I/O in the crate is `include_bytes!` of a checked-in fixture at
  `tests/project_codec.rs:9`, which is compile-time.
- **`main.rs`'s own header says so.** `crates/spectre-app/src/main.rs:4`: *"Interaction
  prototype only; audio and persistence wiring remain out of scope."* There is no menu
  bar, no file dialog, no path field, and no save or open button in the shell.
- **No persisted object collection exists.** `ProjectDoc` (`src/lib.rs:30–38`) has
  exactly four typed fields — `id`, `name`, `tempo_map`, `transport` — plus the flattened
  unknown map. There is no `tracks`, no `devices`, no `view`, and no `id_gen_state`.
  `crates/spectre-core/src/id.rs:4` records the consequence in the source itself:
  *"project-wide duplicate validation arrives with persisted object collections."*
- **`./spectre` produces no sound.** `spectre:9` runs `cargo run -p spectre-app`; the app
  does not depend on `spectre-audio`; R4-1 is specified but **not implemented**. Nothing
  in this spec changes that, and nothing in it should be read as implying otherwise.

**Implemented — `spectre-project`, all of it in memory.**

| Element | Path | What it actually does |
|---|---|---|
| `SCHEMA_VERSION = 1`, `MAX_READABLE_SCHEMA = 1` | `src/lib.rs:13`, `:16` | schema identifiers |
| `ProjectEnvelope`, `ProjectDoc` with `#[serde(flatten)]` unknown maps | `src/lib.rs:20–27`, `:30–38` (flatten at `:24`, `:36`) | CORE-003's forward-field preservation |
| `ProjectDoc::meter_map` | `src/lib.rs:42–49` | decodes optional meter state out of the unknown map |
| `ProjectError { SchemaTooNew, InvalidProject(&'static str), Malformed(serde_json::Error) }` | `src/lib.rs:54–58` | three of the five distinctions the load contract needs; it has **no** I/O variants, because there is no I/O |
| `to_bytes` | `src/lib.rs:80–82` | `serde_json::to_vec_pretty`. **Does not revalidate** — the design authority's "Current codec qualification" section says so explicitly |
| `from_bytes` | `src/lib.rs:85–103` | peeks the version (`:86–91`), gates it (`:92–97`), decodes, then calls `validate` (`:101`) |
| `validate` — **private** | `src/lib.rs:105–128` | nonzero project ID, tempo-map reconstruction, meter-map decode, loop-enabled-requires-region, ordered loop region. Not callable from outside the crate |
| `ProjectCommand::rename`, `Transaction`, `EditHistory` | `src/command.rs:43`, `:72–90`, `:114–180` | in-memory command transactions with generated inverses (`:92–112`) and bounded undo/redo with oldest-edit eviction (`:176–181`). `command.rs:4` calls it the *"App-thread project mutation seam; never callback-reachable"* |

**Verified — the R1 codec fixtures.** `crates/spectre-project/tests/fixtures/r1-canonical.json`
(52 lines, 912 bytes, `schema_version: 1` at `:2`, project id `1311768467463790320` at
`:4`, an `r1_extension` unknown block at `:48–51`), driven by nine tests in
`crates/spectre-project/tests/project_codec.rs`: decode with representative state
(`:21–50`), byte-stable rewrite (`:52–58`), decode-encode-decode round trip (`:60–67`),
unknown-field survival (`:69–83`), newer-schema rejection (`:85–98`), and invalid
tempo/meter/loop rejection (`:100–110`, `:112–122`, `:124–134`). Five more codec unit
tests live in `src/lib.rs:129–221` and eight command tests in
`tests/command_history.rs`. CORE-003 is `verified` in the ledger (`:48`) on this evidence.
**None of these tests opens a file at runtime.**

**Implemented — what the app has that would need to persist.**

- `AppModel` (`crates/spectre-app/src/lib.rs:205–214`): `transport`, `lens`, `tracks:
  Vec<TrackView>`, `devices: Vec<DeviceControl>`, `selected_track`, `selected_device`,
  `ids: IdGen`, `feedback`. **`tracks` is already an ordered collection of objects
  carrying stable `ObjectId`s** (`TrackView` at `:38–45`), created by `prototype()`
  (`:218–262`) and extended by `add_track` (`:405–422`). It is not persisted anywhere.
- `DeviceControl` (`:56–63`) and `ParameterControl` (`:48–53`) each carry an
  `instance_id: ObjectId` minted from the same generator.
- `device_parameter_snapshot` (`:348–383`) publishes exactly four validated
  `DeviceParameterSnapshot` values by canonical identity.
- `IdGen` (`crates/spectre-core/src/id.rs:45–67`) is deterministic splitmix64 seeded at
  `crates/spectre-app/src/lib.rs:219`; it exposes `next_id` but **no accessor for its
  state**, so its position cannot currently be saved.

**Accepted but not implemented — the design this spec implements.**
`docs/03-architecture/project-persistence.md` is `accepted for R4/R5 implementation` and
specifies the two signatures, the load ordering, the save algorithm's nine steps, the
failure vocabulary, the per-stage target-state table, the platform-qualification split,
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
- `crates/spectre-project/tests/render_path_isolation.rs` — U12.
- `crates/spectre-project/tests/fixtures/r4-canonical.json` — schema-2 canonical fixture
  with three tracks, three devices, four parameters, and a view block.
- `crates/spectre-app/src/project.rs` — `project_envelope`, `adopt`, `AdoptError`, the two
  `Lens ↔ LensDoc` conversions.
- `crates/spectre-app/tests/project_io.rs` — I11–I14.

**Modified files**

- `crates/spectre-project/src/lib.rs` — `pub mod fs;`; `SCHEMA_VERSION`/
  `MAX_READABLE_SCHEMA` to 2 (`:13`, `:16`); the four new `ProjectDoc` fields (`:30–38`);
  `TrackDoc`, `DeviceDoc`, `ParameterDoc`, `ViewDoc`, `LensDoc`; `ValidationError`;
  `validate` (`:105–128`) becomes `pub fn validate_envelope` returning `ValidationError`,
  with the existing rules preserved verbatim and the four schema-2 rules added; the call
  site at `:101` adapts. `to_bytes` (`:80–82`) and `from_bytes` (`:85–103`) keep their
  signatures, so `spectre-offline`'s use at `crates/spectre-offline/src/lib.rs:14` is
  unaffected. U15–U18 added to the module's test block.
- `crates/spectre-project/src/command.rs` — `CommandKind::ReorderTracks`;
  `ProjectCommand::reorder_tracks`; `CommandError::TrackIndexOutOfRange` joining `:11–15`;
  U13–U14.
- `crates/spectre-core/src/id.rs` — `IdGen::state()` accessor. One method, no behavior
  change; `next_id` (`:56–67`) and `IdGen::new` (`:51–53`) untouched.
- `crates/spectre-project/tests/project_codec.rs` — **three assertions change, and the
  change is not cosmetic.** `:25` asserts `envelope.schema_version == SCHEMA_VERSION`,
  which is false once the constant is 2, and becomes an assertion against the literal `1`.
  `canonical_fixture_rewrite_is_byte_stable` (`:52–58`) **cannot survive a version bump by
  construction**: rewriting a v1 document stamps the current version, so the bytes
  necessarily differ. It is retargeted to the new `r4-canonical.json`, and its v1 role is
  taken by I8 and I9, which assert the stronger property — that a v1 file still decodes
  completely and that its unknown fields survive the upgrade. The remaining seven tests are
  unchanged. **This touches the acceptance evidence of a `verified` requirement
  (CORE-003, `requirements-ledger.md:48`), so it is flagged here and routed to §8 Q2
  rather than done quietly.**
- `crates/spectre-app/src/lib.rs` — `pub mod project;`. No `AppModel` field or
  method-semantics change.
- `crates/spectre-app/src/main.rs` — the PROJECT block inside `track_list`
  (`:115–158`); `project_path`, `dirty`, `status` on `SpectrePrototype` (`:17–21`) and its
  `Default` (`:23–32`); the dirty marker in `transport` (`:50–84`); dirty-setting on the
  existing mutation call sites; `project=` in the smoke line (`:486–496`); and the header
  Notes line at `:4`, which currently says persistence is out of scope and will no longer
  be true.
- `crates/spectre-app/tests/smoke_cli.rs` — assert `project=none`.
- `crates/spectre-offline/tests/harness.rs` — I15.
- **`docs/01-requirements/requirements-ledger.md`** — three rationale rows, one per
  numeric bound: `SAVE_TEMP_NAME_ATTEMPTS`, `MAX_PROJECT_FILE_BYTES`, and the track level
  range, each carrying the §4.2 text. PROD-003 (`requirements-ledger.md:64`) requires the
  rationale to be recorded **in that ledger**, and decision 16 (`decision-gates.md:40`)
  makes it a standing rule, so §4.2's table does not discharge it on its own. The rows
  are proposed in the CORE family (`:44–49`) as CORE-005, CORE-006, CORE-007, since all
  three are project-envelope bounds. The same edit updates CORE-001's evidence column
  (`:46`) to cite I6/I7 as the reorder evidence and CORE-004's (`:49`) to record the
  implementation, moving it to `implemented`.
- `docs/03-architecture/project-persistence.md` — no content change, but its "Open
  decisions" and "Known gaps" lines shorten when the filesystem implementation lands;
  R5's items stay.
- `docs/status/STATUS.md`, `docs/status/NEXT.md`, `docs/06-plans/current-milestone.md` —
  update when the slice lands, per `docs/README.md`'s working rule. STATUS must move
  CORE-004 to **`implemented`, not `verified`** (§4.6), must not claim crash durability,
  and must not weaken its standing "`./spectre` makes no sound" line, which this slice
  does not change.

**Deliberately not modified:** all of `crates/spectre-audio`, `crates/spectre-graph`,
`crates/spectre-dsp`; `crates/spectre-offline/src/lib.rs`; every RT module scanned by
`crates/spectre-audio/tests/rt_guard.rs`. No render code, no DSP, no callback-reachable
path, and no `Cargo.toml` in the render-path crates is touched — U12 exists to keep that
true.

**Migrations / schema changes:** the version moves 1 → 2 with a **tolerant read plus a
version stamp on save**: absent fields default, and a save always writes the current
version. This is deliberately *not* the migration framework R5 owns — there is no
transformation of existing data, no down-conversion, no salvage, and no per-version code
path. Whether that boundary is drawn where Jeff wants it is §8 Q1.

**New dependencies:** none.

### 7.3 Estimated scope

**L.** Roughly 400–500 new lines across `spectre-project` (the `fs` module dominates),
about 120 in `spectre-app`, and a test surface larger than the implementation: 18 unit
tests, 15 integration tests, one new fixture. It is above **M** for three reasons that are
each independently costly — a schema version bump that touches an existing `verified`
requirement's acceptance tests; a private generic seam that has to be designed so it stays
private while still reaching all seven stages; and a correctness surface where the tests
are the deliverable, since "the old file is intact" is only true if something checks it at
every stage. It is below **XL** because it writes no DSP, adds no dependency, touches no
callback-reachable code, and implements an API that is already designed line by line.

### 7.4 Blocking dependencies

- **Nothing blocks authorship or implementation.** Every piece this composes — the
  envelope, the validator, the command transactions, the ID generator, the app model's
  track collection — exists today and is tested.
- **R4-1 (`live-audio-wiring`) does not block this and is not blocked by it.** They touch
  disjoint files. Two coordination points: the transport-bar edit (this spec adds a dirty
  marker, R4-1 replaces the `ENGINE OFFLINE` and `CPU —` literals at `main.rs:79–80` —
  whichever lands second rebases), and §3.2 step 4's transport-stop-on-open, which only
  becomes load-bearing once an engine exists.
- **R4-4 (`track-model`) will extend `TrackDoc`, not replace it.** This spec persists the
  track collection that exists **today** (`AppModel.tracks`, `lib.rs:208`), so it does not
  wait on R4-4. When R4-4 adds routing and a signal path, those become fields on
  `TrackDoc` and the schema question repeats — which is the argument in §8 Q1 for settling
  the additive-versus-bump rule now rather than per slice.
- **R4-5 (`midi-clips`) and R4-6 (`small synth/effect`) extend the schema the same way.**
  R4's exit evidence requires the round trip to include clips; this spec cannot deliver
  clips that do not exist and does not claim to. It delivers tracks, devices, parameters,
  and view state, and states the remainder as R4-5/R4-6's to add.
- **R4-8 (`offline-bounce`) depends on this**, in that a bounce of a *loaded* project is
  only meaningful once loading exists. I15 is deliberately the shape R4-8 will extend.
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
  This spec treats "absent fields default, save stamps the current version" as tolerant
  reading, not migration, because nothing is transformed and there is no per-version code
  path. The alternative is to keep `SCHEMA_VERSION = 1` and make `tracks`/`devices`
  additive with `skip_serializing_if`, which preserves the existing byte-stability test
  untouched — at the cost that an older build silently opens a project with tracks and
  shows the musician an empty one. §4.2 argues the bump is safer for exactly that reason.
  — *blocks: §4.2, §7.2*
- **Q2 — Retargeting `canonical_fixture_rewrite_is_byte_stable`.** It is acceptance
  evidence for a `verified` requirement (CORE-003, `requirements-ledger.md:48`) and cannot
  survive a version bump by construction. This spec retargets it to the new v2 fixture and
  replaces its v1 role with the stronger I8/I9 pair. Confirm that this is a preserved
  guarantee under a new fixture and not a weakened one. — *blocks: §5.2, §7.2*
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
- **Q5 — Enable `eframe`'s accessibility feature at R4?** `crates/spectre-app/Cargo.toml:13`
  sets `default-features = false`, so no screen-reader integration is compiled in and none
  is claimed. Decision 17 (`decision-gates.md:41`) scopes an audit at R4; turning the
  feature on is one manifest line plus whatever the audit finds. This spec does not do it
  unilaterally. — *blocks: §3.7*
- **Q6 — Is track arm state project state or session state?** Not persisted here, because
  recording does not exist and restoring an armed track into a live engine is a recording
  decision. There is **no citable benchmark evidence either way** in the accepted corpus
  (Appendix A). — *blocks: §4.2*
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

Recorded as a research need: the file-and-set-management chapters of the Ableton manual
are the highest-value extraction for any future persistence work, and they are already
identified as unextracted at `ableton-live-observations.md:19`.

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
`research-state: in-review` (`:10–11`) and untracked in git; its `OBS-OZ-PRESET-009`
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
