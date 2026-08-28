<!--
Author: Jeff
Date: 2026-08-24
Description: Blind independent scorecard for R4-7 project-persistence, iteration 2
Notes: Graded from the spec and from source at HEAD af0e553; every cited path opened and checked
-->

# Scorecard: Project Persistence

**Feature ID:** `R4-7` (`project-persistence`)
**Spec file:** `gauntlet-output/specs/R4-7-project-persistence.md`
**Reviewer agent:** blind verification agent, iteration-2 pass
**Date:** 2026-08-24
**Spec iteration reviewed:** 2

**Graded against:** HEAD `af0e553` (`gauntlet: R4-7 spec remediation 1 answers the feasibility
failure`). The spec declares its own baseline as `8b1633d`; where the two differ I checked both
and say which. `8b1633d` = R4-1, `20f3056` = R4-2, both landed 2026-08-24.

**Prior-scorecard disclosure — read this.** I did not open
`R4-7-project-persistence-scorecard.md` or `R4-7-project-persistence-scorecard-independent.md`.
However, a late repo-wide `grep -rn "ProjectDoc"` returned six matching *lines* from
`R4-7-project-persistence-scorecard-independent.md` (its lines 29, 109, 140, 141, 221, 228) in
the result set, so I saw those fragments. Every finding in this scorecard that overlaps them —
the `ProjectDoc {` census, the `#[derive(Default)]` impossibility, the three test/step counts,
the version stamp's ownership — had already been derived independently and recorded earlier in
my working transcript, before that grep ran. Nothing below is carried over from the prior
verdict, and one of my Priority 1 findings is specifically that a claim the earlier fragment
records as *true* (`crates/spectre-app/Cargo.toml:16`) is **false at this spec's own declared
baseline**.

---

## Verdict: FAIL

**Summary:** This is a strong spec. The six remediations it set out to make all verify against
source — the `ProjectDoc` blast radius is exactly right, the `#[derive(Default)]` escape really
is closed by `ObjectId`'s nonzero invariant, all three disputed counts are now correct, the
version stamp has a real owner whose two halves are pinned from opposite sides by I9 and I15,
I11's device assertion is now genuinely falsifiable, and Q12 is a real defect correctly routed
to Jeff rather than resolved by assertion. It nonetheless **fails the feasibility rule**, for a
narrower reason than iteration 1: §7.1 states that its citations were re-baselined onto
`8b1633d` "verified rather than assumed" and that "This spec pins no line in
[`crates/spectre-app/src/main.rs` or `Cargo.toml`]", and then pins `crates/spectre-app/Cargo.toml:16`
for the single most load-bearing claim in the document — where that line is
`live-audio = ["spectre-audio/cpal-backend"]`, not the `spectre-project` dependency, which is at
`:24`. Eleven further pins into `crates/spectre-app/src/lib.rs` (six of them inside §7.1) are
un-rebaselined `2e005e5` values, off by exactly the `+3` the same section computes and correctly
applies elsewhere. **The single most important fix:** re-derive every `crates/spectre-app`
citation at the current HEAD and delete the `Cargo.toml` line pin in favour of the quoted literal
the spec already uses correctly in §3.7.

---

## Lens 1: Realtime & Correctness (weight: 35%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| **1A. Callback-path discipline** | 2 | §4.1's structural argument is *half* verified and *half* invalidated by the commit the spec re-baselined onto. Verified exactly: `crates/spectre-audio/Cargo.toml:17–21` = `cpal`, `spectre-core`, `spectre-dsp`, `spectre-graph`; `spectre-graph/Cargo.toml:12–14`; `spectre-dsp/Cargo.toml:12–13`; `spectre-audio/Cargo.toml:23–24` dev-depends on `spectre-offline` — every one exact, and none names `spectre-project`. The qualification about `spectre-audio`'s test targets is correct and honest. **But the conclusion overreaches:** "they are **unnameable** in every render-path *library* target, which is the target the callback runs in" is false at `8b1633d`. R4-1 put the render closure at `crates/spectre-app/src/engine.rs:364` — `Box::new(move \|mut block: RenderBlock\| bridge.render(&mut block))` — inside `spectre-app`'s **library** target (`pub mod engine;`, `lib.rs:7`), and `crates/spectre-app/Cargo.toml:24` declares `spectre-project`. So `spectre_project::save_project_atomic` **is** nameable in the crate that now builds the callback closure. Nothing actually calls it there (`engine.rs:4–6` says so, and `grep spectre_project crates/spectre-app/` returns nothing), so the design is sound; the *argument* the spec calls "a compile-time impossibility, not a code-review promise" is not. §4.4's "the callback cannot name this code" inherits the same error. | Restate §4.1: the compile-time impossibility covers `spectre-audio`/`graph`/`dsp`/`core`, and `spectre-app`'s lib target is a code-review property because R4-1 moved the closure there. Extend U12 to `spectre-app` — see 1G. |
| **1B. Control↔render communication** | 3 | §4.4 "The audio thread, precisely" is correct and appropriately narrow: this feature adds no control traffic, so it states its non-effect on the RT-002 lanes instead of inventing a lane. "no parameter, note, or transport message is *sent*; nothing is dropped, because the sender is simply not called; and the RT-002 lanes are neither drained faster nor slower" is exactly right given `engine.rs`'s `control_channel`/`ControlSender` ownership on the app thread. Latest-wins/FIFO are not restated because nothing here uses them, which is the correct handling rather than an omission. §3.2 step 4 routes the loaded transport through `TransportCommand::Stop` (`spectre-core/src/transport.rs:86` — verified exact) on the model, not across the lane. | — |
| **1C. Numerical containment** | 3 | No new DSP node, and the spec says so rather than manufacturing an RT-003 section. It instead applies the contain-at-the-boundary posture where it actually bites here: §4.2 rule 4 rejects non-finite `ParameterDoc.value` **before** encoding, with the correct reason (JSON has no NaN/Inf literal, so an honest encode cannot represent it), and U17 names the injection explicitly for `f32::NAN` and `f32::INFINITY`. U18 adds `-0.0` and `f32::from_bits(1)` bit-exact round-trip. RT-003 (`requirements-ledger.md:30` — verified) is cited *by analogy* and labelled as analogy, not claimed as coverage. Descriptor-range validation is correctly refused for `spectre-project` and located where descriptors live (`DeviceValues::from_snapshot` at `spectre-offline/src/lib.rs:87–108` and `:110–120`, `DeviceParameterSnapshot::new` clamping at `spectre-dsp/src/parameter.rs:89–103` — all three pins exact). | — |
| **1D. Determinism** | 2 | I16 uses the **existing** FNV-1a walk rather than a new comparison method, exactly as 1D requires: `spectre_offline::render_app_snapshot` is real at `crates/spectre-offline/src/lib.rs:306` (exact) and `RenderReport.hash` is the harness's own. The nonzero-peak guard against two silent renders agreeing is the right instinct. **The defect is the one the spec repaired in I11 and did not carry to I16.** As written, I16 takes a snapshot, renders, round-trips *the model*, snapshots again, renders again, and asserts the hashes are **equal**. If the model is an unedited `AppModel::prototype()`, an `adopt` that ignored the persisted devices entirely would leave every parameter at its descriptor default and the hashes would still be equal — the test passes while the feature does nothing. The spec's own §5.2 rationale for I11 spells out this failure mode ("A source model and a fresh target model therefore agree on device `instance_id`s and on every parameter value *by construction*") and does not apply it here. | I16 must edit at least one parameter away from its default before the first render, and must `adopt` into a **fresh** `AppModel::prototype()`, so a dropped or ignored device changes the hash. Say so in the assertion column. |
| **1E. Graph and plan contract** | 3 | Nothing allocates on the render side; no plan recompilation is proposed anywhere, per parameter change or otherwise, so decision 22's rejected option (b) is not approached. §7.2's "Deliberately not modified" names all of `crates/spectre-audio`, `crates/spectre-graph`, `crates/spectre-dsp` and every module scanned by `crates/spectre-audio/tests/rt_guard.rs` (file verified present, and its header confirms it is a module scan, so §5.1's "same structural technique" is accurate). GRAPH-001's editable/compiled split is untouched because this slice never reaches the graph. | — |
| **1F. Failure behavior** | 3 | The strongest section of the spec, and every guarantee traces to the accepted contract, which I read. Fail-closed throughout: validation precedes every filesystem touch (contract `:129`); `target_state: Unchanged` at every pre-commit stage (contract `:139–141`); the one non-`Unchanged` case keeps the project dirty and is forbidden from reading as a plain success (contract `:146` — "keeps the in-memory project dirty ... MUST NOT trigger a second replacement attempt", and §3.6's row honours both); `SchemaTooNew` refuses rather than showing a partial project the user could overwrite; cleanup is best-effort, synchronous, and never masks the primary error (contract `:144`), pinned by U9. Nothing is logged on a callback path because nothing runs there. §3.6's table gives every one of the five `LoadError` and three `SaveError` shapes a distinct message and an explicit data-loss column, with the only "Partial, and named" row being the one the contract says is partial. | — |
| **1G. Test specification** | 2 | Commands are real and exact (see 4E); I ran `cargo test --locked -p spectre-project` and it passes. Most assertions are genuinely falsifiable and several are sharp: U1's **empty** call log, U3's three *distinct* paths, U4's exactly-`SAVE_TEMP_NAME_ATTEMPTS` count, U7's "no `remove` of the *destination* at any point", U8's exactly-one `replace`, U11's pinned step order, I4/I5's byte-identity and continued-absence, I6/I7's identity-across-reorder-and-undo, I9/I15 pinning the stamp rule from opposite directions. The spec also **refuses** two tests that could not fail (`SaveError::Encode`; the concurrency test) and says why, which is the correct behaviour under 1G rather than a gap. I11's repair verifies at source: `SATURATOR_PARAMETERS[0]` is `parameter("drive", "Drive", ParamUnit::Linear, 1.0, 24.0, 1.0)` (`crates/spectre-dsp/src/effect.rs:24`), so `6.0` survives `set_device_parameter`'s `descriptor.clamp` (`spectre-app/src/lib.rs:404`) and `6.0 != default() == 1.0` — the assertion discriminates. **Two holes.** (i) I16 cannot fail as specified (1D). (ii) U12 reads only `spectre-audio`, `spectre-graph`, `spectre-dsp`, `spectre-core` manifests — four crates that never could have depended on `spectre-project` — and omits `spectre-app`, the one crate that both depends on it and now hosts the render closure. The guard is aimed away from the only live risk. | Fix I16 (1D). Extend U12 to assert `spectre_project` is absent from `crates/spectre-app/src/engine.rs` (or from whatever module list the app's render closure lives in), which is the assertion that would actually fire. |

**Lens average:** (2+3+3+2+3+3+2) / 7 = **2.571**
**Lens pass:** Yes — avg ≥ 2.0, zero 1s, zero 0s.
**AF-3 triggered:** No. The spec puts no allocation, lock, I/O, logging, or panic on a
callback-reachable path; save and load are synchronous app-thread calls and the spec is explicit
and correct that a blocked UI thread stalls messages to the engine, never the engine itself. The
1A finding is an over-claimed *argument*, not a violated *design*.

---

## Lens 2: DAW Workflow Depth (weight: 25%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| **2A. Loop-first core loop** | 3 | §3.1 introduces no window, modal, sheet, popover, or drawer; §3.5 states plainly that no navigation transition exists "because no view is entered or left". Selection and lens survive the round trip by design — `ViewDoc` carries `lens`, `selected_track`, `selected_device`, and §4.2 rule 2 makes a dangling selection *invalid* rather than silently dropped, so a reopened project resumes rather than reconstructs. The discard guard is a two-press in-place control, not a focus-stealing dialog, with the rationale given in §3.6. The one real cost — a synchronous save blocking the UI thread — is stated in §3.5 and §4.7 instead of masked with a spinner, and §5.4 requires it be timed by hand. | — |
| **2B. Linked lenses** | 3 | §4.4 is unambiguous: project truth stays in `AppModel`, this feature adds **no** field to it, save serializes the one model and open replaces the one model, "so no lens can hold a private copy". I verified `AppModel`'s eight fields at `crates/spectre-app/src/lib.rs:208–217` (exact) — `transport`, `lens`, `tracks`, `devices`, `selected_track`, `selected_device`, `ids`, `feedback` — and there is no per-lens state to fork. Stable identity is the point of the slice: `TrackDoc.id` is an `ObjectId`, presentation order is vector order, and I6/I7 prove identity survives reorder and undo. Shared selection is persisted once in `ViewDoc`, not per lens. | — |
| **2C. Modulation visibility** | 2 | Correct for what exists and no more: R4 has no automation or modulation layer, and the spec persists base parameter values only, without implying otherwise. `ParameterDoc` carries `value` plus a `#[serde(flatten)]` unknown map, so a later automation/modulation split is not foreclosed at the schema level. What is missing is any statement of *how* the document will keep base, automation, and modulation contributions distinct when they arrive, or where PROD-002's explicit restore action would live — the spec's own §7.4 anticipates R4-4/R4-5/R4-6 extending this schema, so this was the natural place to state the shape. Minor gap, not a contradiction. | Add one line to §4.2 or §7.4 naming that a persisted parameter is the *base* value and that automation/modulation get their own document fields under PROD-002, so the field name is not later overloaded. |
| **2D. Keyboard-first, calm UI** | 3 | Precisely AF-5-aware and correct about it. §3.4: "**No default keyboard shortcut is specified for save or open, and no shortcut map is proposed**", traced to `product-implications.md` §"Prohibited conclusions at current evidence level", with the explicit refusal to claim ⌘S. The commands are *named* (`project.save`, `project.open`) so a later remappable, context-scoped resolver can bind them without this spec fixing a binding — exactly the distinction 2D draws. Existing `Space` and `1`–`4` bindings are stated as unchanged and I verified them at `crates/spectre-app/src/main.rs:612–619`. No density is added: a label, a field, two buttons, one status line, reusing the add-track widget vocabulary already in the same panel (verified at `main.rs:289–301` — `TextEdit::singleline` with `hint_text` beside a small button whose `Err` writes `self.feedback_status`, exactly as §3.1 describes). No cables, no spreadsheet. | — |
| **2E. Convergent-pattern grounding** | 3 | Appendix A finds a real convergence and uses it correctly: `OBS-PP-UNI-003` (Phase Plant — bend range "is saved with the project, not the preset"), `OBS-AB12-WARP-005`, and `OBS-AB12-CLIP-002` (Ableton — set versus `.asd` sidecar) together show two researched products drawing an explicit documented line between project, preset, and sidecar. All three IDs verified present in the accepted corpus. Spectre's divergence is stated as a *deliberate simplification* — no preset layer, no sidecar yet, so everything persisted is project state — and §4.2 draws the line anyway by naming what is excluded (`feedback`, `armed`, every engine measurement), with the note that R5's autosave sidecar under decision 14 must respect it. That is following the pattern and saying where Spectre sits inside it. | — |
| **2F. Differentiation** | 3 | §Appendix A's differentiation paragraph is small, honest, and defensible: the failure vocabulary is part of the product surface, `TargetState` makes "saved and durable" versus "saved but the directory entry may not survive a power loss" machine-checkable rather than editorial, and the project stays dirty until the strong case is reached. Crucially it does **not** claim any benchmark fails to do this — it claims only that Spectre does, which is the correct move given the corpus has no record either way. Parity is explicitly refused as completeness: "R4 has no autosave, no recovery, no migration, and no crash evidence, all of which the benchmark products have had for years". | — |
| **2G. Benchmark evidence discipline** | 3 | The best-executed lens-2 criterion in the spec, and per criteria.md naming the hole is a 3, not a penalty. The gap is stated exactly — "no citable observation in the accepted corpus describing how any benchmark product writes a project file, what it does when a save is interrupted..." — with a checkable reason per product, all of which I verified: `ableton-live-observations.md:12` scopes to chapters 6, 7, 8, 9, 16, 17, 18, 19, 25, 41 (exact); Logic Pro has zero behavioral records; `OBS-SR2-CPU-001`/`OBS-SR2-KB-001` at `synth-modular-observations.md:54–55` (exact) are the only two Serum 2 records and neither concerns persistence; the larger `serum-2-observations.md` carries its **QUARANTINED — NOT ACCEPTED RESEARCH** banner at `:10–12` (exact) and nothing is taken from it. Bitwig's `OBS-BW53-CON-001` is used and explicitly labelled corroborating-only, per criteria.md. `OBS-VCV-VOLT-006` is used "only by analogy, and labelled as such". The Ozone dossier is disclosed as read-and-deliberately-not-leaned-on. The appendix even **retracts** a prior citation of its own: chapter 5 is `section-inventoried` (`ableton-live.md:49`, exact) and is *not* among the unextracted chapters at `ableton-live-observations.md:19` ("2, 10–15, 20–24, 26, 33, 36–40" — exact), and the recorded 5.4 open questions are verbatim ("schema/versioning, atomic save, recovery, merge identity/conflicts, unknown-data preservation, and transactional undo", `ableton-live.md:142`). One blemish, graded under 4A rather than here because it is not a benchmark claim: the appendix says `ozone-observations.md` is "untracked in git"; it is tracked at `8b1633d` and HEAD (added in `b5af060`), untracked only at iteration 1's `2e005e5`. | — |

**Lens average:** (3+3+2+3+3+3+3) / 7 = **2.857**
**Lens pass:** Yes.
**AF-5 triggered:** No — the spec cites the prohibition and complies with it, including the
explicit refusal of a default shortcut map.

---

## Lens 3: Product Identity & Scope Discipline (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| **3A. Milestone fit** | 3 | Dead centre of R4 and verified against the accepted plans: `vision.md:48` names `save/reload` among the six credible-alpha items (exact); `docs/status/NEXT.md:29` is this slice **verbatim** — "Implement CORE-004's atomic save and reload over the accepted persistence contract, and land CORE-001's reorder evidence on the first persisted collection" (exact); `current-milestone.md` §"Inherited debt" items 3 and 4 assign exactly these two obligations to R4 (exact). Everything beyond is deferred **by name** in §7.4 and §4.6 — journaled autosave, recovery selection, migrations, salvage, missing-media, crash qualification, the supported filesystem matrix — each to R5 on the design authority's own milestone table (`project-persistence.md:172–174`, exact) and `rebuild-roadmap.md:30` (exact). Nothing is smuggled. | — |
| **3B. Non-goal respect** | 3 | §6.4 states it and §3 honours it: no plugin hosting of any format, no first-party device published as a plugin, **no cross-DAW project or preset compatibility of any kind**, no cloud, no content store, no video. `vision.md:56` — "No preset/project compatibility with any other DAW or synth." — verified exact. The file format is Spectre's own, and there is no import or export affordance anywhere in §3's inventory, flows, or layout. | — |
| **3C. Deliberately small first devices** | 3 | No device is added, grown, or given a preset system. What is persisted is the four existing parameter values across the three existing device instances — verified: `AppModel::device_parameter_snapshot` publishes exactly four (`crates/spectre-app/src/lib.rs:351–386`, four fixture rows at `:354–359`), and `prototype()` builds exactly three `DeviceControl`s (`:231–253`). Decision 15 at `decision-gates.md:39` verified and untouched. `adopt` deliberately refuses unknown device or parameter keys rather than inventing capacity for them. | — |
| **3D. Originality** | 3 | Schema, field names, error vocabulary, and algorithm are Spectre's; the write-temp/fsync/rename/fsync-dir sequence is correctly characterised as a POSIX idiom taken from the **accepted contract**, not transcribed from any tool, and I confirmed §4.3's table is a faithful implementation of `project-persistence.md:120–127` rather than an import. AF-4 is discharged properly: all three numeric bounds carry Spectre-derived rationales (§4.2's table) *and* proposed ledger rows in §7.2, which is what PROD-003 actually requires — `requirements-ledger.md:53` states the rationale must live in that ledger, and decision 16 at `decision-gates.md:40` makes it standing. `TRACK_LEVEL_RANGE` is honestly presented as making an existing undocumented UI bound explicit rather than as a new number (verified: the inspector slider and the `0.78`/`0.72` seeds are real). The one new checked-in fixture is generated by Spectre's own encoder from Spectre's own types. | — |
| **3E. Platform commitment** | 3 | §4.6 is the model answer for this criterion. It separates *guaranteed on both* (`rename(2)` atomic replacement within one filesystem, `O_CREAT\|O_EXCL`, directory open) from *platform-specific*, names the real divergence concretely (`fsync(2)` versus `fcntl(F_FULLFSYNC)` on Apple; directory-`fsync` defined on Linux, undocumented on APFS and possibly `EINVAL`/`ENOTSUP`), and — the part that earns the 3 — **refuses to treat the `std`-on-Apple mapping as verified**, because "no file in this repository can settle it", routing it to Q9 instead of asserting it. It then shows that the accepted contract already degrades explicitly for the platform that cannot provide step 8 (`ReplacedDurabilityUncertain`, never a silent weakening), and refuses to add any row to a qualification table or claim NFS/SMB/FUSE/cloud folders. §5.4 requires both platforms be run and recorded. Decision 1 (`decision-gates.md:25`) and decision 23 (`:49`) both verified; decision 23's device debt is correctly stated as untouched in either direction. | — |
| **3F. Accessibility trajectory** | 3 | Honest in the direction that costs the spec something. §3.7 quotes the actual manifest line — `eframe = { version = "0.32.3", default-features = false, features = ["default_fonts", "glow"] }`, verified **exact** at `crates/spectre-app/Cargo.toml:19` — to establish that whatever accessibility integration eframe ships is **off**, and therefore makes *no* screen-reader claim. What it does instead is refuse to foreclose decision 17 (`decision-gates.md:41`, verified: keyboard-complete operation and screen-reader labels by beta, scoped audit at R4): every new element is a standard labelled widget, focus order is declared, nothing is icon-only, and no state is conveyed by colour alone (the dirty state is the *word* `unsaved`; `WARM` is redundant emphasis — and `WARM` is a real palette constant, `main.rs:19`). Enabling the feature is routed to Q5 and assigned to the R4 audit. Notably, the dependency is *quoted rather than line-pinned* here with the correct reason given — which makes §7.1's `Cargo.toml:16` pin all the more anomalous. | — |

**Lens average:** 18 / 6 = **3.000**
**Lens pass:** Yes.
**Auto-fail triggered:** No. AF-4 discharged (three bounds, three rationales, three proposed
ledger rows). 3B scores 3, not 0.

---

## Lens 4: Truthfulness & Evidence (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| **4A. Current-state accuracy** | **1** | **This is the failing criterion, and it fails on citation truth rather than on substance.** First, what is right, all of it re-derived rather than trusted. The `ProjectDoc {` census is **exactly** correct: `grep -rn "ProjectDoc {" crates/` returns six hits — three literals (`spectre-project/src/lib.rs:136`, `spectre-offline/src/lib.rs:154`, `spectre-project/tests/command_history.rs:17`), the `struct` at `:30`, the `impl` at `:40`, and the `-> ProjectDoc {` signature at `command_history.rs:13` — and a repo-wide grep finds no fourth literal. The `#[derive(Default)]` claim holds: `ObjectId` derives no `Default` (`crates/spectre-core/src/id.rs:12`), `from_raw` returns `None` for `0` (`:29–35`), and the hand-written `Deserialize` rejects `0` (`:16–25`), so neither `#[derive(Default)]` nor `..Default::default()` is available. All three disputed counts are now right: `project_codec.rs` has **eight** `#[test]`s at exactly `:21, :52, :60, :69, :85, :100, :112, :124`; `command_history.rs` has **five** at `:26, :50, :69, :91, :111`; the crate's own module has **five** more; the contract specifies **eight** steps at `project-persistence.md:120–127` and the spec's ninth row is honestly labelled as its own split of the contract's step 5 (which does read "`sync_all` … **and close or otherwise release handles**"). Every `spectre-project`, `spectre-core`, `spectre-offline`, and `spectre-dsp` pin I checked is exact — roughly forty of them, including `lib.rs:13/:16/:20–26/:30–38/:24/:36/:42–49/:54–58/:80–82/:85–102/:86–91/:92–97/:100/:105–124` and its five rule sub-ranges `/:132–145/:127–221`; `command.rs:3/:4/:11–15/:43/:72–90/:86/:114–180/:122/:176–181`; `id.rs:4/:16–25/:29–35/:45–67/:51–53/:56–58`; `transport.rs:86`; `offline/src/lib.rs:14/:87–108/:110–120/:150–163/:153/:154–160/:306`; `harness.rs:25`; `parameter.rs:66–72/:83/:89–103`; the fixture's 912 bytes, `:2`, `:4`, `:43–46`, `:48–51`. Absence claims verify in the right direction too: `grep -rn "spectre_project" crates/spectre-app/src/ crates/spectre-app/tests/` returns **nothing**; `AppModel` has no `zoom`; no `EditHistory` is constructed outside tests. **Now the defects.** (i) §7.1's central bullet pins `crates/spectre-app/Cargo.toml:16` for `spectre-project = { path = "../spectre-project" }`. At `8b1633d` and at HEAD that line is `live-audio = ["spectre-audio/cpal-backend"]`; the dependency is at **`:24`**. `:16` was correct only at `2e005e5`, iteration 1's baseline. The same paragraph says "**This spec pins no line in either file**", and §3.7 explains why that manifest must not be line-pinned — so the document contradicts itself twice over on this one citation, and the promise "Every row below is checkable in one `Read` at `8b1633d`" fails on the first row. (ii) Six §7.1 pins into `crates/spectre-app/src/lib.rs` are un-rebaselined `2e005e5` values, in a section stating "The pins here are the `8b1633d` values": `TrackView` `:38–45` (actually `41–48`), `ParameterControl` `:48–53` (`51–56`), `DeviceControl` `:56–63` (`59–66`), `prototype()` `:218–262` (`221–265`), `add_track` `:405–422` (`408–425`), `device_parameter_snapshot` `:348–383` (`351–386`). Each is off by exactly the `+3` the same section computes — and which it *correctly applies* at `:208–217`, `:211`, `:222`, so the file is internally inconsistent about its own baseline. (iii) §4.3 repeats it inside one sentence: `tracks()` `:292` is the `8b1633d` value while `lens()` `:281`, `selected_track_id()` `:293`, `devices()` `:313`, `selected_device_id()` `:324` are `2e005e5` values — and at `8b1633d` those four land on `stop()`'s body, `tracks()`'s body, `selected_track_mut`'s body, and `build_presentation`'s brace respectively, i.e. they name a **different symbol** than the one cited. §4.2's `:418` for `level: 0.72` is likewise stale (it is `:421`; `:418` is `muted: false`). (iv) §7.2: "`default_project()`'s only consumer is `default_project_report_is_deterministic`" — false; `crates/spectre-offline/src/main.rs:6` and `:14` are a second consumer. The conclusion drawn survives (the binary asserts nothing), the claim does not. (v) §7.2 instructs STATUS not to weaken "its standing '`./spectre` makes no sound' line" — there is no such line in `docs/status/STATUS.md` at `8b1633d` or HEAD; R4-1 replaced it with the `implemented`-not-`verified` paragraph at `:39` that the spec quotes correctly elsewhere. (vi) §4.1's "`crates/spectre-core/Cargo.toml:12–14` — `serde`, `serde_json`" — that range holds only `serde`; `serde_json` is a **dev**-dependency at `:17`. The load-bearing half ("No path dependencies") is true. (vii) Appendix A's "untracked in git" for `ozone-observations.md` is a `2e005e5` fact. Scored 1 rather than 2 because §7.1 does not merely carry stale pins — it **asserts that it re-baselined them and verified rather than assumed**, and that assertion is false; and because the one Cargo.toml pin it does make is both wrong and self-contradicting. Not scored 0 because no claim describes code that does not exist, no stale range points outside its file, and the substance of every claim is true. | See Priority 1. Mechanical: re-derive every `crates/spectre-app` citation at HEAD, drop the `Cargo.toml` line pin, fix the two false sentences in §7.2, and reconcile the header Notes with §7.1. |
| **4B. Status vocabulary** | 2 | Used correctly and with the distinction respected where it matters most: §4.6 quotes `docs/README.md`'s definitions verbatim — `implemented` = "code exists but its full evidence gate may remain open", `verified` = "stated acceptance evidence passes" (verified exact at `:59–60`) — and requires CORE-004 to move to **`implemented`, not `verified`**, with the reason (R4 cannot prove crash survival). R4-1 is correctly described as `implemented`, not `verified`, matching `STATUS.md:39`. CORE-004's `accepted` status and CORE-001's `implemented`-with-gated-reorder-evidence are quoted correctly from `requirements-ledger.md:49` and `:46` (both exact, including the phrase "gated on the first persisted object collection (R4 intake)"). §7.1's partition into Absent / Implemented / Verified / Accepted-but-not-implemented / Gated is exactly the AF-2 discipline. Docked one for the header Notes' "`./spectre` … still produces no sound", an unevidenced state claim about a baseline where R4-1 has landed — which §7.1 itself explicitly declines to make. | Delete or restate the header Notes' sound claim so it matches §7.1. |
| **4C. Traceability** | 3 | Citation *practice* is exemplary even where individual `spectre-app` pins have rotted. Every normative claim carries a requirement ID, decision row, observation ID, or source path, and every pin into an accepted document verified exact: ledger `:28–30` (RT-001/002/003), `:46`, `:48`, `:49`, `:53` (PROD-003's in-the-ledger rule); gates `:25`, `:27`, `:28`, `:32`, `:37`, `:38`, `:39`, `:40`, `:41`, `:49`; `vision.md:33`, `:37`, `:48`, `:56`; `NEXT.md:29`; `rebuild-roadmap.md:30`; `project-persistence.md:120–127` and `:172–174`; `docs/README.md:59–60`. PROD-003 is deliberately cited **by ID rather than by line**, with the reason stated ("the ledger grows during R4 and its line numbers move") — the correct instinct, and the one the spec failed to extend to `crates/spectre-app`. Every contract phrase the spec quotes is present verbatim in `project-persistence.md`; I checked seventeen of them by string match and all seventeen hit. | — |
| **4D. Honest gaps** | 3 | Twelve open questions, none decorative, none resolved by assertion. **Q12 is real and correctly routed, and I verified it end to end.** `AppModel`'s eight fields (`lib.rs:208–217`) include nothing that could hold the envelope's or the document's `#[serde(flatten)]` unknown map, and §4.4 adds no field, so `project_envelope` necessarily constructs empty unknown maps and open→edit→save through `./spectre` drops any field this build does not know. That is a genuine product-level narrowing of CORE-003 (`requirements-ledger.md:48`, status `verified`, wording "preserved on rewrite **where feasible**"). The spec (a) states it in §4.3 and §4.4 *where the claim is made*, not only in §8; (b) forbids any test, ledger row, or product string from saying otherwise; (c) bounds the R4 exposure correctly — `load_project` refuses anything above `MAX_READABLE_SCHEMA`, so only a same-version writer or a hand edit can plant a droppable unknown field; (d) prices the fix (one opaque field plus a write-back); and (e) hands the actual question — "is unknown-field preservation a crate property or a product property" — to Jeff rather than answering it. That is textbook 4D. Q2 states its full blast radius *before* asking for approval and explicitly says what is being approved is not "retarget one test". Q1 argues against its own choice fairly and corrects the alternative's cost (`skip_serializing_if` only preserves byte-stability if it skips **all four** fields, and the default `ViewDoc` is the awkward one). Q9 and Q11 name things the spec cannot prove from this repository. §5.1 names two tests it refuses to write and why. | — |
| **4E. Evidence commands** | 3 | Exact. `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo test --locked --workspace` — verified verbatim against `docs/status/STATUS.md:48–50`. The three targeted commands are real. `./spectre` is correctly described as `cargo run --locked --quiet -p spectre-app` at `spectre:9` (exact). I personally ran `cargo test --locked -p spectre-project`: it passes, with exactly the eight `project_codec.rs` tests the spec counts. §5.3's honest limit is right — the only process-level assertion available is `smoke_cli.rs` against `smoke_test`'s single println, and the spec extends it rather than inventing a UI harness. | — |
| **4F. No fake surfaces** | 2 | The body is careful and self-auditing. §6.3 audits three user-visible strings line by line: the empty-state copy states a property the code will have **and ships in the same slice**; the `ReplacedDurabilityUncertain` message is forbidden from saying "Saved." and keeps the project dirty; no string says "autosave", "recovered", "backup", or "safe". It also states three negatives the product copy must respect, including that no copy may borrow R4-1's engine as evidence for persistence and that no screen-reader claim is authorised while the manifest reads as it does. §5.4's final row is a standing honesty check. Against that, the spec contradicts itself about the exact surface this criterion governs: the header Notes asserts "`./spectre` … still produces no sound" while §7.1 says "This spec therefore no longer claims `./spectre` makes no sound, and it must not claim the opposite either", and §7.2 tells STATUS to preserve a "makes no sound" line that no longer exists. Two of the three passages cannot be right. | Pick one position — §7.1's is the correct one — and make the header Notes and §7.2's STATUS instruction match it. |

**Lens average:** (1+2+3+3+3+2) / 6 = **2.333**
**Lens pass:** Yes — avg ≥ 2.0, one 1, zero 0s.
**AF-2 triggered:** No. Every source path the spec names *does* contain the thing claimed; the
defect is that six line numbers point to the wrong offsets within the right files, plus two false
sentences about consumers and a STATUS line. Nothing is described as existing that does not.
**AF-6 triggered:** No. The one unevidenced state claim ("still produces no sound") errs
pessimistic, not promotional.

---

## Feasibility Check

Every source file the spec cites was opened and checked against the claim. HEAD `af0e553`.

| Check | Status | Notes |
|---|---|---|
| Types/models exist or are clearly specified | ✓ | `ProjectEnvelope`, `ProjectDoc`, `ProjectError`, `to_bytes`, `from_bytes`, private `validate`, `ProjectCommand`, `Transaction`, `EditHistory`, `CommandError`, `ObjectId`, `IdGen`, `Transport`/`TransportCommand`, `AppModel`, `TrackView`, `DeviceControl`, `ParameterControl`, `DspParameter`, `DeviceParameterSnapshot`, `RenderReport` — all read at their cited paths and all match the spec's description. New types (`TrackDoc`, `DeviceDoc`, `ParameterDoc`, `ViewDoc`, `LensDoc`, `ValidationError`, `SaveStage`, `TargetState`, `SaveReceipt`, `LoadError`, `SaveError`, `AdoptError`) are fully specified with fields and derives. `reorder_tracks`'s inverse is mathematically correct: `remove(from)` + `insert(to, x)` is exactly undone by `remove(to)` + `insert(from, x)`, so `reorder_tracks(to, from)` is the true inverse and U13's `[a,b,c] → [b,c,a]` is right. |
| API/interface changes are feasible with current architecture | ✓ | The schema-2 change is now correctly inventoried. Adding four non-`Option` fields to `ProjectDoc` breaks exactly the three struct literals that exist (two of them out-of-crate consumers of the public API), and §7.2 lists both out-of-crate repairs **with working code**, including the subtle one — `id_gen_state: ids.state()` rather than `0` in `default_project`, because struct-literal fields evaluate in source order and `id: ids.next_id()` has already advanced the generator. The `SCHEMA_VERSION` bump's second-crate breakage (`spectre-offline/tests/harness.rs:25`) is found and repaired. `#[serde(default)]` on all four fields does keep schema-1 files decodable; field-level defaults need only the *field* types to implement `Default`, which `u64`, `Vec`, and the derived `ViewDoc` all do. `validate` going public as `validate_envelope` is a pure widening. |
| Views/screens fit current navigation pattern | ✓ | No new window, modal, sheet, popover, or drawer. Both touched regions exist and are described accurately: `track_list`'s `SidePanel::left("tracks")` with `default_width(220.0)`/`min_width(180.0)` (`main.rs:265–268`), the `.exact_height(62.0)` transport bar, `with_min_inner_size([1060.0, 680.0])`, `request_repaint_after`, the `TRACKS`/`BROWSER` label styling, and the seven `Color32` palette constants (`main.rs:15–21` — exactly seven) all verified. The new block reuses the add-track widget shape already in the same panel. |
| Dependencies are available and version-compatible | ✓ | No new third-party package: `std::fs`/`io`/`path`/`time`/`sync::atomic`/`process` only, and the spec explicitly refuses `tempfile` and `rfd`, stating the cost (no cleanup after a panicking test) rather than discovering it later. `spectre-app → spectre-project` is already declared and this slice is what finally uses it. `spectre-offline` already dev-depends on `spectre-app` (`Cargo.toml:20–21`, exact), so I16 introduces no edge. |
| Platform/renderer requirements are realistic | ✓ | `rename(2)`, `O_CREAT\|O_EXCL`, and directory-descriptor `fsync` are available on both targets; the spec correctly identifies the `F_FULLFSYNC` question as unsettleable from this repository and routes it rather than asserting it, and correctly shows the accepted contract already degrading explicitly where macOS/APFS cannot provide step 8. |
| Test strategy is executable with current infrastructure | ✓ (with one caveat) | The private-seam constraint is correctly derived from the contract, so Group A must be `#[cfg(test)] mod tests` inside `src/fs.rs` — right. Real-filesystem integration tests need only `std::env::temp_dir()`. `cargo test --locked -p spectre-project` runs today (I ran it). **Caveat:** I16 as written is not discriminating, and U12's manifest scan omits the one crate that matters — see 1D and 1G. |
| Performance budget is realistic for target hardware | ✓ | Storage numbers anchor on a real measurement (the R1 fixture is **912 bytes** — verified) rather than a guess; memory is bounded by `MAX_PROJECT_FILE_BYTES` on load; render time is unaffected because the render path does not link this code. The UI-stall figures are explicitly labelled "**estimates from the nature of the calls, not measurements**" and §5.4 requires the real numbers on both platforms before anything is claimed. |
| No undeclared dependency on unbuilt features | ✓ | §7.4 is explicit and correct: nothing blocks implementation; R4-1 has already landed and both coordination points are absorbed rather than left open; R4-4/R4-5/R4-6 **extend** this schema rather than gate it, and the spec persists only the track collection that exists today (`AppModel.tracks`, `lib.rs:211` — verified); R4-8 depends on this rather than the reverse; R5 blocks the durability *claim*, not the code. |

**Feasibility verdict:** **Feasible — but the spec fails criteria.md's feasibility rule.** The
design is implementable as written. The rule that fails it is the §7.1 accuracy clause: the
section states its citations were re-baselined onto `8b1633d` and verified, and then pins
`crates/spectre-app/Cargo.toml:16` (wrong at that commit, right only at `2e005e5`) inside a
paragraph that says it pins no line in that file, alongside six further `2e005e5` line ranges and
two false sentences about `default_project`'s consumers and a STATUS line that no longer exists.

**Caveats:** the working tree moved during this review. The concurrent track-model work added
`pub mod routing;` / `pub mod track;` to `crates/spectre-project/src/lib.rs` after I read it,
shifting that file by +5. I graded the spec's `spectre-project` pins against the state at both
`8b1633d` and the last committed HEAD `af0e553`, where they are exact; they are already rotting
under the uncommitted tree, which is post-baseline drift, not a spec defect. Likewise
`smoke_test`'s `engine=not-started` literal — which §7.2 correctly describes at `8b1633d` — was
changed to a derived `engine={}` by R4-2 (`20f3056`) after the spec's baseline.

---

## Composite Score

| Lens | Average | Weight | Weighted |
|---|---|---|---|
| 1 — Realtime & Correctness | 2.571 | 35% | 0.900 |
| 2 — DAW Workflow Depth | 2.857 | 25% | 0.714 |
| 3 — Product Identity & Scope Discipline | 3.000 | 20% | 0.600 |
| 4 — Truthfulness & Evidence | 2.333 | 20% | 0.467 |
| **Composite** | | | **2.681** |

Arithmetic recomputed independently: lens 1 = 18/7 = 2.5714; lens 2 = 20/7 = 2.8571;
lens 3 = 18/6 = 3.0000; lens 4 = 14/6 = 2.3333.
0.35(2.5714) + 0.25(2.8571) + 0.20(3.0000) + 0.20(2.3333)
= 0.90000 + 0.71429 + 0.60000 + 0.46667 = **2.68095 → 2.681**.

**Pass conditions (criteria.md §Scoring, all four binding):**
- [x] Composite ≥ 2.30 — **2.681**
- [x] Every lens average ≥ 2.00 — 2.571 / 2.857 / 3.000 / 2.333
- [x] No criterion scores 0, and at most two score 1 — **one** 1 (4A), zero 0s
- [x] All auto-fail rules pass — AF-1 no (both touches of accepted material are flagged and
      routed to Q1/Q2); AF-2 no; AF-3 no; AF-4 no; AF-5 no (explicitly complied with); AF-6 no
- [ ] **Feasibility rule — FAILS.** §7.1 misdescribes current state at
      `crates/spectre-app/Cargo.toml:16`, carries six un-rebaselined `2e005e5` line ranges while
      asserting they are `8b1633d` values, and §7.2 states two things about the tree that are not
      true. criteria.md: "A spec whose §7.1 misdescribes current state fails regardless of
      composite."
- [x] Reviewer personally executed the command claimed as passing where it was checkable —
      `cargo test --locked -p spectre-project` run and passing. The full workspace gate was **not**
      run: two other agents are editing `crates/spectre-dsp/`, `crates/spectre-app/`, and
      `crates/spectre-graph/` concurrently, so a workspace result would grade their in-flight work,
      not this spec.

**All conditions met:** **No → FAIL**

---

## Remediation Brief

### Priority 1 — Must fix to pass

Each of these is a false, one-`Read`-checkable statement about the tree. None requires a design
change; all four are edits to §7.1/§7.2 and the header.

1. **§7.1, "Absent" block, first bullet — the `Cargo.toml` pin.** Replace
   "`crates/spectre-app/Cargo.toml:16` reads `spectre-project = { path = "../spectre-project" }`"
   with the quoted literal and **no line number**, matching what §3.7 already does correctly for
   `eframe` and for the stated reason. `:16` is the `2e005e5` value; at `8b1633d` and at HEAD that
   line is `live-audio = ["spectre-audio/cpal-backend"]` and the dependency is at `:24`. As written
   the bullet also contradicts the same section's own sentence, "**This spec pins no line in either
   file**". *Settled by:* `crates/spectre-app/Cargo.toml`, the `[features]` block R4-1 added at
   `:13–16`.

2. **§7.1, "Implemented — what the app has that would need to persist" — six stale line ranges.**
   Apply the `+3` the same section computes: `TrackView` `:38–45` → `:41–48`; `ParameterControl`
   `:48–53` → `:51–56`; `DeviceControl` `:56–63` → `:59–66`; `prototype()` `:218–262` → `:221–265`;
   `add_track` `:405–422` → `:408–425`; `device_parameter_snapshot` `:348–383` → `:351–386`. The
   same section already carries the correct `8b1633d` values at `:208–217`, `:211`, and `:222`, so
   the file currently asserts two different baselines about one file. Fix the same rot in §4.3
   (`lens()` `:281` → `:284`; `selected_track_id()` `:293` → `:296`; `devices()` `:313` → `:316`;
   `selected_device_id()` `:324` → `:327`; `tracks()` `:292` is already correct) and in §4.2
   (`level: 0.72` is `:421`, not `:418`). **Better still — and this would be credited, not
   penalised — cite these by symbol name as the spec already does for `main.rs`,** since R4-2 has
   since shifted `lib.rs` again and R4-4 will shift it further. *Settled by:*
   `crates/spectre-app/src/lib.rs` at `8b1633d`; `git show 8b1633d:crates/spectre-app/src/lib.rs`.

3. **§7.2, `crates/spectre-offline/src/lib.rs` bullet — "only consumer".**
   "`default_project()`'s only consumer is `default_project_report_is_deterministic`" is false.
   `crates/spectre-offline/src/main.rs:6` imports it and `:14` calls it for the `--self-test` /
   no-argument path. The conclusion is unaffected (the binary asserts nothing and byte-compares
   nothing), but say so about both consumers. *Settled by:* `crates/spectre-offline/src/main.rs:6`,
   `:14`.

4. **§7.2's STATUS instruction and the header Notes — the "no sound" contradiction.** §7.2 tells
   STATUS not to weaken "its standing '`./spectre` makes no sound' line"; no such line exists in
   `docs/status/STATUS.md` at `8b1633d` or HEAD — R4-1 replaced it with the paragraph at `:39` the
   spec quotes correctly elsewhere ("Status is `implemented`, not `verified` … no one has confirmed
   by ear that sound leaves the speakers"). Separately, the header Notes still asserts "`./spectre`
   … still produces no sound", which §7.1 explicitly disclaims. Adopt §7.1's position in both
   places. *Settled by:* `docs/status/STATUS.md:39`; the spec's own §7.1 R4-1 bullet.

### Priority 2 — Should fix for quality

5. **I16 cannot fail as written (criterion 1D/1G).** A round trip of an unedited
   `AppModel::prototype()` produces equal hashes even if `adopt` ignores the persisted devices
   entirely — the exact failure mode §5.2 diagnoses for I11 and repairs there. Require I16 to
   (a) `set_device_parameter("saturator", "drive", 6.0)` (or equivalent) before the first render,
   and (b) `adopt` into a **fresh** `AppModel::prototype()`. *Settled by:*
   `crates/spectre-app/src/lib.rs`'s `prototype()` seeding every device value from
   `descriptor.default()` via `DeviceControl::from_descriptors` (`:86`).

6. **§4.1's RT-001 argument overreaches, and U12 guards the wrong crates (criterion 1A/1G).**
   "unnameable in every render-path *library* target, which is the target the callback runs in" is
   false at `8b1633d`: R4-1 builds the render closure at `crates/spectre-app/src/engine.rs:364`
   inside `spectre-app`'s lib target, and `crates/spectre-app/Cargo.toml:24` declares
   `spectre-project`. Restate the compile-time impossibility as covering
   `spectre-audio`/`graph`/`dsp`/`core`, and extend U12 to assert `spectre_project` is absent from
   the app's render-closure module — which is the assertion that could actually fire. *Settled by:*
   `crates/spectre-app/src/engine.rs:364`; `crates/spectre-app/src/lib.rs:7` (`pub mod engine;`).

7. **§4.1's `spectre-core` manifest enumeration.** "`crates/spectre-core/Cargo.toml:12–14` —
   `serde`, `serde_json`" — that range holds only `serde`; `serde_json` is a **dev**-dependency at
   `:17`. The load-bearing half ("No path dependencies") is true; narrow the sentence.

8. **§7.2's blast-radius summary undercounts by one.** "two existing passing tests in two different
   crates" then lists three (`project_codec.rs:25`'s assertion, its byte-stability test, and
   `harness.rs:25`). "Four files, four repairs" is right; the test count is not.

9. **Appendix A: `ozone-observations.md` is tracked in git** at `8b1633d` and HEAD (added in
   `b5af060`); it was untracked only at `2e005e5`. Immaterial to the argument — the spec explicitly
   leans on nothing from it — but it is the same un-rebaselined-fact pattern.

### Priority 3 — Consider for excellence

10. **Criterion 2C.** Add one line saying a persisted `ParameterDoc.value` is the *base* value and
    that automation and modulation contributions get their own document fields under PROD-002, so
    the field is not later overloaded when R4-5/R4-6 extend the schema.

11. **U18's bit-exactness is a claim about the pinned serde stack, not about Spectre.** `-0.0` and
    `f32::from_bits(1)` almost certainly survive `serde_json`'s ryu/parse path, but the spec asserts
    it without evidence and without the hedge it correctly applies to `F_FULLFSYNC` in Q9. Either
    cite the round-trip property or mark it as an assumption the test itself will settle.

12. **§4.3's `command.rs:92–112` range** overshoots `Transaction::execute` (which is `:92–109`;
    `:110` closes the impl). Trivial, but it is the same class as the Priority 1 rot.

---

**Note on what this scorecard does *not* overturn.** Every substantive remediation iteration 2 set
out to make verifies against source, and I checked each independently rather than accepting it:
the `ProjectDoc` blast radius (six grep hits, three literals, two of them out-of-crate) is exactly
right; the `#[derive(Default)]` escape really is closed by `ObjectId`; all three previously wrong
counts (8 / 5 / 8) are now correct; the version stamp has a genuine owner in `project_envelope`
with I9 and I15 pinning opposite halves of the rule, and `save_project_atomic` genuinely cannot
stamp because its parameter is `&ProjectEnvelope` and the contract's eight steps contain no
stamping step; I11's device assertion is now falsifiable because `6.0` lies inside
`SATURATOR_PARAMETERS[0]`'s `1.0..=24.0` and differs from its `1.0` default; and Q12 is a real,
correctly-scoped, correctly-routed question. The failure here is narrower than iteration 1's and
is fully repairable by editing roughly a dozen citations and three sentences.
