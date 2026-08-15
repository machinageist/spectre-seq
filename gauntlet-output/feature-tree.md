<!--
Author: Jeff
Date: 2026-08-14
Description: Confirmed gauntlet feature tree for the R4 Credible Alpha milestone
Notes: Derived from the accepted milestone and slice queue; not discovered from source
-->

# Spectre — Gauntlet Feature Tree

- **Status:** proposed
- **Last verified:** 2026-08-14
- **Scope:** the nine R4 features the gauntlet will spec, in dependency order
- **Decision authority:** Jeff
- **Upstream sources:** `../docs/06-plans/current-milestone.md`, `../docs/status/NEXT.md`
- **Downstream dependents:** `manifest.md`, every file under `specs/`
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** awaits Jeff's sign-off; stays `proposed` until then
- **Known gaps:** R4-3 is hardware-blocked; later milestones are deliberately not decomposed

## Derivation

This tree is **not** discovered from the source tree. It is transcribed from the
accepted R4 slice queue in `docs/status/NEXT.md`, preserving that document's order,
because the roadmap already owns dependency ordering and the gauntlet has no authority
to re-plan it.

The 2026-08-12 run skipped this step and invented a "track management" feature with
APIs that do not exist. Deriving the tree from the accepted plan is the structural fix.

## Tree

```text
R4 Credible Alpha (root — umbrella)
├── R4-1  live-audio-wiring          wire ./spectre to the qualified backend
├── R4-2  runtime-parameter-seam     decision 22 on AudioProcessor
├── R4-3  linux-device-qualification decision 23 debt  [HARDWARE-BLOCKED]
├── R4-4  track-model                track to master signal path
├── R4-5  midi-clips                 clips that play through a track
├── R4-6  first-devices              one original synth + one original effect
├── R4-7  project-persistence        CORE-004 atomic save/reload + CORE-001 reorder
├── R4-8  offline-bounce             bounce matching the live path
└── R4-9  e2e-and-qa                 end-to-end fixture + manual QA protocol
```

Nine leaves, no branch exceeding three children, so dispatch is one agent per leaf.
The root gets an umbrella spec only after the leaves pass, since R4's exit criteria
are the union of theirs.

## Features

### R4-1 — live-audio-wiring
Wire `./spectre` to the qualified backend so Play produces sound through the existing
compiled plan. No new render path, no new DSP.
**Why first:** R3 built a live shell nothing launches. This is what makes every later
R4 slice observable, and `current-milestone.md` calls it "the honest headline."
**Touches:** `crates/spectre-app`, `crates/spectre-audio`.
**Watch:** AF-2 and 4F — the current no-sound state must be stated plainly in §7.1.

### R4-2 — runtime-parameter-seam
Implement decision 22's accepted design — option (a), a callback-safe parameter setter
on `AudioProcessor` — then connect the RT-002 parameter lane the bridge already drains.
**Acceptance:** a Shape edit changes live audio and `parameters_pending` stops
incrementing.
**Design authority:** `docs/03-architecture/dsp-device-io.md` §"Runtime parameter seam".
The spec implements that contract; it does not redesign it (AF-1).
**Watch:** AF-3. Option (b), recompiling a plan per change, is already rejected.

### R4-3 — linux-device-qualification  ·  HARDWARE-BLOCKED
Discharge decision 23's debt: `cargo test -p spectre-audio --test lifecycle_health --
--ignored --nocapture` on a Linux host with a real ALSA device, recorded in the
milestone's qualification table.
**Blocked on:** access to Linux audio hardware. One command; it only needs the box.
**Spec scope:** the qualification protocol and its record format — what counts as a
pass, what gets written down. The spec is authorable now; only execution is blocked.
Marked blocked at intake so it never stalls the loop.
**Watch:** no Linux support claim is authorized until this runs (decision 23).

### R4-4 — track-model
Introduce the track model with a track-to-master signal path, keeping the graph
compilation contract intact.
**Note:** this is the subject the discarded spec claimed to describe. It is unstarted.
Its spec starts from an empty §7.1.
**Watch:** AF-4 on any track-count or channel bound; GRAPH-001 split in 1E.

### R4-5 — midi-clips
MIDI clips that play through a track, reusing the existing bounded event-ordering
contract and the MIDI ingress landed in R3 — not a second event path.
**Watch:** 1D determinism at equal timestamps; the accepted `NoteEventKind::rank`
ordering key is reused, not restated.

### R4-6 — first-devices
One small original synth and one original effect as the alpha's voice.
**Scope guard:** decision 15 makes the smallness deliberate. Criterion 3C fails a spec
that drifts toward the R11 flagship, and AF-5 forbids asserting a synthesis
architecture or modulation limit at this evidence level.

### R4-7 — project-persistence
CORE-004 atomic save and reload over the accepted persistence contract, plus CORE-001's
reorder evidence on the first persisted collection.
**Design authority:** `docs/03-architecture/project-persistence.md` — accepted API
design; the filesystem implementation lands here. Crash qualification is R5, not R4.

### R4-8 — offline-bounce
Offline bounce proven to match the live path's computation, extending the existing
hash-equivalence approach rather than inventing a comparison.
**Watch:** 1D — reuse the FNV-1a walk already shared by `spectre-offline` and the
bridge equivalence test.

### R4-9 — e2e-and-qa
The end-to-end fixture and manual QA protocol R4's exit requires.
**Depends on:** every other leaf. Spec it last.

## Explicitly out of tree

Not R4 scope, not to be spec'd by this run: VST3 hosting (R8), the modular/synth
identity layer (R11), recording (R5+), the launcher and automation surfaces (R9–R10),
and the accessibility audit beyond decision 17's R4 scoping. The parallel research
tasks in `NEXT.md` are research, not features.

## Dispatch order

1. **R4-1** alone — it unblocks observability for everything after it.
2. **R4-2, R4-4, R4-7** — concurrency 3.
3. **R4-5, R4-6, R4-8** — concurrency 3.
4. **R4-3, R4-9** — R4-3's spec is authorable at any point; R4-9 needs the rest first.
