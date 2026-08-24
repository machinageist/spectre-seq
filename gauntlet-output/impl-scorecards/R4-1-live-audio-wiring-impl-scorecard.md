<!--
Author: Jeff
Date: 2026-08-24
Description: Blind implementation verification of R4-1 (commit 8b1633d) against its accepted spec
Notes: Verifier did not write the code. Every cited file was opened at commit 8b1633d in a clean
  clone, because the live working tree is dirty with another agent's in-progress R4-2 work
-->

# Scorecard: Live Audio Wiring — IMPLEMENTATION

**Feature ID:** `R4-1` (`live-audio-wiring`)
**Spec file:** `gauntlet-output/specs/R4-1-live-audio-wiring.md` (iteration 2, accepted at 2.950)
**Artifact reviewed:** commit `8b1633d` — "R4-1: wire ./spectre to the qualified backend"
**Reviewer agent:** blind implementation verifier
**Date:** 2026-08-24
**Spec iteration reviewed:** 2

---

## Verdict: FAIL

**Composite: 2.581 / 3.00** — above the 2.30 threshold, and every lens clears 2.00.
It fails on **pass condition 3**: three criteria score 1 (`1G`, `4A`, `4F`) and the
limit is two.

**Summary.** The engineering is strong and the core of the slice is real: the app
compiles the fixture plan from its own validated snapshot, opens the default device
through the seam at the device's own rate, moves the existing `RenderBridge` into the
render closure, and its live output hashes bit-identically to
`render_app_snapshot` — a falsifiable assertion I ran and watched pass. No second
render path was created, no file on the "deliberately not modified" list was touched,
and the RT-001 re-proof is genuine (the re-declared allocator's positive control does
fire; `rt_guard` passes post-edit). The failure is not in the audio path. It is that
**two assertions cannot fail on the property their own names and comments claim they
prove** — one of them §5.1 test 7, the only test of the "one genuinely new correctness
hazard" the commit message identifies, the other the whole of §5.3's smoke-line
extension — and that the status documents then cite one of those non-results as
established evidence. A 1,157-line undeclared `site/index.html`
rides along in the same commit asserting that the app has no audio thread and that
R4-1 is unimplemented.

**Single most important fix:** make the two-transport binding rule testable and test
it, and derive the smoke line's `engine=` field from actual engine state instead of
printing a literal.

---

## Method and coverage disclosure

- I read **every file the commit touches**, in full or at every hunk, at commit
  `8b1633d`. Nothing below is `[sampled]`. Where I read only part of a large file
  (`site/index.html`, 1,157 lines), I say so at that finding.
- **The live working tree is dirty.** Another agent is implementing R4-2 in
  `/Users/machinageist/spectre-seq` concurrently; during this review
  `crates/spectre-app/src/engine.rs`, `crates/spectre-app/tests/live_engine.rs`,
  `crates/spectre-audio/tests/rt_guard.rs`, `crates/spectre-audio/src/control.rs`,
  `crates/spectre-dsp/src/*` and `crates/spectre-graph/*` all changed under me, and the
  working tree does not compile (`AudioProcessor::set_parameter` is unimplemented in
  `crates/spectre-graph/tests/containment.rs` and `graph_plan.rs`). **That breakage is
  not R4-1's** — none of those files is in commit `8b1633d`. I therefore cloned the
  repository to a scratch path, checked out `8b1633d`, and ran every command there. All
  results below are from that clean clone.
- **Commands I personally executed at `8b1633d`:**

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | **pass**, exit 0, zero diffs |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | **pass**, exit 0, no warnings |
| `cargo test --locked --workspace` | **pass**, exit 0 — **244 passed, 0 failed, 2 ignored** |
| `cargo run --locked -p spectre-app -- --smoke-test` | **pass**, prints `… selected_device=Pulse(pulse) engine=not-started` |
| `cargo test -p spectre-audio --test rt_guard` (in the workspace run) | **pass**, 7/7 |
| `cargo test -p spectre-app --test live_engine` (in the workspace run) | **pass**, 12 passed, 1 ignored |

- I did **not** run the two `#[ignore]`d hardware drills. Their reported numbers
  (175 blocks / 0.974 headroom; 88 blocks / 88 200 Hz / 0.970) are therefore
  **unverified by me**; I take no position on them beyond noting that the drill code
  that would produce them is present and its assertions are real.

---

## Contract conformance — §4.2, §4.3, §7.2 walked against what landed

### §4.2 declared types

| Declared | Shipped | Finding |
|---|---|---|
| `ENGINE_BUFFER_FRAMES = 256` | `engine.rs:31`, identical, rationale comment intact | ✓ |
| `ENGINE_PLAN_FRAME_MARGIN = 2` | `engine.rs:39`, identical | ✓ |
| `EngineUnavailable`, 6 variants, `derive(Debug, Clone, PartialEq, Eq)` | `engine.rs:66–78` — all 6 variants present; derive is **`PartialEq` only, no `Eq`** | Deviation. `BackendError` (`spectre-audio/src/lib.rs:96`), `ControlError` (`control.rs:29`) and `DeviceParameterSnapshotError` (`spectre-app/src/lib.rs:177`) all derive `Eq`, so `Eq` was achievable and was dropped without cause |
| `EngineState`, **3** variants: `Unavailable(EngineUnavailable)`, `Opened{..}`, `Running{..}` | `engine.rs:96–112` — **only 2 variants**; `Unavailable` is absent | **Deviation.** The app substitutes `Option<LiveEngine>` plus a separate `engine_unavailable: EngineUnavailable` field (`main.rs:29–30`). Defensible (a nonexistent engine has no state) but it silently retires §3.3 item 6's "single text label … Data source: `LiveEngine::state()`" for the unavailable case |
| `EngineHealth`, **9** fields | `engine.rs:116–126` — `blocks_rendered`, `xruns`, `worst_headroom`, `last_headroom`, `plan_errors`, `frame_capacity_rejections`, `notes_deferred`, `contaminated_nodes`, `stream_errors` | ✓ **exactly nine, exactly as declared** |
| `EngineParts`, 5 fields | `engine.rs:129–138`, identical | ✓ |
| `AuditionError::{Transport, Note}` | `engine.rs:143–149`, identical | ✓ |
| `LiveEngine<S: ?Sized = dyn AudioStream>` with fields `sender, telemetry, backend_name, device_name, config, next_sequence, voice_id, stream` | `engine.rs:304–314` — every field present **except `voice_id`**, which is replaced by the module constant `AUDITION_VOICE_ID` (`engine.rs:50`) | Minor deviation, arguably an improvement (the audition identity is fixed, not per-engine) |

**Undeclared additions in `engine.rs`:** `ENGINE_NODE_SEED` (`:44`), `AUDITION_VOICE_ID`
/ `AUDITION_CHANNEL` / `AUDITION_NOTE` / `AUDITION_VELOCITY` (`:50–53`),
`canonical_fixture()` (`:56`), `fixture_values()` (`:157`), `plan_error()` (`:152`),
and a `Display` impl for `EngineUnavailable` (`:80`). All are private helpers or
implementation detail the spec did not forbid; the `Display` impl is required by §3.6
and its absence from §4.2 is a spec gap, not an implementation one. See AF-4 below for
the ledger consequence of the constants.

**Undeclared addition in `null.rs`:** `pub const NULL_SAMPLE_RATE: u32 = 48_000`
(`null.rs:20`). §4.3 specified the value but not a named constant. It carries an inline
rationale and a ledger row (ENGINE-003), so this is an improvement over the spec.

### §4.3 declared function signatures — all 13 present

`build_engine_parts` (`engine.rs:177`), `open_default` (`:256`), `from_open_stream`
(`:320`), `stream_mut` (`:341`), `state` (`:346`), `health` (`:369`), `config` (`:384`),
`send_transport` (`:389`), `send_note` (`:396`), `start_audition` (`:408`),
`stop_audition` (`:423`), `reclaim` (`:433`), `close` (`:438`). Signatures match the
declarations. `AudioBackend::default_sample_rate` and `AudioStream::stream_errors` are
both required (not defaulted) trait methods as declared
(`spectre-audio/src/lib.rs:199`, `:215`).

### §7.2 file list vs. `git show 8b1633d --stat`

Every declared new file landed; every declared modified file was touched (13 of 13);
**every file on the "Deliberately not modified" list is absent from the commit** —
`bridge.rs`, `control.rs`, `spsc.rs`, `midi.rs`, and all of `spectre-graph`,
`spectre-dsp`, `spectre-core`, `spectre-project`, `spectre-offline`. That is a real and
creditable result: the "no second render path" claim is structurally true, not just
asserted.

**Two files were modified that §7.2 does not list:**

1. `Cargo.lock` (+3) — unavoidable and trivial.
2. **`site/index.html`, +1,157 lines, an entirely new file** — a "Spectre project
   atlas … live progress board, feature manual, and evidence record". It is unrelated
   to live audio wiring, is in no slice's scope, is not mentioned anywhere in the
   commit message, and accounts for **more than half of the commit's insertions**. See
   findings P1-3 and 3A/4A/4F.

---

## Lens 1: Realtime & Correctness (weight: 35%)

| Criterion | Score | Evidence | Remediation |
|---|---|---|---|
| **1A. Callback-path discipline** | 3 | The installed closure is `Box::new(move \|mut block: RenderBlock\| bridge.render(&mut block))` (`engine.rs:284`) — the R3 closure verbatim. Nothing in `engine.rs` is callback-reachable; `reclaim()` is drained on the app thread each frame (`main.rs:607–609`). I read `RT_MODULES` and `FORBIDDEN` in `rt_guard.rs` **at the commit**: four entries (`src/bridge.rs`, `src/control.rs`, `src/spsc.rs`, `src/null.rs`) and seven needles, exactly as §4.1 states. I ran `--test rt_guard` post-edit: 7/7 pass. `live_engine.rs`'s re-declared `GuardingAllocator` (`:39–56`) has a real positive control (`:347–354`, `Vec::with_capacity(64)` + `black_box`, asserting `> 0`) and the measured section is `engine.stream_mut().pump()` (`:363`) — and `NullStream::pump` (`null.rs`) allocates nothing of its own, so the section really is the render closure. | — |
| **1B. Control↔render communication** | 3 | RT-002 lanes only: `send_transport` → `ControlSender::send_transport`, `send_note` stamps `frame_offset: 0` + monotonic `sequence` (`engine.rs:396–404`). Overflow returns `ControlError::{TransportLaneFull, NoteLaneFull}` and is surfaced on the app thread (`main.rs:100–111`). No parameter targets registered — `control_channel(&[], …)` (`engine.rs:234`) — matching decision 22's deferral. Retired state reclaimed per frame. | — |
| **1C. Numerical containment** | 3 | No new DSP node; RT-003 containment inherited unchanged. `contaminated_nodes` is read into `EngineHealth` and asserted `== 0` by test 10 (`live_engine.rs:337`) and by the app drill (`:444`). | — |
| **1D. Determinism** | 3 | Test 5 (`live_engine.rs:196–218`) hashes the live block with the same FNV-1a walk (basis `0xcbf2_9ce4_8422_2325`, prime `0x0000_0100_0000_01b3`, `:119–131`) and asserts equality with `render_app_snapshot(…).hash`, plus `offline.peak > 0.0` so a match cannot be two silent buffers. Test 6 builds three times and compares hashes. **Both pass in my run.** No new comparison method introduced. | — |
| **1E. Graph and plan contract** | 3 | `build_engine_parts` builds an `EditableGraph`, compiles once on the app thread, and hands an immutable plan to the bridge (`engine.rs:190–243`). Recomputed: `plan_max_frames = 256 × 2 = 512`; the §4.7 arithmetic checks out — 6 × 512 × 4 B = 12,288 B pool, of which the margin is 6 × 256 × 4 B = 6,144 B. No per-change recompilation anywhere. | — |
| **1F. Failure behavior** | 2 | Behaviour is correct and fail-closed: the unsorted same-block pair yields exact silence with `plan_errors += 1` and **no** `blocks_rendered` increment — I confirmed the ordering myself in `RenderBridge::render` (silence + `plan_errors.fetch_add` + `return`, before the `blocks_rendered.fetch_add`), and test 14 asserts all of it. **But the surfacing is not what §3.3 item 8 and §3.6 E8/E9/E10 require.** Those say `contained`/`refused`/`plan errors`/`stream errors` "become visible" when nonzero and "must never be hidden". In `main.rs:194–202` all seven counters live in a single `.on_hover_text(...)` tooltip that is only reachable by hovering a pointer over the headroom label. A nonzero `plan errors` is invisible until someone hovers. | Promote any nonzero counter to a visible label in the transport bar, as §3.3 item 8 specifies; keep the tooltip as detail |
| **1G. Test specification** | **1** | 12 of 14 tests carry real, falsifiable assertions that I watched pass. **Two do not.** (a) Test 7, `a_refused_transport_send_leaves_the_ui_transport_unchanged` (`live_engine.rs:240–257`): it constructs `let mut model = AppModel::prototype();` at `:243`, never passes it to anything, and then asserts `!model.is_playing()` at `:254` under the comment "The binding rule: the app sends first and mutates second, so a refusal leaves the UI alone". A freshly constructed prototype is never playing, so **the assertion cannot fail**, and the binding rule it claims to prove lives in `SpectrePrototype::toggle_transport` (`main.rs:85–114`) — a binary target, unreachable from `tests/`. (b) `smoke_cli.rs:21`'s new `assert!(stdout.contains("engine=not-started"))` is documented as "what fails if engine startup is ever wired into the headless path", but `smoke_test()` prints `engine=not-started` as a **hard-coded literal inside the format string** (`main.rs:675`), so it would still pass if startup were wired in. criteria.md 1G: "A test that cannot fail scores 0." I score 1 rather than 0 only because the other twelve are strong. | See Priority 1 findings 1 and 2 |

**Lens average:** 18 / 7 = **2.571**
**Lens pass:** Yes (≥ 2.0, no 0s, one 1)

---

## Lens 2: DAW Workflow Depth (weight: 25%)

| Criterion | Score | Evidence | Remediation |
|---|---|---|---|
| **2A. Loop-first core loop** | 3 | An engine failure never disturbs workspace context: `open_default` receives only a snapshot, `start_engine` writes only `engine`/`engine_unavailable`/`feedback_status` (`main.rs:71–82`), and a failure leaves the app fully usable with a `Retry engine` control. Transport works with or without an engine (`main.rs:86–89`). | — |
| **2B. Linked lenses** | 3 | Engine state is one global fact rendered in the transport bar, which is drawn once per frame from all four lenses (`main.rs:139`). No per-lens fork; `AppModel` gained no audio field (`lib.rs` diff is `pub mod engine;` and a header note only). | — |
| **2C. Modulation visibility** | 3 | Not yet applicable — no parameter reaches live audio. The implementation ships the honesty copy §3.3 requires in **both** places: Build footer (`main.rs:496–503`) and a new Shape header line (`main.rs:514–521`), both naming R4-2 and decision 22. | — |
| **2D. Keyboard-first, calm UI** | 2 | AF-5 respected: no new binding is assigned, `Retry engine` gets none, and the pre-existing `Space`/`1`–`4` scaffolding is neither extended nor documented as a map. **But** the engine's entire diagnostic surface — seven counters and the unavailability reason — is pointer-only tooltip text (`main.rs:194–202`, `:239`). A keyboard-only operator can read `ENGINE RUNNING · cpal · … · 88200 Hz · 256` and a headroom percentage and nothing else. | Render counters and the failure reason as text, not tooltips |
| **2E. Convergent-pattern grounding** | 3 | The stream is opened once and never closed on Stop — `stop_audition` sends `AllNotesOff` + `TransportCommand::Stop` and touches no lifecycle method (`engine.rs:423–430`) — which is the behaviour §Appendix A derives from `OBS-AB12-ROUTE-001`. Telemetry is published per block from the render thread rather than as a per-track meter, as the spec's stated divergence requires. | — |
| **2F. Differentiation** | 3 | Per-block callback headroom, xruns, containment and frame-capacity counts are read from the render thread and displayed; `ENGINE RUNNING` is gated on `blocks_rendered > 0` (`engine.rs:351`), which is a stronger honesty posture than a conventional transport light. | — |
| **2G. Benchmark evidence discipline** | 3 | No benchmark claim appears in the code or in any document this commit edits; every number cited in the docs is Spectre's own measurement. Nothing is attributed to Logic Pro or to Serum 2 beyond its two records. | — |

**Lens average:** 20 / 7 = **2.857**
**Lens pass:** Yes

---

## Lens 3: Product Identity & Scope Discipline (weight: 20%)

| Criterion | Score | Evidence | Remediation |
|---|---|---|---|
| **3A. Milestone fit** | 2 | The *feature* is scoped exactly right: no track model, no clip, no device, no persistence, no bridge edit, and the entire "deliberately not modified" set is untouched. **But the commit smuggles `site/index.html`** — 1,157 new lines, >50% of the commit's insertions, a project atlas that no slice authorizes, that §7.2 does not list, and that the commit message never mentions. | Land `site/index.html` as its own change with its own scope statement |
| **3B. Non-goal respect** | 3 | No hosting, no plugin format, no cross-DAW compatibility, no cloud. Recording is *removed* from the surface rather than implied: `ui.add_enabled(false, egui::Button::new("●  Record"))` (`main.rs:164`). | — |
| **3C. Deliberately small first devices** | 3 | No device added or modified. `PulseInstrument` still has no envelope; the audition reuses the fixture's own values, which I verified match `spectre_offline::fixture_events` exactly (`id: 1`, `channel: 0`, `note: 45`, `velocity: 0.8`). | — |
| **3D. Originality** | 2 | ENGINE-001/002/003 rows added to `requirements-ledger.md` with Spectre-derived rationale and evidence paths; the four `AUDITION_*` constants are **explicitly dispositioned** in the ledger prose as fixture values rather than bounds — that is exactly the right move. **`ENGINE_NODE_SEED` (`engine.rs:44`) is the one numeric constant the slice introduces that is neither given a row nor dispositioned.** It carries an inline rationale only. Not from a reference product, so AF-4 does not fire; PROD-003's "every numeric limit" is arguable for an identity seed, but the ledger's own list is incomplete. | Add one line to the ledger's ENGINE prose dispositioning `ENGINE_NODE_SEED` the way the audition constants are |
| **3E. Platform commitment** | 3 | One code path. `engine.rs` names no backend type (the only occurrence of "cpal" is a rationale comment at `:33`); the cpal selection is confined to `main.rs:50–57` behind `#[cfg(feature = "live-audio")]`. `default_sample_rate` earned itself immediately — the qualification device's own default is 88 200 Hz, so a hard-coded 48 000 would have been wrong on the first machine. **No Linux row was added**: `current-milestone.md:122` still reads "not run". | — |
| **3F. Accessibility trajectory** | 2 | The primary state words are text and are not colour-only (`ENGINE RUNNING` / `ENGINE OPENED` / `ENGINE UNAVAILABLE`), and every control is a standard egui widget. **Two gaps.** (a) §3.6's presentation rule is "inline, persistent, in the transport bar" and the reason string is instead `.on_hover_text(self.engine_unavailable.to_string())` (`main.rs:239`) — pointer-only. Partly mitigated by `feedback_status` receiving the same text on the failing frame (`main.rs:76`). (b) §3.4's device-name truncation requirement is **not implemented at all** — I grepped `main.rs` for `truncate`/`wrap`: no hits. An OS-supplied device name is unbounded and is formatted straight into the label, so at the 1060 px minimum width (`main.rs:696`) a long name has nothing stopping it from pushing the layout. | Display the reason as a label; add truncation on the device name only |

**Lens average:** 15 / 6 = **2.500**
**Lens pass:** Yes
**Auto-fail triggered:** No (3B ≠ 0; AF-4 not triggered — see below)

---

## Lens 4: Truthfulness & Evidence (weight: 20%)

| Criterion | Score | Evidence | Remediation |
|---|---|---|---|
| **4A. Current-state accuracy** | **1** | Much of the documentation is exactly right and I verified it: "**244** tests pass" — I counted 244 passed / 0 failed / 2 ignored; "formatting, strict Clippy" pass — both verified at the commit; "12 in `live_engine.rs` and 2 in `backend_seam.rs`" — verified. **Four claims do not survive checking.** (i) `STATUS.md` lists among the proven properties "**that a refused transport send leaves the UI unchanged**". Test 7 does not establish that — see 1G. This is a status document citing a non-result as evidence. (ii) `STATUS.md`: "The transport bar reports the backend, device, rate, block size, **and the render thread's own counters**" — the counters are tooltip-only. (iii) `STATUS.md`: "all 244 tests pass **with one ignored hardware drill**, plus **two** `#[ignore]`d hardware drills run by hand" — the suite contains exactly two ignored drills, so the sentence double-counts. (iv) `site/index.html`, **added by this very commit**, states the app "has **no audio thread**", that the header reads `ENGINE OFFLINE` and `CPU —` "because both are literally true" (line 349), and lists R4-1 as `impl:0` (line 811) — under a header promising "Every claim here is transcribed from docs/ or measured from the workspace". All three are false as of `8b1633d`. | See Priority 1 findings 1, 3, 4 |
| **4B. Status vocabulary** | 3 | Best-in-class. `implemented`, never `verified`, in the commit message, `STATUS.md`, and `current-milestone.md`; the milestone exit-evidence row is struck through and re-marked "partially closed … `implemented`, not closed"; the ledger rows are `implemented`. The distinction is respected everywhere I checked. | — |
| **4C. Traceability** | 3 | ENGINE-001/002/003 carry provenance (`PROD-003`; decision 16; the macOS record) and evidence paths naming real symbols and real test names. Constants carry rationale at the definition site. `STATUS.md` names the two drills by function name. | — |
| **4D. Honest gaps** | 3 | Unusually good. The commit, `STATUS.md`, `NEXT.md` and `current-milestone.md` all state that the manual protocol has not run, that nobody has confirmed audibly, that a Shape edit still does nothing live, and that the audition voice is R4-5 scaffolding. `frame_capacity_rejections` was added to the drill's printed line **and asserted zero** (`lifecycle_health.rs:298–302`) specifically to close an evidence gap the old record had. | — |
| **4E. Evidence commands** | 3 | The workspace gate is quoted exactly and all three commands pass at this commit; both drills are named exactly and are genuinely `#[ignore]`d. | — |
| **4F. No fake surfaces** | **1** | The audio-facing anti-fake-surface work is excellent: `ENGINE RUNNING` requires `blocks_rendered > 0` and test 9 proves it; Record is disabled; the position literal became `—`; there is deliberately **no** `NullBackend` fallback (`main.rs:59–63`). **Two fake surfaces ship anyway.** (a) `engine=not-started` in the product's own CLI output is a hard-coded literal in the `println!` (`main.rs:675`) presented — in a code comment, in a test comment, and in spec §5.3 — as a state readout that guards a regression. It is a status field that reports nothing and a guard that cannot fire. (b) `site/index.html` is a shipped "live progress board" that misreports the product's state in the commit that changes it. | Derive the smoke field from real state (e.g. print `engine=` from whether a start was attempted); regenerate or withhold the atlas |

**Lens average:** 14 / 6 = **2.333**
**Lens pass:** Yes (≥ 2.0), but two criteria at 1

---

## Auto-fail rules

| Rule | Verdict | Basis |
|---|---|---|
| **AF-1 — Contradicting accepted authority** | **Pass** | No accepted decision row or requirement is contradicted. Notably the implementation *corrected* the spec here: §3.3 item 3 prescribed the Record hover text "Recording arrives at R5", but `rebuild-roadmap.md:32` places recording at **R7**, and the shipped string says R7 (`main.rs:166`). No Linux claim was added. |
| **AF-2 — Unbacked implementation claims** | **Not triggered as written, but marginal** | AF-2 forbids describing code as existing without a source path that contains it; every symbol the docs name does exist and I opened each one. The nearest thing to a violation is `STATUS.md` citing a *test result* the test does not produce (4A(i)); I grade that at 4A/4F rather than stretch AF-2. A stricter reviewer could rule it triggered. |
| **AF-3 — Realtime discipline violation** | **Pass** | Verified three ways: nothing new is on a callback-reachable path (`engine.rs` is app-thread only and the closure is the R3 closure verbatim); `rt_guard`'s four-module scan passes **after** the `null.rs` edit; the app's own re-declared allocator reports 0 violations across 16 blocks with a positive control that does fire. One precision note in §4.1's argument does not hold: it claims both new `null.rs` bodies "return a constant" and "neither body allocates", but `NullBackend::default_sample_rate` branches and calls `device.clone()` on a `String`-backed `DeviceId` (`null.rs:74–79`, `lib.rs:32`) on the unknown-device path. It is app-thread-only, so RT-001 is untouched — but the accepted argument's stated premise is not what shipped. |
| **AF-4 — Borrowed numeric limits** | **Pass** | Enumerated every constant the diff introduces: `ENGINE_BUFFER_FRAMES` (ENGINE-001), `ENGINE_PLAN_FRAME_MARGIN` (ENGINE-002), `NULL_SAMPLE_RATE` (ENGINE-003), `AUDITION_VOICE_ID/CHANNEL/NOTE/VELOCITY` (explicitly dispositioned in the ledger prose, and I verified they equal `fixture_events`' own values), `ENGINE_NODE_SEED` (**no row, no disposition** — graded at 3D). None traces to a reference product; 256 traces to Spectre's own macOS measurement. |
| **AF-5 — Conclusions the evidence does not support** | **Pass** | No default shortcut map is fixed. `Retry engine` gets no binding; the pre-existing `Space`/`1`–`4` scaffolding is unchanged and undocumented as a map. No native-device list, no monitoring-latency threshold, no platform order asserted. |
| **AF-6 — Optimistic language** | **Pass, narrowly** | The commit and status prose are conspicuously careful ("Recorded as implemented, not verified"). The failures at 4A are factual errors and overstatements of scope, not promotional phrasing. |

---

## Feasibility Check

Read at commit `8b1633d`, in a clean clone, with the full gate executed.

| Check | Status | Notes |
|---|---|---|
| Types/models exist as specified | ✓ | 13/13 declared functions; `EngineHealth` exactly 9 fields. Two type-level deviations (`EngineState` variant count, `LiveEngine::voice_id`) are behaviour-neutral |
| API/interface changes feasible with current architecture | ✓ | Two required trait methods added; every existing `AudioStream`/`AudioBackend` implementor updated — the workspace compiles and clippy is clean |
| Views/screens fit current navigation pattern | ✓ | No new screen; three existing regions modified as §3.1 says |
| Dependencies available and version-compatible | ✓ | The spec's flagged risk did **not** materialise: the `spectre-app` ↔ `spectre-offline` dev-dependency cycle resolves — `cargo test --locked --workspace` builds and runs it. The `live-audio` default feature forwards `spectre-audio/cpal-backend` as specified |
| Platform/renderer requirements realistic | ✓ | Non-`Send` `LiveEngine` sits in the `eframe::App` implementor without incident, as §4.6 predicted |
| Test strategy executable with current infrastructure | ✗ | Executable, but **two assertions cannot fail** (1G), and the binding rule is structurally untestable where it was placed |
| Performance budget realistic | ✓ | Recomputed §4.7's arithmetic: 512-frame plan, 12,288 B pool, 6,144 B margin — all correct. One deviation: §4.7 promised "one `EngineHealth` read per frame — nine relaxed atomic loads"; the app actually reads telemetry **twice** per frame (`engine_state()` then `engine_health()`, `main.rs:178` / `:185`) and `state()` clones the device `String` each frame (`engine.rs:348`). Negligible, but not what was budgeted |
| No undeclared dependency on unbuilt features | ✓ | Nothing depends on R4-2/R4-5 work |

**Feasibility verdict:** Feasible — the artifact builds, lints, and passes its gate.
**The feasibility rule does not fail this artifact.** It fails on pass condition 3.

**Caveats:** one race worth recording rather than fixing blind — `RenderBridge::render`
increments `blocks_rendered` *before* `publish_headroom`, and the app derives "Running"
from `blocks_rendered` in one read and then fetches `worst_headroom` in a second. In the
window between them the bar can render `inf% headroom` (the counter initialises to
`f32::INFINITY`, `bridge.rs:56`). Cosmetic, one frame, at most once per stream.

---

## Composite Score

| Lens | Average | Weight | Weighted |
|---|---|---|---|
| 1 — Realtime & Correctness | 2.571 | 35% | 0.900 |
| 2 — DAW Workflow Depth | 2.857 | 25% | 0.714 |
| 3 — Product Identity & Scope Discipline | 2.500 | 20% | 0.500 |
| 4 — Truthfulness & Evidence | 2.333 | 20% | 0.467 |
| **Composite** | | | **2.581** |

Arithmetic recomputed: 18/7 = 2.5714286; 20/7 = 2.8571429; 15/6 = 2.5; 14/6 = 2.3333333.
(2.5714286 × 0.35) + (2.8571429 × 0.25) + (2.5 × 0.20) + (2.3333333 × 0.20)
= 0.9000000 + 0.7142857 + 0.5000000 + 0.4666667 = **2.5809524**.

**Pass conditions (criteria.md is binding):**

- [x] Composite ≥ **2.30** — 2.581 ✓
- [x] Every lens average ≥ **2.00** — 2.571 / 2.857 / 2.500 / 2.333 ✓
- [x] No criterion scores **0** ✓
- [ ] **At most two criteria score 1** — ✗ **three do: 1G, 4A, 4F**
- [x] All auto-fail rules pass ✓
- [x] Feasibility satisfies criteria.md ✓ (Feasible)
- [x] Reviewer personally executed every command claimed as passing ✓ (fmt, clippy, full workspace test, smoke; the two `#[ignore]`d hardware drills were **not** run and are marked unverified)

**All conditions met:** **No → FAIL**

---

## Remediation Brief

### Priority 1 — Must fix to pass

1. **Make the two-transport binding rule testable, and test it.** The rule §4.4 calls the
   slice's one genuinely new correctness hazard is implemented correctly — I read
   `SpectrePrototype::toggle_transport` (`crates/spectre-app/src/main.rs:85–114`) and all
   three branches match §4.4 exactly: `Ok(())` → `toggle_play()`; `Err(Note(..))` →
   `toggle_play()` + status; `Err(Transport(..))` → **no** mutation + status. Both entry
   points route through it and nothing bypasses it (`toggle_play` is called from exactly
   three sites, all inside `toggle_transport`; `toggle_transport` is called from exactly
   two, `main.rs:246` for the Play button and `main.rs:612` for `Space`). **But it has zero
   test coverage**, because it lives in a binary target, and test 7
   (`crates/spectre-app/tests/live_engine.rs:240–257`) papers over that with
   `assert!(!model.is_playing())` on an `AppModel` that no code under test ever sees. Move
   the decision into `engine.rs` as a pure function over
   `(&mut AppModel, Result<(), AuditionError>)` and assert all three branches, or delete
   the misleading assertion and the comment above it and rename the test to what it
   actually proves (`start_audition` returns the `Transport` variant when the lane is full).

2. **Derive the smoke line's `engine=` field from state.** `crates/spectre-app/src/main.rs`
   `smoke_test()` prints `engine=not-started` as a literal inside the format string, so
   `crates/spectre-app/tests/smoke_cli.rs:21` cannot fail no matter what the headless path
   does. Both the code comment at `main.rs:672–673` and the test comment at
   `smoke_cli.rs:20` assert the opposite. Print a value computed from whether a start was
   attempted (the same `EngineUnavailable::NotAttempted` the app already holds), so the
   assertion can actually catch the regression it is documented to catch.

3. **Remove `site/index.html` from this slice.** It is 1,157 undeclared lines, more than
   half the commit, in no slice's scope and absent from §7.2's file list — and its content
   contradicts the commit that carries it: line 349 says the app "has no audio thread" and
   that the header reads `ENGINE OFFLINE` / `CPU —` "because both are literally true", and
   line 811 records R4-1 as `impl:0`. (I read the file's header, its style block, and the
   two claim regions cited; I did not read all 1,157 lines, and there may be more stale
   claims in the parts I skipped.) Land it separately, regenerated against the post-R4-1
   state.

4. **Correct three claims in `docs/status/STATUS.md`.** (a) Drop "that a refused transport
   send leaves the UI unchanged" from the list of properties the live-engine set proves, or
   restore it after fixing finding 1. (b) "The transport bar reports … the render thread's
   own counters" — the counters are tooltip-only; either say so or make it true (see
   Priority 2). (c) "all 244 tests pass with one ignored hardware drill, plus two
   `#[ignore]`d hardware drills run by hand" double-counts: the suite has exactly two
   ignored drills, and I measured 244 passed / 0 failed / 2 ignored.

### Priority 2 — Should fix for quality

5. **Surface nonzero counters as text, not tooltip.** §3.3 item 8 is explicit — shown only
   when nonzero, and "when nonzero it must never be hidden" — and §3.6 E8/E9/E10 each say a
   counter "becomes visible". `main.rs:194–202` puts all seven behind one `on_hover_text`.
   Same for the failure reason at `main.rs:239`, which §3.6 requires inline and persistent
   in the bar.
6. **Implement §3.4's device-name truncation.** There is no truncation logic anywhere in
   `main.rs`; the OS-supplied name is formatted straight into a label inside a
   `right_to_left` layout, and the spec's constraint is that the name truncates and the Play
   button never moves.
7. **Disposition `ENGINE_NODE_SEED` in the ledger** the way the four `AUDITION_*` constants
   already are, so the ENGINE section's account of this slice's constants is complete.
8. **Use `Display`, not `Debug`, for the E7/E7b status strings.** `main.rs:100–111` formats
   `ControlError` with `{error:?}`, but §3.6 specifies `<Display>` and `ControlError` has a
   `Display` impl (`crates/spectre-audio/src/control.rs:39`). Same at `engine.rs:89` for
   `EngineUnavailable::Control`.

### Priority 3 — Consider for excellence

9. Restore `Eq` to `EngineUnavailable` — every component type derives it, so the spec's
   declared derive was achievable.
10. Read the telemetry once per frame into a single `EngineHealth` and derive both the state
    label and the counters from that snapshot, as §4.7 budgeted. It also closes the one-frame
    `inf% headroom` window created by `blocks_rendered` being published before
    `worst_headroom`.
11. `stop_audition` sends `AllNotesOff { channel: Some(AUDITION_CHANNEL) }` (`engine.rs:424–426`)
    where §3.2 step 8 specifies `channel: None`. Harmless today — `PulseInstrument` matches
    `AllNotesOff { .. }` and ignores the channel (`crates/spectre-dsp/src/source.rs:194`) —
    but a future instrument that honours the filter would leave notes on other channels
    stuck, which is exactly what an all-notes-off is for.
12. Reconcile the copy deviations, or amend the spec: §3.3 asks for the hover reason "Not
    driven by the engine yet; arrives with clips (R4-5)" on the tempo/meter/position group;
    the shipped strings say "Fixed project default; tempo editing arrives with the
    arrangement" and "Playhead position is not reported by the engine yet", neither of which
    names R4-5.

---

## Where the spec, not the code, was wrong

Recorded so the implementation is not graded down for following the codebase over the spec.

1. **§3.3 item 3 said "Recording arrives at R5."** `docs/06-plans/rebuild-roadmap.md:32`
   places audio/MIDI recording at **R7**. The implementation shipped "Recording arrives at
   R7" and is correct; the spec is wrong.
2. **§4.1's RT-001 argument rests on a premise the edit does not satisfy.** It says both new
   `null.rs` bodies "return a constant" and "neither body allocates". `NullBackend::default_sample_rate`
   branches on the device key and clones a `String`-backed `DeviceId` to build
   `BackendError::UnknownDevice` — which is the *right* behaviour (test 13 requires the
   refusal) and is app-thread-only, so RT-001 is unaffected. The correct argument is
   "app-thread-only, never callback-reachable, and re-proved by a post-edit scan", which the
   implementation does satisfy.
3. **§5.1 test 7 is unimplementable as designed.** It asks for an assertion that
   `AppModel::is_playing()` is unchanged after a refused send, but §4.4 places the binding
   rule in `main.rs`, a binary target unreachable from `crates/spectre-app/tests/`. No
   faithful implementation of that test can be falsifiable. The implementation followed the
   spec literally and produced a cannot-fail assertion; the spec set the trap. The fix
   belongs in both.
4. **§4.2 declared `EngineState::Unavailable(EngineUnavailable)`**, but an engine that does
   not exist cannot report a state — the app necessarily holds `Option<LiveEngine>`, so the
   variant is unreachable. The implementation's two-variant enum plus a sibling
   `engine_unavailable` field is the more honest shape.
5. **§4.7's "nine relaxed atomic loads" per frame** was never achievable as written given
   §3.3's requirement that the *state label* also derive from `blocks_rendered`; deriving
   both from one read is possible but the spec did not describe it.

---

**End of scorecard.**
