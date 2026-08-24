<!--
Author: Jeff
Date: 2026-08-15
Description: Independent blind verification scorecard for the R4-7 project-persistence spec, iteration 1
Notes: Second reviewer. The first R4-7 scorecard was written by the orchestrating session and marked
  thirteen of twenty-six criteria [sampled]; this one grades from scratch and does not inherit its
  conclusions. Where the two disagree, the disagreement is stated with the source that settles it.
-->

# Scorecard: Project Persistence (independent review)

**Feature ID:** `R4-7` (`project-persistence`)
**Spec file:** `gauntlet-output/specs/R4-7-project-persistence.md` (1,452 lines)
**Reviewer agent:** independent blind verification, R4-7 iteration 1, second reviewer
**Date:** 2026-08-15
**Spec iteration reviewed:** 1
**Companion scorecard:** `R4-7-project-persistence-scorecard.md` (orchestrating session, PASS / 3.000, thirteen criteria `[sampled]`)

---

## Verdict: FAIL

**Summary:** This is a strong spec on every axis the first scorecard sampled — the platform
treatment of `fsync` versus `F_FULLFSYNC` is real rather than gestural, the
`implemented`/`verified` line holds in seven independent places rather than one, ⌘S is
explicitly refused, cross-DAW compatibility appears nowhere, and all ten cited `OBS-` records
resolve and say exactly what the spec claims. It fails on the one condition the first
scorecard could not test by sampling: **the schema-2 change's blast radius is materially
misdescribed.** Adding four fields to `ProjectDoc` breaks two out-of-crate struct literals —
one of them in a file §7.2 lists under *"Deliberately not modified"* — the constant bump
breaks an existing passing assertion in another crate that §7.2 declares *"unaffected,"* and
the "version stamp on save" that §7.2 and integration test I9 both depend on exists in no
component of the declared API. The single fix that unblocks the rest: rewrite §7.2's delta
against the four files the schema change actually touches.

**Scope of this review — stated honestly.** Read **in full**: the entire spec (all 1,452
lines), `criteria.md`, the scorecard template, `docs/03-architecture/project-persistence.md`
(all 193 lines), and every Rust source file the spec cites, at the cited lines. Verified by
direct `Read`/`grep`: all seven crate manifests; `spectre-project/src/lib.rs` and
`command.rs` complete; `spectre-core/src/id.rs`; `spectre-dsp/src/parameter.rs`;
`spectre-app/src/lib.rs` and `main.rs` at every cited range; `spectre-offline/src/lib.rs` at
every cited range; `tests/project_codec.rs`, `tests/command_history.rs`,
`tests/fixtures/r1-canonical.json`, `tests/smoke_cli.rs`, `tests/rt_guard.rs`, the `spectre`
launcher; every cited row of `requirements-ledger.md`, `decision-gates.md`, `vision.md`,
`docs/README.md`, `rebuild-roadmap.md`, `current-milestone.md`, `NEXT.md`, `STATUS.md`; and
all ten `OBS-` records plus the corpus counts and the Serum 2 quarantine banner.
**Not executed:** `cargo build`/`cargo test` were not run, so the two compile-break findings
(F1) are derived by reading the struct definitions and every construction site rather than
from a compiler. Every other finding is settled by file contents. **Nothing in this scorecard
is inherited from the companion scorecard.**

---

## Lens 1: Realtime & Correctness (weight: 35%)

| Criterion | Score | Evidence from spec | Remediation needed |
|---|---|---|---|
| 1A. Callback-path discipline | 3 | §4.1 proves the boundary structurally rather than by convention: the four render-path manifests are quoted and **all four verify exactly** — `spectre-audio/Cargo.toml:17–21` (cpal, core, dsp, graph), `spectre-graph:12–14`, `spectre-dsp:12–13`, `spectre-core:12–14`, none naming `spectre-project`. U12 turns that into a test. The app-thread argument in §4.4 states precisely what a blocked UI thread does and does not do to the RT-002 lanes. AF-3 clean. | — |
| 1B. Control↔render communication | 3 | §4.4 argues not-applicable-by-design without dodging: nothing is *sent* during a save, nothing is dropped because the sender is not called, and the lanes are neither drained faster nor slower. §3.2 step 4 applies `TransportCommand::Stop` before the loaded transport goes live — verified at `spectre-core/src/transport.rs:86`, exact. | — |
| 1C. Numerical containment | 3 | §4.2 rule 4 rejects non-finite parameter values *before* encoding because JSON cannot represent them, explicitly mirroring RT-003's contain-at-the-boundary posture and labelling it as analogy, not as an RT claim. The division of labour is verified: descriptor-range rejection lives in `spectre-offline/src/lib.rs:87–108` and `:110–120` (**both exact**), clamping in `spectre-dsp/src/parameter.rs:89–103` (exact). U17/U18 cover NaN, ∞, −0.0 and subnormals. | — |
| 1D. Determinism | 3 | I15 reuses the **existing** FNV-1a hash walk via `spectre_offline::render_app_snapshot` — verified at `spectre-offline/src/lib.rs:306`, exact, with `RenderReport.hash` at `:33` — rather than inventing a comparison, which is what 1D demands. The nonzero-`peak` guard defeats vacuous agreement, and I confirmed it holds: prototype defaults are pulse level `0.2`, gain `1.0`, drive `1.0`, mix `1.0`, all nonzero. | — |
| 1E. Graph and plan contract | 3 | Nothing in the graph/plan split is touched; §7.2's "Deliberately not modified" names `spectre-audio`, `spectre-graph`, `spectre-dsp` and every module `rt_guard.rs` scans. No plan recompilation is proposed. | — |
| 1F. Failure behavior | 3 | §4.3's step table gives each of the nine steps a stated purpose and an exact failure target-state, and it matches the accepted contract's failure table (`project-persistence.md:137–146`) row for row. `ReplacedDurabilityUncertain` is never reported as success (§3.6, §6.3), the project stays dirty (§4.4), and no second replacement is attempted. Fail-closed throughout. | — |
| 1G. Test specification | **2** | Commands are real (package names verified) and most assertions are genuinely falsifiable — U7's "no `remove` of the **destination** at any point", U9's cleanup-does-not-mask, U11's ordering pin, and the deliberate refusal to write an unfailable `Encode` test are all 1G-aware. Three defects: **(a)** I9 asserts `schema_version == SCHEMA_VERSION` (2) after load→save→load of the v1 fixture, but §4.3's algorithm has no stamping step, `save_project_atomic` takes `&ProjectEnvelope` (immutable), and step 2 is `to_bytes(snapshot)` unchanged — so the declared API writes `1` and **I9 cannot pass**; **(b)** I11's device/parameter half cannot fail (see F5); **(c)** §1.3 claims injection at "every one of the **seven** `SaveStage` values" while §5.1 specifies six and says so. | Give the version stamp an owner or rewrite I9; mutate a parameter value in I11; reconcile §1.3 with §5.1. |

**Lens average:** 2.857 (20/7)
**Lens pass:** Yes — avg ≥ 2.0, zero 1s, zero 0s

---

## Lens 2: DAW Workflow Depth (weight: 25%)

| Criterion | Score | Evidence from spec | Remediation needed |
|---|---|---|---|
| 2A. Loop-first core loop | 3 | `ViewDoc` (§4.2) persists lens, selected track, and selected device, so reopening restores working context rather than resetting it; §5.4's happy path requires "same tracks, same order, same names, same selected track, same lens". I12 guarantees opening never starts playback. Context survives the round trip, which is what 2A asks. | — |
| 2B. Linked lenses | 3 | §4.4 is explicit: the feature adds **no** field to `AppModel`, save serializes the one model, open replaces the one model, "so no lens can hold a private copy", citing `vision.md:37` — verified exact. Path/dirty/status go on `SpectrePrototype` (`main.rs:17–21`, exact) because "a file path is not project content". No per-lens fork anywhere. | — |
| 2C. Modulation visibility | 3 | No automation or modulation exists to persist, and the spec declines to invent a representation for one. Under criteria.md's own rule, naming the absence is the correct behavior. | — |
| 2D. Keyboard-first, calm UI | 3 | §3.4 names the commands (`project.save`, `project.open`) so a later remappable, context-scoped resolver can bind them, fixes **no** binding, and states in terms that "in particular does not claim ⌘S". Existing `Space` and `1`–`4` bindings verified at `main.rs:425–437`, exact, and left alone. Focus order specified in §3.7. Exactly the shape 2D wants under AF-5. | — |
| 2E. Convergent-pattern grounding | 3 | Appendix A identifies a real convergence — Phase Plant `OBS-PP-UNI-003` ("bend range is saved with the project, not the preset") and Ableton `OBS-AB12-WARP-005` / `OBS-AB12-CLIP-002` (set versus `.asd` sidecar) — **all three verified verbatim** — and states Spectre's R4 position as a deliberate simplification of that line, naming what is excluded (feedback text, arm state, every engine measurement). | — |
| 2F. Differentiation | 3 | The claim is small and correctly bounded: Spectre surfaces the durable/uncertain distinction to the musician and makes it machine-checkable via `TargetState`; the spec explicitly does **not** claim any benchmark fails to. Parity-as-completeness is refused by name — "R4 has no autosave, no recovery, no migration, and no crash evidence, all of which the benchmark products have had for years". | — |
| 2G. Benchmark evidence discipline | 3 | Every `OBS-` ID opened; **all ten resolve and say what the spec claims**, including the two used only as analogy or corroboration. Corpus counts match criteria.md exactly (verified by grep: AB12 **85**, PP **11**, VCV **6**, SR2 **2**). Logic Pro's zero is named. Serum 2 is held to exactly its two records at `synth-modular-observations.md:54–55` (exact), and the wider file is named and refused, citing the quarantine banner at `serum-2-observations.md:10–12` — verified exact. The Ableton hole is named rather than filled, and the reason is checkable (`ableton-live-observations.md:12`, exact). Zero fabricated benchmark claims. | — |

**Lens average:** 3.000
**Lens pass:** Yes

---

## Lens 3: Product Identity & Scope Discipline (weight: 20%)

| Criterion | Score | Evidence from spec | Remediation needed |
|---|---|---|---|
| 3A. Milestone fit | 3 | §6.4 claims this is `NEXT.md` slice 7 verbatim — **it is**, verified at `NEXT.md:29`. `current-milestone.md:26–27` inherited-debt items 3 and 4 assign exactly this split (filesystem implementation at R4, crash qualification at R5, CORE-001 reorder evidence on the first persisted collection) — both verified exact. R5 work is deferred by name in §7.4, not smuggled. | — |
| 3B. Non-goal respect | 3 | **No cross-DAW project or preset compatibility is proposed anywhere.** §6.4 cites `vision.md:56` ("No preset/project compatibility with any other DAW or synth") — verified exact — and the spec earns it: own format, no importer, no exporter, no import/export affordance in §3. The automatic-0 trap for this feature specifically is avoided cleanly. | — |
| 3C. Deliberately small first devices | 3 | No device is added, grown, or given a preset system. §6.4's "four existing parameter values across the three existing device instances" is **exactly right** — verified: `PULSE_PARAMETERS` 1 + `GAIN_PARAMETERS` 1 + `SATURATOR_PARAMETERS` 2 = 4. Decision 15 (`decision-gates.md:39`, exact) untouched. | — |
| 3D. Originality | 3 | Schema, field names, error vocabulary, and algorithm are Spectre's own under decision 3 (`decision-gates.md:27`, exact). The write-temp/fsync/rename/fsync-dir sequence is correctly identified as a POSIX idiom rather than a product's property, and is taken from the accepted contract, not from a tool. The three bounds carry Spectre-derived rationales with ledger rows scheduled — AF-4 clean. | — |
| 3E. Platform commitment | 3 | **Real, not gestural.** §4.6 separates what holds on both platforms (`rename(2)` same-filesystem replacement, `O_CREAT\|O_EXCL`, directory open) from where the two genuinely diverge, and names the divergence concretely: `fsync(2)` flushes to device on Linux but is documented not to flush the drive cache on Apple platforms, where `fcntl(F_FULLFSYNC)` does; directory `fsync` is the defined Linux mechanism but undocumented on APFS and may return `EINVAL`/`ENOTSUP`. It then does the rare thing: **it marks its own `std`-mapping belief as unverified**, because no file in this repository can settle it, and routes it to Q9 instead of asserting it. It shows the contract already absorbs the macOS case via `ReplacedDurabilityUncertain`, refuses to add rows to any qualification table, and §5.4 requires both platforms and both timings. Decision 1 (`decision-gates.md:25`) and decision 23 (`:49`) both verified exact and correctly handled. | — |
| 3F. Accessibility trajectory | 3 | Every new element is a standard labelled widget with a stated focus order and no colour-only state (the dirty signal is the **word** `unsaved`). §3.7 makes **no** screen-reader claim and gives the checkable reason: `crates/spectre-app/Cargo.toml:13` sets `default-features = false` on `eframe` with only `default_fonts` and `glow` — **verified exact**. Decision 17 (`decision-gates.md:41`, exact) is not foreclosed; enabling the feature is routed to Q5. | — |

**Lens average:** 3.000
**Lens pass:** Yes
**Auto-fail triggered:** No — 3B scores 3; AF-4 clean

---

## Lens 4: Truthfulness & Evidence (weight: 20%)

| Criterion | Score | Evidence from spec | Remediation needed |
|---|---|---|---|
| 4A. Current-state accuracy | **2** | The headline claim is **true in both directions** and I verified it as instructed: `crates/spectre-app/Cargo.toml:16` reads `spectre-project = { path = "../spectre-project" }`, and `grep -rn "spectre_project" crates/spectre-app/` returns **no match** across both source files and both test files. The crate inventory, the `AppModel` field list, `IdGen`'s missing state accessor, `validate`'s privacy, the `ProjectDoc` four-field shape, the fixture's 52 lines / 912 bytes / `:2` / `:4` / `:48–51`, and the `main.rs:4` header quote all verify exactly. Against that, §7.1 states three things that are false: **nine** tests in `project_codec.rs` (there are **eight**: `#[test]` at `:21, :52, :60, :69, :85, :100, :112, :124` — and §7.2's own "remaining seven tests are unchanged" only adds up under eight); **eight** command tests in `command_history.rs` (there are **five**); and that the design authority "specifies … the save algorithm's **nine** steps" (`project-persistence.md:118–127` specifies **eight**; the ninth is the spec's own split of `drop(file)`, repeated in §4.6). This sits on the 2/1 boundary: the substance holds, three factual statements do not. | Correct all three counts. |
| 4B. Status vocabulary | 3 | The `implemented`/`verified` line **holds throughout, not in one sentence**. It is stated in the header Notes, in §1's third user story, in §3.6's "Partial, and named" row, in §4.6's dedicated subsection (which quotes `docs/README.md:59–60` verbatim — verified exact), in §5.4's Honesty check, in §6.3's claims audit, in §7.2's STATUS instruction ("move CORE-004 to **`implemented`, not `verified`**"), and in §7.4 ("no product copy may say otherwise"). CORE-004's ledger status is `accepted` and CORE-003's is `verified` — both verified at `requirements-ledger.md:49` and `:48`. This is the criterion the spec handles best. | — |
| 4C. Traceability | **2** | Nearly everything resolves exactly: **all ten** decision rows (1, 3, 4, 8, 13, 14, 15, 16, 17, 23 at `:25, :27, :28, :32, :37, :38, :39, :40, :41, :49`), **all five** ledger rows, **all four** `vision.md` lines, `rebuild-roadmap.md:30`, `NEXT.md:29`, `current-milestone.md` items 3–4, every quotation from the design authority (checked verbatim against all 193 lines), and **all ten** `OBS-` records. **The run's line-pinned-citation rot pattern does not apply here — checked directly: the spec references `criteria.md` five times and every one is by rule name (AF-5, 1G, Lens 2, Lens 3), never by line; it mentions sibling R4 specs twenty times and pins a line in none; and it cites no line in `manifest.md` or `decisions-needed.md` at all. Nothing here can rot.** Against that: `crates/spectre-offline/Cargo.toml:23–24` (cited in §5.2) **does not exist** — the file is 21 lines and the dev-dependency is at `:20–21`; Appendix A's claim that the Ableton file-management chapters are "already identified as unextracted at `ableton-live-observations.md:19`" does not hold (that line lists chapters 2, 10–15, 20–24, 26, 33, 36–40 — chapter 5 is not among them; it is `section-inventoried` at `ableton-live.md:49` and `:132–148`); the front matter's "The four adjacent records … are used in §2/§3" is wrong (no `OBS-` ID appears outside Appendix A); and roughly seven pins drift 1–4 lines. | Fix the three wrong pointers; tighten the drifted pins. |
| 4D. Honest gaps | 3 | Eleven open questions, each with an explicit `blocks:` list. Q2 is correctly routed and correctly framed as touching a **`verified`** requirement's acceptance evidence. Q9 refuses to assert a toolchain fact the repository cannot settle. Q11 refuses to write a test that cannot fail and says why. §7.4 names what R5 owns and what is not blocked. §3.7 and §6.3 both state negatives about the current build. This is not a spec papering over its unknowns. | — |
| 4E. Evidence commands | 3 | The workspace gate is quoted **exactly** as criteria.md 4E requires: `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo test --locked --workspace`. The targeted commands use real package names. `spectre:9` is quoted correctly — verified: `exec cargo run --locked --quiet -p spectre-app -- "$@"`. | — |
| 4F. No fake surfaces | **2** | The product-copy work is genuinely strong: §6.3 audits three user-visible strings line by line, the `ReplacedDurabilityUncertain` message deliberately does not say "Saved", and §5.4 ends with an Honesty check forbidding any crash-proof/recoverable/autosaved claim. But §1.3's success signal claims the slice produces "a fault-injection test at every one of the **seven** `SaveStage` values that asserts the destination is byte-identical to its pre-call contents" — §5.1 specifies **six** (and says `EncodeSnapshot` is deliberately untested), and Group A runs against `FaultFs`, which has no destination to be byte-identical to; the real byte-identity checks are I4 and I5 only. That is a claim about evidence the slice does not produce, in the section most likely to be quoted into STATUS. | Rewrite §1.3(b) to match §5.1. |

**Lens average:** 2.500 (15/6)
**Lens pass:** Yes — avg ≥ 2.0, zero 1s, zero 0s

---

## Auto-fail roll-call

| Rule | Triggered | Finding |
|---|---|---|
| AF-1 — contradicting accepted authority | **No** | The R4/R5 split matches `current-milestone.md:26–27` and `project-persistence.md:172–174` exactly. The two contacts with accepted material — the schema bump and CORE-003's acceptance test — are flagged in §7.2 in bold and routed to §8 Q1/Q2, not asserted. |
| AF-2 — unbacked implementation claims | **No (contested)** | §7.1 correctly distinguishes absent / implemented / verified / accepted-not-implemented / gated, and every named API, path, and structure exists as described. The two false test counts describe tests that do not exist at paths that do; a stricter reading of AF-2 would trigger on that. I classify it under the feasibility rule instead, which is the precise instrument. **The verdict is FAIL either way, so the classification does not change the outcome.** |
| AF-3 — realtime discipline | **No** | The boundary is *stated and proved structurally*, not assumed: four verified manifests plus U12. Nothing callback-reachable is touched. |
| AF-4 — borrowed numeric limits | **No** | Three bounds, each with a Spectre-derived rationale, and **rows are scheduled, not merely argued** — §7.2 commits three rationale rows to `docs/01-requirements/requirements-ledger.md` as CORE-005/006/007, citing PROD-003 at `:64` and decision 16 at `:40`, both verified exact. `TRACK_LEVEL_RANGE` documents an already-existing UI bound (`main.rs:181`, cited as `:180`). No value traces to a reference product. |
| AF-5 — conclusions the evidence does not support | **No** | §3.4 refuses a default shortcut map, cites the prohibition to its source, and disclaims ⌘S **by name** — the trap this feature most invites. No native-device list, synthesis limit, gesture budget, interface list, latency threshold, platform order, or archetype promotion appears. |
| AF-6 — optimistic language | **No** | The header leads with what does not work; §4.6, §5.4, §6.3, §7.2, and §7.4 all police the crash-durability line. §1.3's coverage overclaim (F6) is about test breadth, not product state, and is scored under 4F. |

**3B = 0 check:** not triggered — 3B scores 3.

---

## Feasibility Check

| Check | Status | Notes |
|---|---|---|
| Types/models exist or are clearly specified | ✓ | `ProjectEnvelope`, `ProjectDoc`, `Transaction`, `EditHistory`, `IdGen`, `AppModel`, `DeviceParameterSnapshot` all read and match. `reorder_tracks`'s inverse is correct: `remove(from)`+`insert(to)` is exactly inverted by `reorder_tracks(to, from)`, and U13's `[a,b,c] → [b,c,a]` is right. |
| API/interface changes are feasible with current architecture | ✗ | Adding four non-`Option` fields to `ProjectDoc` breaks every out-of-crate struct literal. `#[serde(default)]` governs serde, not Rust construction. Two such literals exist outside the crate and §7.2 accounts for neither. Separately, the "version stamp on save" has no owner in the declared API. |
| Views/screens fit current navigation pattern | ✓ | The PROJECT block reuses the exact widget shape already at `main.rs:141–152` (verified) inside the existing `SidePanel::left("tracks")` (`:116`, `default_width(220.0)` `:118`, `min_width(180.0)` `:119` — all exact). No new window, modal, or dependency. |
| Dependencies are available and version-compatible | ✓ | `std` only — no `tempfile`, no `rfd`. The one internal edge already exists at `spectre-app/Cargo.toml:16`. |
| Platform/renderer requirements are realistic | ✓ | Every call used is long-standing POSIX and `std`. The one uncertain mapping is flagged as uncertain rather than assumed. |
| Test strategy is executable with current infrastructure | ✗ | I9 cannot pass against the declared API; I11's device half cannot fail; and the constant bump breaks an existing assertion in a crate the spec calls unaffected. The private-seam placement argument (injection tests must be `#[cfg(test)] mod tests` inside `src/fs.rs` because `tests/` sees only the public API) is correct and well reasoned. |
| Performance budget is realistic for target hardware | ✓ | **Every figure recomputed; the arithmetic is clean.** `64 * 1024 * 1024` = 67,108,864 = 64 MiB ✓. The 912-byte fixture is verified byte-exact. 64 MiB / ~2 KB ≈ 4.5 orders of magnitude, so "four orders of magnitude of headroom" is conservative, not inflated. ~2 KB for one track / three devices / four parameters / view block and ~2.5 KB for three tracks are consistent with the measured fixture. §7.3's "18 unit tests, 15 integration tests" is exact (U1–U11 = 11, U12 = 1, U13–U18 = 6; I1–I15 = 15). **No constant contradicts its own derivation.** |
| No undeclared dependency on unbuilt features | ✓ | R4-1 is named as specified-but-not-implemented in three places, and §7.4 states the two coordination points without depending on either. |

**Feasibility verdict:** Infeasible as scoped — the delta in §7.2 does not describe the change the spec actually makes.
**Caveats:** Both ✗ rows resolve into one remediation (F1–F3). Nothing here questions the design; it questions the change inventory.

---

## Composite Score

| Lens | Average | Weight | Weighted |
|---|---|---|---|
| 1 — Realtime & Correctness | 2.857 | 35% | 1.000 |
| 2 — DAW Workflow Depth | 3.000 | 25% | 0.750 |
| 3 — Product Identity & Scope Discipline | 3.000 | 20% | 0.600 |
| 4 — Truthfulness & Evidence | 2.500 | 20% | 0.500 |
| **Composite** | | | **2.850** |

**Pass conditions (values copied from `criteria.md`, which is binding):**
- [x] Composite ≥ **2.30** — 2.850
- [x] Every lens average ≥ **2.00** — 2.857 / 3.000 / 3.000 / 2.500
- [x] No criterion scores 0 — zero 0s
- [x] At most **two** criteria score 1 — zero 1s
- [x] All auto-fail rules pass
- [ ] **Feasibility rule (condition 4) — NOT SATISFIED.** §7.1 misdescribes current state (two false test counts, one false attribution of step count to the accepted contract), and §7.2's delta omits two compile-breaking construction sites and asserts that a crate the change breaks is "unaffected." `criteria.md` states this condition without escape: *"A spec whose §7.1 misdescribes current state fails regardless of composite."*
- [x] Reviewer personally opened every source path cited

**All conditions met:** No → **FAIL**

**Against the 3.000 on record.** This review scores **2.850**, 0.150 below the companion
scorecard, and reaches the opposite verdict. The gap is entirely in the sections that
scorecard marked `[sampled]`: I agree with all thirteen of its unsampled grades and with
twenty of its twenty-six scores overall. The four disagreements are recorded below. The
verdict flips not on composite — 2.850 passes every numeric condition comfortably — but on
condition 4, which no amount of composite can satisfy.

---

## Where this review disagrees with the companion scorecard

1. **"Zero false citations" (its 4A, 4C, and summary) — not sustained.**
   `crates/spectre-offline/Cargo.toml:23–24`, cited in §5.2, **does not exist**: the file is
   21 lines and the `spectre-app` dev-dependency is at `:20–21`. (`spectre-audio/Cargo.toml:23–24`
   *is* `[dev-dependencies]` / `spectre-offline`, which is the likely origin of the slip.)
   Appendix A's `ableton-live-observations.md:19` pointer does not support the claim made from
   it. Roughly seven further pins drift 1–4 lines, including `main.rs:180` for the level slider
   that is actually at `:181` — load-bearing, because it is the evidence for `TRACK_LEVEL_RANGE`
   under AF-4. The *claims* hold in every one of these cases; the *pins* do not. Scored 4C = 2.

2. **Its 4A "Every `AppModel` accessor cited resolves exactly" is right; its §7.1 conclusion is not.**
   The accessors do resolve exactly (`:205`, `:208`, `:218`, `:264`, `:289`, `:308`, `:313`,
   `:348`, `:405–422` — all confirmed). But §7.1 also asserts test counts, and both are wrong:
   `project_codec.rs` has eight `#[test]` functions, not nine; `command_history.rs` has five,
   not eight. Its own scorecard also mis-cites the declaration site as `Cargo.toml:5`; it is `:16`.

3. **Its 1G = 3 "[sampled] — row bodies sampled, not all read."** Reading the bodies changes the
   grade. I9 cannot pass against the API §4 declares, and I11's device/parameter half cannot fail.
   Scored 1G = 2.

4. **Its lens-4 table grades a criterion `criteria.md` does not define.** It lists "4B. Numbers
   carry rationale"; `criteria.md` 4B is **Status vocabulary**, and numeric bounds are AF-4, not a
   lens-4 criterion. The consequence is that the `implemented`/`verified` criterion was never
   scored as such. Graded here on its own terms, it earns a 3 — but on evidence from seven
   locations, not the one sentence the companion quoted.

**Where I agree, emphatically.** Its reading of 3E, 2D/AF-5, 3B, and 4D is correct and I reached
the same conclusions independently from source. Its Priority 2 item on Q2 is the right call and
is upgraded here to Priority 1 for a reason it could not see from a sample (F2).

---

## Remediation Brief

### Priority 1 — Must fix to pass

1. **F1 — Rewrite §7.2's delta for the `ProjectDoc` field additions.** Adding `id_gen_state`,
   `tracks`, `devices`, and `view` breaks every out-of-crate struct literal;
   `#[serde(default)]` affects serde, not Rust construction. Two exist:
   `crates/spectre-offline/src/lib.rs:152–162` (`default_project`) and
   `crates/spectre-project/tests/command_history.rs:17–23` (`project()`). **Remove
   `crates/spectre-offline/src/lib.rs` from the "Deliberately not modified" list** — it cannot
   stay there — add both files to "Modified files", and update both literals explicitly. Note that a
   `#[derive(Default)]` escape is **not** available: `ProjectDoc.id` is an `ObjectId`, whose
   nonzero invariant (`crates/spectre-core/src/id.rs:29–35`) rules out a derived default.

2. **F2 — Correct the claim that `spectre-offline` is unaffected, and widen Q1/Q2's stated
   consequence.** §7.2 says `to_bytes`/`from_bytes` "keep their signatures, so `spectre-offline`'s
   use at `crates/spectre-offline/src/lib.rs:14` is unaffected." That line also imports
   **`SCHEMA_VERSION`**, whose *value* changes; `:153` stamps it into `default_project()`; and
   `crates/spectre-offline/tests/harness.rs:25` asserts `first.schema_version == 1` and therefore
   **fails on the bump**. So the schema bump disturbs an existing passing test in a second crate,
   not only `canonical_fixture_rewrite_is_byte_stable`. Q2 currently tells Jeff the blast radius is
   one test in one file; it is at least two tests in two crates plus two compile sites. A
   `verified` requirement cannot be routed accurately on an understated radius.

3. **F3 — Give the version stamp an owner, or delete the claim.** §7.2 declares the migration rule
   as "a version stamp on save … a save always writes the current version"; §7.2 justifies
   retargeting the byte-stability test with "rewriting a v1 document stamps the current version";
   and I9 asserts `schema_version == SCHEMA_VERSION` after load→save→load. **No component performs
   the stamp.** §4.3 step 2 is `to_bytes(snapshot)` verbatim, `save_project_atomic` takes
   `&ProjectEnvelope`, and §7.2 leaves `to_bytes` unchanged. Either add a stamping step to §4.3
   (which requires the function to clone and stamp), or state that stamping belongs to
   `project_envelope` and rewrite I9 to route through it.

4. **F4 — Fix §7.1's three false statements.** `project_codec.rs` has **eight** tests, not nine
   (`#[test]` at `:21, :52, :60, :69, :85, :100, :112, :124`); `command_history.rs` has **five**,
   not eight; and `docs/03-architecture/project-persistence.md:118–127` specifies **eight**
   algorithm steps, not nine — the ninth is the spec's own decomposition, and §4.6 repeats the
   misattribution when it claims R4 proves "correct ordering of the nine steps."

5. **F6 — Reconcile §1.3 with §5.1.** §1.3(b) promises fault injection "at every one of the seven
   `SaveStage` values" asserting "the destination is byte-identical to its pre-call contents."
   §5.1 injects **six** stages and states outright that `EncodeSnapshot` is deliberately untested;
   Group A asserts `target_state` and a `FaultFs` call log, with no real destination involved. The
   byte-identity property is proved by I4 and I5 on a real filesystem. Restate §1.3 to match.

### Priority 2 — Should fix for quality

1. **F5 — Make I11's device half falsifiable.** `AppModel::prototype()` is seeded with the fixed
   constant at `lib.rs:219` and mints all device and parameter `ObjectId`s **before** any
   `add_track`, and `from_descriptors` sets `value: descriptor.default()` (`lib.rs:83`). So the
   source model and the fresh target model already carry identical device IDs, parameter IDs, and
   values, and "devices() match by `instance_id` … every `ParameterControl.instance_id` and `value`
   matches" would pass even if `adopt` ignored the persisted devices entirely. Mutate one parameter
   via `set_device_parameter` before the round trip, or build the source from a different seed.

2. **Restate why `canonical_fixture_rewrite_is_byte_stable` actually breaks.** Not the version
   stamp (which does not exist) — the four new always-serialized `#[serde(default)]` fields change
   the encoded bytes regardless of the constant. This matters directly to Q1: the
   `skip_serializing_if` alternative Q1 offers preserves byte stability **only** if it also skips
   the empty collections. Q1 should say so.

3. **Fix the three wrong pointers.** `crates/spectre-offline/Cargo.toml:23–24` → `:20–21`
   (file is 21 lines). Appendix A's unextracted-chapter claim → chapter 5 "Managing Files and Sets"
   is not at `ableton-live-observations.md:19`; it is `section-inventoried` at `ableton-live.md:49`
   and `:132–148`. The front matter's "The four adjacent records … are used in §2/§3" → no `OBS-`
   ID appears outside Appendix A.

4. **Name the contract checklist item not discharged.** `project-persistence.md:166` requires fault
   injection to verify that "simultaneous same-target operations are prevented by the app-level
   owner rather than serialized by hidden project-crate state." §4.4 argues this structurally and
   specifies no test. That is defensible at R4 — but say so explicitly in §5.1, the way
   `EncodeSnapshot` is said, rather than leaving it silently uncovered.

### Priority 3 — Consider for excellence

1. **Tighten the drifted pins.** `spectre-project/src/lib.rs:101` → the `validate` call is at `:100`;
   `:105–128` → `validate` is `:105–124`; `:112–118` → `:113–117`; `:119–123` → `:118–122`;
   `:133–147` → the helper is `:132–145`. `main.rs:153` → `BROWSER` is at `:154`; `main.rs:180` →
   the level slider is at `:181`; `spectre-app/src/lib.rs:417` → `level: 0.72` is at `:418`.
   Every claim is true; the pins should be too, especially the slider, which carries an AF-4 rationale.

2. **Qualify §4.1's "unnameable."** `crates/spectre-audio/Cargo.toml:24` dev-depends on
   `spectre-offline`, which depends on `spectre-project` (`:16`), so `spectre_project` **is**
   nameable inside `spectre-audio`'s test targets. The claim is exactly true of the lib target and
   the callback path; U12 still fires on any direct edge. One clause makes it precise.

3. **Extend I2's no-residue property to a failure path.** I2 checks that a *successful* save leaves
   one directory entry, and I4 checks it after a validation failure. A real-filesystem case where
   the temporary is created and then the save fails — the only path where `.spectre-tmp` residue can
   actually occur — is not covered outside the `FaultFs` log assertions.

---

**End of scorecard.**
