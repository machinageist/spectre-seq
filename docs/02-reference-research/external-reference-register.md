<!--
Author: Jeff
Date: 2026-07-11
Description: Classified register of external software references in the Geist DAW repository
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
3. **design anti-reference:** clarifies what Geist should avoid; no parity requirement follows.
4. **compatibility target:** actual interoperability is intended and must have a version/platform test matrix.
5. **incidental/historical mention:** no product requirement may derive from the mention.

A product may have more than one classification when contexts differ. Each adopted behavior still requires a Geist requirement ID and product rationale; classification alone does not approve it.

## Claim tags for all new research

- `OBSERVED`: directly supported by an identified public source section.
- `GEIST-CANDIDATE`: possible Geist product requirement, not approved.
- `GEIST-REQ`: accepted requirement with stable ID and authority.
- `IMPL-DECISION`: original Geist architecture or implementation choice linked to an ADR/spec.
- `REFERENCE-ONLY`: comparison/inspiration without parity commitment.
- `EXCLUDED`: behavior/content/identity intentionally outside scope.
- `SOURCE-GAP`: public source does not establish the detail.
- `METRIC`: corpus metadata only; never correctness evidence.

## DAW and workflow references

| Product | Classification | Repository context | Required treatment |
|---|---|---|---|
| Ableton Live | substantive behavioral reference; bounded VCV interoperability observation | dedicated dossier, UI inspiration, production positioning, VCV host-routing notes | rebuild versioned dossier and field-workflow corpus; no UI cloning or switcher-only identity |
| Bitwig Studio | substantive behavioral reference | dedicated dossier covering DAW, modulation, launcher, routing, Grid | rebuild versioned dossier; separate Grid observations from Geist graph decisions |
| Bitwig Grid | bounded subsystem reference | modular/polyphonic signal and sound-design behavior | section-level source matrix; no inferred implementation reuse |
| FL Studio | substantive workflow reference for field study; current repository mention otherwise incidental | old prompt grouping and VCV supported-host list | create workflow/reference dossier from current official sources; do not infer requirements from host list |
| REAPER | substantive workflow reference for field study; current repository mention otherwise incidental | old prompt grouping and VCV supported-host list | research editing, routing, actions/customization; no parity target |
| Logic Pro | substantive workflow reference for field study; bounded VCV interoperability observation | old prompt/grouping, architecture comparison, VCV routing notes | create current official-source dossier; keep VCV routing observation bounded |
| Cubase | substantive workflow reference for field study; current repository mention otherwise incidental | old prompt grouping and VCV supported-host list | create current official-source dossier, especially recording/editing/scoring workflows if in Geist scope |
| Reason | incidental/historical mention | VCV supported-host list | no dedicated dossier unless future workflows adopt it |
| Harrison Mixbus | incidental/historical mention | VCV supported-host list | no dedicated dossier |
| Studio One | incidental/historical mention | VCV supported-host list | no dedicated dossier unless later workflow evidence justifies it |
| Cakewalk | incidental/historical mention | VCV supported-host list | no dedicated dossier |
| GarageBand | incidental/historical mention | VCV supported-host list | no dedicated dossier |

The VCV supported-host list is an observation about VCV Rack Pro, not an endorsement or compatibility target for Geist.

## Synth and modular references

| Product | Classification | Repository context | Required treatment |
|---|---|---|---|
| VCV Rack | substantive behavioral reference | `docs/modular_rack_spec.md`; modular routing change specs | rebuild dossier with claim-level source anchors; separate volts/polyphony/limits from Geist decisions |
| Kilohearts Phase Plant | substantive behavioral reference | Geist modular synth spec/plan | rebuild dossier; exact macros, lanes, and module limits remain candidates, not commitments |
| Kilohearts Snapins | bounded subsystem reference | Phase Plant device/effect-chain behavior | cover only chain/nesting/modulation workflows relevant to Geist |
| Xfer Serum 2 | substantive behavioral reference | dedicated dossier and sound-design change material | source scope remains incomplete without full official guide; preserve extensive gaps |
| Xfer Serum 1 | bounded historical/source-continuity reference | older support articles used in Serum 2 dossier | claims require proof they remain valid in Serum 2 |
| Max for Live | design anti-reference / excluded compatibility scope | Ableton dossier exclusion | study workflow implications only where relevant; no device/runtime compatibility target |

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
| Bitwig Studio/Grid | draft research only | page-level rather than claim-level provenance; Geist architecture mixed into implications |
| VCV Rack | draft research only | observation/inference/requirement/implementation mixed; vendor-derived limits unapproved |
| Phase Plant | draft research only | “exhaustive” unsupported; public limits promoted directly into Geist commitments |
| Serum 2 | draft research only | no complete official guide in source set; extensive declared gaps; older Serum continuity unproved |
| FL Studio | not started | no dedicated dossier |
| REAPER | not started | no dedicated dossier |
| Logic Pro | not started | no dedicated dossier |
| Cubase | not started | no dedicated dossier |

No reference dossier is accepted for product planning yet.

## Legal and original-design boundary

Research MAY describe publicly documented functional behavior. Geist MUST NOT copy vendor code, private formats, undocumented protocols, screenshots, icons, artwork, presets, samples, wavetables, factory projects, text passages, naming systems, or distinctive UI composition. Vendor numeric limits and defaults do not become Geist requirements without an independent rationale and explicit decision.

Trademarks belong in research, compatibility, attribution, and legal contexts—not first-party device names or product UI unless necessary to identify an actual hosted plugin.

## Next research actions

1. Define the source ledger schema and clean-room methodology.
2. Re-fetch current official source inventories and record exact versions/access dates.
3. Convert existing dossiers to claim-tagged, section-located observations.
4. Create missing major-DAW dossiers at the declared bounded scope.
5. Conduct independent musician workflow research and shortcut analysis.
6. Promote only corroborated, selected implications into `GEIST-CANDIDATE` entries.
7. Assign stable requirement IDs only after product decision.
8. Run a second-pass consistency and legal/originality review before marking any dossier accepted.
