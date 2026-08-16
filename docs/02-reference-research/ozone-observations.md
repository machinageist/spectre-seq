<!--
Author: Jeff
Date: 2026-08-15
Description: Atomic clean-room observations from the official iZotope Ozone 12 user guide and cited public loudness standards
Notes: Documented public behavior only; proprietary DSP and Master Assistant model internals are deliberately not reconstructed
-->

# iZotope Ozone 12 — Atomic Observations

- **Status:** draft
- **Research state:** in-review
- **Last verified:** 2026-08-15
- **Scope:** official Ozone 12 user guide (`docs.izotope.com/ozone12/en/`) module and system pages relevant to a Spectre mastering suite, plus separately cited public loudness/metering standards
- **Decision authority:** Jeff
- **Upstream sources:** `SRC-OZONE12-DOCS-INDEX` and the per-page `SRC-OZONE12-*` records in `source-ledger.json`; `docs/02-reference-research/methodology.md`
- **Downstream dependents:** `docs/02-reference-research/ozone.md`; future Spectre mastering-device specs; requirements ledger
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** whether Spectre builds a mastering suite at all; which loudness standards Spectre implements natively
- **Known gaps:** the guide exposes no revision number or publication date; numeric ranges and defaults are absent from most module pages; latency and oversampling factors are almost never quantified; see the gap register at the end

## Honest coverage note

This is **not** a complete extraction of the Ozone 12 user guide, and it must not be described as one.

What was actually done: the rendered guide index was inventoried, and a selected subset of module and system pages was fetched and converted into atomic claims. Pages were retrieved through an automated fetch-and-summarize path, which means individual wordings carry transcription risk until spot-verified against the rendered page by a human. The guide's signal-flow and interface diagrams are images; their content was not readable by the retrieval tool and no claim here derives from them.

Three structural limits deserve to be stated up front:

1. **The guide is not patch-labeled.** The URL pins major version 12 and the footer carries a 2026 copyright, but no guide revision or publication date is exposed. The Advanced release notes list 12.1.0 (2025-12-01) as the newest build. Claims are therefore scoped to "the Ozone 12 guide as rendered on 2026-08-15", not to a specific application build.
2. **Numeric ranges and defaults are mostly absent from the documentation itself.** Where a range is not printed on the page, it is recorded as a gap. Nothing here infers a limit, a default, a time constant, or a filter order from silence.
3. **Edition scoping is unresolved.** The guide distinguishes "mothership" from "component" plug-ins but the inspected pages did not enumerate Elements/Standard/Advanced differences. Module availability per edition is a gap.

DSP algorithm internals and machine-learning model internals are out of bounds by mandate. What follows records *what a control is called, what the documentation says it does, and what the user sees* — never a reconstruction of how iZotope achieves it. This applies most sharply to the IRC limiting modes and to Master Assistant.

## Claim layer legend

Every record below carries exactly one layer: `OBSERVED` (supported by a cited public source section), `SOURCE-GAP` (relevant fact unavailable or ambiguous in the inspected source), or `SPECTRE-CANDIDATE` (inference or product implication, carrying no authority). No record in this file is a Spectre requirement.

---

## Product identity and version

- `OBS-OZ-VER-001` **OBSERVED** (`SRC-OZONE12-RELEASE-NOTES-ADV`, rendered entry list): The current major version of Ozone is 12. The Advanced release-note list shows 12.1.0 dated 2025-12-01 as its newest entry, preceded by 12.0.2 (2025-09-11), 12.0.1 (2025-09-03), and 12.0.0 (2025-09-02).
- `OBS-OZ-VER-002` **OBSERVED** (`SRC-OZONE12-RELEASE-NOTES-ADV`, 12.1.0 entry): The 12.1.0 entry describes fixes to UI loading and instantiation time, a stereo-processing correction, added Cubase 15 host compatibility, and an adjustment to supported macOS versions. No behavioral claim in this dossier depends on that entry.
- `OBS-OZ-VER-003` **OBSERVED** (`SRC-OZONE12-DOCS-INDEX`, guide index): The guide is published as "Ozone 12 user guide" with a "© 2026 iZotope by Native Instruments" footer. The product is published under the Native Instruments umbrella rather than iZotope standalone branding.
- `OBS-OZ-VER-004` **OBSERVED** (`SRC-OZONE12-DOCS-INDEX`, module navigation): The guide's module navigation lists Master Assistant, Bass Control, Clarity, Dynamics, Dynamic EQ, Equalizer, Exciter, Imager, Impact, Low End Focus, Master Rebalance, Match EQ, Maximizer, Spectral Shaper, Stabilizer, Stem EQ, Unlimiter, Vintage Compressor, Vintage EQ, Vintage Limiter, and Vintage Tape. Utility and analysis pages are Codec Preview, Dither, and Referencing. System pages are General Controls, Preset System, Options, Elements, Glossary, and License Information.

## Chain and routing model

- `OBS-OZ-CHAIN-001` **OBSERVED** (`SRC-OZONE12-GETTING-STARTED`, Common Terms): Two plug-in shapes ship. The "mothership" plug-in hosts multiple processing modules plus a customizable signal chain, Master Assistant, Stem Focus, Referencing, Codec Preview, and Dither. A "component" plug-in exposes a single Ozone processing module as its own plug-in.
- `OBS-OZ-CHAIN-002` **OBSERVED** (`SRC-OZONE12-GETTING-STARTED`, Common Terms): 19 modules are stated to be available as component plug-ins. The guide index lists 21 module pages; the difference between the two counts is not explained on the inspected pages.
- `OBS-OZ-CHAIN-003` **OBSERVED** (`SRC-OZONE12-GETTING-STARTED`, Signal Flow in Ozone): The mothership's signal chain contains **no modules by default**. The user builds the chain rather than bypassing a prebuilt one.
- `OBS-OZ-CHAIN-004` **OBSERVED** (`SRC-OZONE12-GETTING-STARTED`, Signal Flow in Ozone; `SRC-OZONE12-GENERAL-CONTROLS`, Signal Chain): Both the contents and the order of the processing chain are user-adjustable. The Signal Chain surface exposes an add control and a remove control, and 21 modules are selectable for insertion.
- `OBS-OZ-CHAIN-005` **OBSERVED** (`SRC-OZONE12-GETTING-STARTED`, Navigating the Plug-in Interface): The interface is divided into four regions — a global header (Stem Focus, Master Assistant, Presets, IPC instance name, Undo History, Options, Help), the Signal Chain, the module interface for the selected module, and the I/O panel (I/O gain and metering, global bypass, auditioning).
- `OBS-OZ-CHAIN-006` **OBSERVED** (`SRC-OZONE12-GETTING-STARTED`, Plug-in Feature Differences): Feature availability is split between mothership and component contexts. Master Assistant, Stem Focus, the signal chain, Referencing, and Codec Preview are mothership-only. Dither is available in the mothership and in the Maximizer component. The I/O sum-to-mono and swap-channels controls are available in the mothership and in the Imager component.
- `OBS-OZ-CHAIN-007` **OBSERVED** (`SRC-OZONE12-GETTING-STARTED`, Tips for Optimizing Performance): Documented CPU-reduction levers are: remove unused modules from the chain, raise the host buffer size, adjust the Equalizer buffer size in Options when the Equalizer is in Digital mode, adjust the Crossover Buffer Size in Options when using the digital crossover type, reduce the number of enabled bands in multiband modules, and prefer the Stereo channel mode over Mid/Side, Left/Right, or Transient/Sustain.
- `OBS-OZ-CHAIN-008` **SPECTRE-CANDIDATE**: The buffer-size-in-Options levers imply that at least the linear-phase Equalizer mode and the digital crossover are block/FFT-based with a user-visible latency-versus-CPU tradeoff. This is a plausible reading of a performance tip, not a documented statement, and it must not be recorded as Ozone behavior. The Spectre-relevant question it raises is whether a mastering EQ should expose latency-versus-CPU as a user control at all.

## Input/output, gain staging, and channel modes

- `OBS-OZ-IO-001` **OBSERVED** (`SRC-OZONE12-GENERAL-CONTROLS`, I/O Panel; Input/Output Gain and Meters): The I/O panel provides input gain and output gain, each with linked left/right controls, alongside input and output metering.
- `OBS-OZ-IO-002` **OBSERVED** (`SRC-OZONE12-GENERAL-CONTROLS`, Clipping Indicators): The I/O panel exposes dedicated clipping indicators, separate from the level meters themselves.
- `OBS-OZ-IO-003` **OBSERVED** (`SRC-OZONE12-GENERAL-CONTROLS`, Global Processing and Auditioning): A global bypass disables all processing in the current plug-in instance. A Gain Match toggle is available and its behavior depends on a setting in the Options menu — the guide makes gain matching configurable rather than fixed.
- `OBS-OZ-IO-004` **OBSERVED** (`SRC-OZONE12-GENERAL-CONTROLS`, Channel Processing Modes): Channel processing modes are a first-class, chain-level concept with their own controls section, not a per-module afterthought.
- `OBS-OZ-IO-005` **OBSERVED** (`SRC-OZONE12-GETTING-STARTED`, Tips for Optimizing Performance): The available channel processing modes named in the performance guidance are Stereo, Mid/Side, Left/Right, and Transient/Sustain, and Stereo is documented as the cheapest of the four.
- `OBS-OZ-IO-006` **OBSERVED** (`SRC-OZONE12-GETTING-STARTED`, Plug-in Feature Differences): Sum-to-mono and swap-channels are documented as I/O features, available in the mothership and the Imager component.
- `OBS-OZ-IO-007` **SPECTRE-CANDIDATE**: Treating Transient/Sustain as a *channel processing mode* peer to Mid/Side and Left/Right — that is, a general per-module domain split rather than a feature of one compressor — is an architecturally interesting choice for Spectre's device model. It generalizes "which pair of signals does this module process" beyond stereo geometry. Whether Spectre adopts it is an open product question, not a documented Ozone requirement.

## Maximizer (limiter)

- `OBS-OZ-MAX-001` **OBSERVED** (`SRC-OZONE12-MAXIMIZER`, Main Controls): The Maximizer's limiting algorithm is chosen by a **Mode** control whose documented options are IRC Low Latency, IRC 1, IRC 2, IRC 3, IRC 4, and IRC 5. IRC stands for the product's release-control technology; its internals are proprietary and are not reconstructed here.
- `OBS-OZ-MAX-002` **OBSERVED** (`SRC-OZONE12-MAXIMIZER`, Main Controls): Modes are documented with an explicit cost ordering rather than numbers. IRC Low Latency is described as the lowest-latency and least CPU-intensive option; IRC 3 and IRC 4 are each described as very CPU-intensive with high latency; IRC 5 is described as the most CPU-intensive and requiring the most latency.
- `OBS-OZ-MAX-003` **OBSERVED** (`SRC-OZONE12-MAXIMIZER`, Main Controls): Mode character differs by documented intent — IRC 2 is described as preserving transients more than IRC 1, and IRC 3 as the most aggressive limiting.
- `OBS-OZ-MAX-004` **OBSERVED** (`SRC-OZONE12-MAXIMIZER`, Main Controls): Two modes expose a second-level **Character Style** enumeration. IRC 3 offers Clipping, Crisp, Balanced, and Pumping. IRC 4 offers Classic, Modern, and Transient.
- `OBS-OZ-MAX-005` **OBSERVED** (`SRC-OZONE12-MAXIMIZER`, Main Controls): IRC 4 is documented as multiband spectral limiting and IRC 5 as a four-band multiband limiter. The band edges, crossover type, and per-band behavior of either mode are not stated on the page.
- `OBS-OZ-MAX-006` **OBSERVED** (`SRC-OZONE12-MAXIMIZER`, Main Controls): The two principal level controls are **Gain** and **Output Level**, both in dB. Ozone 12 does not name them "Threshold" and "Ceiling" — the Ozone 9-era naming does not survive into the inspected Ozone 12 page. A **link** toggle couples Gain and Output Level inversely, so raising drive lowers the ceiling in step.
- `OBS-OZ-MAX-007` **OBSERVED** (`SRC-OZONE12-MAXIMIZER`, Main Controls): A **True Peak** toggle is documented as making the limiter account for both the levels of each digital sample and the level of the analog signal that D/A conversion will eventually produce. This is the module's inter-sample-peak control. The oversampling factor used to estimate the reconstructed peak is not stated.
- `OBS-OZ-MAX-008` **OBSERVED** (`SRC-OZONE12-MAXIMIZER`, Main Controls): **Character** is a single continuous control with a documented range of 0.0 to 10.0 that moves the attack/release response along a scale described from Clipping at one end to Very Slow at the other. Individual attack and release times in milliseconds are not exposed.
- `OBS-OZ-MAX-009` **OBSERVED** (`SRC-OZONE12-MAXIMIZER`, Secondary Controls): **Upward Compress** is a secondary control calibrated in dB. Its range and default are not stated.
- `OBS-OZ-MAX-010` **OBSERVED** (`SRC-OZONE12-MAXIMIZER`, Secondary Controls): **Soft Clip** is a toggle with an Amount control documented as 0–100% and three modes named Light, Moderate, and Heavy. The soft clipper is documented as 4× oversampled — the only explicit oversampling factor found anywhere in the inspected pages.
- `OBS-OZ-MAX-011` **OBSERVED** (`SRC-OZONE12-MAXIMIZER`, Secondary Controls): **Transient Emphasis** is a toggle with an Amount control; the amount's range is not stated.
- `OBS-OZ-MAX-012` **OBSERVED** (`SRC-OZONE12-MAXIMIZER`, Secondary Controls): **Stereo Independence** is split into two sliders, one for transients and one for sustained material, with a link toggle between them. The documented split means the limiter's stereo-linking behavior is itself time-domain dependent rather than one global link amount. Slider ranges are not stated.
- `OBS-OZ-MAX-013` **OBSERVED** (`SRC-OZONE12-MAXIMIZER`, Module Header): The module header exposes **Learn Input Gain**, a toggle that performs an automatic analysis over roughly five seconds, together with a **Target LUFS** field in LUFS. The documented pattern is: state a loudness target, let the module analyze for a bounded window, and have it set the drive control for you.
- `OBS-OZ-MAX-014` **OBSERVED** (`SRC-OZONE12-MAXIMIZER`, Module Header): The header also exposes a **Delta** control documented as monitoring-only — it changes what you hear for auditioning without being a processing parameter — and a **Reset** that returns the module to defaults.
- `OBS-OZ-MAX-015` **OBSERVED** (`SRC-OZONE12-MAXIMIZER`, Views): The Maximizer offers three switchable visualizations: a spectrum analyzer, a gain trace, and a gain-reduction meter.
- `OBS-OZ-MAX-016` **OBSERVED** (`SRC-OZONE12-GETTING-STARTED`, Plug-in Feature Differences): Dither is reachable from the Maximizer component plug-in as well as the mothership — the dither stage is bound to the last-in-chain limiter rather than existing only as a chain-level facility.
- `OBS-OZ-MAX-017` **SPECTRE-CANDIDATE**: The Gain/Output-Level inverse link is a small, cheap ergonomic idea worth evaluating for any Spectre limiter: it makes "push harder without changing the ceiling" a single gesture. It carries no parity obligation and no numeric commitment.

## Metering — level, loudness, and true peak

This is the densest and most transferable part of the inspected guide. Note that the **General Controls** page names the section "I/O Meter Options" but does not print the standards; the **Options** page is where the vocabulary actually appears.

- `OBS-OZ-METER-001` **OBSERVED** (`SRC-OZONE12-OPTIONS`, I/O Options → Metering): I/O metering can be switched off entirely with an "Enable I/O meters" toggle. Metering is treated as optional work, not as an always-on cost.
- `OBS-OZ-METER-002` **OBSERVED** (`SRC-OZONE12-OPTIONS`, I/O Options → Metering): A **Detect true peaks** toggle selects whether the meter measures digital sample values or estimates the level of the signal that D/A conversion will produce. True-peak detection is thus a *metering option*, independent of the Maximizer's own True Peak limiting toggle (`OBS-OZ-MAX-007`).
- `OBS-OZ-METER-003` **OBSERVED** (`SRC-OZONE12-OPTIONS`, I/O Options → Metering): The **Meter Type** enumeration is RMS, PEAK, RMS + PEAK, K-SYSTEM, MOMENTARY, SHORT TERM, and INTEGRATED. Loudness meters are presented as *types of the same meter*, not as a separate loudness panel.
- `OBS-OZ-METER-004` **OBSERVED** (`SRC-OZONE12-OPTIONS`, I/O Options → Metering): The three loudness meter types are documented with their integration windows printed alongside them — MOMENTARY at 400 ms, SHORT TERM at 3 seconds, and INTEGRATED as indefinite (gated over the whole measured program rather than a sliding window).
- `OBS-OZ-METER-005` **OBSERVED** (`SRC-OZONE12-OPTIONS`, I/O Options → Metering): The **Meter Scale** enumeration is dB (Linear), dB (Non-linear), BS.1771, EBU +9, and EBU +18. Two of the five scale names are external standards references and three are the product's own presentation choices.
- `OBS-OZ-METER-006` **OBSERVED** (`SRC-OZONE12-OPTIONS`, I/O Options → Metering): Documented scale endpoints in LUFS are: BS.1771 spanning −45 to −14.0 LUFS; EBU +9 spanning −41.0 to −14.0 LUFS and identified as the default scale; EBU +18 spanning −59.0 to −5.0 LUFS and characterized as the wide-dynamic-range choice. These are meter *display ranges*, not loudness targets, and must not be read as delivery specifications.
- `OBS-OZ-METER-007` **OBSERVED** (`SRC-OZONE12-OPTIONS`, I/O Options → Metering): The page states the identity **1 LUFS = 1 dB**. This is the unit relationship that makes loudness offsets and gain offsets directly interchangeable in arithmetic.
- `OBS-OZ-METER-008` **OBSERVED** (`SRC-OZONE12-OPTIONS`, I/O Options → Metering): **Meter Source** selects between Stereo and Mid-Side. Metering can therefore be observed in the same domain in which processing happens.
- `OBS-OZ-METER-009` **OBSERVED** (`SRC-OZONE12-OPTIONS`, I/O Options → Metering): **Peak Hold Time** is an enumeration, documented as 5 ms, 250 ms, 500 ms, 1,000 ms, 5,000 ms, and Infinite, paired with a Show Peak Hold toggle and a Readout selector between current level and maximum peak.
- `OBS-OZ-METER-010` **OBSERVED** (`SRC-OZONE12-OPTIONS`, I/O Options → Metering): **Integration time** for the non-loudness meter types is a separate enumeration documented as 10 ms, 50 ms, 300 ms (labeled VU), 1,475 ms, 2,650 ms, 3,825 ms, and 5,000 ms. The 300 ms entry is explicitly annotated as the VU ballistic. The provenance of the four irregular values above 1 second is not explained on the page.
- `OBS-OZ-METER-011` **OBSERVED** (`SRC-OZONE12-MAXIMIZER`, Views; `SRC-OZONE12-DYNAMICS`, Views; `SRC-OZONE12-IMAGER`, Views): Modules carry their own visualizations selected by a **Views** switcher rather than displaying everything at once — Maximizer offers spectrum/gain trace/gain reduction, Dynamics offers crossover spectrum/gain-reduction trace/detection filter/dynamics curve, Imager offers vectorscope forms.
- `OBS-OZ-METER-012` **OBSERVED** (`SRC-OZONE12-IMAGER`, Vectorscope; `SRC-OZONE12-OPTIONS`, Imager Options): Stereo-field metering comprises a vectorscope with three documented forms — Polar Sample, Polar Level, and Lissajous — plus a correlation meter bar and a stereo balance meter. The vectorscope's detection method is separately configurable as Peak, RMS, or Envelope.
- `GAP-OZONE-0001` **SOURCE-GAP**: The inspected pages never name **ITU-R BS.1770** as the loudness measurement algorithm. Only BS.1771 appears, and only as a meter *scale* name. Whether Ozone's MOMENTARY/SHORT TERM/INTEGRATED meters implement BS.1770 gating and K-weighting, and which revision, is not established by the inspected sources.
- `GAP-OZONE-0002` **SOURCE-GAP**: The inspected pages do not state whether the INTEGRATED meter applies the absolute (−70 LUFS) and relative (−10 LU) gating defined by the public standards, nor whether it exposes a loudness range (LRA) readout at all.
- `GAP-OZONE-0003` **SOURCE-GAP**: The oversampling factor used by the "Detect true peaks" metering option and by the Maximizer's True Peak mode is not stated anywhere in the inspected pages.
- `GAP-OZONE-0004` **SOURCE-GAP**: The K-SYSTEM meter type is named but its variant (K-20, K-14, K-12) and reference alignment are not stated on the inspected page.
- `GAP-OZONE-0005` **SOURCE-GAP**: No streaming-platform loudness target values (for example −14 LUFS integrated) are printed on any inspected page, despite Target LUFS controls existing in the Maximizer and Master Assistant.
- `GAP-OZONE-0006` **SOURCE-GAP**: Whether any meter reports PLR (peak-to-loudness ratio) or PSR is not established by the inspected pages.

## Spectrum analysis

- `OBS-OZ-SPEC-001` **OBSERVED** (`SRC-OZONE12-OPTIONS`, Spectrum Options): The analyzer's band mapping is selectable among Linear, 1/3 Octave, Critical, and Full Octave. "Critical" denotes a psychoacoustic critical-band mapping; the specific band table is not printed.
- `OBS-OZ-SPEC-002` **OBSERVED** (`SRC-OZONE12-OPTIONS`, Spectrum Options): Analyzer resolution is user-controlled through a **Window Type** selector and a **Window Size** control, with larger window sizes documented as giving greater frequency resolution. The individual window function names were not returned by the inspected extraction.
- `OBS-OZ-SPEC-003` **OBSERVED** (`SRC-OZONE12-OPTIONS`, Spectrum Options): **Average Time** is an enumeration of Real Time, 1 second, 3 seconds, 5 seconds, 10 seconds, and Infinite. Long averaging is a first-class mastering-analysis mode, not a hidden setting.
- `OBS-OZ-SPEC-004` **OBSERVED** (`SRC-OZONE12-OPTIONS`, Spectrum Options): The frequency axis offers a **Mel** scale in addition to Logarithmic, and the logarithmic option itself has Flat and Extended variants.
- `OBS-OZ-SPEC-005` **OBSERVED** (`SRC-OZONE12-OPTIONS`, Spectrum Options): A **Tilt Slope** control defaults to 3 dB/octave, which renders pink noise as a flat display. The default deliberately biases the display toward a perceptual reference rather than raw magnitude.
- `OBS-OZ-SPEC-006` **OBSERVED** (`SRC-OZONE12-OPTIONS`, Spectrum Options): Peak-hold display for the analyzer is separately configurable with its own Peak Hold Time and Show Peak Hold controls, distinct from the I/O meter's peak hold.
- `OBS-OZ-SPEC-007` **OBSERVED** (`SRC-OZONE12-GENERAL-CONTROLS`, Global Processing and Auditioning): A "Show Reference Spectrum" option overlays the reference track's spectrum inside module views, so reference comparison is a display layer on the working analyzer rather than a separate window.
- `GAP-OZONE-0007` **SOURCE-GAP**: The concrete FFT sizes, window function names, overlap, and update rate behind Window Size and Window Type are not printed on the inspected page.

## Equalizer

- `OBS-OZ-EQ-001` **OBSERVED** (`SRC-OZONE12-EQUALIZER`, Controls): Band frequency spans 20 Hz to 20 kHz and band gain spans −30 dB to +15 dB. The asymmetric gain range — far more cut than boost — is documented product behavior.
- `OBS-OZ-EQ-002` **OBSERVED** (`SRC-OZONE12-EQUALIZER`, Controls): Two filter modes exist. **Analog** mode is documented as minimum-phase IIR filtering; **Digital** mode is documented as linear-phase FIR filtering. The product's user-facing mode names do not describe the phase behavior directly, which is a naming choice Spectre should treat as a cautionary example rather than a model.
- `OBS-OZ-EQ-003` **OBSERVED** (`SRC-OZONE12-EQUALIZER`, Controls): In Digital mode a continuous **Phase** control interpolates the response: 0% yields linear phase and 100% yields minimum phase. Phase behavior is a continuum, not a binary.
- `OBS-OZ-EQ-004` **OBSERVED** (`SRC-OZONE12-EQUALIZER`, Controls): Documented band shapes are Bell, Proportional Q, and Band Shelf; low and high shelves each offer Analog, Baxandall, Vintage, and Resonant variants; lowpass and highpass each offer Flat, Resonant, and Brickwall variants; and a Surgical shape is available in Digital mode only.
- `OBS-OZ-EQ-005` **OBSERVED** (`SRC-OZONE12-EQUALIZER`, Module Header): An **Amount** control spans 0% to 200% with a default of 100%, scaling the whole EQ curve. Values above 100% exaggerate the drawn curve beyond what was dialed in.
- `OBS-OZ-EQ-006` **OBSERVED** (`SRC-OZONE12-EQUALIZER`, Controls): The Equalizer supports all four channel processing modes — Stereo, Mid/Side, Left/Right, and Transient/Sustain.
- `OBS-OZ-EQ-007` **OBSERVED** (`SRC-OZONE12-EQUALIZER`, Working with EQ Nodes): Nodes are editable by dragging and by arrow keys, with modifier keys selecting coarse and fine adjustment. Per-band solo is available from an "S" control in the node HUD.
- `OBS-OZ-EQ-008` **OBSERVED** (`SRC-OZONE12-OPTIONS`, EQ Options → Spectrum): The band-solo bandwidth is itself configurable through an "Alt-Solo Filter Q" option — the audition filter is decoupled from the band being edited.
- `OBS-OZ-EQ-009` **OBSERVED** (`SRC-OZONE12-OPTIONS`, EQ Options → Spectrum): A "Show Extra Curves" option adds Phase Delay, Phase Response, and Group Delay curves to the display. Phase and group-delay visualization is documented product behavior, not a hidden diagnostic.
- `OBS-OZ-EQ-010` **OBSERVED** (`SRC-OZONE12-OPTIONS`, EQ Options → Performance): The Digital-mode EQ exposes a **Buffer Size** in samples, a **Frequency Resolution** enumeration of 3 Hz, 6 Hz, 12 Hz, 24 Hz, and 48 Hz, and a read-only **Filter Size** display of the resulting steepness. Resolution, buffer size, and achievable filter steepness are presented to the user as a single linked tradeoff.
- `OBS-OZ-EQ-011` **OBSERVED** (`SRC-OZONE12-OPTIONS`, EQ Options → Performance): A **Soft Saturation** toggle exists as an EQ *option* rather than a module control.
- `GAP-OZONE-0008` **SOURCE-GAP**: The maximum number of Equalizer bands is not stated on the inspected page.
- `GAP-OZONE-0009` **SOURCE-GAP**: The Q range, per-shape slope values (dB/octave), and the default shape for a newly created node are not stated on the inspected page.
- `GAP-OZONE-0010` **SOURCE-GAP**: The latency incurred by Digital (linear-phase) mode is never quantified, and the mapping from Frequency Resolution to added latency in samples or milliseconds is not printed — even though the option page implies the relationship exists.
- `GAP-OZONE-0011` **SOURCE-GAP**: Whether the Equalizer performs any auto-gain compensation is not stated on the inspected page.

## Dynamics (multiband)

- `OBS-OZ-DYN-001` **OBSERVED** (`SRC-OZONE12-DYNAMICS`, Overview; Band Control Views): The module provides up to four processing bands for multiband compression and limiting. Bands are created by adding crossover points, and each band carries power, solo, and remove controls.
- `OBS-OZ-DYN-002` **OBSERVED** (`SRC-OZONE12-DYNAMICS`, Controls and Meters): Each band contains **both** a compressor stage and a limiter stage with independent thresholds, presented as two handles on one threshold control — limiter on the left, compressor on the right — over a band input-level meter.
- `OBS-OZ-DYN-003` **OBSERVED** (`SRC-OZONE12-DYNAMICS`, Controls and Meters): The compressor ratio spans 0.1:1 to 30:1 with a default of 2:1. The limiter ratio spans 0.4:1 to 30:1 with a default of 10:1. Ratios below 1:1 are reachable in both stages, so upward expansion is inside the documented range.
- `OBS-OZ-DYN-004` **OBSERVED** (`SRC-OZONE12-DYNAMICS`, Controls and Meters): Per-band controls additionally comprise Attack in milliseconds, Release in milliseconds, Knee (documented as the range around the threshold, higher being softer), a **Parallel** dry/wet mix per band, and a post-processing band gain.
- `OBS-OZ-DYN-005` **OBSERVED** (`SRC-OZONE12-DYNAMICS`, Controls and Meters): The detection mode is a **global** setting shared by all bands, with three documented options — Peak (peak level of the incoming signal), RMS (average level of the incoming signal), and Env/Envelope (average level evened out across the frequency spectrum).
- `OBS-OZ-DYN-006` **OBSERVED** (`SRC-OZONE12-DYNAMICS`, Controls and Meters): **Adaptive Release** automatically adjusts release time based on the signal's peak factor, shortening for transients and lengthening for sustained material. **Auto Gain** automatically calculates and applies make-up gain.
- `OBS-OZ-DYN-007` **OBSERVED** (`SRC-OZONE12-DYNAMICS`, Controls and Meters): The module supports Stereo and Mid/Side channel processing modes and offers a **Link Bands** control that couples adjustments across bands.
- `OBS-OZ-DYN-008` **OBSERVED** (`SRC-OZONE12-DYNAMICS`, Views): Views comprise a crossover spectrum, a gain-reduction trace, a detection-filter display, and an interactive dynamics-curve plot. The detection filter is exposed to the user as a visualization, which is unusual and worth noting.
- `OBS-OZ-DYN-009` **OBSERVED** (`SRC-OZONE12-OPTIONS`, Dynamics Options): Lookahead is documented as an *option*, adjustable from 0 ms (described as instantaneous) to 10 ms. This is the only explicit lookahead figure found in the inspected pages.
- `GAP-OZONE-0012` **SOURCE-GAP**: Attack, release, knee, threshold, and band-gain ranges and defaults for the Dynamics module are not stated on the inspected page.
- `GAP-OZONE-0013` **SOURCE-GAP**: No gate or expander stage is documented for the Dynamics module; the inspected page does not say whether one exists.
- `GAP-OZONE-0014` **SOURCE-GAP**: Whether lookahead delay is reported to the host as plug-in latency, and how lookahead interacts with the module's position in the chain, is not stated.

## Crossovers (shared multiband infrastructure)

- `OBS-OZ-XOVER-001` **OBSERVED** (`SRC-OZONE12-OPTIONS`, Dynamics/Imager/Exciter Options): Three separate modules — Dynamics, Imager, and Exciter — each expose the same crossover option triple: a **Crossover Type** of Analog, Digital, or Hybrid; a **Crossover buffer size**; and a **Crossover Q** where higher values give tighter crossovers. Crossover behavior is shared infrastructure configured per module.
- `OBS-OZ-XOVER-002` **OBSERVED** (`SRC-OZONE12-IMAGER`, Controls): Crossover cutoff frequencies are explicitly documented as **not** shared or linked across the multiband modules inside the main plug-in. Each multiband module owns its own band edges.
- `OBS-OZ-XOVER-003` **OBSERVED** (`SRC-OZONE12-GETTING-STARTED`, Tips for Optimizing Performance): The digital crossover type has a Crossover Buffer Size in Options that is documented as a CPU lever, mirroring the EQ's buffer-size tradeoff.
- `GAP-OZONE-0015` **SOURCE-GAP**: The filter topology, order, and phase behavior behind Analog, Digital, and Hybrid crossover types are not stated, nor is the numeric range or unit of Crossover Q.
- `OBS-OZ-XOVER-004` **SPECTRE-CANDIDATE**: Exposing crossover *type* (and therefore its phase/latency character) as a per-module option rather than a global engine choice is a design decision Spectre would have to make deliberately. Independent band edges per module is the opposite of a single shared multiband bus; both are defensible and the choice belongs in a Spectre architecture decision, not in this file.

## Imager

- `OBS-OZ-IMG-001` **OBSERVED** (`SRC-OZONE12-IMAGER`, Controls): The Imager is multiband with up to four bands, each carrying a **Width** control. Positive width values increase perceived stereo width and negative values decrease it; a setting of −100 makes that band's output effectively mono.
- `OBS-OZ-IMG-002` **OBSERVED** (`SRC-OZONE12-IMAGER`, Module Header): A module-level **Amount** control spanning 0% to 100% scales all per-band Width settings at once. A **Link Bands** toggle makes width adjustments track across bands.
- `OBS-OZ-IMG-003` **OBSERVED** (`SRC-OZONE12-IMAGER`, Controls): **Stereoize** is a separate widening effect with its own power control, an Amount slider, and two modes. Mode I is documented as Haas-effect-based decorrelation; Mode II is documented as an alternative with different tonal quality and improved transient preservation. The page states the Stereoize effect is completely mono compatible.
- `OBS-OZ-IMG-004` **OBSERVED** (`SRC-OZONE12-IMAGER`, Controls): **Recover Sides** is a distinct stage with a power control, a gain control, and a solo control that isolates the recovered side channel for auditioning.
- `OBS-OZ-IMG-005` **OBSERVED** (`SRC-OZONE12-OPTIONS`, Imager Options): A **Prevent Antiphase** toggle exists as a module option, constraining the widening processing from producing out-of-phase content.
- `OBS-OZ-IMG-006` **OBSERVED** (`SRC-OZONE12-IMAGER`, Module Header): The Imager's channel processing mode selector offers Stereo or Transient/Sustain — a narrower set than the Equalizer's four, which is consistent with the module already operating on stereo geometry.
- `OBS-OZ-IMG-007` **OBSERVED** (`SRC-OZONE12-IMAGER`, Module Header): The Imager exposes a **Learn** control for automatic crossover placement — an assistive analysis feature scoped to one module rather than to the whole chain.
- `GAP-OZONE-0016` **SOURCE-GAP**: The numeric range and default of the per-band Width control (beyond the documented −100 mono endpoint), the Stereoize Amount range, and the Recover Sides gain range are not stated on the inspected page.
- `GAP-OZONE-0017` **SOURCE-GAP**: The delay time or delay range used by the Haas-based Stereoize Mode I is not stated, and no latency figure is given for the Imager.

## Preferences, state, and undo

- `OBS-OZ-STATE-001` **OBSERVED** (`SRC-OZONE12-GENERAL-CONTROLS`, Undo History; `SRC-OZONE12-OPTIONS`, General Options → Other): Undo is exposed as a browsable **Undo History** in the global header, and its **History Depth** — the number of retained undo events — is a user-configurable option rather than a fixed constant.
- `OBS-OZ-STATE-002` **OBSERVED** (`SRC-OZONE12-OPTIONS`, General Options → Other): Keyboard support is a three-state option — None, Minimal, or Full — acknowledging that a plug-in and its host contend for keystrokes.
- `OBS-OZ-STATE-003` **OBSERVED** (`SRC-OZONE12-OPTIONS`, General Options → Graphics): Graphics options comprise Show Tooltips, Dim Controls When Bypassed, and an adjustable Window Opacity.
- `OBS-OZ-STATE-004` **OBSERVED** (`SRC-OZONE12-GENERAL-CONTROLS`, Resizing): Interface resizing is a documented first-class control.
- `OBS-OZ-STATE-005` **OBSERVED** (`SRC-OZONE12-OPTIONS`, I/O Options → Gain Matching): Gain matching has an explicit "Enable modern bypass gain match behavior" option that changes whether the matched gain is applied to the processed or the bypassed output. The existence of a "modern" alternative implies a preserved legacy behavior for backward compatibility with older sessions; the page does not state that outright, so no claim is made about what the legacy behavior was.
- `OBS-OZ-STATE-006` **OBSERVED** (`SRC-OZONE12-GETTING-STARTED`, Navigating the Plug-in Interface): The global header carries an **IPC instance name**, indicating that instances are named for inter-plug-in communication with other iZotope products.
- `GAP-OZONE-0018` **SOURCE-GAP**: The Preset System page was not inspected; preset scope, format, migration across versions, and whether chain order and module state are both preserved are unestablished.
- `GAP-OZONE-0019` **SOURCE-GAP**: Whether undo history is persisted with plug-in state or discarded on instance reload is not stated on the inspected pages.

## Master Assistant — the assistive layer

**Bounding statement.** Everything below describes what a user does, what the product shows them, and what numbers appear on screen. Nothing below describes, reconstructs, or infers how the analysis works. The target-matching and vocal-separation models are proprietary and are outside the research boundary by mandate. Spectre must design any assistive feature from first principles.

- `OBS-OZ-MA-001` **OBSERVED** (`SRC-OZONE12-MASTER-ASSISTANT`, Workflow Steps): The documented workflow is: place the plug-in on the master or stereo output, open the Master Assistant tab, choose between a One-Click workflow and a Custom workflow, play the track back while it analyzes, and then review and adjust the result.
- `OBS-OZ-MA-002` **OBSERVED** (`SRC-OZONE12-MASTER-ASSISTANT`, Workflow Steps): Analysis requires **at least 8 seconds** of playback, and the guidance is to play the loudest section of the track for best results. The assistant listens to real playback rather than scanning a file offline.
- `OBS-OZ-MA-003` **OBSERVED** (`SRC-OZONE12-MASTER-ASSISTANT`, Processing): Analysis time is adjustable **up to 60 seconds**, and the analysis window can be expressed in seconds or synced to tempo in bars.
- `OBS-OZ-MA-004` **OBSERVED** (`SRC-OZONE12-MASTER-ASSISTANT`, Workflow Steps; Master Assistant View): The assistant's output is a **built processing chain**. When the chain is built, the interface switches to the Master Assistant controls view. The suggestion is materialized as editable device state, not as advisory text.
- `OBS-OZ-MA-005` **OBSERVED** (`SRC-OZONE12-MASTER-ASSISTANT`, Processing): The modules the assistant may place are documented as Equalizer, Maximizer, Master Rebalance, Impact, Imager, Clarity, and Stabilizer, and the user can enable or disable individual modules before analysis.
- `OBS-OZ-MA-006` **OBSERVED** (`SRC-OZONE12-MASTER-ASSISTANT`, Processing): **Intensity** is a five-step enumeration — Subtle, Transparent, Balanced, Bold, Transformative — with Balanced documented as the default. A single ordinal control scales how far the assistant is allowed to go.
- `OBS-OZ-MA-007` **OBSERVED** (`SRC-OZONE12-MASTER-ASSISTANT`, Master Assistant View): After the chain is built, the result is exposed as scaling controls over the assistant's decisions rather than as raw module parameters: the Maximizer's contribution is adjustable over ±4 dB of gain, the Equalizer's over 0–200%, and Dynamics, Width, Clarity, and Stabilizer contributions over 0–100%.
- `OBS-OZ-MA-008` **OBSERVED** (`SRC-OZONE12-MASTER-ASSISTANT`, Loudness): The loudness objective is expressed as a **Target Loudness in LUFS** together with an **Output Level** described as the maximum peak level in dBFS after limiting, plus a True Peak enable/disable.
- `OBS-OZ-MA-009` **OBSERVED** (`SRC-OZONE12-MASTER-ASSISTANT`, Loudness): Two documented destination presets bind those three settings together — **Full Scale** at −0.1 dB output level with True Peak limiting disabled, and **Streaming** at −1 dB output level with True Peak limiting enabled. The product ties true-peak enforcement to the delivery destination, not to a global preference.
- `OBS-OZ-MA-010` **OBSERVED** (`SRC-OZONE12-MASTER-ASSISTANT`, Target Menu; Target Library): Targets come in three documented kinds — ten factory genre targets described as derived from chart-topping hits, a Cinematic target described as derived from film scores, and custom targets created by importing the user's own audio through a plus control in the Target Library. Targets are filterable and favoritable.
- `OBS-OZ-MA-011` **OBSERVED** (`SRC-OZONE12-MASTER-ASSISTANT`, Tonal Balance): The tonal-balance presentation is a target region drawn as a tunnel with the current audio drawn as a line inside or outside it. The user is shown a *tolerance band*, not a single correct curve.
- `OBS-OZ-MA-012` **OBSERVED** (`SRC-OZONE12-MASTER-ASSISTANT`, Loudness): The loudness presentation is a scrolling waveform with a gain trace, so limiting activity is shown against time rather than only as a meter number.
- `OBS-OZ-MA-013` **OBSERVED** (`SRC-OZONE12-MASTER-ASSISTANT`, Vocal Balance): A Vocal Balance stage is documented as separating vocals during analysis and applying the Master Rebalance module accordingly, with checkmark-style visual indicators reporting vocal balance status. The separation method is proprietary and is not characterized here.
- `OBS-OZ-MA-014` **OBSERVED** (`SRC-OZONE12-MASTER-ASSISTANT`, Master Assistant View): Individual modules in the assistant-built chain carry power buttons documented as supporting quick comparisons when Gain Match is enabled — the A/B affordance is explicitly coupled to loudness-compensated bypass.
- `OBS-OZ-MA-015` **OBSERVED** (`SRC-OZONE12-MASTER-ASSISTANT`, Custom Workflow Overview): Custom-workflow settings can be saved as the default for future sessions, so the assistant's configuration is itself persistent user state.
- `GAP-OZONE-0020` **SOURCE-GAP**: The default Target Loudness value in LUFS is not stated on the inspected page, and no per-destination LUFS value is given for either the Full Scale or the Streaming preset. Only their output-level values in dB are printed.
- `GAP-OZONE-0021` **SOURCE-GAP**: The ten genre target names are not enumerated in the inspected extraction, and no statement is made about what a "target" contains (spectral curve, dynamics profile, loudness, or some combination).
- `GAP-OZONE-0022` **SOURCE-GAP**: Whether Master Assistant analysis is deterministic — whether analyzing the same audio twice yields the same chain — is not stated.
- `GAP-OZONE-0023` **SOURCE-GAP**: Stem Focus is named in the Getting Started feature-difference table but was not detailed on any inspected page; its behavior is unestablished.
- `OBS-OZ-MA-016` **SPECTRE-CANDIDATE**: The presentation pattern here — an assistive pass that produces *editable device state* plus a small set of ordinal "how much" scalers over its own decisions, rather than an opaque result or a text recommendation — is the transferable idea, and it is independent of any model. If Spectre ever ships assistive mastering, the reviewable-and-scalable-suggestion shape is worth adopting; the analysis behind it must be Spectre's own.
- `OBS-OZ-MA-017` **SPECTRE-CANDIDATE**: Coupling a destination choice to a true-peak policy (`OBS-OZ-MA-009`) is a good ergonomic answer to a real problem, and the underlying reason is a public standard rather than a vendor behavior. Spectre can arrive at the same ergonomics from the standards directly.

## Dither

- `OBS-OZ-DITH-001` **OBSERVED** (`SRC-OZONE12-DITHER`, Controls): Target bit depth is an enumeration of 24, 20, 16, 12, and 8 bits.
- `OBS-OZ-DITH-002` **OBSERVED** (`SRC-OZONE12-DITHER`, Controls): **Dither Amount** is an ordinal enumeration — Strong, Medium, Low, and Off — rather than a continuous level in LSBs.
- `OBS-OZ-DITH-003` **OBSERVED** (`SRC-OZONE12-DITHER`, Controls): **Noise Shaping** ranges from Off to Max, and Max is documented as providing roughly 14 dB of audible noise suppression. This is the only quantified noise-shaping figure on the page.
- `OBS-OZ-DITH-004` **OBSERVED** (`SRC-OZONE12-DITHER`, Controls): **Auto-Blanking** mutes the dither noise when silence is detected for at least 0.7 seconds.
- `OBS-OZ-DITH-005` **OBSERVED** (`SRC-OZONE12-DITHER`, Controls): **Harmonic Suppression** is available only when Dither Amount is set to Off — the two are mutually exclusive, which is a documented cross-control precedence rule.
- `OBS-OZ-DITH-006` **OBSERVED** (`SRC-OZONE12-DITHER`, Controls): A **Limit Peaks** control suppresses output peaks produced by aggressive dither settings, acknowledging that noise shaping can push the signal above the intended ceiling.
- `OBS-OZ-DITH-007` **OBSERVED** (`SRC-OZONE12-DITHER`, DC Offset): The DC offset filter is documented as a highpass with a 1 Hz cutoff.
- `OBS-OZ-DITH-008` **OBSERVED** (`SRC-OZONE12-DITHER`, Working with Dither): The documented placement rule is that dither must be applied after all other processing; the guidance is to use the last insert slot and a post-fader insert so that dither follows any output gain change, and to disable the host's own export dithering to avoid double-dithering.
- `OBS-OZ-DITH-009` **OBSERVED** (`SRC-OZONE12-DITHER`, Overview): The page identifies the processing by a proprietary iZotope dither algorithm name. That name is recorded here only as a product identifier. No selectable dither *type* control is documented, so the underlying probability density function is not established.
- `GAP-OZONE-0024` **SOURCE-GAP**: The dither PDF (rectangular, triangular, or other), the noise-shaping curve identities and orders, and the amount-to-LSB mapping for Strong/Medium/Low are not stated.
- `GAP-OZONE-0025` **SOURCE-GAP**: What "Harmonic Suppression" does when dither amount is Off — and by what mechanism a system with no added dither noise suppresses harmonic distortion — is not explained on the page.
- `OBS-OZ-DITH-010` **SPECTRE-CANDIDATE**: The cross-control precedence rules here (dither-off gates harmonic suppression; noise shaping can raise peaks and needs its own peak limiter) are real design constraints any Spectre dither stage would hit. They are worth carrying into a Spectre dither spec as *problems to solve*, not as a control layout to copy.

## Codec preview

- `OBS-OZ-CODEC-001` **OBSERVED** (`SRC-OZONE12-CODEC-PREVIEW`, Overview; Working with Codec Preview): Codec Preview auditions lossy compression before export and affects **monitoring only** — it does not alter the rendered output.
- `OBS-OZ-CODEC-002` **OBSERVED** (`SRC-OZONE12-CODEC-PREVIEW`, Bit Rate (Constant)): Constant bitrate options are documented as 96, 112, 128, 160, 192, 224, 256, and 320 kbps, with 256 kbps noted as the maximum for mono files.
- `OBS-OZ-CODEC-003` **OBSERVED** (`SRC-OZONE12-CODEC-PREVIEW`, Solo Artifacts): A **Solo Artifacts** control isolates the signal content that codec compression removes or modifies — documented as the difference between the plug-in's output before and after Codec Preview is applied. It is a difference monitor, not a separate analysis.
- `OBS-OZ-CODEC-004` **OBSERVED** (`SRC-OZONE12-CODEC-PREVIEW`, Headroom and clipping): Codec preview drives clip indicators above the output meters, because lossy encoding and decoding can push peaks above the pre-codec level. The feature exists partly to expose post-codec overs.
- `OBS-OZ-CODEC-005` **OBSERVED** (`SRC-OZONE12-CODEC-PREVIEW`, Sample rates and performance): MP3 does not support sample rates above 48 kHz, so sessions above 48 kHz are resampled automatically in real time, and the page warns this may incur significant latency. No latency figure is given.
- `GAP-OZONE-0026` **SOURCE-GAP**: The complete list of supported codecs is not established; only MP3 was explicitly identified in the inspected extraction, and whether a variable-bitrate option exists was not confirmed either way.

## Referencing and A/B

- `OBS-OZ-REF-001` **OBSERVED** (`SRC-OZONE12-REFERENCING`, Overview): Referencing is reached from a Reference control in the I/O panel and is available only in the mothership plug-in, not in component plug-ins.
- `OBS-OZ-REF-002` **OBSERVED** (`SRC-OZONE12-REFERENCING`, Importing References): Up to **10** reference tracks can be imported at once. Documented accepted formats are WAV, AIF/AIFF, MP3, AAC, and FLAC.
- `OBS-OZ-REF-003` **OBSERVED** (`SRC-OZONE12-REFERENCING`, Reference Track Tabs): References are organized as reorderable tabs; a tab is selected by clicking it and removed by a right-click menu or a dedicated close control.
- `OBS-OZ-REF-004` **OBSERVED** (`SRC-OZONE12-REFERENCING`, Reference Loop Segments): A reference track is automatically divided into predetermined loop segments named with letters (A, B, C, D, E), and those segments can be renamed, resized by drag handles, inserted, and removed. Section-level navigation of a reference is a built-in affordance rather than manual looping.
- `OBS-OZ-REF-005` **OBSERVED** (`SRC-OZONE12-REFERENCING`, Reference Playback Controls): A **Gain** control adjusts the output gain of reference playback. The inspected page documents manual gain only; no automatic loudness matching of the reference to the program is described.
- `OBS-OZ-REF-006` **OBSERVED** (`SRC-OZONE12-REFERENCING`, Reference Metering Options; `SRC-OZONE12-GENERAL-CONTROLS`, Global Processing and Auditioning): When enabled, the reference track's spectrum is drawn inside the module spectrum meters alongside the current track's spectrum. Reference comparison is visual and in-context, not a separate window.
- `GAP-OZONE-0027` **SOURCE-GAP**: Whether reference audio is embedded in plug-in state or stored as a file-path link — and therefore whether a session survives moving or deleting the reference file — is not stated. For a DAW this is a first-order persistence question.
- `GAP-OZONE-0028` **SOURCE-GAP**: Whether reference playback is loudness-matched automatically, and whether any integration exists between Referencing, Master Assistant targets, and Match EQ, is not established by the inspected page.
- `OBS-OZ-REF-007` **SPECTRE-CANDIDATE**: Automatic loudness matching of a reference against the program is the single most obvious improvement over what the inspected documentation describes, and it is implementable purely from public standards (measure both at integrated loudness, apply the difference as gain). This is a Spectre product opportunity, not an observed Ozone behavior.

## Dynamic EQ

- `OBS-OZ-DEQ-001` **OBSERVED** (`SRC-OZONE12-DYNAMIC-EQ`, Dynamic EQ HUD Controls): Frequency spans 20 Hz to 20 kHz and gain spans −30 dB to +15 dB — identical to the static Equalizer's documented ranges (`OBS-OZ-EQ-001`). The two EQ modules share a range contract.
- `OBS-OZ-DEQ-002` **OBSERVED** (`SRC-OZONE12-DYNAMIC-EQ`, Dynamic EQ HUD Controls): Documented band shapes are Baxandall Bass/Treble, Band Shelf, Peak Bell, and Proportional Q — a **narrower** shape set than the static Equalizer offers.
- `OBS-OZ-DEQ-003` **OBSERVED** (`SRC-OZONE12-DYNAMIC-EQ`, Dynamic EQ HUD Controls): Each band has a dynamic direction of **UP** or **DOWN**, indicated by arrows showing which way the filter moves when triggered. Upward and downward dynamic EQ are one control, not two modules.
- `OBS-OZ-DEQ-004` **OBSERVED** (`SRC-OZONE12-DYNAMIC-EQ`, Dynamic EQ HUD Controls): Per-band dynamics controls are Threshold, Attack (time for the dynamic trigger to react once the signal crosses the threshold), and Release (time for the trigger to return the filter to its static setting). Detection is threshold-based level detection with an input meter showing the level that triggers the band.
- `OBS-OZ-DEQ-005` **OBSERVED** (`SRC-OZONE12-DYNAMIC-EQ`, Dynamic EQ HUD Controls): An **Auto Scale** feature scales attack and release by frequency, so time constants are not uniform across the spectrum by default.
- `OBS-OZ-DEQ-006` **OBSERVED** (`SRC-OZONE12-DYNAMIC-EQ`, Dynamic EQ HUD Controls): All four channel processing modes are supported — Stereo, Mid/Side, Left/Right, and Transient/Sustain.
- `OBS-OZ-DEQ-007` **OBSERVED** (`SRC-OZONE12-DYNAMIC-EQ`, Spectrum View): The display shows the spectrum analyzer, a composite curve, the individual filter response curve, a per-band input level meter, and a meter showing gain reduction or gain addition. The meter is signed — upward dynamic movement is displayed, not just reduction.
- `OBS-OZ-DEQ-008` **OBSERVED** (`SRC-OZONE12-OPTIONS`, Dynamic EQ Options): The Dynamic EQ's alt-solo filter Q is configurable over a documented range of **0.2 to 12.0**. This is the only printed Q range found anywhere in the inspected pages, and it applies to the audition filter rather than to a processing band.
- `GAP-OZONE-0029` **SOURCE-GAP**: The maximum number of Dynamic EQ bands is not stated.
- `GAP-OZONE-0030` **SOURCE-GAP**: The Q range of a processing band, the threshold range and units, and the attack and release ranges in milliseconds are not stated. Whether a ratio or amount control exists at all is not established.
- `GAP-OZONE-0031` **SOURCE-GAP**: No latency figure is stated for the Dynamic EQ, and the page does not say whether its filters are minimum-phase or offer a linear-phase option as the static Equalizer does.

## Exciter

- `OBS-OZ-EXC-001` **OBSERVED** (`SRC-OZONE12-EXCITER`, Overview): The Exciter provides up to four bands of configurable saturation.
- `OBS-OZ-EXC-002` **OBSERVED** (`SRC-OZONE12-EXCITER`, Modes): Seven saturation modes are documented by name — Analog, Retro, Tape, Tube, Warm, Triode, and Dual Triode. These names are recorded as documented enumerations; their transfer functions and circuit models are proprietary and are not reconstructed here.
- `OBS-OZ-EXC-003` **OBSERVED** (`SRC-OZONE12-EXCITER`, Oversampling): The module exposes an **Oversampling** control documented as raising the internal sampling rate to reduce aliasing. The factor is not printed. This is the second explicit oversampling control found in the inspected set, after the Maximizer's 4× soft clipper.
- `OBS-OZ-EXC-004` **OBSERVED** (`SRC-OZONE12-EXCITER`, Post Filter): A **Post Filter** high-shelf acts on the wet output only, and the module's histogram display updates to reflect the post-filter state.
- `OBS-OZ-EXC-005` **OBSERVED** (`SRC-OZONE12-EXCITER`, Controls): Per-band **Amount** and **Mix** controls exist; neither range nor default is printed. A **Link Bands** control couples bands.
- `OBS-OZ-EXC-006` **OBSERVED** (`SRC-OZONE12-EXCITER`, Module Header): A **Learn** feature automatically positions crossovers at minima in the spectrum. The same assistive pattern appears in the Imager (`OBS-OZ-IMG-007`) — assistive crossover placement is a per-module convention, not a one-off.
- `OBS-OZ-EXC-007` **OBSERVED** (`SRC-OZONE12-EXCITER`, Controls): Channel processing modes are Stereo, Mid/Side, and Transient/Sustain — Left/Right is absent from this module's documented set.
- `GAP-OZONE-0032` **SOURCE-GAP**: The Exciter's oversampling factor, its available oversampling settings, and the resulting latency are not stated.
- `GAP-OZONE-0033` **SOURCE-GAP**: Amount and Mix ranges, units, and defaults are not stated.

## Low End Focus

- `OBS-OZ-LEF-001` **OBSERVED** (`SRC-OZONE12-LOW-END-FOCUS`, Difference Meter and Action Region): The module operates on a bounded **action region** spanning 20 Hz to 300 Hz, whose cutoffs are adjustable by dragging handles individually, dragging both together, or typing values.
- `OBS-OZ-LEF-002` **OBSERVED** (`SRC-OZONE12-LOW-END-FOCUS`, Controls): Two modes are documented — **Punchy**, using faster response times to emphasize transient content, and **Smooth**, using slower response times to enhance sustained content. No time constants are printed for either.
- `OBS-OZ-LEF-003` **OBSERVED** (`SRC-OZONE12-LOW-END-FOCUS`, Controls): **Contrast** is a bipolar control over the spectral contrast between low- and high-level signals. Positive values increase the difference and attenuate low-level content for a punchier result; negative values decrease the difference and blur the low end in a way the page compares to saturation.
- `OBS-OZ-LEF-004` **OBSERVED** (`SRC-OZONE12-LOW-END-FOCUS`, Controls): **Gain** sets makeup gain applied to the action region only, not to the full-band signal.
- `OBS-OZ-LEF-005` **OBSERVED** (`SRC-OZONE12-LOW-END-FOCUS`, Module Header; Difference Meter and Action Region): Auditioning is served by a Delta meter monitoring the before/after difference, a Difference Meter display showing tonal change adaptively over time, and a Solo control that isolates the **input** signal within the action-region cutoffs — that is, solo auditions the band pre-processing, not the processed result.
- `GAP-OZONE-0034` **SOURCE-GAP**: Contrast and Gain ranges, units, and defaults are not stated, and no response-time values are given for Punchy or Smooth.

## Master Rebalance

- `OBS-OZ-REBAL-001` **OBSERVED** (`SRC-OZONE12-MASTER-REBALANCE`, Controls → Focus): A **Focus** control selects exactly one of three stem elements — Vocals, Bass, or Drums. Only one focus element can be active at a time.
- `OBS-OZ-REBAL-002` **OBSERVED** (`SRC-OZONE12-MASTER-REBALANCE`, Controls → Gain): A single **Gain** control adjusts the level of the selected focus element. Range, units, and default are not printed.
- `OBS-OZ-REBAL-003` **OBSERVED** (`SRC-OZONE12-MASTER-REBALANCE`, Overview): The page describes real-time gain adjustment of the focus element. No separate analysis pass is documented as a precondition.
- `OBS-OZ-REBAL-004` **OBSERVED** (`SRC-OZONE12-MASTER-REBALANCE`, Spectrum View): The display shows the focus spectrum and the residual spectrum in distinct colors, so the user can see what has been separated from what. A Delta monitor and a Reset are also present.
- `OBS-OZ-REBAL-005` **OBSERVED** (`SRC-OZONE12-MASTER-ASSISTANT`, Vocal Balance; `SRC-OZONE12-MASTER-REBALANCE`, Overview): Master Rebalance is the module Master Assistant applies when its Vocal Balance stage decides the vocal level needs correcting. The separation module and the assistive layer are coupled by design.
- `GAP-OZONE-0035` **SOURCE-GAP**: The Gain range, units, and default for Master Rebalance are not stated.
- `GAP-OZONE-0036` **SOURCE-GAP**: The page states no latency figure, no sample-rate limitation, no quality/performance setting, and no artifact caveat for a module that separates and re-mixes already-mixed material. Silence is not evidence that these constraints do not exist; it is a documentation gap, and a significant one for anyone reasoning about real-time use.
- `OBS-OZ-REBAL-006` **SPECTRE-CANDIDATE**: The user-facing contract worth noting is narrow and honest: one selected element, one gain, and a visualization of what was separated. Any Spectre equivalent would need original separation work and would need to document the latency and sample-rate constraints this page omits.

## Match EQ

- `OBS-OZ-MATCH-001` **OBSERVED** (`SRC-OZONE12-MATCH-EQ`, Reference Spectrum Snapshot; Apply To Spectrum Snapshot): Matching uses **two** captured snapshots, not one. A Reference Spectrum snapshot captures the target, and an "Apply To" snapshot captures the current material. Each is captured by starting a capture, playing audio back, and stopping; each can be cleared.
- `OBS-OZ-MATCH-002` **OBSERVED** (`SRC-OZONE12-MATCH-EQ`, Reference Spectrum Snapshot; Apply To Spectrum Snapshot): The two snapshots have **asymmetric persistence** — Reference Spectrum snapshots are saved with Ozone presets, and Apply To snapshots are not. A preset therefore carries the target but re-derives the source.
- `OBS-OZ-MATCH-003` **OBSERVED** (`SRC-OZONE12-MATCH-EQ`, Overview): The matching filter is documented as a digital linear-phase EQ, and matching resolution is characterized as over 8,000 frequency bands.
- `OBS-OZ-MATCH-004` **OBSERVED** (`SRC-OZONE12-MATCH-EQ`, Fine Tune): **Smoothing** trades precision for smoothness — higher values are less precise. **Amount** scales the matching intensity. Neither range nor default is printed; the page's own worked example suggests keeping Amount under 50%.
- `OBS-OZ-MATCH-005` **OBSERVED** (`SRC-OZONE12-MATCH-EQ`, Spectrum and Matched EQ Curve): The display distinguishes the reference curve, the Apply To curve, and the resulting matched curve by color, so the user can see target, source, and correction simultaneously.
- `GAP-OZONE-0037` **SOURCE-GAP**: Match EQ's latency is not stated, despite the page identifying the filter as linear-phase, which necessarily implies a latency.
- `GAP-OZONE-0038` **SOURCE-GAP**: Smoothing and Amount ranges, units, and defaults are not stated, and the "over 8,000 bands" figure is not tied to a stated FFT size or bin spacing.

## Stabilizer

- `OBS-OZ-STAB-001` **OBSERVED** (`SRC-OZONE12-STABILIZER`, Controls): A **Target** control selects the tonal-balance profile the module corrects toward. Documented entries include All-purpose and Bass Heavy, plus an **Assistant** target that becomes available when a reference file has been selected in Master Assistant. The complete target list was not enumerated in the inspected extraction.
- `OBS-OZ-STAB-002` **OBSERVED** (`SRC-OZONE12-STABILIZER`, Controls): **Mode** has two documented settings — **Shape**, which applies both boosts and cuts and is described as loudness-neutral, and **Cut**, which applies cuts only to tame resonances exceeding the target bounds.
- `OBS-OZ-STAB-003` **OBSERVED** (`SRC-OZONE12-STABILIZER`, Controls): **Amount** scales the adaptive correction gain, and the page states a maximum boost of **9 dB** at Amount 100. This is the module's only printed dB figure.
- `OBS-OZ-STAB-004` **OBSERVED** (`SRC-OZONE12-STABILIZER`, Controls): **Speed** controls how quickly the correction reacts to incoming audio; the page warns that higher speeds are more precise but may introduce artifacts.
- `OBS-OZ-STAB-005` **OBSERVED** (`SRC-OZONE12-STABILIZER`, Controls): **Smoothing**, available in Shape mode only, is documented in terms of *effective filter count* rather than bandwidth — at 100 the correction is equivalent to roughly three or four filters, and at 0 it spreads across many filters. Lower values are more precise but riskier.
- `OBS-OZ-STAB-006` **OBSERVED** (`SRC-OZONE12-STABILIZER`, Controls): **Sensitivity** spans 0 to 100. At 0 the module permits everything but excessive resonances; at 100 it suppresses any deviation from the target. It is the tolerance-band width control.
- `OBS-OZ-STAB-007` **OBSERVED** (`SRC-OZONE12-STABILIZER`, Controls): **Tame Transients** enables instant tonal correction for transient material, sitting outside the Speed control's normal reaction behavior.
- `OBS-OZ-STAB-008` **OBSERVED** (`SRC-OZONE12-STABILIZER`, Controls): Correction amount is separately scalable in three fixed frequency regions with printed boundaries — Low below 100 Hz, Mid from 100 Hz to 5.6 kHz, and High above 5.6 kHz.
- `GAP-OZONE-0039` **SOURCE-GAP**: Defaults for Amount, Speed, Smoothing, and Sensitivity are not stated; no latency figure is given; and the complete Target enumeration was not captured.

## Presets and persistence

- `OBS-OZ-PRESET-001` **OBSERVED** (`SRC-OZONE12-PRESET-SYSTEM`, Factory and Custom Presets): The preset manager separates factory presets (an iZotope tab) from user presets (a Custom tab listing presets the user saved or modified).
- `OBS-OZ-PRESET-002` **OBSERVED** (`SRC-OZONE12-PRESET-SYSTEM`, Overview): Presets exist at **two scopes** — a global preset manager reached from the header, and module-specific preset managers. A mastering chain and a single module are both presettable objects.
- `OBS-OZ-PRESET-003` **OBSERVED** (`SRC-OZONE12-PRESET-SYSTEM`, Preset Manager Footer): The footer offers New (create a preset from current settings) and Update (save changes to a modified custom preset, documented as available in the global preset manager only).
- `OBS-OZ-PRESET-004` **OBSERVED** (`SRC-OZONE12-PRESET-SYSTEM`, Custom Default Preset; Default and Working Settings): A user-defined **custom default preset** is supported, and the guide distinguishes default settings from working settings as separate concepts.
- `OBS-OZ-PRESET-005` **OBSERVED** (`SRC-OZONE12-PRESET-SYSTEM`, Custom Preset Names and Comments): Presets carry both a name and a free-text comment field.
- `OBS-OZ-PRESET-006` **OBSERVED** (`SRC-OZONE12-PRESET-SYSTEM`, Preset Locations): Presets are filesystem objects in documented locations. Factory presets live under `C:\Program Files\iZotope\Ozone\Presets\` on Windows and `/Library/Application Support/iZotope/Ozone/Presets/` on macOS. User presets default to `Documents\iZotope\Ozone\User Presets\` under the user profile on both platforms. Folders are organizable by drag and drop.
- `OBS-OZ-PRESET-007` **OBSERVED** (`SRC-OZONE12-PRESET-SYSTEM`, Preset Locations): The page acknowledges legacy storage locations for presets created by Ozone 10 builds before 10.2.0 and by Ozone Pro. Legacy locations are documented; a migration mechanism is not.
- `OBS-OZ-PRESET-008` **OBSERVED** (`SRC-OZONE12-MATCH-EQ`, Reference Spectrum Snapshot): At least one module stores captured *analysis data* — not just parameter values — inside a preset (`OBS-OZ-MATCH-002`). Preset state is therefore not purely a parameter snapshot.
- `GAP-OZONE-0040` **SOURCE-GAP**: The page does not state what a preset contains. Whether a global preset stores the module chain order, I/O gains, channel processing mode, metering options, or reference tracks is unestablished.
- `GAP-OZONE-0041` **SOURCE-GAP**: The preset file extension and serialization format are not stated, and no forward or backward compatibility rule across Ozone versions is documented.
- `OBS-OZ-PRESET-009` **SPECTRE-CANDIDATE**: Two facts here are worth carrying into a Spectre device-state contract as *questions Spectre must answer explicitly*, since Ozone's documentation does not: (a) what exactly is in a device preset versus what is session state, and (b) what happens to captured analysis data inside a preset when the analysis format changes between versions. Both are persistence-contract problems, and Spectre should specify them rather than leave them to be discovered.
