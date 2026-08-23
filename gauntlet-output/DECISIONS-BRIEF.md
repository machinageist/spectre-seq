<!--
Author: Jeff
Date: 2026-08-23
Description: One-page decision sheet — the six open decisions, their real choices, and what each costs
Notes: Full context lives in decisions-needed.md; this exists so the decisions can be made without re-reading it
-->

# Decision Sheet

- **Status:** accepted
- **Last verified:** 2026-08-23
- **Scope:** every open decision blocking Spectre work, with the actual options and their consequences
- **Decision authority:** Jeff
- **Upstream sources:** `decisions-needed.md`, `manifest.md`, `summary.md`
- **Downstream dependents:** R4 implementation; the mixing/mastering loop
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** all six below
- **Known gaps:** none — this is a restatement, not new analysis

Nine specs pass. **Six decisions block what happens next**, and none can be made by an agent:
each is a conflict between accepted authority and reality, which AF-1 reserves to Jeff.
Ordered by what unblocks the most.

---

## 1. D-R4 — What does R4's exit mean while the Linux row cannot close? **← decide first**

R4's exit is a **ten-row conjunction**. `current-milestone.md:22` says *"None is optional."*
One row needs a Linux ALSA device that has never existed in this project.

| Option | Consequence |
|---|---|
| **(a) Strict conjunction** | R4 stays open until hardware is bought. Honest; gates a milestone on a purchase. |
| **(b) Defer Linux again** | **Makes R4 the second consecutive milestone** to defer decision 1's co-first-class commitment at the point it could be discharged. |
| **(c) Split the exit** | Nine rows close; milestone carries a named `R4-exit-pending-Linux` state. |

**Recommendation:** (c) if hardware is coming, (a) if it is not. **(b) only with the
second-deferral consequence written into the decision row** — deferring twice without naming it
is how a co-first-class commitment quietly becomes a single-platform product.

**Until answered:** no document may say "R4 passed" unqualified; per-platform outcomes only.

---

## 2. D-R3 — `Gain` smoothing: the document or the code? **blocks R4-2**

`dsp-device-io.md:94` and `:104` both say `Gain` smooths. The shipped `Gain` is one `f32` with
an instantaneous clamped setter and **no smoothing state**.

R4-2 exists to make a slider drag reach a live processor — so this is exactly where a click
comes from.

| Option | Consequence |
|---|---|
| **(a) Document is wrong** | Correct two lines; decide separately when smoothing is owed. Cheapest, honest. |
| **(b) Code owes an implementation** | Smoothing lands in R4-2, growing the slice and adding per-device state with an RT-001 consequence. |
| **(c) Split** | Correct the document now; open a scoped follow-on for all four fixture devices. |

**Recommendation:** (a) or (c). (b) quietly doubles R4-2's scope, and decision 15's deliberate
smallness argues against it.

---

## 3. R4-7 Q2 — May a schema bump disturb a `verified` requirement? **blocks R4-7**

CORE-003 is **`verified`**. R4-7's schema-version bump may invalidate one of its acceptance
tests. Landing it unanswered would silently downgrade a verified requirement to unverified —
the exact drift `STATUS.md`'s header exists to prevent.

**Choose:** update the test and re-verify CORE-003 in the same slice, **or** change its status
explicitly and visibly. Not: let it lapse quietly.

---

## 4. D-R2 — Serum 2 guide provenance **gates 250 observations**

A research pass produced ~250 Serum 2 observations from a 354-page user guide whose provenance
was never recorded. `methodology.md` permits **publicly authorized sources only**.

**Only you can answer this** — it is a question about where a file on your machine came from.

| Answer | Consequence |
|---|---|
| Official public endpoint | Inside methodology. Authorize a completion pass; `GAP-SERUM2-0001` may close. |
| Authenticated customer download | This *is* the open question `serum-2.md` has carried since July. |
| Third-party upload | Outside methodology — the file is deleted, not quarantined. |

**Until answered:** the file stays quarantined and cited by nothing. Nine specs respected that
without being asked; R4-8 found the one convenient record and refused it.

---

## 5. D-MM2 and D-MM3 — Build a mixing/mastering suite, and where? **gates the whole MM loop**

`criteria-mixing-mastering.md`, both dossiers, and the component tree describe a suite with **no
accepted decision behind it**. Nothing in the vision, requirements, or roadmap commits Spectre
to it. The roadmap has no mastering milestone; Tier-0 infrastructure fits R6, Tier-1 devices fit
R11, and **nothing fits R4**.

**Answering "not yet, and the research stands" is a perfectly good outcome.** The dossiers,
criteria, and tree keep their value under deferral — they are what make the question answerable
later without redoing the work.

---

## 6. D-MM1 and D-MM4 — MM lens weighting; native loudness measurement

**D-MM1:** accept or amend the proposed weights (Truthfulness at 25% rather than 20%, taken from
Realtime). Only matters if D-MM2 says yes.

**D-MM4:** the research half is **done** — four standards read, 95 records, four ledger entries.
What remains is product judgment, and the reading surfaced two facts that decide it:

- **BS.1770-5 gives K-weighting coefficients for 48 kHz only.** Any other sample rate is an
  original Spectre engineering decision. No further reading removes this.
- **Correct Integrated Loudness collides with RT-001/RT-002** — it requires recomputation from
  stored per-block history, ~36,000 growing entries per hour, which cannot live on the audio
  thread. A bounded approximation is available but is a **declared deviation**, not conformance.

**Choose:** measure loudness natively accepting both burdens; decline to make loudness claims;
or commission the EBU test-signal sets, which is the only path to a *conformance* claim rather
than a *measurement* claim.

---

## What is not blocked

Implementation of **R4-1, R4-3, R4-4, R4-5, R4-6, R4-8, R4-9** is blocked by none of the above —
only R4-2 (D-R3) and R4-7 (Q2) are gated. R4-1 is the correct first slice regardless: it makes
every later slice observable, and its spec passed at 2.950.
