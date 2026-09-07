<!--
Author: Jeff
Date: 2026-07-11
Description: Unresolved product/architecture decisions with recommendations and safe defaults
Notes: Safe defaults accepted by Jeff through delegated decision authority on 2026-07-12
-->

# Decision Gates

- **Status:** accepted
- **Last verified:** 2026-08-31
- **Scope:** every decision the audits and mandate flagged as open, with recommendation, reversibility, and safe default
- **Decision authority:** Jeff
- **Upstream sources:** product vision, implementation evidence, and reference research
- **Downstream dependents:** requirements ledger, architecture contracts, R0+ implementation
- **Supersedes:** removed prototype decision material
- **Superseded by:** none
- **Open decisions:** only rows explicitly gated at a future milestone
- **Known gaps:** benchmark and license-audit evidence still missing for several rows

Legend: **SD** = safe default adopted so work can proceed; **GATE** = must be ratified by Jeff before the stated milestone ships.

| # | Decision | Recommendation | Reversibility | Status |
|---:|---|---|---|---|
| 1 | First-class platform order | macOS and Linux co-first-class; Windows at beta. | Medium | **Accepted** |
| 2 | Workspace posture | Repository root is the only workspace; prototype code is removed. | High | **Implemented** |
| 3 | Project encoding | Versioned JSON envelope for R1 because it is inspectable and preserves unknown fields; reconsider binary packing only with measured need. | Medium | **Accepted** |
| 4 | Stable ID strategy | Deterministic seeded 64-bit IDs with zero excluded and project-scoped uniqueness validation; creation sequence owns ordering. | Medium | **Accepted** |
| 5 | Canonical pitch/time types | `SampleTime(i64)`, fixed-point `BeatTicks(i64)` at 960 ticks/beat, and explicit pitch newtypes when pitch enters scope; conversions defined once. | Medium | **Accepted** |
| 6 | f32 vs f64 processing | f32 buffers, f64 for accumulators/coefficients where numerically warranted, documented per module | Medium | **SD adopted** |
| 7 | Graph feedback policy | Explicit one-block delay on declared feedback edges (all three studied modular systems price feedback in explicit unit delays); no implicit cycles | Medium | **SD adopted** for design work; GATE before R11 modular surface |
| 8 | UI stack | egui for the R4 shell; accessibility audit before beta; custom rendering only if measured limits justify it. | Low once beta ships | **Accepted** |
| 9 | VST3 bindings + license | Decide at R8 intake with a fresh audit of SDK tag v3.8.0_build_66 (pinned commit + submodule licenses) vs third-party Rust bindings; no decision now | Medium | GATE at R8 intake |
| 10 | Plugin crash isolation | Out-of-process scanning from day one; in-process hosting with containment at R8; full process isolation evaluated post-1.0 | Medium | **SD adopted** |
| 11 | Sample decode/streaming deps | symphonia for decode (pure Rust) evaluated first; OS codecs as fallback; decision recorded at R7 | High | **SD adopted** for planning |
| 12 | Time-stretch strategy | Own phase-vocoder/transient path long-term (DSPREF-JOS-SASP); no third-party stretch library in the identity path; interim: no stretch until R7 | Medium | **SD adopted** |
| 13 | Undo architecture | Command-pattern transactions over the project model with grouped edits; no state-snapshot diffing. **Ratified 2026-09-06 on R5's own evidence:** inverses are generated at apply time rather than diffed, `InsertTrack`/`RemoveTrack` are exact mutual inverses because the inverse carries the removed `Track` whole, and every level inverse captures the stored — already clamped — value. Commands borrow an `EditScope` (name + track list) rather than a whole document, so the app keeps its resolved device descriptors instead of being reshaped into the wire format. No snapshot-diffing design was built, so nothing here is retro-justified | Low after R5 | **Accepted**, gate discharged at R5 exit review 2026-09-06 |
| 14 | Autosave/recovery model | Journaled autosave to sidecar + atomic rename saves; recovery drill required at R5 exit. **Resolved 2026-08-31:** accepting recovery loads the autosaved state into memory as unsaved, leaves the saved project and sidecar intact, and removes the obsolete sidecar only after a later successful manual Save. | Medium | **Accepted**, recovery-commit semantics ratified by Jeff 2026-08-31 |
| 15 | Small synth vs flagship relationship | R4 ships a deliberately small original synth; R11 designs the flagship and modular identity from accepted requirements without prototype-code reuse. | High now | **Accepted** |
| 16 | Reference-product numeric limits | No copied limits (e.g., wavetable frame sizes, unison caps); every numeric bound in Spectre needs its own rationale row in the requirements ledger | n/a | **SD adopted** (standing rule) |
| 17 | Accessibility baseline | Keyboard-complete operation and screen-reader labels on all commands/params by beta; scoped audit at R4 | Low if deferred | GATE before beta |
| 18 | Config/scripting boundary | Declarative, versioned, validated config; no embedded scripting language pre-1.0 | High | **SD adopted** |
| 19 | Audio backend | `cpal` behind an `AudioBackend` trait seam with a null implementation for CI and offline. Pure Rust, dual MIT/Apache matching this workspace, and covers CoreAudio, ALSA, and JACK from one API. Adopted to get the callback bridge built, not because it is the endgame: cpal owns its stream thread and its device-change and error-callback semantics are uneven across hosts. | High — the trait seam is the point; swapping the implementation must not touch the bridge | **Accepted** 2026-08-09, ratified by Jeff; qualification deferred to the R3 lifecycle drill |
| 20 | Linux backend baseline | ALSA is the qualification baseline because it is present on every Linux host and PipeWire exposes an ALSA compatibility layer. JACK stays available behind a cargo feature for pro routing and never becomes a hard dependency. | High | **Accepted** 2026-08-09, ratified by Jeff |
| 21 | RT-002 overflow policy | Split lanes. Parameter changes are latest-wins per `(device, parameter)` target, so an arbitrarily fast knob sweep occupies one slot and cannot starve the queue. Note and transport messages are strict FIFO and are never dropped; overflow on those lanes is a counted defect surfaced off-thread, not silent loss. | Medium — lane split is structural; per-lane policy is tunable | **Accepted** 2026-08-09, ratified by Jeff; satisfies RT-002's "defined overflow policy" clause |

| 22 | Runtime parameter seam on `AudioProcessor` | Discovered at R3 slice 4: the accepted DSP device I/O contract bakes parameters in at processor construction and exposes no way to change them on a live plan, so callback-side parameter edits have nowhere to go. Options are (a) extend `AudioProcessor` with a callback-safe parameter setter, (b) swap a recompiled plan per change and reclaim the old one, or (c) a separate parameter-application seam alongside the process seam. (b) is rejected on its face: plan compilation allocates and cannot run per knob turn. | Medium — this changes an accepted architecture contract | **Accepted (design)** 2026-08-09, ratified by Jeff: option (a), specified in `../03-architecture/dsp-device-io.md` under "Runtime parameter seam". Implementation lands at R4 following the CORE-004 precedent. R3 does not implement it and R3's exit does not depend on it; the RT-002 parameter lane stays implemented but unconsumed, with the bridge counting pending changes |

| 23 | R3 exit on single-platform qualification | Exit R3 with macOS hardware qualification only, deferring the Linux device drill to R4 as tracked debt. Raised as working against decision 1 (macOS and Linux co-first-class) and reaffirmed by Jeff on 2026-08-09. This narrows R3's exit bar specifically; it does not amend decision 1, and no Linux support claim is authorized until the drill runs on real Linux hardware. | High — the drill exists and runs unchanged on Linux; discharging the debt is one command | **Accepted** 2026-08-09, ratified by Jeff; **debt discharged 2026-08-28** on Arch Linux across three qualification rows, one on the raw ALSA path. The "one command" confidence was wrong: the drill first exposed a real `find_device` defect on ALSA that had to be fixed before any device would open |
| 24 | **Note routing and note-effect devices** | Note events become a routed graph signal with their own buses and edges, a `DeviceClass::NoteEffect` with no audio output, and a separate `NoteProcessor` trait rather than a widened `AudioProcessor::process`. Full design at `../03-architecture/note-routing.md`. Amends two accepted contracts: `dsp-device-io.md` (event output buses, a sixth device layout) and `graph-compilation.md` (a node with zero audio outputs). The existing `PlanNoteInput` bridge path is unchanged, so R4's evidence stays valid. **Four sub-decisions in §6 need answers first** — sequence bands and note IDs for generated events, the state-versus-feedback boundary, and where a note effect sits in a track chain | High — it is the note half of the typed signal model R11 owns, so getting it wrong is paid for twice | **GATE before MIDI effects are implemented**; proposed 2026-09-06 |

Rows marked **SD adopted** proceed now and are re-opened only by evidence. GATE rows block their named milestone, not current work.

Row 19 carries a standing re-open trigger: if the R3 lifecycle drill or the RT-001 guards show cpal allocating, locking, or blocking on a callback-reachable path, the row re-opens and the direct CoreAudio/ALSA option is reconsidered. The trait seam exists so that re-opening costs an implementation, not a redesign.
