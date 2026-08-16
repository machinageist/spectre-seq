<!--
Author: Jeff
Date: 2026-08-15
Description: Official-public-source scope, classification proposal, and gap dossier for FabFilter mixing/mastering research
Notes: The online help is the primary surface; PDF manuals were not downloaded and no plug-in exposes a point version
-->

# FabFilter Reference Dossier

- **Status:** draft
- **Research state:** in-review
- **Last verified:** 2026-08-15
- **Scope:** publicly documented behavior of the seven FabFilter mixing/mastering plug-ins relevant to a Spectre device suite — Pro-Q 4, Pro-C 3, Pro-L 2, Pro-MB, Pro-DS, Saturn 2, Pro-R 2
- **Decision authority:** Jeff
- **Upstream sources:** `SRC-FABFILTER-HELP-INDEX`; `SRC-FABFILTER-NEWS-INDEX`; the seven per-plug-in `SRC-FABFILTER-*-HELP` records in `source-ledger.json`; `docs/02-reference-research/methodology.md`
- **Downstream dependents:** `fabfilter-observations.md`; `external-reference-register.md`; any future Spectre mixing/mastering device requirements
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** the register classification proposed below; whether Spectre builds a mixing/mastering device suite at all
- **Known gaps:** no plug-in exposes a point version or manual revision date; three plug-ins carry no generation number; most control ranges, defaults, and every time constant are qualitative in the help; no latency figure is stated for any mode of any plug-in

## Completion finding

FabFilter is **not** source-complete and must not be treated as an accepted reference.

What exists is a claim-tagged extraction of **199 records** across seven plug-ins in `fabfilter-observations.md` — 196 tagged `OBSERVED` and 3 `SPECTRE-CANDIDATE` — each anchored to a named help page, plus a running `GAP-FABFILTER-*` register for what the help does not establish. Per plug-in: Pro-Q 4 64, Pro-C 3 32, Pro-L 2 27, Pro-MB 26, Saturn 2 20, Pro-R 2 16, Pro-DS 14. What does not exist is any version anchor, any latency number, and — for most controls outside Pro-Q 4 — any numeric range at all.

The dossier is `in-review` rather than `blocked-source-gap`: unlike Serum 2, a complete official public manual **is** exposed, and the limits below are limits of *reading depth and vendor disclosure*, not of access.

## Source matrix

| Source ID | Source role | What it can support | What it cannot support | State |
|---|---|---|---|---|
| `SRC-FABFILTER-HELP-INDEX` | official documentation index | which product generation is current, where its documentation lives | any behavior | inventory-only |
| `SRC-FABFILTER-NEWS-INDEX` | official news listing | that a point release was announced and when | what changed in it, or which release the help describes | inventory-only |
| `SRC-FABFILTER-PROQ4-HELP` | official manual | EQ band model, dynamic and spectral EQ, processing modes, immersive channel handling, analyzer and matching, interaction vocabulary | latency in samples, filter design, automation payloads | claims-extracted (19 of 37 pages) |
| `SRC-FABFILTER-PROC3-HELP` | official manual | compressor control set, styles, side chain, metering, oversampling and mix | time-constant ranges, detector design, style internals | claims-extracted (8 of ~30) |
| `SRC-FABFILTER-PROL2-HELP` | official manual | limiting styles, true-peak and ceiling behavior, loudness metering, dither and utility options | algorithm behavior, per-style latency, oversampling latency | claims-extracted (6 of ~28) |
| `SRC-FABFILTER-PROMB-HELP` | official manual | band model, per-band dynamics, processing-mode tradeoffs, stereo-link semantics, display gestures | crossover design, any latency figure, band deletion behavior | claims-extracted (5 of ~28) |
| `SRC-FABFILTER-PRODS-HELP` | official manual | two detection modes, side-chain filter range, full/split-band topology, lookahead ceiling, metering honesty properties | Threshold/Range numerics, per-mode latency, oversampling latency | claims-extracted (4 of 25) |
| `SRC-FABFILTER-SATURN2-HELP` | official manual | band split model, crossover slopes, per-band control set, distortion styles, slot-owned modulation, oversampling factors, partial linear-phase scope | per-control ranges other than Level and I/O, source/slot count limits, latency | claims-extracted (6 of 32) |
| `SRC-FABFILTER-PROR2-HELP` | official manual | single-axis Space control, proportional decay, predelay sync model, both six-band EQs, automatic gain compensation, parameter-converting IR import | Distance/Brightness/Thickness/Ducking ranges, Decay Rate EQ units, IR limits, surround behavior | claims-extracted (5 of 26) |

## What the corpus establishes well

Four things are documented consistently enough across plug-ins to be worth treating as observed patterns rather than single-product facts. Each is a **pattern in the evidence**, not an approved Spectre requirement.

1. **Detection and application are separate stages.** Pro-DS splits threshold from range (`OBS-FF-PRODS-002`, `-003`); Pro-MB and Pro-C 3 expose the filtered, stereo-linked trigger signal for audition (`OBS-FF-PROMB-013`, `OBS-FF-PROC-019`). The user can hear what the detector hears.
2. **Phase and quality modes are scoped explicitly, and the scope is stated.** Pro-Q 4 and Pro-MB name their processing modes with their tradeoffs; Saturn 2 goes furthest and says outright that Linear Phase applies to the crossover and oversampling but **not** to tone EQ or amp modeling (`OBS-FF-SAT-018`).
3. **Gain consequences are absorbed rather than exported.** Saturn 2's Drive auto-compensates output (`OBS-FF-SAT-006`); Pro-R 2 compensates reverb gain for Post EQ edits so Mix need not be re-touched (`OBS-FF-PROR-012`).
4. **One interaction vocabulary spans the range.** The same modifier set — wheel or `Ctrl`/`Cmd`+drag for width, `Shift` to fine-tune, `Alt` to escape an axis constraint, `Ctrl`/`Cmd`+click for exclusive solo, press-and-hold for a momentary state — recurs across Pro-Q 4, Pro-MB, and Saturn 2 (`OBS-FF-PROQ-0xx`, `OBS-FF-PROMB-020`–`022`, `OBS-FF-SAT-005`, `-010`).

## What the corpus cannot establish

- **No latency figure exists anywhere in the inspected corpus.** Every plug-in that offers linear phase, oversampling, or lookahead describes the latency *consequence* qualitatively and never states samples or milliseconds — except Pro-DS's 15 ms lookahead ceiling, which is a control range, not a reported latency. A Spectre latency-reporting requirement therefore cannot cite FabFilter for a number, only for the observation that vendors leave this undisclosed.
- **Most control ranges are absent.** Pro-Q 4 is the exception, not the rule. Saturn 2 states a range for Level and I/O and for nothing else; Pro-DS states none for its two primary knobs.
- **No time constants.** Not one attack, release, or hold range appears in the inspected pages.
- **No preset payload, file format, or state-compatibility rule** for any plug-in.
- **No version anchor.** `GAP-FABFILTER-0001` covers this and applies to every record.

## Proposed register classification

For `external-reference-register.md`. **Proposed, not accepted** — this is the open decision this dossier carries.

| Product | Proposed classification | Rationale |
|---|---|---|
| FabFilter Pro-Q 4 | substantive behavioral reference | The deepest extraction in the corpus (64 records) and the only plug-in with systematic numeric ranges. It is the reference for band models, processing-mode tradeoffs, and analyzer behavior. |
| FabFilter Pro-C 3, Pro-L 2, Pro-MB | bounded subsystem reference | Each is relevant to exactly one Spectre subsystem — dynamics, limiting/loudness, multiband dynamics — and exhaustive product coverage is unnecessary. |
| FabFilter Pro-DS, Saturn 2, Pro-R 2 | bounded subsystem reference | Same, for de-essing, saturation, and reverb. Spectre has no reverb and no convolution, so Pro-R 2's value is the import-and-conversion *pattern* (`OBS-FF-PROR-016`), not reverb behavior. |
| FabFilter plug-in range (as a suite) | design reference for interaction consistency only | The recurring modifier vocabulary is the suite-level observation. It carries **no** parity requirement and no UI-composition license — see the register's legal boundary. |

No FabFilter numeric limit becomes a Spectre limit without an independent product rationale and an explicit decision row. This restates decision 16 / PROD-003 and applies with particular force here, because the ranges that *are* documented (Pro-Q 4's ±30 dB, 10 Hz–30 kHz; Saturn 2's ±36 dB) are exactly the kind of number that gets copied unexamined.

## Source-gap records

- `GAP-FABFILTER-0001`: no point version, build number, or revision date rendered on any help page; all claims scoped to "online help as rendered 2026-08-15".
- `GAP-FABFILTER-0002`: PDF manuals were not downloaded; the online help is the sole inspected surface and may differ from the PDF.
- `GAP-FABFILTER-0003`: no latency figure in samples or milliseconds is stated for any processing mode, oversampling factor, or lookahead setting across all seven plug-ins.
- `GAP-FABFILTER-0004`: no attack, release, hold, or other time-constant range is stated for any plug-in in the inspected pages.
- `GAP-FABFILTER-0005`: no preset payload, file format, or cross-version state-compatibility rule is documented for any plug-in.
- `GAP-FABFILTER-0006`: extraction used assisted page summarization over the rendered HTML; numerals are reproduced as read and are **not** spot-verified against the PDF manuals.
- Per-plug-in gaps are recorded inline in `fabfilter-observations.md` as `GAP-FABFILTER-00xx`.

## Acceptance blocker

This dossier stays `in-review` until:

1. every numeral intended for a Spectre decision is re-read directly from the vendor PDF and the transcription risk in `GAP-FABFILTER-0006` is discharged for that numeral; and
2. Jeff accepts or amends the classification proposal above; and
3. the declared scope is either widened by reading the unreviewed pages the coverage table names, or narrowed in the header to the pages actually read.

Until then, no FabFilter record may be cited as settled behavior in an accepted Spectre requirement — only in research and in explicitly-tagged `SPECTRE-CANDIDATE` material.
