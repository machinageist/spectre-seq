<!--
Author: Jeff
Date: 2026-08-15
Description: Official-public-source scope, classification proposal, and gap dossier for iZotope Ozone 12 research
Notes: Records where vendor behavior ends and public loudness standards begin; the two are deliberately not merged
-->

# iZotope Ozone Reference Dossier

- **Status:** draft
- **Research state:** in-review
- **Last verified:** 2026-08-15
- **Scope:** publicly documented behavior of iZotope Ozone 12 relevant to a Spectre mastering device suite, as rendered at `docs.izotope.com/ozone12/en/` on 2026-08-15
- **Decision authority:** Jeff
- **Upstream sources:** the 19 `SRC-OZONE12-*` records in `source-ledger.json`; `SRC-ITU-BS1770-5`; `docs/02-reference-research/methodology.md`
- **Downstream dependents:** `ozone-observations.md`; `external-reference-register.md`; any future Spectre mastering device requirements
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** the register classification proposed below; whether Spectre builds a mastering suite at all; which loudness standards Spectre would implement natively
- **Known gaps:** the guide exposes no revision number or publication date; numeric ranges and defaults are absent from most module pages; latency and oversampling factors are almost never quantified; signal-flow diagrams are images and were not readable; edition scoping (Elements/Standard/Advanced) is unresolved

## Completion finding

Ozone is **not** source-complete and must not be treated as an accepted reference.

What exists is a claim-tagged extraction of **179 records** across 22 functional areas in `ozone-observations.md` — 169 tagged `OBSERVED` and 10 `SPECTRE-CANDIDATE` — each anchored to a named guide page, with **41 `GAP-OZONE-*` records** for what the guide does not establish. Coverage is broad across the module surface and thin on numbers: the guide reliably says what a control is called and what it does, and rarely says what range it has.

Like FabFilter and unlike Serum 2, the limiting factor is **vendor disclosure and reading depth**, not access. A complete official guide is public.

## The line between vendor behavior and public standards

This dossier's most important structural rule, and the reason it exists separately from the observations file:

**Nothing in the Ozone corpus is evidence about a loudness standard, and nothing in a loudness standard is evidence about Ozone.**

The inspected Ozone pages **never name ITU-R BS.1770** (`GAP-OZONE-0001`). What appears is `BS.1771` — and only as one of five *meter scale* names alongside "dB (Linear)", "dB (Non-linear)", "EBU +9", and "EBU +18" (`OBS-OZ-METER-005`). Those scales carry display endpoints in LUFS (`OBS-OZ-METER-006`), which are **display ranges, not delivery targets**. The guide states the unit identity 1 LUFS = 1 dB (`OBS-OZ-METER-007`) and offers Target LUFS controls in both the Maximizer and Master Assistant (`OBS-OZ-MAX-013`, `OBS-OZ-MA-008`) — while printing **no streaming-platform target value anywhere** (`GAP-OZONE-0005`).

Three consequences bind any Spectre work that follows:

1. Whether Ozone's momentary / short-term / integrated meters implement BS.1770 gating and K-weighting, and at which revision, is **unestablished** (`GAP-OZONE-0001`, `GAP-OZONE-0002`).
2. **This changed on 2026-08-15 and the change is what makes loudness work possible.** `SRC-ITU-BS1770-5` is now `claims-extracted` — Annexes 1 and 2 were read in full — and `SRC-EBU-R128`, `SRC-EBU-TECH3341`, and `SRC-EBU-TECH3342` were added, all `claims-extracted`. The four documents produced 95 records in `loudness-standards-observations.md`. The algorithm, K-weighting coefficients, gating thresholds, true-peak method, and LRA definition are now citable facts with section anchors.
3. **What is still not citable is conformance.** Reading a specification is not passing it. Both EBU documents gate compliance on test-signal sets hosted separately that were not retrieved (`GAP-LOUDNESS-0008`), and Tech 3341 §2.9 / Tech 3342 §4 both state that passing even those does not imply full accuracy. "Conforms to", "implements", and "is compliant with" remain prohibited.
4. **No streaming-platform target exists in any of the four documents** (`GAP-LOUDNESS-0003`), and no platform is named anywhere in them. The only citable target is R 128's **broadcast** −23.0 LUFS / −1 dBTP, and `OBS-R128-014` flags the temptation to launder a broadcast recommendation into a general mastering target.

Any Spectre document that cites a loudness number without a body-level standards record is making it up. That is what the mixing/mastering criteria's loudness auto-fail exists to catch — and it is now a **narrower** prohibition than it was, because the records exist.

## Source matrix

| Source ID | Source role | What it can support | What it cannot support | State |
|---|---|---|---|---|
| `SRC-OZONE12-DOCS-INDEX` | official guide index | the module inventory and page structure | any behavior | inventory-only |
| `SRC-OZONE12-RELEASE-NOTES-ADV` | official release notes | that 12.1.0 (2025-12-01) is the newest listed build | that the guide describes that build | claims-extracted |
| `SRC-OZONE12-GETTING-STARTED` | official guide | mothership/component split, empty-by-default chain, interface regions, CPU levers | per-module behavior | claims-extracted |
| `SRC-OZONE12-GENERAL-CONTROLS` | official guide | I/O panel, clipping indicators, gain match, channel processing modes | numeric ranges | claims-extracted |
| `SRC-OZONE12-MAXIMIZER` | official guide | limiter control set, Learn Input Gain, Target LUFS | IRC algorithm internals, latency | claims-extracted |
| `SRC-OZONE12-OPTIONS` | official guide | meter scale enumeration and display endpoints, buffer-size options | standards conformance | claims-extracted |
| `SRC-OZONE12-EQUALIZER`, `-DYNAMICS`, `-IMAGER`, `-DYNAMIC-EQ`, `-EXCITER`, `-LOW-END-FOCUS`, `-MASTER-REBALANCE`, `-MATCH-EQ`, `-STABILIZER` | official guide | per-module control sets and documented behavior | ranges, defaults, time constants, filter design | claims-extracted |
| `SRC-OZONE12-MASTER-ASSISTANT` | official guide | the assistive workflow's inputs, objectives, and outputs | the model's internals, which are out of bounds by mandate | claims-extracted |
| `SRC-OZONE12-DITHER`, `-CODEC-PREVIEW`, `-REFERENCING`, `-PRESET-SYSTEM` | official guide | dither options, codec preview, referencing/A-B, preset scopes and locations | preset payload contents, format, migration | claims-extracted |
| `SRC-ITU-BS1770-5` | public standard | K-weighting cascade and coefficients, gating block geometry and two-stage gating, true-peak method | conformance; coefficients at any rate other than 48 kHz (`GAP-LOUDNESS-0001`); channel-combination rule for true peak (`GAP-LOUDNESS-0012`) | claims-extracted |
| `SRC-EBU-R128` | public standard | the −23.0 LUFS broadcast target, the −1 dBTP production limit, the BS.1770 measurement reference | any streaming or mastering target — R 128 is scoped to broadcast programme production (`OBS-R128-014`) | claims-extracted |
| `SRC-EBU-TECH3341` | public standard | EBU Mode meter timing, momentary/short-term/integrated specs, true-peak tolerance +0.2/−0.4 dBTP | an "EBU Mode" claim, which requires the unretrieved test-signal set (`GAP-LOUDNESS-0008`, `OBS-XSTD-004`) | claims-extracted |
| `SRC-EBU-TECH3342` | public standard | the LRA definition — 3 s window, −70 LUFS absolute and −20 LU relative gates, 10th-to-95th percentile | conformance; the §5 MATLAB reference was deliberately not extracted, so Spectre LRA code derives from the §3.1 text |  claims-extracted |

## What the corpus establishes well

Recorded as observed patterns, not approved requirements.

1. **The chain is empty by default and user-ordered.** Ozone ships no prebuilt chain; the user adds modules and sets their order (`OBS-OZ-CHAIN-003`, `-004`). A mastering suite is presented as a graph the user builds, not a fixed strip they bypass.
2. **Channel processing mode is a chain-level concept.** Stereo, Mid/Side, Left/Right, and Transient/Sustain are peers at the chain level rather than per-module features (`OBS-OZ-IO-004`, `-005`). Generalizing "which pair of signals does this module process" beyond stereo geometry is architecturally interesting for Spectre and is recorded as `OBS-OZ-IO-007`, a candidate only.
3. **Assistive features state an objective and then move ordinary controls.** Learn Input Gain analyzes for a bounded window and sets the drive control (`OBS-OZ-MAX-013`); Master Assistant takes a target and produces a chain the user can then edit (`OBS-OZ-MA-*`). The output lands in the parameter space the user already edits — the same property that makes Pro-R 2's IR import worth noting (`OBS-FF-PROR-016`).
4. **Presets exist at two scopes and can contain analysis data.** Chain-level and module-level presets both exist (`OBS-OZ-PRESET-002`), and at least one module stores captured *analysis* inside a preset rather than only parameter values (`OBS-OZ-PRESET-008`) — while the guide never states what a preset contains (`GAP-OZONE-0040`) or its format (`GAP-OZONE-0041`).

## What the corpus cannot establish

- **Loudness standard conformance.** See the section above. This is the single most important boundary in this dossier.
- **Latency.** Almost never quantified, for any module or oversampling setting.
- **Numeric ranges and defaults** for most module controls, and time constants throughout.
- **Edition scoping.** Elements/Standard/Advanced module availability was not enumerated on the inspected pages.
- **Signal flow beyond prose.** The guide's flow and interface diagrams are images; the retrieval tool could not read them and **no record here derives from a diagram**.
- **Anything about DSP or ML internals**, which are out of bounds by mandate — most sharply for the IRC limiting modes and Master Assistant.

## Proposed register classification

For `external-reference-register.md`. **Proposed, not accepted.**

| Product | Proposed classification | Rationale |
|---|---|---|
| iZotope Ozone 12 (mothership) | substantive behavioral reference | 179 records across 22 areas make it the deepest mastering reference available, and its chain/module architecture is directly relevant to Spectre's graph model. |
| Ozone Master Assistant | bounded subsystem reference | Relevant only as an assistive-workflow pattern — objective in, editable chain out. Model internals are excluded by mandate, not merely unresearched. |
| Ozone component plug-ins | bounded subsystem reference | The mothership/component split is itself the observation; individual components add no separate behavioral surface. |
| ITU-R BS.1770 / EBU R 128 | **public standard, not a product reference** | Needs its own row class in the register. A standard is not a competitor and carries no originality or trademark concern; it is a specification Spectre may choose to implement, and implementing it requires reading it. |

No Ozone numeric limit becomes a Spectre limit without an independent product rationale and an explicit decision row (decision 16 / PROD-003). The meter-scale endpoints in `OBS-OZ-METER-006` are the trap: they are display ranges and read like targets.

## Source-gap records

The 41 `GAP-OZONE-*` records live inline in `ozone-observations.md`. The four that most constrain Spectre work:

- `GAP-OZONE-0001`: BS.1770 is never named; conformance of Ozone's loudness meters is unestablished.
- `GAP-OZONE-0002`: absolute (−70 LUFS) and relative (−10 LU) gating, and any LRA readout, are not documented.
- `GAP-OZONE-0005`: no streaming-platform loudness target value appears anywhere, despite Target LUFS controls existing.
- `GAP-OZONE-0040`/`0041`: preset contents, format, and cross-version compatibility are undocumented.

Additionally, at the dossier level:

- `GAP-OZONE-0042`: the guide exposes no revision or publication date, so no claim can be tied to an application build.
- ~~`GAP-OZONE-0043`~~ — **closed 2026-08-15.** EBU R 128 and Tech 3341/3342 now have `claims-extracted` ledger records, and `SRC-ITU-BS1770-5` moved to `claims-extracted` on a body read. The standards side of loudness is covered; see `loudness-standards-observations.md`. Two successor gaps replace it: `GAP-LOUDNESS-0008` (the EBU minimum-requirement test-signal sets were not retrieved, so no conformance claim is possible) and `GAP-LOUDNESS-0001` (BS.1770-5 supplies K-weighting coefficients for 48 kHz only).

## Acceptance blocker

This dossier stays `in-review` until:

1. ~~`SRC-ITU-BS1770-5` moves to `claims-extracted` and EBU records are added~~ — **discharged 2026-08-15.** All four documents were read and recorded; 95 records in `loudness-standards-observations.md`. What remains open is conformance, not coverage; and
2. Jeff accepts or amends the classification proposal, including the new "public standard" row class; and
3. numerals intended for a Spectre decision are re-read directly from the rendered page, discharging the transcription risk the observations file declares.

Until then, no Ozone record may be cited as settled behavior in an accepted Spectre requirement — only in research and in explicitly-tagged `SPECTRE-CANDIDATE` material.
