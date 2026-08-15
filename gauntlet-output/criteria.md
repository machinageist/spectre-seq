<!--
Author: Jeff
Date: 2026-08-14
Description: Binding grading standard for every Spectre spec the gauntlet produces
Notes: Subordinate to docs/README.md conflict precedence; auto-fails trace to accepted documents
-->

# Spectre — Quality Criteria

- **Status:** proposed
- **Last verified:** 2026-08-14
- **Scope:** grading standard for all gauntlet-authored specs and blind scorecards
- **Decision authority:** Jeff
- **Upstream sources:** `../docs/00-product/vision.md`, `../docs/01-requirements/decision-gates.md`, `../docs/01-requirements/requirements-ledger.md`, `../docs/02-reference-research/`, `../docs/06-plans/current-milestone.md`
- **Downstream dependents:** every file under `specs/` and `spec-scorecards/`
- **Supersedes:** the uncriteria'd 2026-08-12 run
- **Superseded by:** none
- **Open decisions:** lens weights await Jeff's sign-off; this file stays `proposed` until then
- **Known gaps:** lens 2 grades against a research corpus the field study itself marks below saturation

**Criteria version:** 1

## Authority

This file is **subordinate** to the conflict precedence in `docs/README.md`. It
grades specs; it does not create product authority. Where a spec, this file, and an
accepted document disagree, the accepted document wins in `docs/README.md` order:
product vision → requirements and decisions → architecture contracts → specs →
quality → plans → status → research.

`docs/README.md` states the governing rule directly: *"Implementation proves behavior
but does not silently redefine accepted requirements."* A spec is a proposal about
behavior, so the same rule binds it. A spec that wants to change accepted direction
must say so explicitly and route through `decisions-needed.md` — never by assertion.

## Scoring

Each criterion is graded 0–3:

- **0 — Missing:** not addressed.
- **1 — Inadequate:** addressed but wrong, superficial, or contradicts constraints.
- **2 — Acceptable:** correct and functional; minor gaps.
- **3 — Excellent:** would ship in a best-in-class product; no meaningful gap.

**Pass threshold, all four conditions binding:**

1. Weighted composite ≥ **2.30 / 3.00**.
2. Every lens ≥ **2.00** on its own. A strong lens cannot carry a weak one.
3. No criterion scores **0**, and at most **two** criteria score 1.
4. **Feasibility rule.** The reviewer reads every source path the spec cites and
   confirms the claim against the file. A spec whose §7.1 misdescribes current state
   fails regardless of composite — the reviewer must open the files, not trust §7.1.

Reviewers are blind: they grade the finished spec, never the author's reasoning.

## Auto-fail rules

These override all scoring. Each traces to an accepted document.

**AF-1 — Contradicting accepted authority.**
A spec that contradicts an **Accepted** row in `docs/01-requirements/decision-gates.md`
or an accepted requirement in `requirements-ledger.md` fails, unless it explicitly
flags the conflict, proposes supersession, and logs it in `decisions-needed.md`.
*Source: `docs/README.md` conflict precedence.*

**AF-2 — Unbacked implementation claims.**
A spec that describes code as existing, partial, or implemented without a source path
that actually contains it fails. §7.1 must distinguish implemented / prototyped /
planned / gated / absent, and each claim must be checkable in one `Read`.
*Source: `docs/00-product/vision.md` release bar "honest telemetry, no fake surfaces";
`docs/status/STATUS.md` "claims here link to live evidence".*

**AF-3 — Realtime discipline violation.**
A spec that puts allocation, deallocation, blocking locks, I/O, logging, or a panic
across the callback boundary on a callback-reachable path fails. Bounded wait-free
structures with a defined overflow policy and off-thread reclamation are mandatory
for control↔render traffic. Denormals flush; NaN/Inf isolate the node and output
silence, never noise.
*Source: RT-001, RT-002, RT-003 in `requirements-ledger.md`; decision 21.*

**AF-4 — Borrowed numeric limits.**
Any numeric bound taken from a reference product without its own rationale row fails.
*Source: decision 16 (standing rule); PROD-003.*

**AF-5 — Conclusions the evidence does not support.**
The workflow field study names conclusions its corpus cannot justify. A spec that
asserts any of them as settled fails: a final native-device list, a synthesis
architecture or modulation limit, **a default shortcut map**, a gesture-count or time
budget, a supported-interface model list, a monitoring-latency threshold,
platform/backend order, command-frequency or feature-priority scores, or promotion of
any workflow archetype.
*Source: `docs/02-reference-research/workflow-field-study/product-implications.md`
§"Prohibited conclusions at current evidence level".*

**AF-6 — Optimistic language.**
Unevidenced or promotional phrasing about state fails. `docs/status/STATUS.md` names
this prohibition in its own header, and the standard applies to specs describing that
same state.

> The three defects in the discarded 2026-08-12 `track-management.md` map to AF-2
> (invented `AppModel::add_track`, `Track::arm`, `Track::mute`), AF-5 (a ⌘T/⌘U/⌘M
> shortcut map), and the feasibility rule (a §7.1 describing a track model that does
> not exist). Any one of them fails the spec on its own. That is the regression this
> criteria file exists to prevent.

---

## Lens 1: Realtime & Correctness (weight: 35%)

**Standard:** the accepted realtime contract in `requirements-ledger.md` §RT and the
architecture contracts in `docs/03-architecture/`.

**1A. Callback-path discipline.** Every path reachable from the audio callback is
identified and shown allocation-free, lock-free, and non-blocking. The spec names
which RT-001 guard covers it.

**1B. Control↔render communication.** Control traffic crosses via the accepted
split-lane RT-002 transport. The spec states its lane, its overflow behavior, and
where retired state is reclaimed. Latest-wins for parameters; strict FIFO with counted
overflow for notes and transport.

**1C. Numerical containment.** Denormal flush and NaN/Inf isolation per RT-003 are
addressed for any new DSP node, with injection tests named.

**1D. Determinism.** Identical input yields identical output. Where the spec claims
equivalence between live and offline paths, it names the comparison method — the
existing FNV-1a hash walk in `spectre-offline` and `bridge`, not a new one.

**1E. Graph and plan contract.** Editable-graph and compiled-plan responsibilities stay
split per GRAPH-001. Nothing that allocates runs on the render side. Plan recompilation
is never proposed per parameter change (decision 22 rejects option (b) on its face).

**1F. Failure behavior.** Refusals are explicit and fail closed — silence rather than
stale audio, counted rather than silently dropped, surfaced off-thread rather than
logged on the callback.

**1G. Test specification.** §5 names commands that actually run and assertions that
would fail if the behavior regressed. A test that cannot fail scores 0.

---

## Lens 2: DAW Workflow Depth (weight: 25%)

**Standard:** the accepted research corpus in `docs/02-reference-research/` —
Ableton Live, Bitwig Studio, Cubase, FL Studio, Logic Pro, Reaper, Serum 2,
Phase Plant, VCV Rack — plus `workflow-field-study/`. Cite by observation ID
(`OBS-…`) or source-ledger entry, never from memory.

**Benchmark handling:** the corpus is already accepted research. Reuse it. Do not
re-derive competitor behavior from recollection, and respect the field study's own
saturation warning — it grades what the evidence supports, not what a spec wishes.

**2A. Loop-first core loop.** The feature supports sketch → branch → audition → grow
without losing selection, zoom, or transport context (`vision.md`).

**2B. Linked lenses.** Timeline, performance grid, mixer, and sound-flow are views over
one model with shared selection and stable identity. A feature that forks state per
lens scores 1 or lower.

**2C. Modulation visibility.** Where parameters appear, base value, automation, and
modulation contribution stay distinct, with an explicit restore action (PROD-002).

**2D. Keyboard-first, calm UI.** Context-scoped command resolution, searchable and
remappable commands, no spreadsheet density, no default cable spaghetti. Note the
interaction with AF-5: the spec may specify that commands are remappable and
context-scoped; it may **not** fix a default shortcut map.

**2E. Convergent-pattern grounding.** Where two or more researched products converge on
a pattern, the spec either follows it or states why Spectre diverges.

**2F. Differentiation.** The spec says what Spectre does that the benchmark set does
not, without claiming parity as completeness (`vision.md` non-goals).

---

## Lens 3: Product Identity & Scope Discipline (weight: 20%)

**Standard:** `docs/00-product/vision.md` and the active milestone in
`docs/06-plans/current-milestone.md`.

**3A. Milestone fit.** The feature belongs to R4 Credible Alpha as scoped: one track,
MIDI clip, native synth + effect, transport, save/reload, offline bounce. Scope beyond
that is named as such and deferred, not smuggled in.

**3B. Non-goal respect.** No CLAP/LV2/AU hosting, no plugin-format authoring of
first-party devices, no cross-DAW preset or project compatibility, no cloud services or
content stores, no video scoring. Proposing one is an automatic 0.

**3C. Deliberately small first devices.** R4's synth and effect stay small by decision
15. A spec that grows them toward the R11 flagship scores 1 or lower.

**3D. Originality.** Original code, DSP, names, content, and formats. Reference research
informs; it is never transcribed. This is the qualitative partner to AF-4.

**3E. Platform commitment.** macOS and Linux are co-first-class (decision 1). A spec that
assumes macOS-only behavior without naming the Linux path scores 1 or lower — decision
23's undischarged Linux debt makes this the live risk in R4, not a hypothetical.

**3F. Accessibility trajectory.** Keyboard-complete operation and screen-reader labels
are a beta gate (decision 17) with a scoped audit at R4. The spec does not have to
deliver them; it must not design a surface that forecloses them.

---

## Lens 4: Truthfulness & Evidence (weight: 20%)

**Standard:** `docs/status/STATUS.md`, whose header prohibits optimistic language, and
the metadata discipline in `docs/README.md`.

**4A. Current-state accuracy.** §7.1 matches what the source files actually contain.
The reviewer verifies by reading them. This is the feasibility rule's criterion.

**4B. Status vocabulary.** Claims use `docs/README.md`'s vocabulary — `draft`,
`proposed`, `accepted`, `implemented`, `verified`, `superseded`, `archived` — with the
`implemented` / `verified` distinction respected: code existing is not evidence passing.

**4C. Traceability.** Behavioral claims cite a requirement ID, a decision row, an
observation ID, or a source path. Uncited normative claims score 1 or lower.

**4D. Honest gaps.** §7.2 and §8 state what is unknown or blocked rather than papering
over it. A spec with no open questions on genuinely novel work is suspect, not strong.

**4E. Evidence commands.** Verification commands are real, runnable, and named exactly
— the workspace gate is `cargo fmt --all -- --check`, `cargo clippy --locked
--workspace --all-targets -- -D warnings`, `cargo test --locked --workspace`.

**4F. No fake surfaces.** Nothing in the spec implies a working surface that would not
work. `./spectre` currently produces no sound; a spec that reads as though it does
fails 4F and AF-2 together.

---

## Scoring summary

| Lens | Criteria | Weight | Auto-fail conditions |
|---|---|---|---|
| 1 — Realtime & Correctness | 7 | 35% | AF-3 |
| 2 — DAW Workflow Depth | 6 | 25% | AF-5 |
| 3 — Product Identity & Scope Discipline | 6 | 20% | AF-4; 3B = 0 fails the spec |
| 4 — Truthfulness & Evidence | 6 | 20% | AF-2, AF-6 |

Weights sum to 100%. AF-1 applies across all four lenses.
