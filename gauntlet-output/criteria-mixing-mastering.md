<!--
Author: Jeff
Date: 2026-08-15
Description: Binding grading standard for every Spectre mixing/mastering suite spec, with FabFilter and Ozone as the AAA benchmark
Notes: Subordinate to docs/README.md conflict precedence; parallel to criteria.md, which grades the R4 loop
-->

# Spectre — Mixing & Mastering Suite Quality Criteria

- **Status:** proposed
- **Last verified:** 2026-08-15
- **Scope:** grading standard for every spec and blind scorecard produced for the Spectre mixing/mastering device suite
- **Decision authority:** Jeff
- **Upstream sources:** `../docs/00-product/vision.md`, `../docs/01-requirements/decision-gates.md`, `../docs/01-requirements/requirements-ledger.md`, `../docs/02-reference-research/fabfilter.md`, `../docs/02-reference-research/ozone.md`, `../docs/02-reference-research/external-reference-register.md`, `../docs/06-plans/rebuild-roadmap.md`
- **Downstream dependents:** every mixing/mastering spec and scorecard; the MM gauntlet prompt
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** **D-MM1** lens weighting; **D-MM2** whether Spectre builds this suite at all; **D-MM3** the suite's roadmap placement, since no mastering milestone exists; **D-MM4** which loudness standards Spectre implements natively — its *research* half was discharged 2026-08-15, leaving the product question
- **Known gaps:** the benchmark corpus contains **no latency figure for any FabFilter plug-in and almost none for Ozone**, **no time constant for any plug-in**, and **no evidence of loudness-standard conformance for either vendor**. Those three holes are not incidental — they define three of this file's auto-fails. The loudness *standards* are now read and recorded (95 records), which narrowed MM-AF-5 but did not remove it: conformance and delivery targets remain prohibited.

**Criteria version:** 1 (mixing/mastering)

## Authority

This file is **subordinate** to the conflict precedence in `docs/README.md`, exactly as `criteria.md` is. It grades specs; it creates no product authority. Where a spec, this file, and an accepted document disagree, the accepted document wins in `docs/README.md` order: product vision → requirements and decisions → architecture contracts → specs → quality → plans → status → research.

It is a **sibling** of `criteria.md`, not a replacement. `criteria.md` grades the R4 loop. Where both apply, the stricter rule binds; where they conflict on a shared concern (realtime discipline, borrowed numbers, optimistic language), `criteria.md`'s wording governs, because those rules trace to accepted requirements and this file only sharpens their application to a new domain.

**This file grades specs for work that has not been approved.** D-MM2 is open: Spectre has no accepted decision to build a mixing/mastering suite. Nothing here implies one. A criteria file can exist for a proposal — grading a spec is how a proposal gets examined, not how it gets approved.

## The benchmark set

**The AAA bench for this suite is FabFilter and iZotope Ozone 12.** They are the A's in "AAA quality" for mixing and mastering, the way Ableton Live and Serum 2 are for the R4 loop.

Grade against the research corpus in `docs/02-reference-research/`, citing by observation ID (`OBS-FF-*`, `OBS-OZ-*`) or source-ledger entry — **never from memory**. Both dossiers are `in-review` and neither is accepted for product planning; see `fabfilter.md` and `ozone.md` for their acceptance blockers.

### Evidence inventory, verified 2026-08-15

Citable depth is very uneven across the suite's surface, and grading must respect that. This is the count of `OBS-` records available per component. The corpus holds **199 FabFilter records** (196 `OBSERVED`, 3 `SPECTRE-CANDIDATE`) and **179 Ozone records** (169 `OBSERVED`, 10 `SPECTRE-CANDIDATE`); the counts below are per-prefix totals, so a handful are candidates rather than observations, and a `SPECTRE-CANDIDATE` record is **never** citable as benchmark behavior.

| Suite surface | FabFilter | Ozone | Total | Grading consequence |
|---|---|---|---|---|
| Equalization | Pro-Q 4: 64 | `EQ` 11, `DEQ` 8, `MATCH` 5 | **88** | richest surface in the corpus; a thin EQ spec has no excuse |
| Compression | Pro-C 3: 32 | `DYN` 9 | **41** | strong on control set, **zero on time constants** |
| Limiting | Pro-L 2: 27 | `MAX` 17 | **44** | strong on styles and ceiling, **zero on latency** |
| Multiband dynamics | Pro-MB: 26 | `DYN` 9, `XOVER` 4 | **39** | strong on band model and mode tradeoffs; `DYN` is shared with Compression above |
| Saturation / harmonics | Saturn 2: 20 | `EXC` 7 | **27** | moderate; Saturn 2 is the only source for slot-owned modulation |
| De-essing | Pro-DS: 14 | — | **14** | **FabFilter only** — Ozone's module list contains no de-esser |
| Stereo imaging | — | `IMG` 7 | **7** | **Ozone only** — thin; name the gap rather than filling it |
| Metering & analysis | Pro-L 2 loudness, Pro-DS metering | `METER` 12, `SPEC` 7 | **~30** | moderate in count, **null on standards conformance** |
| Chain & routing | — | `CHAIN` 8, `IO` 7 | **15** | **Ozone only**; the chain architecture is its distinctive contribution |
| Referencing / A-B | — | `REF` 7 | **7** | **Ozone only** |
| Dither & export | Pro-L 2 dither | `DITH` 10, `CODEC` 5 | **~18** | moderate |
| Assistive / auto-chain | — | `MA` 17 | **17** | **Ozone only; model internals excluded by mandate**, not merely unresearched |
| Presets & state | — | `PRESET` 9, `STATE` 6 | **15** | **Ozone only**, and the guide never states what a preset contains |
| Specialty modules | — | `LEF` 5, `REBAL` 6, `STAB` 8 | **19** | Ozone-only; out of any plausible first scope |
| Reverb | Pro-R 2: 16 | — | **16** | **not a mixing/mastering surface**; present in the corpus for its import pattern only |
| **Latency, any device** | **0** | near-0 | **~0** | see MM-AF-6 |
| **Time constants, any device** | **0** | 0 | **0** | no attack/release/hold range exists anywhere in the corpus |
| **Loudness-standard conformance** | 0 | **0** (`GAP-OZONE-0001`) | **0 from vendors** | neither vendor evidences conformance; see MM-AF-5 |
| Loudness standards themselves | — | — | **95** (`loudness-standards-observations.md`) | added 2026-08-15 from BS.1770-5, R 128, Tech 3341, Tech 3342 — **a separate evidence class from the vendor corpus, never interchangeable with it** |

**The rule this creates.** A spec cites the benchmark evidence that exists and **names the gap where it does not**. "No citable evidence for stereo imaging beyond `OBS-OZ-IMG-001`–`007`; recorded as a research need" is a **3**, not a penalty. Inventing the hole's contents is the failure. This mirrors `criteria.md`'s handling of Logic Pro's zero records, and it applies with more force here, because three of the holes are exactly the numbers a spec most wants to state.

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
4. **Feasibility rule.** The reviewer reads every source path the spec cites and confirms the claim against the file, and reads every `OBS-` ID the spec cites and confirms the record says what the spec says it says. A spec whose §7.1 misdescribes current state fails regardless of composite.

Reviewers are blind: they grade the finished spec, never the author's reasoning.

**Weights (D-MM1, open):** Realtime & Signal Correctness 30%, Mastering Depth & Metering Truth 25%, Product Identity & Scope Discipline 20%, Truthfulness & Evidence 25%.

Truthfulness carries 25% here against 20% in `criteria.md` for one reason: this domain's characteristic failure is not a bad design, it is a **confident false number** — a loudness target, a latency figure, a time constant, a standards conformance claim. The corpus contains none of those, so every one that appears in a spec was invented somewhere.

---

## Auto-fail rules

These override all scoring. `MM-AF-1` through `MM-AF-4` and `MM-AF-8` are inherited from `criteria.md` and restated in domain terms; `MM-AF-5`, `MM-AF-6`, and `MM-AF-7` are new to this suite.

**MM-AF-1 — Contradicting accepted authority.**
A spec that contradicts an Accepted row in `decision-gates.md` or an accepted requirement in `requirements-ledger.md` fails, unless it explicitly flags the conflict, proposes supersession, and logs it in `decisions-needed.md`.
*Source: `docs/README.md` conflict precedence.*

**MM-AF-2 — Unbacked implementation claims.**
A spec that describes code as existing, partial, or implemented without a source path that actually contains it fails. §7.1 must distinguish implemented / prototyped / planned / gated / absent, each checkable in one `Read`.
*Source: `vision.md` release bar "honest telemetry, no fake surfaces"; `STATUS.md` "claims here link to live evidence".*

**MM-AF-3 — Realtime discipline violation.**
Allocation, deallocation, blocking locks, I/O, logging, or a panic across the callback boundary on a callback-reachable path fails. Bounded wait-free structures with a defined overflow policy and off-thread reclamation are mandatory for control↔render traffic. Denormals flush; NaN/Inf isolate the node and output silence, never noise.
*Source: RT-001, RT-002, RT-003; decision 21.*

> This bites harder in this suite than in R4. Analyzers, loudness meters, and gain-reduction histories all want to accumulate history and publish it to a UI. Every one of those is a callback-reachable path that must publish off-thread through bounded structures — the pattern `BridgeTelemetry` already establishes — not a `Vec` that grows on the render thread.

**MM-AF-4 — Borrowed numeric limits.**
Any numeric bound taken from a reference product without its own rationale row fails.
*Source: decision 16 (standing rule); PROD-003.*

> The specific trap here is that the corpus's *documented* numbers are the tempting ones: Pro-Q 4's ±30 dB and 10 Hz–30 kHz, Saturn 2's ±36 dB and 8×/32× oversampling, Pro-DS's 2–20 kHz side-chain range and 15 ms lookahead, Pro-R 2's 25–400 % and 0–500 ms. Every one of these is a vendor's choice with a vendor's rationale. Reproducing any of them without an independent Spectre rationale row in `requirements-ledger.md` fails, **even when the number is obviously reasonable.**

**MM-AF-5 — Fabricated loudness or standards claim.** *(new; narrowed 2026-08-15)*
A spec fails if it states a loudness, true-peak, or loudness-range fact without citing a record that actually establishes it.

**This rule was narrowed on 2026-08-15 and the narrowing is load-bearing.** When it was written, `SRC-ITU-BS1770-5` was `inventory-only` and no EBU record existed, so the rule was a near-total prohibition. A standards reading pass then read ITU-R BS.1770-5 Annexes 1 and 2, EBU R 128, EBU Tech 3341, and EBU Tech 3342 in full, producing 95 records in `docs/02-reference-research/loudness-standards-observations.md` and four `claims-extracted` ledger records. **Four of this rule's six original prohibitions are lifted. Two stand, and they are the two that matter most.**

**Now citable — cite the record, do not restate from memory:**

| Fact | Cite |
|---|---|
| K-weighting cascade and its exact coefficients | `OBS-BS1770-003/004/005` |
| Gating block geometry (400 ms, 75% overlap) and two-stage gating | `OBS-BS1770-014/015/017`, `OBS-T3341-008` |
| True-peak method (−12.04 dB attenuation, 4× oversample, LPF) and tolerances | `OBS-BS1770-019/020/024`, `OBS-T3341-018`, `OBS-R128-007` |
| LRA — 3 s window, −70 LUFS absolute and −20 LU relative gates, 10th-to-95th percentile | `OBS-T3342-005/006/007` |

**Still prohibited, and these two are absolute:**

1. **Any conformance claim.** "Conforms to", "implements", "is compliant with", "EBU Mode", or any equivalent, for any standard. **Reading a specification is not passing it.** Both EBU documents gate compliance on test-signal sets hosted separately that were never retrieved (`GAP-LOUDNESS-0008`), and Tech 3341 §2.9 and Tech 3342 §4 each state that passing even those does not imply full accuracy (`OBS-XSTD-004`). A spec may say Spectre *measures per the algorithm in* BS.1770-5. It may not say Spectre *conforms to* it.
2. **Any streaming or mastering delivery target.** No streaming-platform target exists in any of the four documents and no platform is named in any of them (`GAP-LOUDNESS-0003`). The single citable target is R 128's **broadcast** −23.0 LUFS / −1 dBTP, and `OBS-R128-014` exists specifically to flag the temptation to launder a broadcast recommendation into a general mastering target. A spec that writes "−14 LUFS for streaming" fails, cited or not, because no retrieved source contains it.

**Two further traps, each scoring 1 or lower on 4C if mishandled:**

- **Oversampling factor.** `GAP-LOUDNESS-0004`: no factor is mandated. Annex 2 is titled "Guidelines", the recommends clause permits any method with "similar or superior results", and 4× is a worked example at 48 kHz — not a floor. A spec calling 4× "required" misstates its source.
- **Sample rate.** `GAP-LOUDNESS-0001`: **BS.1770-5 supplies K-weighting coefficients for 48 kHz only**, and says other rates need values "chosen to provide the same frequency response" while supplying neither the values, a derivation procedure, nor a tolerance. A DAW runs at whatever rate the device is set to. A spec that presents 48 kHz coefficients as universal fails; one that names this as an original Spectre engineering decision requiring its own rationale row scores 3.

*Source: `loudness-standards-observations.md`; `ozone.md` §"The line between vendor behavior and public standards"; `external-reference-register.md` §"Public standards".*

**MM-AF-6 — Undeclared latency.** *(new)*
Any device or mode that introduces latency — linear phase, oversampling, lookahead, FFT analysis, or block-based crossover — fails unless the spec states:

1. the latency in **samples**, derived from the spec's own design rather than a vendor's;
2. how it is **reported** to the host and to the graph; and
3. how it is **compensated** across parallel paths.

A qualitative statement — "introduces some latency", "adds a small delay" — is a **0** on criterion 1D and triggers this auto-fail.

**This is where Spectre must exceed the benchmark rather than match it.** The evidence inventory records zero latency figures across all seven FabFilter plug-ins and near-zero across Ozone. Both vendors describe latency as a consequence and never as a number. A spec may therefore **not** cite a benchmark for a latency figure — there is none to cite — and must derive its own. Naming this as a place Spectre is deliberately more honest than the AAA bench is a **3** on criterion 2F.

**MM-AF-7 — Algorithm reconstruction or DSP non-originality.** *(new)*
A spec fails if it reconstructs, reverse-engineers, or approximates a vendor's proprietary processing, or presents vendor-derived processing as a Spectre design. Specifically prohibited:

- describing a Spectre algorithm as "matching", "modeling", "emulating", or "equivalent to" a named vendor's;
- specifying behavior for an iZotope IRC limiting mode, Master Assistant's model, or any FabFilter internal;
- naming a Spectre device after a vendor's device, style, or mode (no "Pro-", no "IRC", no "Maximizer");
- transcribing a vendor's control *set* as a Spectre control set, even where each individual name is generic.

Documented public **behavior** is a legitimate research input. The **means** is not, and neither is the identity.
*Source: `external-reference-register.md` §"Legal and original-design boundary"; `ozone.md` — DSP and ML internals are out of bounds by mandate; criterion 3D.*

**MM-AF-8 — Optimistic language.**
Unevidenced or promotional phrasing about state fails. `STATUS.md` names this prohibition in its own header and it binds specs describing that state.

**Inherited: AF-5 from `criteria.md`** — conclusions the workflow field study's evidence cannot support (a final native-device list, a default shortcut map, a gesture-count or time budget, a monitoring-latency threshold, platform/backend order, command-frequency or feature-priority scores, promotion of any workflow archetype) remain auto-fails here without restatement.

---

## Lens 1: Realtime & Signal Correctness (weight: 30%)

**Standard:** `requirements-ledger.md` §RT and the architecture contracts in `docs/03-architecture/`.

**1A. Callback-path discipline.** Every path reachable from the audio callback is identified and shown allocation-free, lock-free, and non-blocking, and the spec names which RT-001 guard covers it. Analyzer and meter history accumulate in preallocated storage and publish off-thread.

> **The known collision, and the sharpest test of this criterion.** `OBS-T3341-020` records that EBU Tech 3341 §2.3 requires Integrated Loudness to be **recomputed from stored per-block loudness history on every update** — the measurement is not incrementally summarisable, because the relative gate moves as the programme proceeds and retroactively changes which past blocks count. At the mandated 100 ms block hop that is 36,000 entries per hour of monotonically growing state. **That cannot live on the audio thread under RT-001 and RT-002.**
>
> This is the single most likely place a mastering spec violates realtime discipline while believing it is being rigorous, because the standard and the realtime contract genuinely pull in opposite directions. A spec scores 3 by resolving it explicitly — history off-thread, or a bounded approximation such as a histogram — **and by declaring the approximation a deviation rather than describing it as conformance** (MM-AF-5). A spec that silently accumulates history on the render thread fails AF-3 outright.

**1B. Control↔render communication.** Control traffic crosses via the accepted split-lane RT-002 transport, with lane, overflow behavior, and reclamation stated. Latest-wins for parameters; strict FIFO with counted overflow for anything ordered.

**1C. Numerical containment.** Denormal flush and NaN/Inf isolation per RT-003 are addressed for every new DSP node, with injection tests named. **Weight this heavily here:** feedback paths, high-Q filters, deep limiting, and long reverb-style tails are the classic denormal generators, and a saturation stage is the classic NaN source.

**1D. Latency: declared, reported, compensated.** The spec states latency in samples for every mode, how it reaches the host and the graph, and how parallel paths stay aligned. See MM-AF-6. A qualitative statement scores 0.

**1E. Gain staging and unity.** The spec states what unity means for the device, whether bypass is gain-matched, and whether any automatic compensation exists. Where compensation exists it must be specified as behavior, not implied — the two benchmarks both do this (`OBS-FF-SAT-006`, `OBS-FF-PROR-012`, `OBS-OZ-IO-003`) and it is a common source of silent level drift.

**1F. Determinism.** Identical input yields identical output. Where the spec claims live/offline equivalence it names the comparison method — the existing FNV-1a hash walk in `spectre-offline` and `bridge`, not a new one.

**1G. Failure behavior.** Refusals are explicit and fail closed — silence rather than stale audio, counted rather than silently dropped, surfaced off-thread rather than logged on the callback.

**1H. Test specification.** §5 names commands that actually run and assertions that would fail if the behavior regressed. A test that cannot fail scores 0. For any metering or analysis surface, at least one test must assert the **reported number** against a synthesized signal with a known answer.

---

## Lens 2: Mastering Depth & Metering Truth (weight: 25%)

**Standard:** the FabFilter and Ozone corpora, cited by ID; and the vision's release bar on honest telemetry.

**2A. Benchmark evidence discipline.** Every benchmark claim carries an `OBS-` ID or ledger entry. Claims about a surface with no citable record are named as gaps, not filled in from recollection. See the evidence inventory.

**2B. Detection versus application.** Where a device detects and then acts, the spec separates the two and says whether the user can hear what the detector hears. Both benchmarks converge on this (`OBS-FF-PRODS-002`/`-003`, `OBS-FF-PROMB-013`, `OBS-FF-PROC-019`); a spec that fuses them scores 1 or lower unless it says why.

**2C. Meters report, they do not act.** Every meter, indicator, and analyzer states exactly what signal it shows and at what point in the chain, and whether it modifies audio. Pro-DS is the model: its clip indicator fires above 0 dBFS and the help states outright that the audio is not clipped (`OBS-FF-PRODS-013`), and its analyzer shows the **post-filter** detector signal, not the device input (`OBS-FF-PRODS-012`). A meter whose displayed quantity is ambiguous scores 1 or lower.

**2D. Mode scope is stated.** Where a device offers a phase, quality, or oversampling mode that applies to some stages and not others, the spec says which stages. Saturn 2 is the benchmark and the corpus's best documentation-honesty example: Linear Phase applies to the crossover and oversampling, and explicitly **not** to tone EQ or amp modeling (`OBS-FF-SAT-018`). A global-sounding label over a partial guarantee scores 1 or lower.

**2E. Chain architecture.** Where the spec touches the mastering chain, it respects that the chain is a user-built, user-ordered graph rather than a fixed strip (`OBS-OZ-CHAIN-003`, `-004`), and it states how the suite's devices compose with Spectre's existing `EditableGraph`/`CompiledPlan` split rather than introducing a second chain concept.

**2F. Differentiation, including where Spectre is more honest.** The spec says what Spectre does that the benchmarks do not, without claiming parity as completeness. Declaring latency in samples where neither vendor declares any is the clearest available example and scores well here.

**2G. Convergent-pattern grounding.** Where both benchmarks converge on a pattern — separated detection, absorbed gain consequences, momentary press-and-hold states, exclusive solo on a modifier — the spec either follows it or states why Spectre diverges.

---

## Lens 3: Product Identity & Scope Discipline (weight: 20%)

**Standard:** `docs/00-product/vision.md`, `docs/06-plans/rebuild-roadmap.md`, and D-MM2/D-MM3.

**3A. Roadmap honesty.** **No mastering milestone exists.** The roadmap runs R0–R12 and contains no mixing/mastering suite; the nearest homes are R6 (which already owns meters, sends/returns, compensation, and a latency matrix) and R11 (which owns the effect catalog). A spec must state where its work lands and that the placement is open under D-MM3. A spec that writes itself into R4 fails 3A outright — R4 is track→master, one MIDI clip, one small synth and effect, save/reload, and bounce, and decision 15 keeps those deliberately small.

**3B. Non-goal respect.** No CLAP/LV2/AU hosting, no plugin-format authoring of first-party devices, no cross-DAW preset or project compatibility, no cloud services or content stores, no video scoring. Proposing one is an automatic 0.

**3C. Infrastructure before devices.** Metering, analysis, latency declaration and compensation, oversampling, and crossover infrastructure are shared and must exist before or alongside the devices that need them. A spec that builds a limiter with its own private loudness meter, or a multiband device with its own private crossover, scores 1 or lower — that is how a suite becomes seven incompatible devices.

**3D. Originality.** Original code, DSP, names, control layouts, and formats. Reference research informs; it is never transcribed. This is the qualitative partner to MM-AF-4 and MM-AF-7.

**3E. Platform commitment.** macOS and Linux are co-first-class (decision 1). Any SIMD, FFT, or vector-math dependency states its Linux path. Decision 23's undischarged Linux debt makes this live, not hypothetical.

**3F. Accessibility trajectory.** Keyboard-complete operation and screen-reader labels are a beta gate (decision 17). A mastering suite is the hardest case in the product — analyzers, drag-driven curves, and multi-band displays are exactly what forecloses non-visual operation. The spec need not deliver accessibility; it must not design a surface where the only way to read a value or move a band is with a mouse on a graph.

---

## Lens 4: Truthfulness & Evidence (weight: 25%)

**Standard:** `docs/status/STATUS.md`, whose header prohibits optimistic language, and the metadata discipline in `docs/README.md`.

**4A. Current-state accuracy.** §7.1 matches what the source files actually contain, verified by the reviewer reading them. Today that means: Spectre has **no** mixing/mastering device, no metering beyond `BridgeTelemetry`'s counters, no analyzer, no oversampling, no crossover, no latency declaration, and no loudness measurement of any kind. A §7.1 that implies otherwise fails.

**4B. Numbers carry rationale.** Every numeric bound has its own rationale row scheduled into `requirements-ledger.md` in §7.2's modified-files list — not merely argued in prose. PROD-003 requires the row to exist *in the ledger*, and decision 16 makes it standing.

**4C. Standards claims are ledger-backed.** See MM-AF-5. A loudness or true-peak claim without a body-level standards record fails; naming the gap scores 3.

**4D. Status vocabulary.** Claims use `docs/README.md`'s vocabulary — `draft`, `proposed`, `accepted`, `implemented`, `verified`, `superseded`, `archived` — with the `implemented`/`verified` distinction respected: code existing is not evidence passing.

**4E. Traceability.** Behavioral claims cite a requirement ID, decision row, observation ID, or source path. Uncited normative claims score 1 or lower.

**4F. Honest gaps.** §7.2 and §8 state what is unknown or blocked rather than papering over it. Given that three whole categories of number are absent from the corpus, a mastering spec with no open questions is **suspect, not strong**.

**4G. Evidence commands.** Verification commands are real, runnable, and named exactly — the workspace gate is `cargo fmt --all -- --check`, `cargo clippy --locked --workspace --all-targets -- -D warnings`, `cargo test --locked --workspace`.

---

## The suite's component tree

**Proposed, not accepted.** This is the decomposition the criteria assume when grading "does this spec fit the suite". It is derived from the two benchmarks' *surface*, not their internals, and every node is a Spectre-original design question.

### Tier 0 — shared infrastructure (must precede the devices)

| ID | Component | Why it is shared | Benchmark evidence |
|---|---|---|---|
| `MM-0.1` | Metering core — peak, RMS, true peak, loudness | Every device and the chain need one definition of "level", or meters disagree | ~30 records; **null on standards** |
| `MM-0.2` | Spectrum analysis core | EQ, multiband, de-esser, and the chain view all need one analyzer | `OBS-OZ-SPEC-*` (7), Pro-Q 4 analyzer records |
| `MM-0.3` | Latency declaration and compensation | Nothing else in the tier is correct without it; see MM-AF-6 | **~0 records — Spectre must derive its own** |
| `MM-0.4` | Device I/O and gain-staging contract | Unity, bypass, and gain matching must mean one thing suite-wide | `OBS-OZ-IO-*` (7), `OBS-FF-SAT-006`, `OBS-FF-PROR-012` |
| `MM-0.5` | Oversampling seam | Shared by saturation, limiting, and dynamics | `OBS-FF-SAT-017` (8×/32×), `OBS-FF-PRODS-008` (4×) |
| `MM-0.6` | Crossover / multiband infrastructure | Shared by multiband dynamics, saturation, and de-essing | `OBS-FF-PROMB-*`, `OBS-OZ-XOVER-*` (4) |
| `MM-0.7` | Device preset and state contract | Two benchmarks and neither documents what a preset contains | `OBS-OZ-PRESET-*` (9), `GAP-OZONE-0040`/`0041` |

### Tier 1 — devices

| ID | Device | First-scope note | Evidence depth |
|---|---|---|---|
| `MM-1` | Equalizer | richest evidence; the natural first device | 88 |
| `MM-2` | Compressor | strong on controls, **zero on time constants** | 41 |
| `MM-3` | Limiter | needs `MM-0.1` and `MM-0.3` first | 44 |
| `MM-4` | Multiband dynamics | needs `MM-0.6`; composition of `MM-2` | 39 |
| `MM-5` | De-esser | FabFilter-only evidence; smallest useful device | 14 |
| `MM-6` | Saturation | needs `MM-0.5`; Spectre already ships a `Saturator` seed | 27 |
| `MM-7` | Stereo imaging | thinnest device evidence in the corpus | 7 |

### Tier 2 — chain-level surfaces

| ID | Surface | Note | Evidence depth |
|---|---|---|---|
| `MM-8` | Metering and analysis surface | the user-facing half of `MM-0.1`/`MM-0.2` | ~30 |
| `MM-9` | Mastering chain container | must compose with `EditableGraph`/`CompiledPlan`, not fork it | 15 |
| `MM-10` | Referencing / A-B | Ozone-only evidence | 7 |
| `MM-11` | Dither and export path | pairs with R4-8's offline bounce | ~18 |

**Deliberately excluded from the tree:** reverb (not a mastering surface; Pro-R 2 is in the corpus for its import pattern only), assistive auto-chain (`OBS-OZ-MA-*` — the pattern is interesting, the internals are excluded by mandate, and it presupposes the whole tier below it), and Ozone's specialty modules (Low End Focus, Master Rebalance, Stabilizer). Proposing any of these in a first scope scores 1 or lower on 3A.

## Roadmap placement

**There is no mastering milestone, and this file does not create one.** `rebuild-roadmap.md` runs R0 through R12 with no mixing/mastering entry. The honest reading of the existing roadmap:

- **Tier 0 infrastructure has a natural home at R6** (tracks/routing/mixer), which already owns monitoring, compensation, meters, and a latency matrix as its exit gate. `MM-0.3` in particular is arguably *already* R6 work under a different name.
- **Tier 1 devices have a natural home at or after R11** (Spectre identity), which owns the effect catalog.
- **Tier 2 chain surfaces straddle both** and cannot be placed until D-MM2 resolves.
- **Nothing in this tree belongs in R4.** R4 is a credible alpha: one track to master, one MIDI clip, one small synth and one small effect, save/reload, bounce. Decision 15 keeps the first devices deliberately small.

This leaves three real options, and choosing among them is **D-MM3**, which needs Jeff:

1. **Distribute** — Tier 0 into R6, Tier 1 into R11. No new milestone; the suite is never a coherent deliverable.
2. **Insert a milestone** — a new R6.5 or R11.5 that owns the suite end to end. Coherent, but it renumbers the roadmap.
3. **Defer entirely** — record the tree and the criteria, build nothing until R11 arrives and the question is live.

Option 3 is the honest default while D-MM2 is open, and it costs nothing: this file and the two dossiers are the durable artifact, and they keep their value under any of the three.

---

## Scoring summary

| Lens | Criteria | Weight | Auto-fail conditions |
|---|---|---|---|
| 1 — Realtime & Signal Correctness | 8 | 30% | MM-AF-3, MM-AF-6 |
| 2 — Mastering Depth & Metering Truth | 7 | 25% | inherited AF-5 |
| 3 — Product Identity & Scope Discipline | 6 | 20% | MM-AF-4, MM-AF-7; 3B = 0 fails the spec |
| 4 — Truthfulness & Evidence | 7 | 25% | MM-AF-2, MM-AF-5, MM-AF-8 |

Weights sum to 100%. MM-AF-1 applies across all four lenses.
