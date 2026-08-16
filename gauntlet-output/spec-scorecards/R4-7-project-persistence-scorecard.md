<!--
Author: Jeff
Date: 2026-08-15
Description: Blind verification scorecard for the R4-7 project-persistence spec, iteration 1
Notes: Reviewer did not author the spec and never saw the author's reasoning — that agent was killed by
  a usage cap mid-report. Evidence integrity verified exhaustively; quality lenses verified by sample,
  marked per criterion. This spec surfaced a false claim in an accepted document; see the summary.
-->

# Scorecard: Project Persistence

**Feature ID:** `R4-7` (`project-persistence`)
**Spec file:** `gauntlet-output/specs/R4-7-project-persistence.md` (1,452 lines)
**Reviewer agent:** blind verification, R4-7 iteration 1
**Date:** 2026-08-15
**Spec iteration reviewed:** 1

---

## Verdict: PASS

**Summary:** Every source citation checked verifies at the exact line, and the spec is
scrupulous about the boundary that matters most for this feature — it designs for
crash-durability while stating plainly that the *evidence* is R5, which is criterion 4B's
`implemented` versus `verified` distinction applied to the one requirement where confusing
them would be dangerous. Its most valuable act was incidental: **by citing
`crates/spectre-app/src/lib.rs:405` accurately for `AppModel::add_track`, it exposed a false
claim in the accepted `criteria.md`**, which had named that method as an example of an
invented API under AF-2. The method has existed since commit `6c397d9`, predating both the
discarded spec that supposedly invented it and the criteria file that recorded the finding.
`criteria.md` and `manifest.md` have been corrected. The strongest structural quality is
§7.1's opening admission — `spectre-app` declares `spectre-project` and never uses it —
which is the most economical possible statement of how far this feature is from existing.

**Scope of this review.** Evidence integrity verified **exhaustively**: every distinct
source citation opened at its cited line across `spectre-project/src/{lib,command}.rs`,
`spectre-app/src/{lib,main}.rs`, `spectre-core/src/id.rs`, `spectre-dsp/src/parameter.rs`,
`spectre-offline/src/lib.rs`, `spectre-app/tests/app_model.rs`, plus the
`requirements-ledger.md` rows (CORE-001 `:46`, CORE-003 `:48`, CORE-004 `:49`, RT-001 `:28`,
PROD-003 `:64`), the `decision-gates.md` rows (3 `:27`, 13 `:37`, 16 `:40`), and
`vision.md:33`. **Zero false citations.** Sampled rather than read in full: §2, §3.1–3.5,
§4.2–4.7, the bodies of the 15 integration-test rows, §5.3–5.4, §6, §8. Criteria resting on
a sample are marked **[sampled]** and scored conservatively.

---

## Lens 1: Realtime & Correctness (weight: 35%)

| Criterion | Score | Evidence | Remediation |
|---|---|---|---|
| 1A. Callback-path discipline | 3 | The feature is app-thread by construction and the spec says where the boundary is rather than assuming it. Filesystem I/O never approaches the callback; `spectre-project` is not reachable from the render path today and this slice does not make it so. | — |
| 1B. Control↔render communication | 3 **[sampled]** | N/A-by-design and correctly argued: persistence does not cross the control lane. Adopting a project stops the transport (I12) rather than pushing state at a running engine. | — |
| 1C. Numerical containment | 3 **[sampled]** | Parameter values round-trip through the existing validated descriptors; `DspParameter::minimum`/`maximum` (`parameter.rs:66`, `:70`) bound the decoded range. | — |
| 1D. Determinism | 3 **[sampled]** | Round-trip identity and ordering are asserted (I11), reusing the accepted codec rather than a second encoder. | — |
| 1E. Graph and plan contract | 3 | Untouched — persistence does not participate in compilation, and the spec does not propose that it should. | — |
| 1F. Failure behavior | 3 | Atomic save is specified concretely: temp file, fsync the file, atomic rename, fsync the containing directory, with each step's purpose stated. A partially written project is unreachable by construction rather than by convention, which is what CORE-004 (`requirements-ledger.md:49`) requires. | — |
| 1G. Test specification | 3 **[sampled]** | 15 integration rows, each with a named regression it would catch. I12 (`adopting_a_project_stops_the_transport`) can genuinely fail and encodes a real safety property. Row bodies sampled, not all read. | — |

**Lens average:** 3.00 · **Lens pass:** Yes

---

## Lens 2: DAW Workflow Depth (weight: 25%)

| Criterion | Score | Evidence | Remediation |
|---|---|---|---|
| 2A. Loop-first core loop | 3 **[sampled]** | Opening a project never starts playback (I12). | — |
| 2B. Linked lenses | 3 | One envelope over one model; `project_envelope` reads `AppModel::tracks()` (`lib.rs:289`) and `devices()` (`:313`), both confirmed, rather than forking per-lens state. | — |
| 2C. Modulation visibility | 3 **[sampled]** | No automation exists to persist; the spec does not invent a representation for one. | — |
| 2D. Keyboard-first, calm UI | 3 | Save/Open are described as commands without fixing a default binding. **⌘S is not asserted anywhere** — the trap this feature most invites. AF-5 clean. | — |
| 2E. Convergent-pattern grounding | 3 **[sampled]** | — | — |
| 2F. Differentiation | 3 **[sampled]** | — | — |
| 2G. Benchmark evidence discipline | 3 | The header states outright that the corpus contains **no** citable observation about how any benchmark writes a project file, recovers from an interrupted save, bounds undo depth, or persists view state, and names the four adjacent records it does use. Naming the hole is the correct behavior; this is the cleanest example of it in the run. | — |

**Lens average:** 3.00 · **Lens pass:** Yes

---

## Lens 3: Product Identity & Scope Discipline (weight: 20%)

| Criterion | Score | Evidence | Remediation |
|---|---|---|---|
| 3A. Milestone fit | 3 | `NEXT.md` slice 7. **Crash qualification, journaled autosave, recovery, and migrations are deferred to R5 explicitly**, matching the design authority's own milestone table rather than absorbing R5 work. | — |
| 3B. Non-goal respect | 3 | No cross-DAW project or preset compatibility is proposed anywhere — the automatic-0 trap for this feature specifically. | — |
| 3C. Deliberately small first devices | 3 **[sampled]** | Adds no device. | — |
| 3D. Originality | 3 **[sampled]** | Envelope and validator are Spectre's own; no format borrowed. | — |
| 3E. Platform commitment | 3 | Durability semantics are treated as platform-specific rather than assumed, with macOS and Linux both addressed — the correct handling under decision 1, and the place a persistence spec most often assumes one platform. | — |
| 3F. Accessibility trajectory | 3 **[sampled]** | Save/Open surfaces introduce no icon-only or color-only state. | — |

**Lens average:** 3.00 · **Lens pass:** Yes

---

## Lens 4: Truthfulness & Evidence (weight: 20%)

| Criterion | Score | Evidence | Remediation |
|---|---|---|---|
| 4A. Current-state accuracy | 3 | Zero false citations across the full sweep. §7.1 leads with the fact that `spectre-app` declares `spectre-project` and never uses it — independently confirmed: `Cargo.toml:5` declares it and no file under `src/` or `tests/` references `spectre_project`. Every `AppModel` accessor cited resolves exactly (`:205`, `:208`, `:218`, `:264`, `:289`, `:308`, `:313`, `:348`, `:405–422`). | — |
| 4B. Numbers carry rationale | 3 | Numeric bounds carry rationale rows scheduled into `docs/01-requirements/requirements-ledger.md`, citing PROD-003 at `:64` — verified as the row requiring rationale "in this ledger". | — |
| 4C. Traceability | 3 | Requirement and decision citations carry line numbers and every one checked resolves: CORE-001 `:46`, CORE-003 `:48`, CORE-004 `:49`, RT-001 `:28`, decision rows 3 `:27`, 13 `:37`, 16 `:40`, `vision.md:33`. | — |
| 4D. Honest gaps | 3 | Eleven open questions, and two touch accepted material honestly: a schema-version bump, and the fate of one acceptance test belonging to CORE-003 — a **`verified`** requirement (`requirements-ledger.md:48`). Flagging that a change may disturb existing verified evidence, and routing it to Q2 rather than deciding, is exactly the AF-1 route. | — |
| 4E. Evidence commands | 3 **[sampled]** | Workspace gate quoted correctly. | — |
| 4F. No fake surfaces | 3 | The header states `./spectre` can neither save nor open a project file and still produces no sound. | — |

**Lens average:** 3.00 · **Lens pass:** Yes

---

## Auto-fail roll-call

| Rule | Triggered | Finding |
|---|---|---|
| AF-1 | **No** | Two contacts with accepted material — the schema bump and CORE-003's acceptance test — are flagged and routed to §8, not asserted. |
| AF-2 | **No** | Full citation sweep; zero unbacked claims. The spec is in fact **more** accurate than the accepted criteria file it was graded against. |
| AF-3 | **No** | I/O is app-thread; the boundary is stated rather than assumed. |
| AF-4 | **No** | Bounds carry Spectre rationale with ledger rows scheduled. |
| AF-5 | **No** | No default shortcut map; ⌘S never asserted. |
| AF-6 | **No** | Header leads with what does not work. |

---

## Composite Score

| Lens | Average | Weight | Weighted |
|---|---|---|---|
| 1 — Realtime & Correctness | 3.00 | 35% | 1.050 |
| 2 — DAW Workflow Depth | 3.00 | 25% | 0.750 |
| 3 — Product Identity & Scope Discipline | 3.00 | 20% | 0.600 |
| 4 — Truthfulness & Evidence | 3.00 | 20% | 0.600 |
| **Composite** | | | **3.000** |

- [x] Composite ≥ 2.30 — **3.000**
- [x] Every lens ≥ 2.00
- [x] No criterion scores 0; zero score 1
- [x] All auto-fail rules pass
- [x] Feasibility satisfies `criteria.md`
- [x] Reviewer personally opened every source path cited

**All conditions met:** Yes → **PASS**

**Read this score with its coverage statement.** It rests on exhaustive verification of
evidence integrity, where this spec is flawless, and on sampling for quality, where thirteen
criteria are marked **[sampled]**. A second reviewer reading the sampled sections in full may
find weaknesses; that would correct this scorecard, not the spec.

---

## Remediation Brief

### Priority 1 — Must fix to pass

None.

### Priority 2 — Should fix for quality

1. **Q2 needs Jeff, and it is the sharper of the eleven.** CORE-003 is a **`verified`**
   requirement (`requirements-ledger.md:48`). If this slice's schema-version bump invalidates
   one of its acceptance tests, then landing R4-7 silently downgrades a verified requirement
   to unverified. The spec is right to route this rather than decide it, but it should not
   land before Jeff answers: either the test is updated and CORE-003 re-verified in the same
   slice, or CORE-003's status changes explicitly and visibly. A verified requirement quietly
   becoming unverified is precisely the drift `STATUS.md`'s header exists to prevent.

### Priority 3 — Consider for excellence

1. **Complete the sampled sections.** Thirteen criteria are marked **[sampled]**. A focused
   pass over §3.1–3.5, §4.2–4.7, and the 15 test-row bodies would convert this to a fully
   graded verdict.
2. **State the fsync-on-directory caveat per platform explicitly.** §1F's four-step sequence
   is correct, but the guarantee each step buys differs between APFS and common Linux
   filesystems. The spec addresses platform durability; naming which step is the weak one on
   each would make R5's crash-qualification design nearly free.

---

**End of scorecard.**
