<!--
Author: Jeff
Date: 2026-07-11
Description: Vetted public DSP engineering literature register for Geist native DSP work
Notes: Engineering literature, not product clean-room sources; the product source ledger stays separate
-->

# DSP Engineering Literature Register

- **Status:** draft
- **Last verified:** 2026-07-11
- **Scope:** authoritative public literature Geist native DSP contracts and implementations may cite
- **Decision authority:** Jeff
- **Upstream sources:** verification fetches recorded per entry
- **Downstream dependents:** DSP verification strategy, native-device specs, flagship-synth DSP, code-level algorithm citations
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** whether to vendor local snapshots of freely copyable texts into `fixtures/` for offline reproducibility
- **Known gaps:** fractional-delay and physical-modeling primary papers are identified but not yet URL-verified

## Role and rules

This register exists so that every Geist DSP algorithm can cite its mathematical basis per rebuild-mandate §12.4 ("original algorithms and clearly documented references where standard DSP literature is used"). These are engineering references: standard public knowledge, not competitor products. Rules:

1. Geist implements algorithms from theory; it does not copy source code from books or forums unless the license explicitly permits and the provenance is recorded.
2. Community snippet archives are inspiration and cross-checks only — snippet licensing is per-post and usually unstated, so no code may be copied from them.
3. Every DSP module in the rebuild MUST cite the register entries it draws on, in its spec or code header.

## Verified entries (accessed 2026-07-11)

| ID | Work | Author | Access | License / terms | Geist relevance |
|---|---|---|---|---|---|
| `DSPREF-JOS-MDFT` | Mathematics of the Discrete Fourier Transform | Julius O. Smith III | `https://ccrma.stanford.edu/~jos/mdft/` (free online; verified via author homepage) | online reading free; standard copyright | DFT/FFT foundations, windowing, spectral tests |
| `DSPREF-JOS-FILTERS` | Introduction to Digital Filters | Julius O. Smith III | `https://ccrma.stanford.edu/~jos/filters/` | as above | filter theory, transfer functions, stability, state-space |
| `DSPREF-JOS-PASP` | Physical Audio Signal Processing | Julius O. Smith III | `https://ccrma.stanford.edu/~jos/pasp/` | as above | delay lines, waveguides, reverberation, physical models |
| `DSPREF-JOS-SASP` | Spectral Audio Signal Processing | Julius O. Smith III | `https://ccrma.stanford.edu/~jos/sasp/` | as above | STFT, phase vocoder, time-stretch/pitch-shift theory |
| `DSPREF-SWSMITH-GUIDE` | The Scientist and Engineer's Guide to Digital Signal Processing | Steven W. Smith | `https://www.dspguide.com/` (entire book downloadable without charge; verified) | free browsing/download per site | convolution, recursion, Fourier, fixed/float numerics |
| `DSPREF-RBJ-EQ-COOKBOOK` | Audio EQ Cookbook (W3C Working Group Note, 2021-06-08, ed. Raymond Toy; original by Robert Bristow-Johnson) | RBJ / W3C | `https://www.w3.org/TR/audio-eq-cookbook/` (verified) | W3C permissive document license | biquad coefficient formulas: LPF/HPF/BPF×2/notch/APF/peaking/shelves via bilinear transform |
| `DSPREF-ZAVALISHIN-VA` | The Art of VA Filter Design | Vadim Zavalishin (Native Instruments) | official host `https://www.native-instruments.com/fileadmin/ni_media/downloads/pdf/VAFilterDesign_1.1.1.pdf` (rev 1.1.1 URL verified via search index; current rev 2.1.2, 2020-02-14, same host path pattern — direct fetch returned 403 to our tool, browser access expected) | book grants right to freely copy in full with copyright note, unmodified | TPT / zero-delay-feedback virtual-analog filter design — the standard text for Geist's VA filters |

## Identified, not yet URL-verified

| ID | Work | Author | Why it matters | Verification gap |
|---|---|---|---|---|
| `DSPREF-STILSON-BLIT` | Alias-Free Digital Synthesis of Classic Analog Waveforms (BLIT) | Stilson & Smith (CCRMA) | bandlimited oscillator theory behind BLIT/BLEP/polyBLEP families | canonical CCRMA URL not yet fetched |
| `DSPREF-LAAKSO-FRACDELAY` | Splitting the Unit Delay (IEEE SP Magazine, 1996) | Laakso, Välimäki, Karjalainen, Laine | fractional-delay interpolation for resampling, warping, modulated delays | paywalled; cite by identity, use JOS coverage for free access |
| `DSPREF-SCHROEDER-REVERB` | Natural Sounding Artificial Reverberation (1962) + Moorer (1979) | Schroeder; Moorer | comb/allpass reverb topologies | historical papers; JOS PASP covers the material freely |

## Community resources (cross-check only, no code copying)

- `https://www.musicdsp.org/` — snippet archive; licensing per-post and mostly unstated. Use to discover technique names and cross-check behavior, never as a code source.
- KVR Audio DSP and Plugin Development forum — practitioner discussion (e.g., the VA Filter Design book thread); evidence of practice, not of correctness.

## How these map to likely Geist DSP domains

| Geist domain | Primary references |
|---|---|
| Oscillators (VA waveforms, wavetables) | DSPREF-STILSON-BLIT, DSPREF-JOS-FILTERS (bandlimiting), DSPREF-ZAVALISHIN-VA (phase structures) |
| Filters (musical VA) | DSPREF-ZAVALISHIN-VA (primary), DSPREF-RBJ-EQ-COOKBOOK (fixed-coefficient EQ), DSPREF-JOS-FILTERS (theory/stability) |
| Delays/chorus/flanger | DSPREF-JOS-PASP, DSPREF-LAAKSO-FRACDELAY |
| Reverb | DSPREF-JOS-PASP, DSPREF-SCHROEDER-REVERB |
| Time-stretch/warp/pitch | DSPREF-JOS-SASP (phase vocoder, STFT) |
| Resampling/interpolation | DSPREF-JOS-PASP + JOS resampling material, DSPREF-LAAKSO-FRACDELAY |
| Metering/analysis | DSPREF-SWSMITH-GUIDE, DSPREF-JOS-MDFT |
| Numerics (denormals, float behavior) | DSPREF-SWSMITH-GUIDE + platform documentation |
