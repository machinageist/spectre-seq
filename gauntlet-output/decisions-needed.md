<!--
Author: Jeff
Date: 2026-08-14
Description: Open decisions escalated out of specs and scorecards, awaiting Jeff
Notes: Questions that outgrow a spec's section 8 land here; nothing here blocks silently
-->

# Decisions Needed

- **Status:** accepted
- **Last verified:** 2026-08-14
- **Scope:** decisions the gauntlet cannot make for itself
- **Decision authority:** Jeff
- **Upstream sources:** spec §8 entries, scorecard escalations
- **Downstream dependents:** `manifest.md`, affected specs
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** none blocking; D-R1 is a research need, not a gate
- **Known gaps:** none

## How this file is used

A spec records its own uncertainty in §8. A question escalates here when it cannot be
resolved inside one feature — when it changes accepted authority, spans features, or
needs Jeff's product judgment. Auto-fail rule AF-1 requires this route: a spec may not
contradict an accepted decision by assertion, only by proposing supersession here.

Entries are never deleted. Resolved ones move to the resolved section with the outcome.

## Open

### D-R1 — Benchmark evidence gaps for Logic Pro and Serum 2

**Raised:** 2026-08-14, at Phase 1 dispatch.
**Blocks:** nothing. Recorded so specs name the gap instead of filling it in.

Jeff set the AAA benchmark set to Ableton Live, Logic Pro, Serum 2, Phase Plant, and
VCV Rack 2. The citable evidence behind those five is uneven:

| Benchmark | `OBS-` records | Dossier research state |
|---|---|---|
| Ableton Live 12 | 85 | inventory-only |
| Phase Plant | 11 | inventory-only |
| VCV Rack 2 | 6 | inventory-only |
| Serum 2 | 2 | **blocked-source-gap** |
| Logic Pro | **0** | inventory-only |

Serum 2's dossier states it "cannot currently be marked source-complete or accepted
for product planning." Logic Pro has no behavioral observations at all.

**Consequence for this run:** lens 2 criterion 2G requires specs to cite what exists
and name what does not. A spec that says "no citable Logic Pro evidence for this
surface" is correct and scores well; one that invents Logic behavior fails AF-6.

**Decision for Jeff, not blocking:** whether to commission Logic Pro and Serum 2
observation passes before the R4 UI-facing features (R4-4 track model, R4-6 first
devices) reach implementation. Until then those two benchmarks contribute little to
grading and the effective bench is Ableton, Phase Plant, and VCV.

### D-R2 — Provenance of the Serum 2 user guide, and whether its extraction can be accepted

**Raised:** 2026-08-15, after the Serum 2 research pass was cut off by a usage cap.
**Blocks:** promotion of any record in `docs/02-reference-research/serum-2-observations.md`.
Blocks nothing in the R4 gauntlet.

A research pass produced ~250 atomic Serum 2 observations, the large majority sourced
from what it describes as an official **354-page Serum 2 User Guide, manual 1.0.3,
documenting product 2.0.18**. If that guide is publicly authorized, it closes
`GAP-SERUM2-0001` — the blocker that has held the Serum 2 dossier at
`blocked-source-gap` since 2026-07-11 — and materially upgrades the weakest benchmark
in Jeff's AAA set.

**The problem.** The run was terminated before it wrote any source records. The ledger
has no URL, publisher, access date, source class, or access limitation for that guide
or for the three practitioner sources. So the file's central evidentiary claim rests on
provenance that was never recorded.

This is not a formality. `methodology.md` §"Research boundary" permits **publicly
authorized sources only** and explicitly forbids leaked or private manuals.
`serum-2.md` still carries the open decision *"whether a legitimately available
installed/customer guide can be reviewed without redistribution,"* and
`GAP-SERUM2-0004` records that authenticated-customer documentation was never
assessed. Whether this guide sits inside or outside that boundary **is** the open
question, and the extraction cannot answer it about itself.

**Decision for Jeff:**

1. Where did the guide come from — an official public endpoint, an authenticated
   customer download, or a third-party upload? Only the first is unambiguously inside
   the methodology; the second is the open decision; the third is outside.
2. If inside: authorize a completion pass to write the ledger records, fix the
   self-reported count errors, and run the second-pass contradiction review the
   review gates require. `GAP-SERUM2-0001` and `-0002` may then close.
3. If outside or unresolved: the file is deleted rather than quarantined, and the
   dossier stays `blocked-source-gap`.

**Recommendation:** do not promote anything from the file until (1) is answered. The
work is preserved under a quarantine banner and cited by nothing. Note the run also
disclosed that roughly half the guide's chapters remain `unreviewed`, so even the
favorable path is a partial coverage matrix, not a source-complete dossier.

### D-R4 — What does R4's exit mean while the Linux row cannot close?

**Raised:** 2026-08-21, by the R4-9 spec, which routed it rather than deciding it.
**Blocks:** R4's exit, and nothing before it. All nine specs pass regardless.

> **RESOLVED BY FACT, 2026-08-28 — the question's premise is void.** The Linux row closed. This
> repository's own development host is Linux (Arch, kernel 7.1.9-arch1-2, PipeWire 1.6.8) with
> three usable audio cards, and `hardware_lifecycle_drill` ran on it: 363 and 369 blocks through
> pipewire-alsa on a C-Media USB interface, and 198 blocks on the raw ALSA path to the onboard
> ALC285 with no sound server, all with 0 xruns, 0 plan errors, 0 contaminated nodes, and 0
> frame-capacity rejections. **The hardware was never missing.**
>
> Each disposition below collapses accordingly. **(a) strict conjunction** is now satisfiable and
> was never gated on a purchase. **(b) a second deferral** must not happen — there is nothing left
> to defer, and taking it now would defer a commitment that has already been discharged.
> **(c) split exit** is unnecessary for the Linux row.
>
> What remains open is a *different* obligation that D-R4 correctly identified as a strict
> superset: R4-9's manual protocol needs an **operator** at the host, not new hardware. That is
> the single remaining open R4 exit row. The per-platform rule still holds — no document may say
> "R4 passed" unqualified, because no operator pass exists on any platform.
>
> Two things surfaced in the same run that bear on how this decision came to look hardware-shaped:
> the workspace did not compile on Linux at all until 2026-08-28, and CI had been failing on
> exactly that since 2026-08-06 while watching only `main`, which no R4 slice ever touched. The
> drill also did not "run unchanged" as decision 23 predicted — it exposed a real `find_device`
> defect on ALSA first. A Linux host had been available the whole time; what was missing was
> anything that ran against one.

R4's exit evidence is a **ten-row conjunction** (`current-milestone.md`, "Exit evidence"),
and the milestone's own inherited-debt section says at `:22`: *"R4 carries four obligations
from earlier milestones. **None is optional** and none should be rediscovered later."*

One row — "Linux device qualification runs, discharging decision 23's debt" — is closable only
by running `cargo test -p spectre-audio --test lifecycle_health -- --ignored --nocapture` on a
Linux host with a real ALSA device. **No Linux audio device has ever been opened in this
project.** Decision 23 (`decision-gates.md:49`) authorizes no Linux support claim until it
runs, and records itself as "working against decision 1".

R4-9's own Linux column is blocked by a **strict superset** of that requirement: R4-3 needs one
command on the host; R4-9's manual protocol needs an operator *at* that host with an audio
interface — listening, unplugging it mid-playback, driving a screen reader.

**Three dispositions. The spec deliberately takes none:**

**(a) Strict conjunction.** R4 stays open until Linux hardware exists. Honest, and it means the
milestone's completion is gated on a purchase rather than on engineering.

**(b) A second scoped narrowing, in the shape of decision 23 itself.** Exit R4 on macOS
evidence and defer Linux again. **This would make R4 the second consecutive milestone to defer
decision 1's co-first-class commitment at the exact point where it could have been
discharged.** If this is the choice, it should be a decision row that says so in those words —
a pattern of deferral is a different thing from a one-time exception, and the second instance
is where it becomes one.

**(c) Split the exit.** Nine rows close; the milestone carries a named
`R4-exit-pending-Linux` state until the tenth does. Keeps the conjunction honest without
blocking everything behind hardware.

**Until this is answered**, R4-9 §4.4 rule 7 and §5.6 item 2 hold the only line consistent with
all three: **per-platform outcomes only, no aggregate word.** No document may say "R4 passed"
unqualified.

**Recommendation:** (c) if the hardware is genuinely coming, (a) if it is not, and (b) only with
the second-deferral consequence written into the row. What should not happen is (b) by
default — deferring twice without naming it is how a co-first-class commitment quietly becomes
a single-platform product.

### D-R3 — An accepted architecture document asserts `Gain` smooths; the shipped `Gain` does not

**Raised:** 2026-08-15, by the R4-2 spec and confirmed independently at blind verification.
**Blocks:** R4-2's implementation, and it should — R4-2 exists to make a slider drag reach a
live processor, and this is exactly where a click would come from.

`docs/03-architecture/dsp-device-io.md` states it twice. Line 94: *"Smoothing stays the
device's concern. **`Gain` already smooths**; devices whose parameters would click MUST
smooth internally rather than requiring the caller to ramp."* Line 104: *"`Gain`: stereo
linear gain with **click-resistant smoothing**."*

The shipped type is:

```rust
// crates/spectre-dsp/src/effect.rs — "Stereo gain with callback-ready target state"
pub struct Gain {
    gain: f32,
}
```

One field. `set_gain` performs `self.gain = GAIN_PARAMETERS[0].clamp(gain)` — an
instantaneous assignment (`effect.rs:45–47`). **There is no smoothing state, no target/current
pair, no ramp, and no coefficient.** The struct's own comment describes "callback-ready target
state" that does not exist either.

This is an accepted document making a false claim about shipped code, which
`docs/README.md`'s precedence does not resolve on its own: the architecture contract outranks
implementation on *direction*, but implementation is what actually runs. Under AF-1 a spec may
not resolve this by assertion, and R4-2 correctly did not.

**Decision for Jeff — three options:**

1. **The document is aspirational and wrong.** Correct `dsp-device-io.md:94` and `:104` to
   describe what `Gain` is, and decide separately whether smoothing is owed at R4-2, R4-6, or
   later. Cheapest, and honest.
2. **The document is right and `Gain` owes an implementation.** Smoothing lands inside R4-2,
   which grows the slice and adds per-device state to a processor that currently has none —
   with an RT-001 consequence, since the ramp runs on the callback.
3. **Split it.** Correct the document now, and open a scoped follow-on for click-resistance
   across all four fixture devices, since `Saturator`, `ToneSource`, and `PulseInstrument`
   have the same exposure the moment their parameters become live.

**Recommendation:** option 1 or 3. Option 2 quietly doubles R4-2's scope, and decision 15's
deliberate smallness argues against loading it in. Whichever is chosen, the audible
consequence should be verified by hand — §5.4 of R4-2 is where that check belongs.

### D-MM1 — Lens weighting for the mixing/mastering criteria

**Raised:** 2026-08-15, on authoring `criteria-mixing-mastering.md`.
**Blocks:** dispatching the MM gauntlet. Blocks nothing in the R4 loop.

Four lenses, weighted Realtime & Signal Correctness 30 %, Mastering Depth & Metering
Truth 25 %, Product Identity & Scope Discipline 20 %, Truthfulness & Evidence 25 %.

The one deliberate departure from `criteria.md`'s accepted split is **Truthfulness at
25 % rather than 20 %**, taken out of Realtime's share. The reasoning: this domain's
characteristic failure is not a bad design, it is a confident false number. The
benchmark corpus contains **no latency figure, no time constant, and no evidence of
loudness-standard conformance** for either vendor, so every such number appearing in a
spec was invented somewhere. Accept, amend, or reject.

### D-MM2 — Does Spectre build a mixing/mastering suite at all?

**Raised:** 2026-08-15. **Blocks:** the entire MM gauntlet, and it should.

`criteria-mixing-mastering.md` and the two dossiers grade and describe a suite that has
no accepted decision behind it. Nothing in the vision, the requirements ledger, or the
roadmap commits Spectre to mixing or mastering devices. The criteria file says so in its
own authority section.

This is a genuine product question, not a formality: a mastering suite is a large,
long-lived surface with its own metering, analysis, latency, and standards obligations,
and the roadmap already carries R11's effect catalog without specifying its contents.

**Recommendation:** answering "not yet, and the research stands" is a perfectly good
outcome. The dossiers, the criteria, and the component tree keep their value under
deferral; they are what makes the question answerable later without redoing the work.

### D-MM3 — Roadmap placement, since no mastering milestone exists

**Raised:** 2026-08-15. **Blocks:** criterion 3A can't be graded without it.

`rebuild-roadmap.md` runs R0–R12 with no mixing/mastering entry. The tree's Tier 0
infrastructure (metering, analysis, latency declaration and compensation, oversampling,
crossover) has a natural home at **R6**, which already owns monitoring, compensation,
meters, and a latency matrix as its exit gate — `MM-0.3` is arguably already R6 work
under another name. Tier 1 devices have a natural home at or after **R11**'s effect
catalog. **Nothing belongs in R4**, which decision 15 keeps deliberately small.

Three options: **distribute** across R6 and R11 (no new milestone, suite never coherent);
**insert** a milestone that owns it end to end (coherent, renumbers the roadmap); or
**defer** until R11 arrives. Deferral is the honest default while D-MM2 is open.

### D-MM4 — Which loudness standards, if any, does Spectre implement natively?

**Raised:** 2026-08-15. **Research half discharged the same day.** **Still blocks:** nothing
mechanically; it is now a product question rather than an evidence gap.

The reading pass ran. ITU-R BS.1770-5 (Annexes 1 and 2), EBU R 128, EBU Tech 3341, and
EBU Tech 3342 were retrieved and read in full, producing **95 records** in
`docs/02-reference-research/loudness-standards-observations.md` and four
`claims-extracted` ledger records. MM-AF-5 was narrowed accordingly: the algorithm,
K-weighting coefficients, gating rules, true-peak method, and LRA definition are now
citable facts. Conformance claims and delivery targets remain prohibited.

**What the pass surfaced that Jeff should decide on, because none of it is a research
question any longer:**

1. **BS.1770-5 supplies K-weighting coefficients for 48 kHz only** (`GAP-LOUDNESS-0001`).
   It says other rates need values "chosen to provide the same frequency response" and
   supplies neither the values, a derivation procedure, nor a tolerance. A DAW runs at
   whatever rate the device is set to. **Any Spectre meter at 44.1 or 96 kHz requires an
   original engineering decision here** — this is the real implementation blocker, and no
   amount of further reading removes it.
2. **Correct Integrated Loudness collides with Spectre's realtime contract**
   (`OBS-T3341-020`). Tech 3341 §2.3 requires recomputation from stored per-block history
   on every update — the measurement is not incrementally summarisable, because the
   relative gate moves and retroactively changes which past blocks count. At the mandated
   100 ms hop that is 36,000 entries per hour of monotonically growing state, which cannot
   live on the audio thread under RT-001/RT-002. A bounded approximation is available but
   is a **declared deviation**, not conformance.
3. **"EBU Mode" is a claim with a test suite attached, and Spectre cannot currently make
   it** (`OBS-XSTD-004`, `GAP-LOUDNESS-0008`). Both EBU documents gate compliance on
   test-signal sets hosted separately that were not retrieved.

**Decision for Jeff:** whether Spectre measures loudness natively at all; if so, whether it
accepts (1)'s original-engineering burden and (2)'s declared deviation; and whether to
commission retrieval of the EBU test-signal sets, which is the only path to a conformance
claim rather than a measurement claim.

## Resolved

### D-G1 — Lens weighting for `criteria.md` — **accepted as proposed, 2026-08-14**

Four lenses rather than the template's three, on the reasoning that a realtime audio
engine has a hard-constraint dimension no UI or competitive lens covers. Weights:
Realtime & Correctness 35%, DAW Workflow Depth 25%, Product Identity & Scope
Discipline 20%, Truthfulness & Evidence 20%. Jeff authorized the run without
amending them; `feature-tree.md`'s nine features were confirmed in the same breath.
