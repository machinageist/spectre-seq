<!--
Author: Jeff
Date: 2026-07-12
Description: Current verified state of Spectre
Notes: Claims here link to live evidence; optimistic language is prohibited
-->

# Status

- **Status:** accepted
- **Last verified:** 2026-09-06
- **Scope:** current implementation, documentation, and research state
- **Decision authority:** Jeff
- **Upstream sources:** workspace tests, `../01-requirements/traceability.md`, research ledgers
- **Downstream dependents:** `NEXT.md`, `../06-plans/current-milestone.md`
- **Supersedes:** all removed prototype-era status and handoff material
- **Superseded by:** none
- **Open decisions:** milestone-gated decisions in `../01-requirements/decision-gates.md`; rows 19-22 ratified 2026-08-09
- **Known gaps:** ~~callback-headroom measurements are still taken on a three-node plan~~ — **measured 2026-08-28**: the alpha as R4-9 specifies it (11 nodes, `Filament → Gloam` per track) runs at 0.634 median worst-case headroom against the qualification chain's 0.849 when each is measured alone — about 2.4× the callback cost. **One of six alpha samples was negative**, meaning that block overran on an idle machine. Absolute headroom is dominated by host load and an earlier set of figures was measured under contention; the ratio held, the absolute numbers did not. `worst_headroom` is noisy run to run and every single-value figure in the record is one sample. ~~The alpha contains no effect~~ — **fixed 2026-08-28**: `Track` carries an optional `TrackInsert`, so `Gloam` is in the signal path and on the parameter lane, which R4-9's spec required and no implementation had delivered; `./spectre`'s transport button, engine status cluster, Retry control, and Shape sliders have not been exercised by hand on any platform, so no slice has an operator protocol pass

## Repository state

The prototype implementation and all prototype-only plans, assets, CI, audits, archives, feature lanes, and agent scaffolding were removed on 2026-07-12. The former namespaced workspace was promoted to the repository root. Git history retains committed historical material.

The project was renamed from Geist to Spectre on 2026-08-09, aligning the code with the `spectre-seq` repository. The rename covered all six crate directories and package names (`spectre-core`, `spectre-dsp`, `spectre-graph`, `spectre-project`, `spectre-offline`, `spectre-app`), the `./spectre` launcher and its binary, in-app strings, the research taxonomy tags `SPECTRE-CANDIDATE`/`SPECTRE-REQ`, the local agent skills, and all documentation. It was a pure identifier and prose rename: no behavior, contract, schema, or fixture content changed, and the checked-in project fixtures never carried the old name. The copyright holder `machinageist` is unrelated to the product name and is unchanged.

The active workspace contains:

- `spectre-core`: stable IDs, explicit time types, tempo and meter maps, transport, bounded event ordering, and parameter descriptors;
- `spectre-dsp`: planar-buffer processing contract, a callback-safe runtime parameter seam, bounded note events, deterministic tone source, Pulse instrument, Gain, Saturator, the `SumBus` summing device, and the R4-6 pair — `Filament`, a one-voice phase-warped instrument behind a linear amplitude contour, and `Gloam`, a stereo one-pole damper whose corner opens with the signal's own level;
- `spectre-graph`: app-thread editable graph and immutable compiled plan (GRAPH-001 split) with validated compilation, implicit-cycle diagnostics, and measured allocation-free execution;
- `spectre-project`: versioned JSON envelope at schema 2, one public semantic validator run after every decode **and before every encode**, CORE-004's atomic save and bounded load (`fs.rs`), atomic command transactions, bounded undo/redo, the R4-5 clip model (`MidiClip`, `ClipPlacement`, and a per-track `TrackClips` whose placements are ordered and non-overlapping), and the R4-4 track model — an ordered `TrackList` with stable identity, mute/solo/level, and the graph builder that turns it into an instrument → track gain → sum → master render path;
- `spectre-offline`: deterministic project inspection, a Pulse → Gain → Saturator fixture and a Filament → Gloam voice chain rendered through the compiled plan, including an offline snapshot entrypoint that requires the complete four-parameter fixture and validates exact backend identities and authoritative values before processor construction; and, since R4-8, the blocked offline bounce — a shared FNV-1a fold (`hash.rs`), the single definition of the fixture chain (`fixture.rs`), a 32-bit float WAVE writer (`wav.rs`), the block loop and its cancellation, progress, and divergence reporting (`bounce.rs`), and a `--bounce` CLI mode;
- `spectre-audio`: audio backend trait seam with validated stream configuration, a deterministic pump-driven null backend for CI and offline, a cpal implementation behind a default-on feature, the RT-002 split-lane control transport over a bounded wait-free SPSC ring, a callback bridge that drives the existing compiled plan, timestamped MIDI ingress, and off-thread health telemetry;
- `spectre-app`: native egui shell with backend-derived Build/Shape device surfaces, stable project-instance device focus, an owned offline device-parameter snapshot seam, a selection-aware feedback-report seam, the R4-1 live engine host (`engine.rs`) that compiles a plan from the app snapshot, opens the default device through the `AudioBackend` seam, and owns the stream on the UI thread, and the R4-8 bounce panel (`bounce_panel.rs`) whose pre-render checks are pure functions the shell draws from, and the R4-7 persistence bridge (`project.rs`) that builds the saved snapshot and adopts a loaded one.

`./spectre` launches the graphical interaction prototype. Build shows the native device signal path and offers a visible `Open in Shape` action on every card. The action atomically preserves track selection, focuses that existing device by stable `ObjectId`, and enters Shape; Shape renders only the selected device using backend parameter descriptors and setters. Ordinary lens changes and parameter edits preserve device focus. App parameter edits can be transferred as an owned snapshot to deterministic offline rendering independently of focus, where the complete fixed fixture is validated before values construct processors in the immutable compiled plan. **R4 slice 1 landed 2026-08-24 and this is now a live engine.** On its first frame `./spectre` compiles the track plan, queries the default device's own sample rate through the seam, opens it at 256 frames, and moves the `RenderBridge` into the render closure. Play starts the project's clips when placements exist; a project with no clips uses the held audition voice. Stop sends all-notes-off and Stop. The app sends first and mutates second, so UI and render transport do not diverge. The transport bar reports backend/device/geometry and render-thread counters, and distinguishes an opened stream from one that has rendered.

Status is `implemented`, not `verified`: the manual protocol has not run, and no one has confirmed by ear that signal reaching the driver reaches the speakers. Hardware drills exercise both the legacy fixture and the `open_track_engine` path the binary uses. Runtime parameter routing reaches the selected track's instrument, insert, and gain; `Gain` smoothing was implemented on 2026-08-28, resolving D-R3. R4-4 §8 Q2's visible flat-device seam remains R6 work. No VST3 host, recording path, or project editing canvas exists; Record is disabled and says so. A live CoreAudio driver has been opened and driven by the bridge from the slice 7 drill, and, since R4-1, through the app's own engine path as well.

The accepted JSON project codec and 960-PPQ `BeatTicks` representation have checked-in R1 fixtures. Tempo conversion evidence now covers signed pre-roll, fractional piecewise boundaries, 24-hour positions, unrounded-anchor accumulation, and nearest-tick sample quantization without claiming impossible one-sample arbitrary-sample round trips.

### Arrange, and what it drew before

Until 2026-08-28 the `Arrange` lens drew a hardcoded `Pulse Pattern · 8 bars` rectangle with
sixteen invented step lines, sized from the panel and corresponding to no project data. The clip
model had shipped at R4-5 and nothing read it: `create_clip`, `select_clip`, `selected_clip`,
`set_clip_active`, and `clip_label` had zero callers outside `crates/spectre-app/tests/`.

`Arrange` now draws one lane per track from the project's own placements, positioned by start tick
over a fixed eight-bar span, with inactive placements dimmed rather than hidden, click-to-select,
and an inspector carrying the selected clip's length, note count, active state, and a scrollable
note list. The transport's position reads bars.beats.sixteenths from the render thread's own
published playhead, and reads `—` before any block has rendered rather than a fabricated `1.1.1`.

**Still not drawn:** a piano roll, a session grid, an automation lane, a record-arm surface, and
per-clip time tools — each deferred by name in R4-5 §3.1 rather than overlooked.

### AAA quality pass, 2026-08-29

`gauntlet-output/criteria.md`'s auto-fail rules were run against the active documents rather than
against specs, because they grade claims about state and this repository's claims had drifted.

- **AF-2, paths:** two citations pointed at prototype-era files removed on 2026-07-12 and are now
  marked as history rather than left dangling. Every other cited path under `docs/` resolves.
- **AF-2, symbols:** five backticked identifiers in `docs/` are absent from `crates/`, and all five
  are correct — four are Serum 2's own folder names in an observation file, one is ALSA's
  `snd_pcm_open`. No Spectre symbol is claimed that does not exist.
- **AF-6:** six promotional words appear under `docs/`, all inside quoted external product claims
  in research files. None describes Spectre's own state.
- **Citation drift**, which the gauntlet handoff carried as an undecided rule, is measured: 712
  `path:line` citations across the nine specs, one out of range after a session that rewrote six
  of the cited files. `docs/` carries none at all — it already cites by path and symbol.

**Lens 1 carried the real gap**, and it is the heaviest at 35%. RT-001's allocation evidence
guarded a three-node chain and a device pair; the composed alpha — multi-voice, summed, with a
Gloam insert per track — had none. The structural lock scan reached only `spectre-audio`'s own
modules, so every `AudioProcessor::process` body and `CompiledPlan::process` were unscanned. Both
are closed and both were falsified rather than assumed.

This is the fourth appearance of one pattern: evidence taken on a fixture the product does not
run. The others were the headroom record, R4-1's driver drill, and R4-2's parameter seam.

## Validation

The current gate is:

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

Latest full-gate result (2026-08-31, after the crash-handshake, forward-field, and migration slices):
formatting passed, strict workspace/all-target Clippy passed, and the workspace reported **487
passed / 0 failed / 3 ignored** across 61 test binaries. The ignored rows are hardware/operator
drills and were not reclassified as passing. The earlier 2026-08-28 R4 gate was 443 passed with
two ignored hardware drills; the implementation evidence described below remains the provenance
for those slices rather than being rewritten around the new aggregate.

**R4 slices 4, 5, 6, 7, 8, and 9 landed after that count was first written, and the 443 above includes them.** Slice 4 added the track model, the `SumBus`, and the track → master graph builder; slice 6 added `Filament` and `Gloam`. Slice 6's evidence is 25 counted tests: 15 device-contract tests in `crates/spectre-dsp/tests/devices.rs` (30 total, up from 15) and 3 device-boundary RT tests in its new `device_rt.rs`, 3 app-model tests, 2 offline harness tests, 1 containment test, and 1 allocation-guarded plan test in `crates/spectre-audio/tests/rt_guard.rs` that drives a `Filament → Gloam` plan through the null backend's callback and records zero allocations across seven blocks. That last one was falsified before being trusted: a deliberate `vec!` inserted into `Gloam::process` made it fail, and removing it made it pass again.

Slice 6 arrived on a branch that predated slices 1, 2, and 4, and its integration is recorded here because it changed shipped code. It had defined its own `ParameterError` in a `runtime_parameter` module and implemented `set_parameter` as an inherent method on both devices, because `AudioProcessor::set_parameter` did not exist at its branch point. Both were reconciled onto the seam slice 2 landed: the duplicate module is gone and both devices implement the trait method. `spectre-offline`'s `render_plan` also carried its own copy of the peak/FNV-1a fold; it now returns through `report_from`, so the crate holds one definition of the equivalence walk. Three hand-wired tests that compute their own reference inside the test crate still pass unchanged, which is what proves the extraction changed no bit.

**R4 slice 5 landed the same day and is what turns the signal chain into a sequencer.** A clip is tick-domain material in `spectre-project`; `spectre_audio::clip::ClipSchedule` bakes it to absolute sample positions through `TempoMap::ticks_to_samples` and nowhere else, so no second time conversion exists; and `ClipPlayer` emits `NoteEvent` on the render thread from at most two contiguous transport segments per block. The feature's real content is the merge rule: two producers now feed one note node, and `collect_notes` previously pushed lane events without sorting — correct only because `MidiIngress` sorted before its events entered the lane. Clip and lane events are now collected into one array and sorted once by `(frame_offset, NoteEventKind::rank, sequence)`, the same tuple `ProcessContext::new` validates; `contract_order` is that key's one definition and both the player and the bridge call it. Sequence uniqueness across producers is by band assignment rather than renumbering — clips take `1 << 63` upward, ingress numbers from 0 — so the deliberate tie-break is that a live-performance event resolves before a clip event at an identical offset and rank.

**A bridge with no clip player behaves exactly as it did before slice 5, and that is load-bearing rather than convenient.** R4-1's evidence pins that a Play and a Stop landing in one block are refused into counted silence, and that refusal comes from the lane's own order. Sorting that pair would place the all-notes-off before the note-on by rank and leave the note sounding after Stop — a stuck note where there was a clean refusal. The merge rule exists for a second producer, so it applies only where there is one. **The consequence is recorded rather than resolved:** the app has not yet attached a player to the audition path, and whichever slice does must replace the audition note rather than merge with it.

**Two limits slice 5 states rather than hides.** Its `CLIP_EVENT_RESERVE` of 512 clears the densest *monophonic* block Spectre's own tempo and tick bounds admit, with 2.5× margin, but it does not bound polyphony: notes stacked on a shared start tick are representable, so density is bounded by counted refusal into `AllNotesOff`, never by a partial event set. And `MAX_BLOCK_SEGMENTS` of 2 refuses a loop region shorter than one granted block; the granted block is a driver property, so if a backend ever negotiates above 1 024 frames that bound must be derived from `plan.max_frames()` rather than left at 2. Both are CLIP ledger rows with their own arithmetic.

**Releases follow TIME-002's half-open rule.** A note sounds on `[start, end)`, so its release is emitted at frame `end` — the first silent frame — not at `end - 1`. The R4-5 spec's S-5 expects the earlier placement; adopting it would shorten every note by one sample, so the accepted contract won and the test says why.

**R4 slice 9 landed 2026-08-28 and is the first evidence in this project whose subject is a composition rather than a seam.** One checked-in project — `crates/spectre-offline/tests/fixtures/r4-alpha.json`, three tracks running Filament into Gloam, one clip each, one muted — is decoded from disk, compiled through R4-4's track graph, scheduled through R4-5's clip player, rendered offline, saved through R4-7's atomic writer, reloaded, and rendered again. Fifteen assertions, each naming the sibling whose claim it composes, so a failure says which slice's contract the composition broke rather than only that something broke.

**No checked-in golden hash, and that is the design rather than an omission.** `tanh` and `powf` route through the platform's libm, whose transcendental results are not guaranteed identical across operating systems or libm versions, so a cross-platform constant would encode an open decision as a passing test on one platform and a red CI on the other. What is asserted is equality between two computations on the same host in the same process; the observed hash is printed for the run record and recorded per platform.

**The slice found a real composition gap, and closing it is what R4-9 delivered beyond its own spec.** `RenderBridge` carried exactly one `note_node`, so a project with three instrument tracks played **one** of them live while the offline path played all three: every seam passed its own test and the composition did not work. The bridge now carries additional clip voices — a fixed `Box<[ClipVoice]>` built on the app thread, each with its own preallocated scratch — and `the_live_bridge_plays_every_instrument_track` asserts the whole project live equals the whole project offline. `one_voice_does_not_sound_like_three` is the control that stops that from being two identically-truncated renders agreeing.

**The note-input array is sized from the graph's own already-accepted bound, not a new number.** `MAX_FLAT_INPUTS` (32) became public for this; the bridge builds one block's inputs in a stack array of that size and `with_clip_voices` refuses anything past it, so a caller cannot overrun the array and the callback allocates nothing. `multi_voice_bridge_render_is_rt_clean` drives three instruments into a sum through the guarding allocator across seven blocks at zero allocations, and `a_duplicate_voice_node_is_refused_without_allocating` proves the fail-closed path is RT-clean too: a caller that gives two voices the same node gets counted silence, never stale audio.

**`./spectre` now plays the project's clips rather than an audition note.** `build_track_engine_parts` bakes each track's active placements on the app thread before the stream opens — through `TempoMap::ticks_to_samples` and nowhere else — attaches the selected track's schedule to the primary note node and every other track's to its own clip voice, and reports `has_clips`. Play then sends the transport command **alone**: the project's material *replaces* the audition note rather than merging with it, which is the rule R4-5 recorded and did not act on. A project with no clip placements attaches no player at all and auditions exactly as it did at R4-1, so that slice's evidence stays valid unchanged.

**The rule is pinned by the case it exists for, and the first version of the test could not fire.** Separating Play and Stop across blocks passes whether or not the audition note is sent, because the merge never conflicts. `play_then_stop_in_one_block_stays_silent_with_clips_attached` puts both in one block, where an audition note-on merged into the sorted block would land *after* the all-notes-off by rank and leave a note sounding after Stop. Writing it immediately caught a real defect — in the test harness rather than the product: `engine_over_null` bypassed `open_with_parts` and dropped the flag, so the engine auditioned over its own clips. The helper now carries it and the test asserts `engine.has_clips()` rather than `parts.has_clips`, so a future helper that forgets fails loudly.

**One piece of speculative code was written and removed rather than kept.** The voice emitter first sorted each voice's events; deliberately breaking that sort failed nothing, because `ClipPlayer::emit_block` already leaves its own output in contract order and a voice is one producer feeding one destination. The merge rule exists where two producers share a node — the primary node's case, not a voice's — so the sort came out.

**Two smaller findings, both recorded rather than absorbed.** `TrackInstrument` had one variant, so R4-6's synth was in no plan `./spectre` ran; slice 9 added `TrackInstrument::Filament`, which is what makes "one original synth ships as the alpha's voice" a statement about the product rather than about a crate. And an exact note-off on a block boundary turned out not to be expressible at 120 BPM, 48 kHz, and 256 frames: one tick is 25 samples, so a boundary falls on a whole tick only at 6,400 samples — 25 blocks, past the fixture's 16-block note span. The fixture exercises the other fragile case, a release and an attack sharing one tick, and the arithmetic is written into the builder so the constraint is not rediscovered.

**The manual half exists and has not been run.** `docs/05-quality/r4-qa-protocol.md` carries fifteen manual rows, the environment schema, the per-platform outcome vocabulary, and an explicit list of what a PASS does and does not authorize. `docs/05-quality/r4-qa-records.md` carries block Q-1 — the automated half on Arch Linux, Flow A passing 16/16, a clean workspace gate, thirteen manual rows `NOT RUN`, outcome `INCONCLUSIVE`. **R4's end-to-end exit row is half closed and is now the only open one:** the fixture passes, the protocol is written and corrected, no operator has worked it.

**R4 slice 7 landed 2026-08-28 and is the first time work survives quitting the app.** Schema 2 adds the first persisted object collections — the track list, device instances and their parameter values, the working view context, and the ID generator's position — and `save_project_atomic` writes them the way the accepted persistence contract requires: validate, encode, create a hidden sibling with `O_CREAT|O_EXCL`, write, `fsync` the file, release the handle, `rename(2)`, `fsync` the parent directory. **The destination is never deleted, truncated, or moved aside first**, and the only failure that is not `Unchanged` is the parent-directory sync after a successful replacement — which reports `ReplacedDurabilityUncertain`, keeps the project dirty, and attempts no second replacement.

**Six of the seven save stages are covered by fault injection through a private seam, and the seventh is uncovered on purpose.** `FaultFs` implements the private `FsOps` trait inside `fs.rs`, records an ordered call log, and fails at a chosen stage; the tests assert the exact `target_state` and the exact log for each. `EncodeSnapshot` has no test because a validated envelope cannot fail to encode with the current encoder, so any such test could not fail — the variant exists because the contract requires the distinction, and that it is currently unreachable is stated rather than papered over with a green check.

**CORE-001's persistence half is closed.** R4-4 proved identity survives `TrackList::reorder` in memory; `reorder_preserves_identity_across_save_and_reload` proves it survives the reorder *and the file*, and `undone_reorder_reloads_in_the_original_order` proves the undo does too. The new `ProjectCommand::reorder_tracks` addresses a track by `ObjectId` rather than by a from-index, and builds its inverse from the index `TrackList::reorder` returns, so the inverse is exact by construction.

**The validator became public and gained four rules, because deserialization bypasses every constructor.** `Track`'s mutators clamp, `Track::new` refuses a blank name, and `TrackList::insert` refuses a duplicate `ObjectId` — but `#[derive(Deserialize)]` calls none of them, so a hand-edited file could carry a blank name, a level outside the descriptor's range, duplicate IDs, or more than `MAX_TRACKS` tracks. Schema 2 adds project-wide ID uniqueness, referential integrity of the restored selection, the `Track` invariants the derive skips, and finite parameter values with non-empty unique keys. **No new number is introduced by any of it** — every bound named is an already-accepted one being re-checked on a path that skipped it. Rule 4 is correctness, not hygiene: JSON has no literal for NaN or infinity, so validating before encoding turns a non-finite value into a refusal with the destination never opened rather than a file that no longer means what the project meant.

**Two deviations from the spec, both stated rather than absorbed.** First, `AppModel` gained two fields — `project_id` and `tempo_map` — where §4.4 claimed persistence would add none. Without them an app-built envelope has nothing to put in `ProjectDoc.id` or `ProjectDoc.tempo_map`, so every save would mint a new project identity and reset the tempo; a project ID that changes on every save makes CORE-001's project-level identity meaningless, which is worse than one field. Second, the four schema-2 document fields are `skip_serializing_if` when they carry nothing, which the spec did not call for. Without that, reading the R1 fixture and rewriting it injects four empty fields into a schema-1 document and breaks `canonical_fixture_rewrite_is_byte_stable` — accepted CORE-003 evidence. Skipping them is what keeps a schema-1 file byte-identical through a load and a save.

**The dirty marker is derived, not maintained.** It compares the bytes a save would write right now against the bytes last written, rather than being set by hand at each mutation site — a flag maintained at call sites is a flag someone forgets at the next one, and a marker reading "Saved" over unsaved work is exactly the defect the project-safety pillar names. The two shell decisions that are not drawing — that comparison and the one-press-never-discards rule — live in `spectre_app::project` rather than in `main.rs`, because `main.rs` is a binary target no test can reach; that is the lesson R4-1's review already paid for.

**R5's engineering queue closed 2026-09-06 with eight of nine slices done; the milestone has not
exited.** Slice 8, missing-media diagnostics, is blocked on a persisted media reference the
workspace does not contain — there is no audio clip, sample, or media type in the project model at
all — so it is carried to the instrument milestone under the concurrent-milestone rule rather than
faked against an invented fixture. Decision 13 (command-pattern undo) was gated at R5 exit and is
ratified on this milestone's own evidence. The product-path drill
(`crates/spectre-project/tests/recovery_qualification.rs`) kills a process writing **the sidecar**
rather than the project, which tests a claim the older crash drill does not make: `journal.rs`
never opens the project file. Three drills, 24 randomized-phase kills each; a mutation that tears
the project file inside the autosave fails two of them.

**Autosave became product-reachable 2026-09-06.** `./spectre` journals unsaved work to the
sidecar on a content trigger and retires it on a durable save. The trigger is
`spectre_app::project::autosave_action`, a pure function of four inputs, deliberately not a timer
— an interval would be a number with no rationale row, which PROD-003 forbids. `commit_save` owns
the save-then-retire order and **keeps** the sidecar when the parent-directory sync fails, since
that save left the project in place but its directory entry may not survive a power loss. The recovery half landed the same
day: opening a project inspects for a sidecar and offers it, `adopt_recovered` loads it as
**unsaved** by returning the project file's own bytes for the dirty marker, and neither disk
version is written until a manual Save retires the sidecar. Autosave is suppressed while an offer
is pending, because the model still holds the saved project then and journaling it would
overwrite the sidecar with the work the offer protects.

**R5 persistence seams now exist but are not product-reachable.** Process-death crash qualification,
sidecar autosave, and explicit recovery inspection/accept/decline landed on 2026-08-29. No shell
surface calls the autosave or recovery APIs.

**Recovery now commits on manual Save rather than on accept, per decision 14 as Jeff ratified it
on 2026-08-31.** `recovery::accept` previously wrote the autosaved envelope over the project
through the atomic save and discarded the sidecar; it now returns the envelope and touches
neither disk version, so both survive until a later successful manual Save. **The change also
closes a real defect rather than only re-shaping an API:** `inspect` decided redundancy from the
shallow `compare` summary, so two envelopes that differed only outside the five compared fields —
parameter values, note content, view state, or forward fields — were reported `Redundant` and the
unsaved work was silently dropped. Redundancy is now exact envelope equality, and an offer whose
summary names no row says so in `describe` rather than presenting an empty difference list as
though nothing differed. As of 2026-08-31, the shell carries every unknown map
the codec currently represents — envelope, project, device, and parameter — through open → edit →
save by stable identity. Track, clip, placement, note, and view types expose no flatten map, so no
deeper preservation claim is made. Nothing in the shell's own `main.rs` — the buttons, the
two-press discard, the status text — has been exercised by hand.

**The first explicit migration landed 2026-08-31.** `spectre_project::migrate_to_current` upgrades
the checked-in schema-1 document to schema 2 without changing project identity or represented
unknown fields, initializes the generator position for the collection-bearing schema, and refuses
a schema-1 label that already carries schema-2 state. `spectre_app::project::adopt` invokes that
boundary before mutating the model and restores the schema-1 meter map rather than merely carrying
its bytes. Already-current documents are exact no-ops.

**R4 slice 8 landed 2026-08-28 and is the first evidence that live and offline are one computation over a whole render rather than over one quantum.** A bounce compiles the fixture plan once, renders `frames` in `block_frames` quanta, folds every block into one frame-major FNV-1a hash, and writes 32-bit float WAVE bytes that are a copy of the engine's own `f32` buffers rather than a conversion. The central gate is `a_multi_block_bounce_matches_the_live_path_block_for_block`: sixteen 256-frame `RenderBlock`s through `RenderBridge` and a sixteen-block `bounce_report` produce the same hash and the same per-block hash vector. The `--bounce` CLI prints that same number from a separate process, asserted against a literal the live path produces.

**The refactor under it is a de-duplication of the specimen and deliberately not of the instrument.** `fixture.rs` is now the single definition of the Pulse → Gain → Saturator chain, its ID seed, and its four device values, replacing three hand-maintained copies; `hash.rs` is the single *shared* FNV-1a fold. Three hand-written folds remain by design — two in `crates/spectre-offline/tests/harness.rs` and the interleaved walk in `crates/spectre-audio/tests/bridge_plan.rs` — because two independently written instruments agreeing over one specimen is verification, while one instrument compared against itself is not. No assertion in either file changed, and both suites pass unchanged, which is what makes "bit-exact extraction" a claim with evidence rather than an intention.

**Block size is part of a render, not a preference, and slice 8 proves it rather than asserting it.** RT-003 containment silences a whole quantum, so a contaminated render is *not* identical across block sizes: `a_contaminated_render_is_not_identical_across_block_sizes` renders 512 frames with a poison at absolute frame 300 once as one quantum and once as two, and the frame-0 sample is silence in the first and intact in the second. This narrows the accepted claim at `docs/03-architecture/dsp-device-io.md` that "identical initial state and input produce bit-identical offline output" — that statement holds unconditionally only while containment does not fire. **Closed 2026-08-28, and not by narrowing anything.** The clause says "identical initial state and input"; block size is part of that state, which is what this section's own first sentence says. Two renders at different quanta are not the same render, so the clause never covered them and nothing needed weakening. `a_contaminated_render_is_bit_identical_to_itself_at_the_same_block_size` supplies the half that was missing — determinism survives contamination within a configuration — and the contract now states the scope explicitly.

**Cross-machine bit-reproducibility is not claimed.** `Saturator` calls `f32::tanh` and `PulseInstrument` calls `f64::powf`, both of which dispatch to the platform's libm, whose transcendental results are not guaranteed identical across operating systems, architectures, or libm versions. What slice 8 proves is precisely: within one process and one build, the bounce and the live path produce bit-identical output.

**Two defects the slice found in its own work, both fixed and both now pinned.** The block loop first selected each block's events by comparing an absolute offset against the block *length* rather than the block's end, so a note-off past the first block never fired at all; and the first equivalence test could not catch it, because `fixture_events` places its note-on at frame 0 and its note-off on the final frame, where a rebase is either the identity or one sample from the end. `a_mid_timeline_note_lands_on_the_same_frame_in_both_paths` places events at frames 700 and 2 500 and fails under both mutations. Every central mechanism in the slice was deliberately broken before being trusted.

**Historical R4-6 limit, closed 2026-08-28.** Filament and Gloam initially existed only in the
device crate and offline harness. `TrackInstrument::Filament` plus the track insert slot now make
every alpha track `Filament → Gloam`, and the product's parameter route reaches both roles.

**Corrected 2026-08-24 after R4-1's implementation review.** This paragraph previously also listed "a refused transport send leaves the UI unchanged" among what the set proved. It did not: the test asserting it built an `AppModel`, never passed it to the rule under test, and asserted that a fresh prototype was not playing — an assertion that cannot fail. The rule was correct in `main.rs`, but `main.rs` is a binary target unreachable from `crates/spectre-app/tests/`, so it had no coverage at all. The rule now lives in `spectre_app::engine::toggle_transport` and carries three falsifiable tests, one per branch. Two further review findings were fixed in the same pass: the smoke line's `engine=` field was a hard-coded literal, so the assertion guarding the headless path could not fire either, and it is now derived from the shell's own engine.

The previous full-gate result (2026-08-09, after the Geist → Spectre rename and R3 slices 2-9): formatting, strict Clippy, and all 230 tests passed with one ignored hardware drill; the selected-device headless launch check and offline self-test pass. The app evidence includes 27/27 model tests and 1/1 process smoke test; the offline harness retains 21/21 tests. Selection tests cover deterministic Pulse focus, atomic valid drill-in, invalid-ID rollback of lens/selection/device state, all-lens and parameter-edit continuity, selected descriptor and parameter identity, focus-independent complete snapshots with correct edited-value attribution, renderer-neutral Build/Shape presentation, recoverable UI-thread error reporting, and selected-device feedback/smoke reporting. Existing snapshot tests continue to cover validated nonzero `ObjectId` decode at the public snapshot/render boundary, stable project-instance device/parameter identity, private-field DTO access through getters, canonical constructor containment of NaN/infinities/out-of-range input, exact signed-zero/subnormal publication, read-only app schema with identity-based value attribution and explicit invariant errors, exact complete fixture membership, duplicate/alias/partial/unknown/mismatched snapshot rejection, exact hand-wired render equivalence for all four mappings, backend-default equivalence, and deterministic repeated rendering. Native descriptor tests also pin the documented `f32` normalization and boundary policy. The existing R2 silence, impulse, allocation, and deterministic-hash evidence remains on the unchanged compiled-plan process path.

The rename changed no test count and no assertion: the same 155 tests pass before and after, and the pre-rename run on 2026-08-09 reproduced the 2026-08-06 result exactly. The only source churn beyond identifiers was rustfmt reordering `use` blocks, because `spectre_*` sorts differently than `geist_*`.

R0/R1 exited 2026-07-17 and R2 (offline graph) exited 2026-08-09 on its four render gates. R3
(live shell) exited 2026-08-09 with all ten rows closed. R5 is the active engineering milestone
under the concurrent-milestone rule accepted 2026-08-31; R4 remains exit-pending on its operator
protocol and has not passed. Two of R3's rows closed on macOS hardware alone under decision 23: cpal opened an M-Audio AIR 192|6 through CoreAudio for 173 driver callbacks with 0 xruns and 0.990 worst-case headroom. **Decision 23's Linux debt was discharged 2026-08-28** on Arch Linux (kernel 7.1.9-arch1-2, PipeWire 1.6.8): three qualification rows, two through pipewire-alsa on a C-Media USB interface (363 and 369 blocks) and one on the raw ALSA path to the onboard ALC285 with no sound server (198 blocks), all with 0 xruns, 0 plan errors, 0 contaminated nodes, and 0 frame-capacity rejections. The drill did **not** run unchanged as decision 23 predicted: it first exposed a real defect in `CpalBackend::find_device`, which matched device ids only against `host.output_devices()` while ALSA's default PCM is named `default` and absent from that enumeration, so every open failed `UnknownDevice`. Separately, the workspace had never compiled on Linux at all — `spectre-app` named neither `x11` nor `wayland` with `eframe`'s default features off — and CI had been failing on exactly that since 2026-08-06 unread, because `ci.yml` triggers only on `main` while every R4 slice landed on `gauntlet/r4-spec-phase`. Both are fixed. Decision 1's co-first-class commitment is now discharged for the device seam and still open for the shell, which has no operator pass on any platform. R2's exit does not claim full GRAPH-002 satisfaction: only implicit-cycle rejection exists, and explicit priced feedback edges stay gated at decision row 7 before R11.

The R1 exit disposition is complete: CORE-004's design, implementation, and process-death crash
qualification are present. CORE-001's persisted reorder evidence landed at R4 and its first schema
migration evidence landed at R5 on 2026-08-31.

## Product and requirements

The product vision, requirement seed, decision defaults, roadmap, and current milestone are accepted. Decisions explicitly assigned to later intakes remain gated there rather than blocking R1.

## Research

- 29 unique ledger sources across 10 products.
- Four timestamped FL Studio action-sequence observations.
- Two Ableton thematic self-reports, not action-sequence evidence.
- No frequency, convergence, or priority claim is authorized by the current corpus.
- Visible-session Bitwig and Ableton evidence remains the highest-value workflow gap.