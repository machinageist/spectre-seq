<!--
Author: Jeff
Date: 2026-09-07
Description: Complete section-level coverage map for the final-product source
Notes: Required detail remains in the preserved source; phase allocation is proposed, not ratified
-->

# Final-product capability map

- **Status:** proposed
- **Last verified:** 2026-09-07
- **Scope:** every numbered section of `final-product-source.md`
- **Decision authority:** Jeff
- **Upstream sources:** `product-contract.md`, `final-product-source.md`, current source audit
- **Downstream dependents:** `../06-plans/product-convergence.md`, subsystem intake specifications
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** phase/release allocation and subsystem behavioral contracts
- **Known gaps:** this is section-level coverage, not a claim that each bullet has an executable acceptance test

Each section's required subfeatures remain in scope even when grouped in one row. At intake,
expand its source bullets into requirement IDs, dependency edges, testable acceptance conditions,
explicit exclusions and deferred rows. Do not close a row by shipping a similarly named subset.
Proposed phase IDs are defined in `../06-plans/product-convergence.md` and are not replacements
for the accepted R0–R12 order until ratified.

| Source section | Capability | Current disposition | Proposed owner |
|---|---|---|---|
| §1 | Product vision | Product/engineering contract; not a completed feature. | All |
| §2 | Core architecture | Partial: compiled audio graph; not general typed routing. | P0/P3 |
| §3 | Audio engine | Partial: realtime bridge and guards; no complete compensation/scheduling/export capability. | P0/P3/P10 |
| §4 | Project system | Partial: project codec, atomic save, migrations and recovery; device ownership and media gaps. | P0/P3 |
| §5 | Track system | Partial: instrument tracks, ordered inserts, gains and summing. | P3/P4 |
| §6 | Arrangement view | Partial: MIDI clip lanes and list editing; no full timeline editor. | P4 |
| §7 | Clip launcher / Session view | Specified destination; full capability not established by this audit. | P5 |
| §8 | Audio recording | Specified destination; full capability not established by this audit. | P4 |
| §9 | MIDI engine | Partial: bounded note ingress/conversion; no physical input backend. | P3/P4/P5/P8 |
| §10 | Wavetable synthesizer | Specified destination; full capability not established by this audit. | P6 |
| §11 | Wavetable engine | Specified destination; full capability not established by this audit. | P6 |
| §12 | Wavetable synthesis modes | Specified destination; full capability not established by this audit. | P6 |
| §13 | Filters | Limited device DSP, not the specified filter catalog. | P6/P7 |
| §14 | Synth modulation system | Specified destination; full capability not established by this audit. | P3/P6 |
| §15 | Envelopes | Specified destination; full capability not established by this audit. | P6 |
| §16 | LFOs | Specified destination; full capability not established by this audit. | P6 |
| §17 | Polyphony | Basic instrument polyphony; not the specified full voice contract. | P6/P10 |
| §18 | Modular synthesis environment | Specified destination; full capability not established by this audit. | P8 |
| §19 | Modular patch cables | Specified destination; full capability not established by this audit. | P8 |
| §20 | Modular module SDK | Specified destination; full capability not established by this audit. | P8 |
| §21 | Full effects suite | Specified destination; full capability not established by this audit. | P7 |
| §22 | Creative effects | Specified destination; full capability not established by this audit. | P7 |
| §23 | Mixer | Partial: levels/mute/solo/master; not full mixer/routing. | P3/P7 |
| §24 | Mastering | Fixture WAVE bounce is not musician-project mastering/export. | P7/P10 |
| §25 | Automation | Parameter delivery only; no automation/modulation system. | P3/P4/P5/P6/P8/P9 |
| §26 | Plugin support | Specified destination; full capability not established by this audit. | P9 |
| §27 | Routing architecture | Partial: compiled audio edges and track topology; general routing absent. | P3/P8 |
| §28 | Sidechain system | Specified destination; full capability not established by this audit. | P3/P7 |
| §29 | Warp/time-stretch engine | Specified destination; full capability not established by this audit. | P4/P6/P7 |
| §30 | Sampler | Specified destination; full capability not established by this audit. | P4/P6 |
| §31 | Granular engine | Specified destination; full capability not established by this audit. | P6/P7 |
| §32 | Browser / library | Specified destination; full capability not established by this audit. | P2/P4/P6/P8/P9 |
| §33 | Macro system | Specified destination; full capability not established by this audit. | P3/P6/P8 |
| §34 | UI architecture | egui prototype exists; custom-surface framework not chosen. | P1/P2 |
| §35 | Synth UI | Specified destination; full capability not established by this audit. | P2/P6 |
| §36 | Modular UI | Specified destination; full capability not established by this audit. | P2/P8 |
| §37 | Clip system | Partial: MIDI clips and placements; audio/session/expression absent. | P3/P4/P5 |
| §38 | Groove system | Specified destination; full capability not established by this audit. | P4/P5 |
| §39 | Performance / live mode | Specified destination; full capability not established by this audit. | P5/P10 |
| §40 | Undo/redo | Command history exists; Shape parameter edits bypass it. | All |
| §41 | Accessibility | No fresh accessibility qualification. | P1/P2/P10; every surface |
| §42 | Crash recovery | Atomic save/sidecar/recovery exist; standalone-sidecar and lifecycle concerns remain. | P0/P3/P4/P9/P10 |
| §43 | Testing requirements | Full automated gate passes; no fresh operator qualification. | All |
| §44 | Audio quality requirements | Containment/RT evidence exists; full DSP quality matrix absent. | P3/P6/P7/P8/P10 |
| §45 | Preset/state architecture | Partial project serialization; track-owned complete device state absent. | P0/P3/P6/P8/P9 |
| §46 | Agent-development architecture | Product/engineering contract; not a completed feature. | All |
| §47 | Critical engineering rule | Product/engineering contract; not a completed feature. | All |
| §48 | Definition of “done” | Product/engineering contract; not a completed feature. | All |
| §49 | Suggested master architecture | Product/engineering contract; not a completed feature. | P0/P3/P8/P9 |

## Coverage rule

This map covers source §§1–49 exactly once. Optional and conditional capabilities retain their
source modality. The complete source, rather than this abbreviated disposition column, defines
what must be accounted for. Test execution does not establish any missing feature listed here.
