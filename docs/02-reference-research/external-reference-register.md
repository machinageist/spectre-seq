<!--
Author: Jeff
Date: 2026-07-11
Description: Classified register of external software references in the Spectre repository
Notes: Classification controls research scope; it does not approve parity, compatibility, or implementation requirements
-->

# External Reference Register

- **Status:** draft
- **Last verified:** 2026-07-11
- **Scope:** commercial/open-source DAWs, synths, modular systems, plugin formats, frameworks, and creative tools named in repository material
- **Decision authority:** Jeff
- **Upstream sources:** repository-wide reference audit, existing clean-room dossiers, legacy plans/specs/ADRs
- **Downstream dependents:** clean-room research documents, workflow field study, requirements provenance, trademark policy
- **Supersedes:** implicit parity/compatibility implications in legacy plans and specs
- **Superseded by:** none
- **Open decisions:** final product scope derived from research; supported VST3 platform/version matrix
- **Known gaps:** current official public sources have not yet been re-fetched or version-verified under this mandate

## Classification vocabulary

1. **substantive behavioral reference:** concrete workflows/capabilities are research inputs; requires a bounded, source-traceable reference dossier.
2. **bounded subsystem reference:** only a named workflow or subsystem is relevant; exhaustive product coverage is unnecessary.
3. **design anti-reference:** clarifies what Spectre should avoid; no parity requirement follows.
4. **compatibility target:** actual interoperability is intended and must have a version/platform test matrix.
5. **incidental/historical mention:** no product requirement may derive from the mention.

A product may have more than one classification when contexts differ. Each adopted behavior still requires a Spectre requirement ID and product rationale; classification alone does not approve it.

## Claim tags for all new research

- `OBSERVED`: directly supported by an identified public source section.
- `SPECTRE-CANDIDATE`: possible Spectre product requirement, not approved.
- `SPECTRE-REQ`: accepted requirement with stable ID and authority.
- `IMPL-DECISION`: original Spectre architecture or implementation choice linked to an ADR/spec.
- `REFERENCE-ONLY`: comparison/inspiration without parity commitment.
- `EXCLUDED`: behavior/content/identity intentionally outside scope.
- `SOURCE-GAP`: public source does not establish the detail.
- `METRIC`: corpus metadata only; never correctness evidence.

## DAW and workflow references

| Product | Classification | Repository context | Required treatment |
|---|---|---|---|
| Ableton Live | substantive behavioral reference; bounded VCV interoperability observation | dedicated dossier, UI inspiration, production positioning, VCV host-routing notes | rebuild versioned dossier and field-workflow corpus; no UI cloning or switcher-only identity |
| Bitwig Studio | substantive behavioral reference | dedicated dossier covering DAW, modulation, launcher, routing, Grid | rebuild versioned dossier; separate Grid observations from Spectre graph decisions |
| Bitwig Grid | bounded subsystem reference | modular/polyphonic signal and sound-design behavior | section-level source matrix; no inferred implementation reuse |
| FL Studio | substantive workflow reference for field study; current repository mention otherwise incidental | old prompt grouping and VCV supported-host list | create workflow/reference dossier from current official sources; do not infer requirements from host list |
| REAPER | substantive workflow reference for field study; current repository mention otherwise incidental | old prompt grouping and VCV supported-host list | research editing, routing, actions/customization; no parity target |
| Logic Pro | substantive workflow reference for field study; bounded VCV interoperability observation | old prompt/grouping, architecture comparison, VCV routing notes | create current official-source dossier; keep VCV routing observation bounded |
| Cubase | substantive workflow reference for field study; current repository mention otherwise incidental | old prompt grouping and VCV supported-host list | create current official-source dossier, especially recording/editing/scoring workflows if in Spectre scope |
| Reason | incidental/historical mention | VCV supported-host list | no dedicated dossier unless future workflows adopt it |
| Harrison Mixbus | incidental/historical mention | VCV supported-host list | no dedicated dossier |
| Studio One | incidental/historical mention | VCV supported-host list | no dedicated dossier unless later workflow evidence justifies it |
| Cakewalk | incidental/historical mention | VCV supported-host list | no dedicated dossier |
| GarageBand | incidental/historical mention | VCV supported-host list | no dedicated dossier |

The VCV supported-host list is an observation about VCV Rack Pro, not an endorsement or compatibility target for Spectre.

## Synth and modular references

| Product | Classification | Repository context | Required treatment |
|---|---|---|---|
| VCV Rack | substantive behavioral reference | `docs/modular_rack_spec.md`; modular routing change specs | rebuild dossier with claim-level source anchors; separate volts/polyphony/limits from Spectre decisions |
| Kilohearts Phase Plant | substantive behavioral reference | Spectre modular synth spec/plan | rebuild dossier; exact macros, lanes, and module limits remain candidates, not commitments |
| Kilohearts Snapins | bounded subsystem reference | Phase Plant device/effect-chain behavior | cover only chain/nesting/modulation workflows relevant to Spectre |
| Xfer Serum 2 | substantive behavioral reference | dedicated dossier and sound-design change material | source scope remains incomplete without full official guide; preserve extensive gaps |
| Xfer Serum 1 | bounded historical/source-continuity reference | older support articles used in Serum 2 dossier | claims require proof they remain valid in Serum 2 |
| Max for Live | design anti-reference / excluded compatibility scope | Ableton dossier exclusion | study workflow implications only where relevant; no device/runtime compatibility target |

## Mixing and mastering device references

Classifications in this table are **proposed, not accepted** — they are the open decision carried by `fabfilter.md` and `ozone.md`. See those dossiers for the source matrices and acceptance blockers.

| Product | Proposed classification | Repository context | Required treatment |
|---|---|---|---|
| FabFilter Pro-Q 4 | substantive behavioral reference | `fabfilter.md`, `fabfilter-observations.md` (64 records) | deepest EQ reference and the only plug-in with systematic numeric ranges; re-read numerals from the vendor PDF before any promotion |
| FabFilter Pro-C 3 | bounded subsystem reference | dynamics observations (32 records) | compressor control set, styles, side chain only; no time constants exist in the source |
| FabFilter Pro-L 2 | bounded subsystem reference | limiting/loudness observations (27 records) | limiting styles, true peak, loudness metering; no latency figure exists in the source |
| FabFilter Pro-MB | bounded subsystem reference | multiband dynamics observations (26 records) | band model, processing-mode tradeoffs, stereo link |
| FabFilter Pro-DS | bounded subsystem reference | de-esser observations (14 records) | detection/application split, split-band topology, metering honesty properties |
| FabFilter Saturn 2 | bounded subsystem reference | saturation observations (20 records) | band split model, slot-owned modulation, explicitly partial linear-phase scope |
| FabFilter Pro-R 2 | bounded subsystem reference | reverb observations (16 records) | Spectre has no reverb; the value is the parameter-converting import pattern, not reverb behavior |
| FabFilter plug-in range (suite) | design reference for interaction consistency only | recurring modifier vocabulary across plug-ins | no parity requirement, no UI-composition license; the legal boundary below applies in full |
| iZotope Ozone 12 (mothership) | substantive behavioral reference | `ozone.md`, `ozone-observations.md` (179 records, 22 areas) | chain/module architecture and channel-processing-mode model; no numeric limit promoted without its own rationale |
| Ozone Master Assistant | bounded subsystem reference | assistive-workflow observations | pattern only — objective in, editable chain out; **model internals are excluded by mandate**, not merely unresearched |
| Ozone component plug-ins | bounded subsystem reference | mothership/component split | the split itself is the observation; components add no separate behavioral surface |

## Public standards

A standard is **not** a product reference. It carries no parity question, no originality concern, and no trademark constraint — it is a specification Spectre may choose to implement, and implementing it means reading it. This row class is new and is proposed with the tables above.

| Standard | Classification | Repository context | Required treatment |
|---|---|---|---|
| ITU-R BS.1770-5 (11/2023) | public standard — implementable specification | `SRC-ITU-BS1770-5`, **`claims-extracted`**; loudness and true-peak measurement | body read 2026-08-15 from the official ITU PDF. Annex 1 and Annex 2 normative text read in full — K-weighting coefficients, channel weights, gating thresholds, block/overlap structure, LKFS designation, true-peak stages — plus Annex 3/4 and the informative Attachment 1 to Annex 2. Claims live in `loudness-standards-observations.md` as `OBS-BS1770-*`. **Not read:** Attachment 1 to Annex 4; Attachment 1 to Annex 1 §5 onward; all figures (raster, unreadable). Coefficients are **48 kHz only** — `GAP-LOUDNESS-0001` |
| EBU R 128 (V5, 11/2023) | public standard — implementable specification | `SRC-EBU-R128`, **`claims-extracted`**; loudness normalisation and permitted maximum level | body read 2026-08-15, in full — this is the **only** delivery target now citable from this ledger: −23.0 LUFS with a ±1.0 LU practical tolerance, a separate ±0.2 LU workflow tolerance, and −1 dBTP ±0.3 dB in production. Claims are `OBS-R128-*`. It is a **broadcast** recommendation and must never be cited as a streaming target; R 128 s2 (streaming) was not retrieved — `GAP-LOUDNESS-0003`. URL is unversioned, so `mutable_url: true` |
| EBU Tech 3341 (V4, 11/2023) | public standard — implementable specification | `SRC-EBU-TECH3341`, **`claims-extracted`**; "EBU Mode" loudness metering | §1–§2.9 read 2026-08-15 — momentary/short-term/integrated window lengths and update rates, the ban on extra ballistics, gating restatement, scales, display obligations, calibration, and the true-peak tolerance. Claims are `OBS-T3341-*`. **Not read:** §2.10, §3, §4, Table 1 cases past 19. It defines the "EBU +9"/"EBU +18" scale names — which is evidence about **this document only** and never about a vendor UI that reuses the label. "EBU Mode" is a testable claim Spectre cannot yet make — `GAP-LOUDNESS-0008` |
| EBU Tech 3342 (V4, 11/2023) | public standard — implementable specification | `SRC-EBU-TECH3342`, **`claims-extracted`**; Loudness Range (LRA) | §1–§4 read 2026-08-15 — 3 s sliding window, ≥10 Hz sampling, cascaded −70 LUFS absolute and **−20 LU** relative gates, and the 10th-to-95th-percentile definition. Claims are `OBS-T3342-*`. §5's MATLAB reference implementation was **deliberately not extracted**; any Spectre LRA code must derive from the §3.1 text. Its relative gate differs from BS.1770's −10 LU and must not share a code path — `OBS-XSTD-001` |
| ITU-R BS.1771; Report ITU-R BS.2217 | public standard — **no ledger record yet** | cited by Tech 3341 (Requirement PLD-4, IIR ballistics) and by BS.1770-5 NOTE 2 (compliance test material) | open research need — `GAP-LOUDNESS-0011`. Note that the "BS.1771" name appearing as an Ozone meter-scale label refers to a standard this repository has **never read** |

**The binding rule this table creates.** Vendor documentation is not evidence about a standard, and a standard is not evidence about a vendor. `GAP-OZONE-0001` records that the inspected Ozone 12 pages never name BS.1770 at all, and reading the standards changed nothing about that — the four records above describe what a conforming measurement *is*, and this repository still holds no evidence that any vendor implements any of it. Any Spectre loudness number without a body-level standards record is fabricated, regardless of how many vendor observations surround it; and any Spectre *conformance* claim additionally requires test evidence, which `GAP-LOUDNESS-0008` records does not exist here.

## Plugin formats and SDKs

| Technology | Classification | Repository context | Required treatment |
|---|---|---|---|
| VST3 | compatibility target | active third-party host architecture | current official Steinberg docs/license; exact SDK/bindings; per-platform fixture matrix |
| VST2 | incidental/historical and excluded | ADR alternative | no implementation; retain legal/history note only after source verification |
| CLAP | incidental/historical; excluded active architecture | shelved host and stale docs | archive; no active feature work unless Jeff changes product decision |
| LV2 | incidental/historical; excluded active architecture | shelved host and stale docs | archive; no active feature work unless Jeff changes product decision |
| Audio Unit | incidental comparison; excluded current target | broad plugin-format lists | no compatibility claim or implementation scope |

Only VST3 is a compatibility target. Comparable native-device capability is not plugin-format compatibility.

## Frameworks and implementation tools

| Product/tool | Classification | Context | Treatment |
|---|---|---|---|
| JUCE | design anti-reference / incidental implementation comparison | old mega prompt framework non-default | no behavioral parity dossier; architecture alternatives may cite public technical facts |
| Tracktion Engine | design anti-reference / incidental implementation comparison | old mega prompt | no reuse or parity implication |
| Dplug | incidental implementation comparison | old mega prompt | no product requirement |
| iPlug2 | incidental implementation comparison | old mega prompt | no product requirement |
| WebAudio | design anti-reference / incidental implementation comparison | old mega prompt | no product requirement |
| Electron | design anti-reference | old mega prompt | records native-desktop preference only |
| Pure Data | incidental modular comparison | old mega prompt | no requirement without explicit workflow evidence |
| Max | incidental modular comparison | old mega prompt | no requirement without explicit workflow evidence |
| SuperCollider | incidental modular/programming comparison | old mega prompt | no requirement without explicit workflow evidence |
| egui/eframe | implementation technology, not behavioral reference | live UI and placeholder ADR 004 | re-decide through UI/accessibility architecture gate |
| wgpu | implementation candidate, not behavioral reference | placeholder ADR 004 | no accepted migration commitment |
| ArcSwap | implementation candidate, not behavioral reference | placeholder ADR 002 filename | no accepted decision; compare alternatives in future ADR |
| CBOR | implementation candidate, not behavioral reference | current project crate and placeholder ADR 003 | no accepted project-format commitment |

## Existing dossier acceptance status

| Dossier | Current status | Principal blockers |
|---|---|---|
| Ableton Live | draft research only | claim-level locators absent; observations and mappings mixed; mutable sources/version drift |
| Bitwig Studio/Grid | draft research only | page-level rather than claim-level provenance; Spectre architecture mixed into implications |
| VCV Rack | draft research only | observation/inference/requirement/implementation mixed; vendor-derived limits unapproved |
| Phase Plant | draft research only | “exhaustive” unsupported; public limits promoted directly into Spectre commitments |
| Serum 2 | draft research only | no complete official guide in source set; extensive declared gaps; older Serum continuity unproved |
| FabFilter | draft research, `in-review` | 199 records across 7 plug-ins; no point version anywhere; **no latency figure anywhere**; no time constants; most ranges absent outside Pro-Q 4; numerals not spot-verified against the vendor PDFs |
| iZotope Ozone | draft research, `in-review` | 179 records across 22 areas; no guide revision date; latency almost never quantified; edition scoping unresolved; signal-flow diagrams unreadable; loudness-standard conformance unestablished |
| FL Studio | not started | no dedicated dossier |
| REAPER | not started | no dedicated dossier |
| Logic Pro | not started | no dedicated dossier |
| Cubase | not started | no dedicated dossier |

No reference dossier is accepted for product planning yet. FabFilter and Ozone are `in-review` rather than `blocked-source-gap` because, unlike Serum 2, complete official public documentation **is** exposed for both — their limits are vendor disclosure and reading depth, not access.

## Legal and original-design boundary

Research MAY describe publicly documented functional behavior. Spectre MUST NOT copy vendor code, private formats, undocumented protocols, screenshots, icons, artwork, presets, samples, wavetables, factory projects, text passages, naming systems, or distinctive UI composition. Vendor numeric limits and defaults do not become Spectre requirements without an independent rationale and explicit decision.

Trademarks belong in research, compatibility, attribution, and legal contexts—not first-party device names or product UI unless necessary to identify an actual hosted plugin.

## Next research actions

1. Define the source ledger schema and clean-room methodology.
2. Re-fetch current official source inventories and record exact versions/access dates.
3. Convert existing dossiers to claim-tagged, section-located observations.
4. Create missing major-DAW dossiers at the declared bounded scope.
5. Conduct independent musician workflow research and shortcut analysis.
6. Promote only corroborated, selected implications into `SPECTRE-CANDIDATE` entries.
7. Assign stable requirement IDs only after product decision.
8. Run a second-pass consistency and legal/originality review before marking any dossier accepted.
