<!--
Author: Jeff
Date: 2026-08-16
Description: Blind verification scorecard for the R4-8 offline-bounce spec, iteration 1
Notes: Read in full — the spec, criteria.md, the scorecard template, and these sources:
  crates/spectre-offline/src/lib.rs (all 335 lines), crates/spectre-offline/src/main.rs,
  crates/spectre-audio/tests/bridge_plan.rs (all 246 lines), crates/spectre-graph/src/lib.rs
  :190-349 and :395-546, crates/spectre-audio/src/bridge.rs :1-40 and :115-291,
  crates/spectre-dsp/src/source.rs :105-216, crates/spectre-dsp/src/effect.rs :45-134,
  crates/spectre-dsp/src/io.rs :45-110, crates/spectre-graph/tests/containment.rs :1-115,
  crates/spectre-audio/tests/rt_guard.rs :285-321, crates/spectre-project/src/lib.rs :1-45,
  all three cited Cargo.toml manifests, and every OBS- record the spec cites.
  Sampled: docs/06-plans/current-milestone.md, requirements-ledger.md, decision-gates.md,
  docs/status/NEXT.md, docs/status/STATUS.md, docs/03-architecture/dsp-device-io.md — read only
  the cited line ranges plus surrounding context, not the whole files. Did not read
  gauntlet-output/specs/R4-1-live-audio-wiring.md in full; grepped it for the four symbols R4-8
  cites. Did not execute cargo — no test in this spec exists yet to run, and the four regression
  targets it names were verified to exist as files rather than by running them.
  All arithmetic in the spec was recomputed independently.
-->

# Scorecard: Offline Bounce

**Feature ID:** `R4-8` (`offline-bounce`)
**Spec file:** gauntlet-output/specs/R4-8-offline-bounce.md
**Reviewer agent:** blind verification agent, R4-8 iteration 1
**Date:** 2026-08-16
**Spec iteration reviewed:** 1

---

## Verdict: PASS

**Summary:** This is the most accurate §7.1 the gauntlet has produced — I opened every source
path it cites and checked roughly fifty-five line references, including the ones easiest to get
wrong (the duplicated FNV constants, RT-003's whole-quantum silencing, `RT_MODULES`'s exact four
entries, and the claim that `crates/` contains exactly two filesystem accesses and both are
reads), and found no error in any of them; §4.3–§4.4 then do the hard part of criterion 1D
honestly, keeping **one** FNV-1a fold and adding a traversal rather than a second hash, and
stating what legitimately differs and why. The critical gap is arithmetic: `BOUNCE_MAX_SECONDS`'s
rationale — the sentence the spec itself says is "what makes the ceiling honest rather than
arbitrary" — computes 24 h at 48 kHz / 256 frames as **337,500 blocks / 2.7 MB** when the correct
figures are **16,200,000 blocks / 129.6 MB**, a factor-of-48 error that propagates into §4.4(8)'s
"cheap enough to always be on" and §4.7's memory budget, and that would be written verbatim into
`requirements-ledger.md`. Secondary but real: §5.1 test 1 cannot fail as written, because after the
§7.2 refactor both sides of its assertion route through `hash_planar_quantum`, and that same
refactor deletes `bridge_plan.rs:80-92` — the workspace's only independent implementation of the
FNV walk.

---

## Lens 1: Realtime & Correctness (weight: 35%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 1A Callback-path discipline | 3 | §4.1's argument is structural, not rhetorical, and it checks out. Verified `RenderBridge`'s plan is private (`bridge.rs:120`) and its public surface is exactly `new`/`telemetry`/`transport`/`render` (`:133/:152/:157/:162`), so a bounce genuinely has no API path to the live plan. Verified R4-8 modifies nothing under `crates/spectre-audio/src/` — its only `spectre-audio` touch is `tests/`. §5.2 test 18 re-runs `rt_guard` as evidence rather than resting on the paragraph. | — |
| 1B Control↔render communication | 3 | §4.1 correctly establishes there is *no* RT-002 traffic: the bounce sends nothing on the lanes and reads nothing from them. §4.4's state table names the only shared state (`Arc<AtomicBool>`, `Arc<BounceProgress>`), its owner, its threads, and its one-render lifetime, and states it is shared app↔worker and never with a callback. Gap, not a scoring hit: the spec never says how the finished `BounceReport` crosses back from the worker to the app thread, and §3.1's promise that "the app stays fully usable" forbids a naive `join()` on the UI thread. | Name the completion handoff in §4.4 (poll `JoinHandle::is_finished`, or a one-slot channel) — P2 |
| 1C Numerical containment | 3 | R4-8 adds no DSP node, so RT-003 is inherited rather than re-implemented. §4.4(3) is verified correct against `crates/spectre-graph/src/lib.rs`: per-sample flush to signed zero at `:407-411` (`:409` is the sign-preserving branch), whole-quantum `fill(0.0)` at `:517-518`, `contaminated_nodes += 1` at `:519`. §3.6 E9 surfaces a nonzero count as a warning rather than shipping a clean-looking report, and §5.1 test 6 is a real injection test in the file where the poisoning devices already live. | — |
| 1D Determinism | 3 | The heart of the feature, and it is done properly. §4.3 identifies that the existing walk (`lib.rs:272`, channel-major) does not compose across blocks and that `hash_interleaved` de-interleaves to reproduce it (`bridge_plan.rs:82-84`) — both verified. It keeps **one** fold with the same offset basis and prime (verified byte-identical at `lib.rs:271`/`:276` and `bridge_plan.rs:81`/`:87`), adds a *traversal*, and adds **no third copy** of the constants. §4.3 states the consequence out loud (`BounceReport::hash != RenderReport::hash`) and §5.1 test 4 pins the inequality. §4.4(8) answers all three parts of the criterion: *what* is compared (whole-render streaming hash), *what a mismatch reports* (per-block hash log to localize a block, then `first_divergence` in **bits**, with the signed-zero and NaN reasoning verified against `lib.rs:409`), and *what legitimately differs* (§4.4(3) block geometry under containment; §4.6 cross-machine libm, refused rather than claimed). I independently verified §4.4(4): `PulseInstrument`'s per-frame body (`source.rs:202-209`) reads only carried state and `sample_rate()`, `Gain` (`effect.rs:62-71`) and `Saturator` (`effect.rs:121-129`) are memoryless — so the block-invariance claim is true for today's device set, exactly as scoped. | — |
| 1E Graph and plan contract | 3 | §4.1's GRAPH-001 paragraph is correct: `compile` allocates the channel pool (verified at `lib.rs:324`, `vec![vec![0.0; max_frames]; channel_count]`) and runs once before the block loop, never inside it. No plan recompilation per parameter change is proposed anywhere; §8 Q2 raises the R4-2 interaction as an open question instead of deciding it. | — |
| 1F Failure behavior | 3 | §3.6's E1–E9 are explicit and fail closed. Pre-render refusals (E1–E3) happen before anything is written; mid-render failures (E5, E6) delete the partial file; E7 keeps the file *because it is evidence* and says so; cancellation (E8) removes the partial. §5.2 test 15 asserts `frame_capacity_rejections() == 0` and `blocks_rendered() == 16`, which is the correct guard against `bridge.rs:167-173`'s silent-refusal path — that path increments rejections without incrementing `blocks_rendered`, and the spec's block-count claim survives it because block size equals `max_frames`. | — |
| 1G Test specification | 2 | 18 of 20 tests name a real target and a falsifiable assertion; the four regression targets (`harness`, `bridge_plan`, `rt_guard`, `containment`) all exist as files. Test 6's arithmetic is verified correct: a NaN at absolute frame 300 silences 0–511 in one 512-frame quantum and only 256–511 in two 256-frame quanta, with `contaminated_nodes == 1` both ways. **But §5.1 test 1 cannot fail for the reason it is designated to catch.** After §7.2's refactor, `render_vertical_slice(…).hash` *is* `hash_planar_quantum(output)`, so asserting the two are equal compares the implementation under test against itself; the only divergence it can still detect is the test rebuilding the fixture with a different seed or different device values. The spec calls it "pinned differently and more strongly" and "the test that fails if the refactor changes any existing hash" — it is pinned weaker, and it would not fire on a changed hash. Compounding this, §7.2 reduces `bridge_plan.rs:80-92` to a call into the same module, deleting the workspace's only *independent* implementation of the FNV walk; tests 17 and 19 then also compare shared code against itself for the hash function (they still pin the audio). | Replace test 1 with a literal golden vector: `SampleHasher` over a hand-specified `[f32]` array asserted against a checked-in `u64`. No libm is involved in a literal array, so §8 Q4's cross-machine objection does not apply. — P1 |

**Lens average:** 2.857
**Lens pass:** Yes — avg ≥ 2.0, no criterion scores 1, no 0s

---

## Lens 2: DAW Workflow Depth (weight: 25%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 2A Loop-first core loop | 3 | §3.1 argues the non-modal panel from the workflow rather than from taste: "a modal … converts a background job into a hostage situation," and it forecloses the only useful thing to do during a render. §3.2 step 4 states that selection, zoom, lens, and transport are untouched; §5.4's "Long render behavior" row makes that a manual check. | — |
| 2B Linked lenses | 3 | §3.1 puts the entry point in the transport bar because it is "the one region visible from every lens," keeping "a render is running" one global fact rather than a per-lens fork. Verified the transport bar exists today (`crates/spectre-app/src/main.rs:50-51`, `TopBottomPanel::top("transport")`), so "modified" is the correct word. §4.4 keeps `BouncePanel` off `AppModel`, so no lens forks state. | Note that `SidePanel::right("inspector")` (`main.rs:163`) already owns the right edge; §5.4's screen-size row checks the transport bar and lens body but not the existing left and right panels — P3 |
| 2C Modulation visibility | 2 | Genuinely inapplicable — R4-8 exposes no parameter surface — but the spec never says so in PROD-002's terms. It does engage the underlying problem substantively: §8 Q2 asks what "matches the live path" means once R4-2 makes parameters changeable mid-render, and §7.1's closing paragraph reports that `dsp-device-io.md:94`/`:104` describe a smoothing `Gain` that `effect.rs:55-73` does not implement (verified true) and ties it to the same question. | One line stating PROD-002 does not apply because no parameter is displayed — P3 |
| 2D Keyboard-first, calm UI | 3 | §3.4 is the AF-5-correct handling: it refuses to assign any shortcut, cites the prohibited-conclusions list by name, states the remappable/context-scoped requirement the accepted direction does support, and gives the structural reason nothing forecloses it ("a single app-thread function with no UI state in its signature"). No spreadsheet density, no color-only state (§3.7). | — |
| 2E Convergent-pattern grounding | 3 | Appendix A both converges and diverges with stated reasons. Converges with `OBS-AB12-MIX-002` (verified verbatim at `ableton-live-observations.md:99`: 32-bit float engine, clipping matters at physical outputs, main output, or file export) and with `OBS-PP-ARCH-001` and `OBS-VCV-VOLT-006` (both verified). Diverges deliberately from `OBS-AB12-ARR-008` (verified: Consolidate incorporates clip-level gain/warp/pitch/envelopes **but not track effects**) because a partial-chain render would be a second computation. | — |
| 2F Differentiation | 3 | Appendix A's "Differentiation" paragraph states the claim precisely and then bounds it: Spectre makes equivalence a CI gate with a localizable failure, and the spec explicitly does **not** claim no other DAW does this — "only that this repository holds no record of one." That is the correct shape for a claim against a corpus that does not document competitors' internals. | — |
| 2G Benchmark evidence discipline | 3 | Every `OBS-` ID I opened says what the spec claims. The gap-naming is exact and verbatim-accurate: `ableton-live.md:64` is `inventory-only` for chapter 20 "Bounce to Audio"; `:113` and `:139` are `section-inventoried`, and `:139`'s unresolved-question list ("signal-path equivalence, interruption, plugin realtime requirements, metadata, dither, SRC, partial-output cleanup, and failure reporting") is quoted verbatim; `logic-pro.md:64` reads `unreviewed`. Best of all, the spec finds `OBS-SR2-GLOB-008` — the one directly convenient record — and **refuses it**, citing the quarantine banner at `serum-2-observations.md:10-38` (verified, including the missing `source-ledger.json` record for the 354-page guide) and D-R2 in `decisions-needed.md` (verified). Recording a refusal is the behavior criteria.md calls a 3. | — |

**Lens average:** 2.857
**Lens pass:** Yes

---

## Lens 3: Product Identity & Scope Discipline (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 3A Milestone fit | 3 | Verified all three anchors exactly: `current-milestone.md:12` scope line ends "save/reload, bounce"; `:86` is verbatim "Offline bounce renders the same project deterministically and matches the live path's computation"; `NEXT.md:30` is slice 8, "extending the hash-equivalence approach the callback bridge already uses." §7.4 defers stem export, codecs, dither, 16/24-bit, SRC, region export, realtime bounce, metadata, and batch renders by name. | — |
| 3B Non-goal respect | 3 | §6.4 enumerates each non-goal and states the output is a plain WAV of Spectre's own render, "not a project-interchange format, and not a preset." Verified against `current-milestone.md:77`, which also covers R4-8's correct treatment of recording and latency compensation as R5+. | — |
| 3C Deliberately small first devices | 3 | "**No device is added or modified.**" §6.4 goes further and declines a limiter, a fade-out, and a normalizer precisely because each "would make a bounced file sound better" — new DSP in a slice whose claim is that it adds none. | — |
| 3D Originality | 2 | Both constants are Spectre-derived, not borrowed: `BOUNCE_FALLBACK_BLOCK_FRAMES` from Spectre's own macOS measurement (verified at `current-milestone.md:113`: 173 blocks, 0 xruns, 0.990 worst headroom) by way of R4-1 §4.2's `ENGINE_BUFFER_FRAMES = 256` (verified at that spec's line 393), and `BOUNCE_MAX_SECONDS` from TIME-003's own 24-hour horizon (verified at `requirements-ledger.md:38`). §7.2 schedules **both** rationale rows into `docs/01-requirements/requirements-ledger.md`, so PROD-003 is properly discharged and AF-4 does not fire. **The defect is the arithmetic inside the second rationale.** 24 h × 48 kHz ÷ 256 = **16,200,000** blocks, and 16,200,000 × 8 B = **129.6 MB**. The spec states 337,500 blocks and 2.7 MB — the figures for a **30-minute** render, wrong by exactly 48×. That sentence is the whole justification ("bounded and cheap, which is what makes the ceiling honest rather than arbitrary") and it would enter the ledger as written. §3.2 step 4's derived example is independently wrong too: 4,210 of 337,500 is 1.25%, not the "12.6%" shown. Every other figure I recomputed is correct — 6,144 B, 2,048 B, 207 MB, 69.1 MB, 16 blocks, 8 blocks. | Correct §4.2, §4.4(8), §4.7, and §3.2 step 4; then re-argue whether a 129.6 MB always-on hash log is still "cheap enough to always be on" or needs a cap — P1 |
| 3E Platform commitment | 3 | §4.6 is the strongest platform section I have seen in this run: one code path, `std::fs` and `std::thread` only, no audio device, so the whole proof runs headless on both platforms. It explicitly discharges **none** of decision 23's debt and forbids recording any Linux result from this slice. Verified: the bounce composes only crates that build on both platforms and `spectre-offline` has no platform-conditional dependency. | — |
| 3F Accessibility trajectory | 3 | §3.7's central claim is verified exactly: `crates/spectre-app/Cargo.toml` builds eframe with `default-features = false` and only `default_fonts` and `glow`, so no accessibility feature is enabled and the spec claims no screen-reader support. It forecloses nothing — standard widgets, text-carried progress and results, no icon-only control, and an explicit rule that E7's `MISMATCH` must not be color-only. Focus order is reasoned against `main.rs:78`'s `right_to_left` cluster (verified to exist). | — |

**Lens average:** 2.833
**Lens pass:** Yes
**Auto-fail triggered:** No — see roll-call below

---

## Lens 4: Truthfulness & Evidence (weight: 20%)

| Criterion | Score (0–3) | Evidence from spec | Remediation needed |
|---|---|---|---|
| 4A Current-state accuracy | 3 | I opened every source path §7.1 cites and checked every line number. All correct. Spot list of the load-bearing ones: `RenderReport`'s four fields at `lib.rs:29-34`; `render_plan` private at `:204-285`, compiling at `:243` and calling `process` exactly once at `:259-267`; FNV constants at `:271`/`:276` and their verbatim duplicates at `bridge_plan.rs:81`/`:87`; the `frames < 2` guards at `:289`/`:307`/`:320`; `containment.rs`'s `PoisonEffect` writing `outputs[0][0]` at `:100`; `RT_MODULES` at `rt_guard.rs:293-298` with exactly the four named entries at `:294-297` and `FORBIDDEN` at `:299-307` with exactly the seven named needles; `ProjectDoc`'s exactly five fields at `spectre-project/src/lib.rs:29-38`; `spectre-app`'s exactly six dependencies; `spectre-offline`'s dev-dependency on `spectre-app` (so §4.5's cycle hazard is real, not hypothetical). The claim easiest to overreach on — "the only filesystem write in `crates/` outside a test is none: `main.rs:16` reads and `rt_guard.rs:311` reads" — is **exactly true**; a workspace-wide grep returns those two lines and nothing else. §4.4(6)'s three-way transport check is also exactly true: `Transport` appears nowhere in `spectre-graph/src/` or `spectre-dsp/src/`, and `advance` (`transport.rs:96`) is called only from `spectre-core`'s own tests. | — |
| 4B Status vocabulary | 3 | §7.1 partitions into Implemented / Planned (spec'd, not implemented) / Gated / Absent, and the partition is accurate: R4-1 "spec'd and passed at 2.950" but `crates/spectre-app/src/engine.rs` does not exist (verified — `src/` holds only `lib.rs` and `main.rs`), R4-7 passed at 3.000 (both verdicts verified in the scorecards). §7.2 states status moves to `implemented`, **not** `verified`, until §5.4's manual checks pass. | — |
| 4C Traceability | 2 | Nearly every normative claim carries a requirement ID, decision row, `OBS-` ID, or source path, and every one I checked resolves correctly — including the four ledger IDs (RT-001 `:28`, RT-002 `:29`, RT-003 `:30`, TIME-003 `:38`, GRAPH-001 `:55`, PROD-003 `:64`) and both decision rows (16 at `decision-gates.md:40`, 17 at `:41`). The exception is the §4.2 / §4.4(8) / §4.7 hash-log figure, which cites its own computation and is wrong by 48× (see 3D). A claim traced to arithmetic that does not hold is not traced. | Same fix as 3D — P1 |
| 4D Honest gaps | 3 | Ten open questions, none decorative. Q4 refuses a checked-in golden hash for render output and gives the real reason (`f32::tanh` at `effect.rs:119`/`:127` and `f64::powf` at `source.rs:204` route to platform libm — both verified). Q5 refuses to edit `dsp-device-io.md:113` by assertion and routes the narrowing instead (verified: that line reads "Identical initial state and input produce bit-identical offline output"). Q9 argues against the spec's own bundling of `hash.rs` with the bounce; Q10 argues against its own hand-rolled WAV writer. §7.1 also reports the `Gain`-smoothing documentation divergence (`dsp-device-io.md:94`/`:104` versus `effect.rs:55-73`) rather than silently correcting it — verified true in both directions. | — |
| 4E Evidence commands | 3 | The workspace gate is quoted exactly as criteria.md requires (`cargo fmt --all -- --check`; `cargo clippy --locked --workspace --all-targets -- -D warnings`; `cargo test --locked --workspace`). Every `-p X --test Y` target either exists today (`spectre-offline --test harness`, `spectre-audio --test bridge_plan`, `--test rt_guard`, `spectre-graph --test containment` — all four verified present) or is created by this spec and named as such. | — |
| 4F No fake surfaces | 3 | The spec says three separate times that `./spectre` produces no sound, verified against `STATUS.md:19` ("`./spectre` does not use the audio crate, so nothing launchable makes sound") and `:37`. §3.3 goes further than required: because `ProjectDoc` has no tracks or clips, it *mandates* an empty-state string saying the bounce renders the built-in fixture, and calls anything shorter "a fake surface." §4.7 reports a measured realtime factor and explicitly asserts no target. | — |

**Lens average:** 2.833
**Lens pass:** Yes

---

## Auto-fail roll-call

| Rule | Result | Basis |
|---|---|---|
| AF-1 Contradicting accepted authority | **Pass** | The one place R4-8 narrows an accepted statement (`dsp-device-io.md:113`) is flagged as such in §7.2 and routed to §8 Q5 rather than edited. No accepted ledger requirement or decision row is contradicted. |
| AF-2 Unbacked implementation claims | **Pass** | §7.1 distinguishes implemented / planned / gated / absent, and every claim resolved in one `Read`. I found no misdescription in either direction — including the absences, which are stated as absences (`engine.rs` does not exist; no bounce, no hasher, no WAV writer, no divergence locator, no filesystem write). |
| AF-3 Realtime discipline violation | **Pass** | R4-8 adds nothing on a callback-reachable path. §4.1 places all file I/O on the bounce worker inside `wav.rs`, states the boundary exactly ("everything R4-8 adds sits strictly outside `process`, on the caller's side of it"), and the structural claim is verified: `RenderBridge` exposes no API through which a bounce could reach the live plan. Shared state is app↔worker only. |
| AF-4 Borrowed numeric limits | **Pass** | Two constants, both Spectre-derived, both with rationale rows scheduled into `requirements-ledger.md` in §7.2's modified-files list. The `BOUNCE_MAX_SECONDS` rationale is arithmetically wrong but it is not borrowed; §7.2 also records that no tail and no timeout constant is introduced, and gives the reason for each. |
| AF-5 Conclusions the evidence does not support | **Pass** | §3.4 refuses a default shortcut map by name and cites the prohibited-conclusions list. No native-device list, synthesis architecture, gesture budget, interface model list, monitoring-latency threshold, platform order, or archetype promotion appears. |
| AF-6 Optimistic language | **Pass** | Close call, stated for the record. "Bounded and cheap" in §4.2 is false at the stated bound, but it is an arithmetic error with its working shown, not unevidenced promotional phrasing about state. Graded under 3D and 4C instead. The spec's actual posture on state is the opposite of optimistic — it restates three times that nothing makes sound and that its own app panel is blocked on an unimplemented spec. |

---

## Feasibility Check

Read the actual source files referenced in the spec before filling this table.

| Check | Status | Notes |
|---|---|---|
| Types/models exist or are clearly specified | ✓ | Every new type is fully specified in §4.2 with fields and signatures. `DeviceParameterSnapshot`, `NoteEvent`, `GraphError`, `PlanError` are all real public types. `bounce.rs` sits as a child module of `spectre-offline`'s crate root, so it can reach the private `DeviceValues::from_snapshot` (`lib.rs:45-146`). |
| API/interface changes are feasible with current architecture | ✓ | `SampleHasher::new/write/finish` being public is what makes §7.2's `hash_interleaved` delegation possible — the channel-major-over-interleaved walk has no direct equivalent among the three named functions, but it can keep its loop and swap the inline constants for the shared hasher. |
| Views/screens fit current navigation pattern | ✓ | Transport bar exists (`main.rs:50-51`); egui side panels exist (`main.rs:116`, `:163`). Caveat: the right edge is already `SidePanel::right("inspector")`, unmentioned. |
| Dependencies are available and version-compatible | ✓ | No new crate dependency at all. `spectre-audio` already dev-depends on `spectre-offline` (verified), so the equivalence test needs no manifest change. The `spectre-app → spectre-offline` edge does create a dev-dependency cycle through `spectre-offline`'s existing dev-dependency on `spectre-app` (verified); Cargo permits this, and §4.5 flags it for verification rather than assuming either way. |
| Platform/renderer requirements are realistic | ✓ | `std::fs` and `std::thread` only; no device; headless on both platforms. Verified nothing in the bounce path is platform-conditional. |
| Test strategy is executable with current infrastructure | ✓ with one exception | All four regression targets exist as files; the two new test files use only public APIs plus the crate's own `[dependencies]`, which integration tests can link. The exception is §5.1 test 1, which is executable but tautological — see 1G. |
| Performance budget is realistic for target hardware | ✗ | Every figure recomputed correct except the hash log: 129.6 MB at the ceiling, not 2.7 MB. §4.7's "Total steady-state working set is well under 16 KB plus the hash log" is true but the log is the dominant term at long lengths, and `log_block_hashes` is default-on. |
| No undeclared dependency on unbuilt features | ✓ | §7.4 declares each: R4-1 blocks the panel (not the proof), R4-7 blocks the default path, R4-4/R4-5 block bouncing real content, R4-6 is the risk to the block-invariance claim with test 5 as the tripwire. The core proof composes only implemented components — verified. |

**Feasibility verdict:** Feasible with caveats
**Caveats:** the per-block hash log's memory budget is understated by 48×, and §5.1 test 1 as written cannot detect the regression it is designated to catch.

---

## Composite Score

| Lens | Average | Weight | Weighted |
|---|---|---|---|
| 1 — Realtime & Correctness | 2.857 | 35% | 1.000 |
| 2 — DAW Workflow Depth | 2.857 | 25% | 0.714 |
| 3 — Product Identity & Scope Discipline | 2.833 | 20% | 0.567 |
| 4 — Truthfulness & Evidence | 2.833 | 20% | 0.567 |
| **Composite** | | | **2.848** |

**Pass conditions (values copied from criteria.md, which is binding):**
- [x] Composite ≥ **2.30** — 2.848
- [x] Every lens average ≥ **2.00** — 2.857 / 2.857 / 2.833 / 2.833
- [x] No criterion scores **0** — lowest score is 2
- [x] At most **two** criteria score 1 — zero criteria score 1
- [x] All auto-fail rules pass — AF-1 through AF-6 all clear; see roll-call
- [x] Feasibility rule satisfied — I opened every source path the spec cites and confirmed each claim against the file. §7.1 does not misdescribe current state anywhere I could find.
- [x] Reviewer personally executed every command claimed as passing where required — no command in this spec is *claimed* as currently passing except the four existing regression targets, which are asserted only as "must continue to pass" after a change that has not been made; I verified those targets exist rather than running them, and recomputed every arithmetic claim by hand.

**All conditions met:** Yes → **PASS**

---

## Remediation Brief

The spec passes; these are required and recommended corrections, not blockers to acceptance.
Priority 1 items should be fixed before implementation begins, because both would otherwise
propagate into the requirements ledger and into the test suite respectively.

### Priority 1 — Must fix before implementation

1. **Correct the `BOUNCE_MAX_SECONDS` arithmetic in §4.2.** 24 h × 48,000 Hz = 4,147,200,000
   frames; ÷ 256 = **16,200,000 blocks**; × 8 B = **129.6 MB**. The spec states 337,500 blocks
   and 2.7 MB, which are the figures for a 30-minute render. Fix the same numbers where they
   repeat: §4.4(8) item 1 ("8 bytes per block, 2.7 MB at the 24-hour ceiling") and §4.7's
   "Per-block hash log" bullet.
2. **Re-argue §4.4(8)'s "cheap enough to always be on" against the corrected figure.** A
   129.6 MB always-on allocation is a different design claim from a 2.7 MB one. Either keep
   `log_block_hashes` default-on and state the real ceiling cost plainly, or introduce a cap on
   logged blocks — which would be a third numeric bound and would need its own §4.2 rationale
   and its own scheduled row in §7.2's `requirements-ledger.md` entry.
3. **Fix §3.2 step 4's progress example.** `block 4210 of 337500 · 12.6%` is internally
   inconsistent: 4,210 of 337,500 is 1.25%. Recompute against whatever total the corrected
   ceiling implies.
4. **Replace §5.1 test 1 with an independent pin.** As written, after §7.2's refactor both sides
   of `hash::hash_planar_quantum(output) == render_vertical_slice(48_000.0, 512)?.hash` route
   through the same function, so the assertion cannot fail and the spec's claim that it "is the
   test that fails if the refactor changes any existing hash" is false. Assert `SampleHasher`
   against a checked-in `u64` computed over a hand-specified literal `[f32]` array. No
   transcendental function is involved, so §8 Q4's libm objection does not apply and the vector
   is safe to check in.
5. **State in §7.2 what is lost by collapsing `bridge_plan.rs:80-92`.** Today that function is an
   independent second implementation of the FNV walk, and the agreement of two implementations is
   part of what makes `bridge_output_matches_the_offline_render_of_identical_input` a real
   cross-check. After the refactor, tests 1, 17, and 19 all route through one implementation.
   Either say plainly that the golden vector from item 4 replaces that cross-check, or keep the
   independent walk in `bridge_plan.rs` and remove only the duplicated constants.

### Priority 2 — Should fix for quality

1. **§4.4 — name the worker-to-app completion handoff.** The state table covers the cancellation
   flag and the progress counters but not how the finished `BounceReport` (or `BounceError`)
   reaches the app thread. §3.1 promises the app "stays fully usable," which rules out a blocking
   `join()` on the UI thread; say whether the panel polls `JoinHandle::is_finished` or the worker
   pushes into a one-slot channel.
2. **§4.4(1) / §7.2 — schedule the shared fixture builder, or say a copy is accepted.** §4.4(1)
   says the bounce "builds the same three-node chain `render_plan` builds (`lib.rs:215-258`)," but
   §7.2's `lib.rs` entry schedules only the module declarations and the hash-loop swap. Without
   extracting a plan builder, `bounce.rs` lands a third verbatim copy of the fixture chain
   (after `render_plan` and `bridge_plan.rs::fixture_plan`) — which is the same duplication
   failure mode the spec is otherwise careful about with the FNV constants.
3. **§4.7 — state the ceiling case for the plan's channel pool.** The section correctly derives
   6,144 B at 256 frames and correctly shows why a single-pass render is rejected (207 MB at
   3 minutes), but does not state that the pool scales with `block_frames`, so a caller passing a
   large live block size pays proportionally.

### Priority 3 — Consider for excellence

1. **§3.1 / §5.4 — account for the existing right-edge panel.** `SidePanel::right("inspector")`
   (`main.rs:163`) already owns the right edge, and `SidePanel::left("tracks")` (`:116`) owns the
   left. §5.4's screen-size row checks coexistence with the transport bar and lens body but not
   with those two panels at the 1060 px minimum (`main.rs:508`).
2. **§6.3 / Lens 2 — state PROD-002's N/A explicitly.** The bounce displays no parameter, so
   base/automation/modulation distinctness does not arise; §8 Q2 engages the underlying issue
   well, but the criterion is never answered in its own terms.
3. **§4.7 — the realtime-factor implication could be tightened.** "Roughly two orders of
   magnitude faster than real time" is derived from 0.990 headroom, and the spec correctly
   disclaims it as an implication of one measurement rather than a target. Noting that the
   measurement's block size (256 frames) comes from R4-1 §4.2 and not from
   `current-milestone.md:113` — whose table has no block-size column — would make the
   provenance exact.

---

**End of scorecard.**
