<!--
Author: Jeff
Date: 2026-08-23
Description: Independent blind verification scorecard for the R4-2 runtime-parameter-seam spec, iteration 1
Notes: Second reviewer. The first scorecard was written by the orchestrating session rather than an
  independent agent and marked nine of twenty-six criteria [sampled]; it invited correction in its own
  words. This grade was built from source, not inherited. Where it disagrees with the earlier scorecard
  the disagreement is stated explicitly with the file and line that settles it.
-->

# Scorecard: Runtime Parameter Seam — Independent Review

**Feature ID:** `R4-2` (`runtime-parameter-seam`)
**Spec file:** `gauntlet-output/specs/R4-2-runtime-parameter-seam.md` (1,686 lines)
**Reviewer agent:** independent blind verification agent (second reviewer)
**Date:** 2026-08-23
**Spec iteration reviewed:** 1
**Prior scorecard:** `spec-scorecards/R4-2-runtime-parameter-seam-scorecard.md` (3.000, PASS, nine criteria `[sampled]`)

---

## Verdict: PASS

**Summary:** The spec's evidence integrity is genuinely outstanding and its architectural
reasoning is correct end to end — of roughly sixty source citations opened at their exact
lines, all but three land precisely, §7.1 contains no false row in either direction, the
`rt_guard` and no-panic arguments both hold, and it carries **zero** line-pinned citations
into the moving `gauntlet-output/` documents that rotted three sibling specs. But §5 does
not survive a full read: §5.1's celebrated "which tests hold which handles" preamble is a
15-test map laid over an 18-test list and is false for five of them, and the two end-to-end
tests that encode the acceptance criterion both fail to compile as written — one on a type
that cannot coerce, and both on an `EngineHealth.parameters_pending` field that neither R4-1
nor R4-2's own §7.2 ever creates. **The single most important fix: §7.2 must add
`parameters_pending` to `EngineHealth` and §4.3 must make `apply_parameter_edit` generic over
`S`, or test 16 — the acceptance criterion — cannot be written.**

**Scope of this review, stated honestly.** Read **in full**: the entire 1,686-line spec,
`criteria.md`, the scorecard template, and the prior scorecard. Opened **at the exact cited
lines**: every source path the spec cites in §1.2, §3.1–3.7, §4.1–4.7, §5.1–5.4, §6, §7.1–7.4,
§8, and Appendix A — roughly sixty distinct citations across `spectre-core`, `spectre-dsp`,
`spectre-graph`, `spectre-audio`, `spectre-app`, `docs/`, and `gauntlet-output/`. Opened and
read: **all seven** cited `OBS-` records (the prior scorecard opened five). Verified by
recomputation: every figure in §4.2's rationale and §4.7's budget. Verified by grep across
`crates/`: the seven `impl AudioProcessor` sites, the three uncalled inherent setters, and
`DeviceParameterKey`'s conversion surface. **Not independently executed:** no `cargo`
command was run — `criteria.md` requires the reviewer to read cited files, which was done
exhaustively; it does not require compiling a spec that describes unwritten code. **No
section of this spec was sampled.**

---

## Lens 1: Realtime & Correctness (weight: 35%)

| Criterion | Score | Evidence from spec | Remediation needed |
|---|---|---|---|
| 1A. Callback-path discipline | 3 | §4.1's RT-001 argument verifies exactly and refuses the easy version of itself. `RT_MODULES` is at `rt_guard.rs:293–298` with the four entries at `:294–297` reading verbatim `src/bridge.rs`, `src/control.rs`, `src/spsc.rs`, `src/null.rs`; `FORBIDDEN` is at `:299–307` with seven needles at `:300–306`; the substring assert spans `:310–319`. `midi.rs` is indeed unscanned. **Two of the four scanned modules are in this slice's change list**, so the spec argues from edit content instead: `control.rs`'s length check lands in a constructor that already allocates (`control.rs:205–217` — confirmed, `Vec::with_capacity` through `bounded::<RetiredState>`), and `bridge.rs:181` is verbatim `let pending = self.control.drain_parameters(\|_, _\| {});`. It then adds its own new `route.rs` to the scan as `[&str; 5]`, scheduled in §7.2. The argument holds in full. | — |
| 1B. Control↔render communication | 3 | Uses the accepted RT-002 lane, does not invent one. Drain position confirmed: `apply_transport`/`collect_notes` at `bridge.rs:175–176`, parameters at `:178–186`, `plan.process` at `:192–195`. Lane depths 1,024/64/32 at `control.rs:17–19`; `ParameterTarget` derives `PartialOrd, Ord` at `:22` as §4.2 claims, so the binary search needs no new ordering. Latest-wins is decision 21 (verified at `decision-gates.md:45`) and already pinned by `control_channel.rs:45`. §4.4's reclamation paragraph is correct and load-bearing: option (a) mutates in place and retires nothing, which is exactly why RT-002's off-thread-reclamation clause is untouched. | — |
| 1C. Numerical containment | 3 | Containment is doubled and both layers verified. `ParamSpec::clamp` returns the descriptor default for non-finite input at `param.rs:119–124`, with the `!plain.is_finite()` branch at `:120–122`; RT-003's `contain_channel` at `graph/lib.rs:401–414` is a software FTZ-equivalent (not a control-register write), which is what makes §4.6's aarch64 portability claim true rather than asserted. Test 2 proves the non-finite→default mapping **at the new seam**, and quotes `dsp-device-io.md:72` correctly. | — |
| 1D. Determinism | 3 | No second render path. §5.2 makes the existing `bridge_output_matches_the_offline_render_of_identical_input` the load-bearing regression gate and requires it to pass **unchanged** — and that test hashes against `spectre_offline::render_vertical_slice` with the existing FNV-1a walk (`bridge_plan.rs:110–116`), which is the comparison method 1D names. §4.3's refusal to add smoothing is argued precisely because it would break that bit-exactness. Test 5 asserts hash identity across a refused call. | — |
| 1E. Graph and plan contract | 3 | GRAPH-001 intact. `CompiledPlan` at `graph/lib.rs:418–545` exposes exactly `max_frames`, `step_count`, `containment`, `process`, `last_output` — confirmed by enumerating the impl block; there is no parameter method. The parallel-index invariant is real: `steps.push` at `:310`, `processors.push` at `:317`, zipped at `:482`. The proposed node lookup reuses the linear scan `process` already performs at `:466–470`. **No recompilation is proposed anywhere in 1,686 lines**; decision 22's option (b) never appears. | — |
| 1F. Failure behavior | 3 | §3.6's E1–E8 each carry trigger, presentation, recovery, and data-loss. Fail-closed throughout: an unrouted target renders the **previous** value and increments a counter rather than going stale-and-hidden. The rule that no error string is formatted on the audio thread is stated as binding and is corroborated by `ControlError`'s own doc comment at `control.rs:28` ("never constructed on the render path"). §3.6 explicitly declines a threshold-derived health badge under decision 16/PROD-003. | — |
| 1G. Test specification | **1** | Eighteen tests, most excellent — but a full read of the sampled range finds five defective and a false structural preamble. **(a)** §5.1's handle map is a 15-test map over an 18-test list: tests 4–5 are in `crates/spectre-graph/tests/graph_plan.rs` and cannot "build … a `RenderBridge` directly" because `RenderBridge` is in `spectre-audio`, which *depends on* `spectre-graph` (`crates/spectre-audio/Cargo.toml`) while `crates/spectre-graph/Cargo.toml` lists only `spectre-core` and `spectre-dsp` and has **no** dev-dependencies at all; tests 12–14 are in `control_channel.rs`/`rt_guard.rs` and hold no `LiveEngine` at all (test 14 is a text scan of source files); tests 16–18, which actually hold `LiveEngine<NullStream>`, are never mentioned. **(b)** Test 8 sits in `crates/spectre-audio/tests/bridge_plan.rs` but builds from `AppModel::prototype()` — `spectre-app` is not a dependency of `spectre-audio` in any form (`[dev-dependencies]` is `spectre-offline` alone). **(c)** Tests 16 and 18 assert `engine.health().parameters_pending`, a field R4-1's `EngineHealth` does not have and §7.2 does not add. **(d)** Tests 16 and 18 pass `Some(&engine)` where `engine: LiveEngine<NullStream>` into `apply_parameter_edit(…, Option<&LiveEngine>, …)`, which defaults to `LiveEngine<dyn AudioStream>`; R4-1's field is `stream: Box<S>`, so the two types have different layouts and the reference does not coerce. **(e)** Tests 2–3 pass `&str` literals where §4.3 declares `DeviceParameterKey`, which has no `From<&str>` and no `Deref` (`parameter.rs:13–27`). No test is *unfailable*, so 1G's explicit 0 does not fire — but four of eighteen do not compile or do not run as specified, and two of those are the acceptance-criterion tests. | See Priority 1 items 1–4 |

**Lens average:** 2.714 · **Lens pass:** Yes (avg ≥ 2.00, one 1, no 0s)

---

## Lens 2: DAW Workflow Depth (weight: 25%)

| Criterion | Score | Evidence from spec | Remediation needed |
|---|---|---|---|
| 2A. Loop-first core loop | 3 | §3.2's primary flow keeps selection, lens, and transport untouched — the edit path is the existing `edits` collection at `main.rs:380–382` applied at `:389–397`, with only the downstream call changing. §3.6 E1 makes editing with no engine the *normal* state with non-error copy, which is what keeps sketching usable today. §3.2's fast-drag branch relies on the lane's own coalescing rather than adding a throttle, explicitly because a throttle would be an unevidenced numeric bound. | — |
| 2B. Linked lenses | 3 | §4.4's "there is exactly one value of record, and it is the model's" is precisely what this criterion asks for, and it is made structural rather than aspirational: the render side holds a published *copy* and never originates or returns a value. §3.1 confirms Build's read-only values (`main.rs:297–310`) keep tracking the same model. The spec ties this to `vision.md:37` and to R4-8's bounce equivalence. `ParameterEdit` carries two `ObjectId`s and an `f32` and names nothing from `spectre-audio`, so `AppModel` stays renderer-neutral. | — |
| 2C. Modulation visibility | 3 | Defers rather than diverges, correctly. PROD-002 (`requirements-ledger.md:63`) needs automated/overridden states with a restore action; R4-2 introduces no automation, so §3.3 states there is no automated state to distinguish, keeps `Reset` as return-to-default rather than restore-automation, and makes it a normative copy rule in §6.3. It does not foreclose PROD-002, because one value lives in one place. | — |
| 2D. Keyboard-first, calm UI | 3 | §3.4: "**none are added, ratified, or documented**," naming `product-implications.md` §"Prohibited conclusions at current evidence level" — verified at `:90–102`, with "a default shortcut map" at `:96`. The pre-existing `Space`/`1`–`4` bindings at `main.rs:425–437` are named as scaffolding and neither extended nor blessed. Remappability and context-scoping are stated as future requirements per `vision.md:41`, not fixed here. | — |
| 2E. Convergent-pattern grounding | 3 | Appendix A takes a per-record posture instead of a blanket one: converges on `OBS-VCV-VOLT-006` and on descriptor-declared ranges from `OBS-PP-ARCH-004`, **diverges** on that record's numbers under decision 16, and **defers** on the `OBS-AB12-AUTO-004` / `OBS-BW53-AUTO-002` convergence. Each posture is argued rather than asserted. | — |
| 2F. Differentiation | 3 | §Appendix A's final gap explicitly refuses to claim the `params pending` counter as superior on evidence, deriving it instead from `vision.md:40` and the alpha release bar at `:48`. No parity-as-completeness claim appears anywhere. | — |
| 2G. Benchmark evidence discipline | 3 | **All seven cited `OBS-` records opened; every one says what the spec claims** — `OBS-VCV-VOLT-006` (`synth-modular-observations.md:50`), `OBS-AB12-AUTO-004` (`ableton-live-observations.md:127`, §25.4 as cited), `OBS-BW53-AUTO-002` (`bitwig-studio-observations.md:37`), `OBS-AB12-AUTO-003` (`ableton-live-observations.md:126`, §25.2.1, Touch/Latch verbatim), `OBS-PP-ARCH-003` and `OBS-PP-ARCH-004` (`:27`, `:28`), `OBS-SR2-CPU-001` (`:54`, support article 51). Serum 2 is held to exactly its two accepted records with the wider corpus correctly untouched — `serum-2-observations.md` carries a "⚠ QUARANTINED — NOT ACCEPTED RESEARCH" banner and the spec cites nothing from it. Logic Pro is asserted nowhere and named as a zero. The header and Appendix A both state that **no** benchmark has a citable record about internal control→render transport, and Appendix A declares `OBS-AB12-AUTO-003` **not sufficient** to settle Q5 rather than stretching it. **Correction to the prior scorecard:** it verified five OBS IDs and reported five; the spec cites seven. The two it missed also verify. | — |

**Lens average:** 3.000 · **Lens pass:** Yes

---

## Lens 3: Product Identity & Scope Discipline (weight: 20%)

| Criterion | Score | Evidence from spec | Remediation needed |
|---|---|---|---|
| 3A. Milestone fit | 3 | §6.4 quotes `docs/status/NEXT.md` slice 2 and the quote is verbatim against `NEXT.md:24`, including the acceptance clause about `parameters_pending`. The R4 exit-evidence row at `current-milestone.md:82` and the inherited-debt item at `:25` ("A playable alpha needs this closed") are both quoted exactly. Sample-accurate automation is left to PROD-002 at R9 — which is the architecture contract's own disposition (`dsp-device-io.md`, "Sample-accurate automation is PROD-002 at R9 and will extend this seam rather than replace it"). | — |
| 3B. Non-goal respect | 3 | §6.4 checks each non-goal by name. Verified against `current-milestone.md:77`: automation and modulation are R5+, and §3.3 excludes both explicitly. No hosting, no plugin-format authoring, no cross-DAW formats, no cloud, no video. | — |
| 3C. Deliberately small first devices | 3 | No device added, none grown. The four existing devices gain one method each that assigns to fields they already have. The spec **declines** the one change that would grow a device — smoothing on `Gain` — and routes it to R4-6 under decision 15 (`decision-gates.md:39`) with its evidence cost priced in §4.3. That refusal is the strongest scope-discipline act in the spec. | — |
| 3D. Originality | 3 | One numeric bound, `MAX_PARAMETER_TARGETS`, with a rationale computed from this codebase's own struct sizes — and the arithmetic is exactly right: 16 B `ParameterTarget` (two `ObjectId(u64)`, `id.rs:14`) + 8 B `ParameterSlot` (two `AtomicU32`, `control.rs:67–70`) + 4 B `seen` (`:87`) + 40 B `ParameterRoute` = 68 B, × 1,024 = **69,632 B** exactly as stated. The bound's justification traces to `ParameterReader::drain` walking every slot per block at `control.rs:127`, which is real. Appendix A explicitly refuses Phase Plant's numbers under decision 16. | — |
| 3E. Platform commitment | 3 | §4.6 is right on the merits: no `#[cfg]` anywhere in the change list, and the denormal-portability argument is verified rather than assumed — `contain_channel` (`graph/lib.rs:401–414`) really is a software flush, so it behaves identically on x86-64 and aarch64. **No Linux claim is authorized**; decision 23's row still reads "not run" (`current-milestone.md:113`), and §5.4's manual protocol is owed on both platforms with only macOS recordable. The risk is correctly framed as inherited from decision 23, not created here. | — |
| 3F. Accessibility trajectory | 3 | §3.7 adds no icon-only control, no color-only state, no custom-painted widget; delivery state is a sentence plus two named counters. The eframe accessibility gap is disclosed from source rather than hidden — `crates/spectre-app/Cargo.toml` really does build eframe with `default-features = false` and only `default_fonts`/`glow`. Focus order is preserved because no interactive element is added. Decision 17's R4 audit is named, not claimed as delivered. | — |

**Lens average:** 3.000 · **Lens pass:** Yes
**Auto-fail triggered:** No

---

## Lens 4: Truthfulness & Evidence (weight: 20%)

| Criterion | Score | Evidence from spec | Remediation needed |
|---|---|---|---|
| 4A. Current-state accuracy | **2** | §7.1 itself is **clean in both directions** — I could not falsify a single row. Spot-verified: `AudioProcessor` has exactly `io` and `process` (`io.rs:163–172`); `CompiledPlan` really exposes no parameter method; `control_channel` really has no target cap (only the duplicate scan at `control.rs:200–204`); `Gain::set_gain`/`Saturator::set_drive`/`set_mix` really have **zero callers** anywhere under `crates/` (grep returns only their three declarations); `PulseInstrument` (`source.rs:113–158`) and `ToneSource` (`:50–74`) really have no setters; `crates/spectre-app/Cargo.toml`'s dependency list is exactly the six named, with no `spectre-audio`; `ENGINE OFFLINE` at `main.rs:79` and `CPU —` at `:80` really are hard-coded. **But the prior scorecard's "zero false" claim does not survive a wider read.** Three citations do not land on the claimed element: §4.3's "`new` … is called by `bridge_plan.rs:33`" — line 33 is `fn fixture_plan(…)`, and the real call sites are `:99`, `:128`, `:148`, `:172`, `:215`, `:234` (six, not one); §4.3's `phase` citation `source.rs:52` is `level: f32`, with `phase: f64` at `:53`; §5.2's `bridge_plan.rs:94–124` spans a test that ends at `:121`. And §4.7 carries one figure that contradicts the spec's own change list (see remediation 5) and one mischaracterized memory ordering (remediation 6). None describes code that does not exist, so AF-2 does not fire and the feasibility rule's specific trigger — a §7.1 that misdescribes current state — is passed. | See Priority 2 items 5–7 |
| 4B. Status vocabulary | 3 | Header is `proposed`. §7.1 partitions implemented / prototyped / absent / gated using the vocabulary correctly, and the `implemented` / `verified` distinction is respected explicitly: §7.2 states status moves to `implemented`, **not** `verified`, until §5.4's manual protocol and a hardware re-run pass. That is the distinction stated in the exact terms `docs/README.md` requires. | — |
| 4C. Traceability | 3 | Essentially every normative claim carries a requirement ID, decision row, `OBS-` ID, or source path, and the structure resolves. Verified: `requirements-ledger.md:64` is PROD-003 and reads "recorded **in this ledger**"; decision rows 1, 15, 16, 17, 19, 21, 22, 23 all say what the spec says, including decision 22's "option (a) … Implementation lands at R4 following the CORE-004 precedent" quoted essentially verbatim. **On the pattern that rotted three sibling specs, this spec is the best in the run:** it carries **zero** line-pinned citations into `gauntlet-output/manifest.md`, `decisions-needed.md`, or a sibling spec — every such reference is by name or by section (`R4-1 §4.3`), and its one line-pinned `docs/` citation still resolves eight days later. | — |
| 4D. Honest gaps | **2** | §8's eight questions are among the strongest in the run: each names its options, their costs, and a "blocks §X" blast radius, and Q7 goes as far as asking whether the research gap is fillable at all. D-R3 is genuinely escalated rather than decided. **But §7.4 makes one positive claim that is false**: "everything from `AudioProcessor::set_parameter` through `RenderBridge`'s route table and tests 1–14 can be implemented and proved today against the existing crates and the null backend, with no device and no app wiring." Test 8 needs `AppModel::prototype()` from `spectre-app`, i.e. a new dev-dependency edge — which §4.5 separately declares does not exist ("New crate dependencies: none") and §7.2 does not schedule. A blocking-dependency section that understates a blocker is exactly what this criterion grades. | See Priority 1 item 3 |
| 4E. Evidence commands | 3 | The workspace gate is quoted character-exact against `criteria.md`: `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo test --locked --workspace`. Every feature-scoped target exists on disk — `spectre-dsp/tests/devices.rs`, `spectre-graph/tests/graph_plan.rs`, `spectre-audio/tests/{control_channel,bridge_plan,rt_guard,lifecycle_health}.rs`, `spectre-app/tests/app_model.rs` — and §5's lead is correctly hedged from the first sentence: "All of them run today except `cargo test -p spectre-app --test live_engine`, whose file **R4-1** creates." §5.2 makes `rt_guard` a required post-edit gate and says outright that "§4.1 is not itself the evidence." | — |
| 4F. No fake surfaces | 3 | §1.2 states in bold that `./spectre` produces no sound of any kind today and backs it with four independent facts, each verified. §3.6 E1 makes the no-engine case *normal* with non-error copy. §3.3's three-state Shape header exists specifically so that R4-1's "live application does not exist" string is not left in place after R4-2 lands — a false surface in the opposite direction — and §6.3 makes both that and the never-suppress-`pending` rule normative. | — |

**Lens average:** 2.667 · **Lens pass:** Yes

---

## Auto-fail roll-call

| Rule | Triggered | Finding |
|---|---|---|
| AF-1 — Contradicting accepted authority | **No** | The spec implements `dsp-device-io.md` §"Runtime parameter seam" point for point — I compared its §4.3 against the contract's six accepted bullets and each maps. Where it meets a genuine conflict with accepted authority, it flags and routes: `dsp-device-io.md:94` says "`Gain` already smooths" and `:104` says "click-resistant smoothing", while the shipped struct is `pub struct Gain { gain: f32 }` (`effect.rs:29–31`) multiplied directly at `:62–71` with `set_gain` assigning immediately at `:45–47`. **D-R3 is still routed, not resolved**: §8 Q1 lays out three resolutions with costs and refuses to choose, §7.2 schedules the `decisions-needed.md` entry, and that entry exists at `decisions-needed.md:146`. §7.2 further forbids editing the `dsp-device-io.md` sentences until Q1 is answered. This is the route AF-1 requires. |
| AF-2 — Unbacked implementation claims | **No** | §7.1 distinguishes implemented / prototyped / absent / gated, and no row describes code that is not there. Both directions were checked, including the claimed-absent ones: `CompiledPlan` genuinely has no parameter method, `control_channel` genuinely has no cap, and the three inherent setters genuinely have zero callers. Three citations elsewhere point at the wrong line (4A) but none names code that does not exist. |
| AF-3 — Realtime discipline violation | **No** | The added render-side sequence is one `binary_search_by` over `Box<[ParameterRoute]>`, the linear step scan `process` already performs at `graph/lib.rs:466–470`, one dynamic dispatch, and ≤ 2 `&'static str` comparisons plus a clamp. **The no-panic argument was traced end to end and holds:** `f64::clamp` panics only when `min > max`; `ParamSpec::new` rejects `max <= min` at `param.rs:77–79` and is `pub const fn` at `:68`; all four descriptor tables are `const` (`source.rs:27`, `:39`; `effect.rs:19`, `:22`), so an ill-ordered spec is a compile-time failure. The counters accumulate as plain locals and publish two relaxed adds per block, matching the pattern `CompiledPlan` already uses (comment verified verbatim at `graph/lib.rs:387–388`). §4.4 adds no reclaim-lane traffic. |
| AF-4 — Borrowed numeric limits | **No** | One bound. Its rationale is derived from this codebase's own struct sizes and arithmetically exact, and — decisively — §7.2's modified-files list **schedules the row**, not merely the argument: "`docs/01-requirements/requirements-ledger.md` — one rationale row for `MAX_PARAMETER_TARGETS`, carrying §4.2's text," citing PROD-003 at `requirements-ledger.md:64`, which I opened and which does require the rationale "in this ledger." Prose alone would not have discharged this; the scheduled row does. |
| AF-5 — Conclusions the evidence cannot support | **No** | Checked against all nine prohibited conclusions at `product-implications.md:94–102`. No default shortcut map (§3.4 refuses one by name and cites the section). No gesture-count or time budget — §3.2 derives 256/48,000 = 5.33 ms as a block period and then states outright that "**No latency threshold is asserted anywhere in this spec**," citing the monitoring-latency prohibition at `:99`. No promoted workflow archetype. No native-device list, no modulation limit, no interface model list, no platform/backend order, no frequency scores. §4.7 declines a CPU budget threshold on the grounds that one hardware measurement cannot justify one. |
| AF-6 — Optimistic language | **No** | The header leads with two uncomfortable facts. §1.2 states in bold that `./spectre` makes no sound. §3.2 volunteers that a large gain move **will** click. §4.6 refuses a Linux claim outright. §6.3's audit is answered honestly rather than checked off. |

---

## Feasibility Check

| Check | Status | Notes |
|---|---|---|
| Types/models exist or are clearly specified | ✓ | Every existing type built on is real and at the cited line. `AudioProcessor` is `pub trait AudioProcessor: Send` (`io.rs:163`), so the new required method lands on a `Send`-bounded trait. `DspParameter.key` is a **public** field (`parameter.rs:31`), so `GAIN_PARAMETERS[0].key` resolves from other crates. `ParameterTarget` derives `Ord` at `control.rs:22`. `DeviceParameterSnapshot` (`parameter.rs:81–126`) carries exactly the four identity fields the route join needs. |
| API/interface changes feasible with current architecture | ✓ | `spectre-dsp/src/lib.rs` already re-exports from `io`, so adding `ParameterError` is a one-line change. Exactly **seven** `impl AudioProcessor` sites exist, at exactly the seven lines §5.2 names — so a required method really is a compile-time forcing function. `spectre-audio/src/lib.rs`'s module list accepts `pub mod route;` cleanly. |
| Views/screens fit current navigation pattern | ✓ | No new screen. Shape's structure (`main.rs:336–398`), the transport cluster (`:78–80`), and the inspector status line (`:210`) all exist as described, down to the 520 px row minimum at `:362` and the 360 × 26 slider at `:370`. |
| Dependencies are available and version-compatible | **✗** | §4.5 says "New crate dependencies: none," but test 8 requires `spectre-app` reachable from `crates/spectre-audio/tests/`. `crates/spectre-audio/Cargo.toml`'s `[dev-dependencies]` is `spectre-offline` alone, and `spectre-app` depends on `spectre-audio` after R4-1, so this is a dev-dependency cycle that must be declared. Recoverable in one line, but undeclared. |
| Platform/renderer requirements are realistic | ✓ | No `#[cfg]`, no host API, no driver. Denormal behavior is software (`graph/lib.rs:401–414`) and therefore genuinely identical across targets. Edition 2021 confirmed at `Cargo.toml:19`. |
| Test strategy is executable with current infrastructure | **✗** | Four of eighteen tests do not run as written: test 8 (crate dependency), tests 16 and 18 (a `LiveEngine` type that does not coerce, plus a nonexistent `EngineHealth` field), and tests 2–3 (`&str` where `DeviceParameterKey` is required). Every target *file* exists or is correctly attributed to R4-1. Tests 6, 7, 9, 11, and 13 also need the Gain node's `NodeId`, which the existing `fixture_plan`/`fixture` helpers do not return — recoverable from `FIXTURE_SEED` (`bridge_plan.rs:30`) but unmentioned. |
| Performance budget is realistic for target hardware | ✓ | Recomputed every figure. `ParameterRoute` = 16 + 8 + 16 = **40 B** ✓ (no padding; max align 8). Four routes = **160 B** ✓. Lane slots 4 × (16 + 8 + 4) = **112 B** ✓. §4.2's cap figure 68 × 1,024 = **69,632 B** ✓ exact. Block period 256/48,000 = **5.33 ms** ✓. Two figures are wrong: the counter line item (16 B for one new counter) and the "relaxed" characterization of `Acquire` loads. The headline "under 300 B" survives either way. |
| No undeclared dependency on unbuilt features | **✗** | R4-1 is declared as a hard blocker and its API was cross-checked — `EngineUnavailable::Control`, `EngineState`, `EngineParts`, `LiveEngine<S: ?Sized = dyn AudioStream>`, `build_engine_parts`, `from_open_stream`, `stream_mut`, and the `live-audio` feature all exist in R4-1's spec. **But `EngineHealth.parameters_pending` does not.** R4-1's `EngineHealth` has exactly nine fields (`R4-1-live-audio-wiring.md:431–441`, corroborated by its own §4.7 "nine relaxed atomic loads"), none of them a parameter counter, and R4-2's §7.2 schedules only `parameters_applied`. §3.3's counter data source and test 16's acceptance assertion both depend on a field nothing creates. |

**Feasibility verdict:** **Feasible with caveats.** No defect found is architectural; all four are declaration or signature fixes confined to §4.3, §4.5, §5.1, and §7.2. The design itself — the trait method, the route table, the plan method, the publication rule — is sound and implementable exactly as described.

---

## Composite Score

| Lens | Average | Weight | Weighted |
|---|---|---|---|
| 1 — Realtime & Correctness | 2.714 | 35% | 0.950 |
| 2 — DAW Workflow Depth | 3.000 | 25% | 0.750 |
| 3 — Product Identity & Scope Discipline | 3.000 | 20% | 0.600 |
| 4 — Truthfulness & Evidence | 2.667 | 20% | 0.533 |
| **Composite** | | | **2.833** |

**Pass conditions (from `criteria.md`, binding):**
- [x] Composite ≥ **2.30** — 2.833
- [x] Every lens average ≥ **2.00** — 2.714 / 3.000 / 3.000 / 2.667
- [x] No criterion scores 0 — none
- [x] At most **two** criteria score 1 — exactly one (1G)
- [x] All auto-fail rules pass — AF-1 through AF-6 all clear
- [x] Feasibility satisfies `criteria.md` — the feasibility rule's stated trigger is "a §7.1 that misdescribes current state"; §7.1 was read row by row against source and does not
- [x] Reviewer personally opened every source path graded above — roughly sixty citations at their exact lines, plus all seven `OBS-` records; no `cargo` command was executed, and `criteria.md` does not require one for a spec describing unwritten code

**All conditions met:** Yes → **PASS at 2.833**

---

## Disagreement with the prior scorecard, stated explicitly

The prior scorecard scored **3.000** with nine criteria `[sampled]`. This review reads every
section and lowers the composite to **2.833**. Four specific disagreements, each with its
source:

1. **1G: 3 → 1.** The prior scorecard's evidence line reads "Fifteen tests, each with setup,
   assertion, and a named edge case that can genuinely fail," and singles out §5.1's handle
   preamble as "the exact fix R4-1 needed at iteration 2, present here at iteration 1."
   There are **eighteen** numbered tests, and the preamble is a fifteen-test map: it is
   false for tests 4–5, 12, 13, and 14, and silent on 16–18. Four tests do not compile or
   run as written. The artifact the prior scorecard praised is the artifact that fails.

2. **4A: 3 → 2.** The prior scorecard states "**Forty-six citations opened individually;
   zero false.**" Three do not land on the claimed element: `bridge_plan.rs:33` is
   `fn fixture_plan(…)`, not a `RenderBridge::new` call site (those are `:99`, `:128`,
   `:148`, `:172`, `:215`, `:234`); `source.rs:52` is `level: f32`, not `phase: f64` (`:53`);
   `bridge_plan.rs:94–124` overshoots a test ending at `:121`. §4.7 additionally contains a
   figure contradicting the spec's own §4.1/§4.2/§7.2 and calls `Acquire` loads relaxed.

3. **4D: 3 → 2.** §7.4's claim that "tests 1–14 can be implemented and proved today …
   with no device and no app wiring" is false for test 8, and contradicts §4.5's "New crate
   dependencies: none." The prior scorecard did not read §7.4.

4. **Feasibility: "Feasible. Every path checked contains its claim" → "Feasible with
   caveats."** Three of the template's eight feasibility rows fail. The prior scorecard
   filled only five rows and did not include a dependencies row or an unbuilt-features row
   that reached R4-1's `EngineHealth` definition.

Two findings **confirm and strengthen** the prior scorecard rather than correcting it: the
`rt_guard` argument and the no-panic argument both hold under full independent re-derivation,
and the benchmark discipline is better than reported — the spec cites seven `OBS-` records,
not five, and all seven verify.

---

## Remediation Brief

### Priority 1 — Must fix before implementation

1. **Add `parameters_pending` to `EngineHealth` in §7.2.** §3.3 names
   `EngineHealth.parameters_pending` as the `params pending` counter's data source, §1.3
   states the success signal in terms of it, and tests 16 and 18 assert
   `engine.health().parameters_pending == 0` — the assertion §1.3 calls "the one that
   encodes the acceptance criterion." R4-1's `EngineHealth` has exactly nine fields
   (`gauntlet-output/specs/R4-1-live-audio-wiring.md:431–441`; its own §4.7 confirms "nine
   relaxed atomic loads") and none is a parameter counter. §7.2's `engine.rs` entry
   schedules only `EngineHealth.parameters_applied`. Add `parameters_pending` to that entry,
   and state that it reads `BridgeTelemetry::parameters_pending()` (`bridge.rs:82–85`),
   which already exists.

2. **Make `apply_parameter_edit` generic over the stream type in §4.3.** As declared,
   `engine: Option<&LiveEngine>` resolves to `Option<&LiveEngine<dyn spectre_audio::AudioStream>>`
   via R4-1's default parameter (`R4-1-live-audio-wiring.md:472`). Tests 16 and 18 hold a
   `LiveEngine<NullStream>` and pass `Some(&engine)`. R4-1's field is `stream: Box<S>`
   (`:483`), so `LiveEngine<NullStream>` and `LiveEngine<dyn AudioStream>` have different
   layouts and `&LiveEngine<NullStream>` does not coerce. Change the signature to
   `pub fn apply_parameter_edit<S: spectre_audio::AudioStream + ?Sized>(model: &mut AppModel,
   engine: Option<&LiveEngine<S>>, …)` — the same fix R4-1 applied to `LiveEngine`'s own
   impl block (`:546`) and which §4.3 already applied correctly to `LiveEngine::send_parameter`.

3. **Resolve test 8's crate dependency, and correct §4.5 and §7.4.** Test 8 lives in
   `crates/spectre-audio/tests/bridge_plan.rs` and builds from
   `AppModel::prototype().device_parameter_snapshot()?`. `crates/spectre-audio/Cargo.toml`'s
   `[dev-dependencies]` is `spectre-offline` alone, and `spectre-app` depends on
   `spectre-audio` after R4-1. Either (a) move test 8 to
   `crates/spectre-app/tests/live_engine.rs` where `AppModel` and `build_parameter_wiring`
   are both in scope — the cleaner option, since the route table is built by app-side code —
   or (b) declare `spectre-app` as a `spectre-audio` dev-dependency, add
   `crates/spectre-audio/Cargo.toml` to §7.2's modified-files list, and correct §4.5's
   "New crate dependencies: none." Either way, §7.4's "tests 1–14 … with no device and no
   app wiring" must be corrected.

4. **Fix §5.1's handle preamble, which is a fifteen-test map over an eighteen-test list.**
   Concretely: tests 4–5 are in `crates/spectre-graph/tests/graph_plan.rs` and drive
   `CompiledPlan::process` directly — they cannot build a `RenderBridge`, because
   `spectre-audio` depends on `spectre-graph` and not the reverse, and
   `crates/spectre-graph/Cargo.toml` declares no dev-dependencies through which it could. Tests 6, 7, 9, and 11 are
   the ones that build a bridge. Test 12 is in `control_channel.rs` and tests 13–14 in
   `rt_guard.rs`; none holds a `LiveEngine`, and test 14 reads source files as text. Tests
   **16–18**, not 12–14, are the ones holding `LiveEngine<NullStream>` via `from_open_stream`
   and pumping through `stream_mut()`. Rewrite the paragraph against the final list.

### Priority 2 — Should fix for quality

5. **Correct §4.7's counter line item.** It reads "Two `AtomicU64` counters on
   `BridgeTelemetry`: **16 B**," but §4.1, §4.2, and §7.2 all add exactly one
   (`parameters_applied`); `parameters_pending` already exists at `bridge.rs:29` with its
   accessor at `:82–85`. Added memory is **8 B**. The headline "under 300 B" survives (280 B,
   or 296 B if the unlisted 16-byte `Box<[ParameterRoute]>` fat pointer on `RenderBridge` is
   counted), but §4.7 opens with "All figures are computed from source, not estimated," and
   this one is not.

6. **Correct §4.7's "relaxed" characterization of the drain loads.** §4.7 states twice that
   the steady-state added cost is "four relaxed atomic loads." `ParameterReader::drain` reads
   `slot.version.load(Ordering::Acquire)` at `control.rs:129` — a line §5.1 test 6 cites
   correctly as `control.rs:129–131`. On aarch64 an Acquire load is `ldar`, not `ldr`, so the
   mischaracterization runs against §4.6's own x86-64/aarch64 portability argument. The UI-cost
   bullet's "two relaxed atomic loads" **is** correct (`bridge.rs:84`).

7. **Repair three off-target line citations.** §4.3's `bridge_plan.rs:33` should be the six
   real `RenderBridge::new` call sites at `:99`, `:128`, `:148`, `:172`, `:215`, `:234` — the
   correction strengthens the argument, since six tests compile unchanged rather than one.
   §4.3's `source.rs:52` for `phase` should be `:53` (`:52` is `level: f32`); the `:96` half is
   correct. §5.2's `bridge_plan.rs:94–124` should be `:94–121`.

8. **Use `DeviceParameterKey` in tests 2 and 3.** `Gain::set_parameter("gain", 99.0)` and
   `set_parameter("frequency_hz", 880.0)` pass `&str` where §4.3 declares
   `key: DeviceParameterKey`. That type is `pub struct DeviceParameterKey(&'static str)`
   (`parameter.rs:13`) with no `From<&str>` and no `Deref` — only
   `DeviceParameterKey::new(&'static str) -> Option<Self>` at `:16`. Use the form test 4 and
   §4.3's `Gain` impl already use: `GAIN_PARAMETERS[0].key`, `TONE_PARAMETERS[0].key`.

9. **Schedule the existing test whose premise this slice invalidates.**
   `crates/spectre-audio/tests/bridge_plan.rs:205–225`,
   `parameter_changes_are_counted_while_the_live_seam_is_missing`, will still **pass** after
   R4-2 (`RenderBridge::new` delegates to `ParameterRoutes::empty()`, so the target is
   unrouted and counted) — but its name and its comment ("Applying it to a live processor
   needs an AudioProcessor parameter seam that the accepted contract lacks") become false
   statements about the codebase, and it duplicates new test 9. §5.2 names only
   `bridge_output_matches_the_offline_render_of_identical_input` among existing tests, and
   §7.2's `bridge_plan.rs` entry lists additions only. Rename and re-comment it, or retire it
   in favor of test 9.

### Priority 3 — Consider for excellence

10. **Drop test 11's vacuous clause.** "assert … that `denormals_flushed()` is a finite
    count" cannot fail: the accessor returns `u64` (`bridge.rs:93–95`). The test's other
    assertions are genuinely discriminating, so 1G's zero does not apply — but replace this
    clause with a bound that can break, e.g. that `denormals_flushed()` did not increase
    across a block whose output is exactly zero.

11. **Say how tests 6, 7, 9, 11, and 13 obtain the Gain node's `NodeId`.** Both fixture
    helpers return only the note node — `bridge_plan.rs`'s `fixture_plan` at `:33` and
    `rt_guard.rs`'s `fixture` — while routing requires the Gain node. The IDs are
    deterministic from `FIXTURE_SEED` (`bridge_plan.rs:30`, allocated in order at `:34–37`),
    so this is recoverable, but §7.2 lists both files as gaining tests only. Either widen the
    helpers' return type or state the seed-regeneration approach.

12. **Assert `RT_MODULES`'s length against the callback-reachable module set.** §7.2
    correctly grows the array to `[&str; 5]`. An assertion that its length matches the number
    of callback-reachable `src` modules would make the *next* slice that adds one fail loudly
    — the exact hole this spec spotted on its own.

13. **Note test 12's construction cost.** `control_channel`'s duplicate scan is O(n²)
    (`control.rs:200–204`), so the "cap value itself constructs successfully" half of test 12
    performs roughly 524,000 comparisons at 1,024 targets. Fast enough, but worth one clause
    so a future reader does not mistake it for a hang.

---

**End of scorecard.**
