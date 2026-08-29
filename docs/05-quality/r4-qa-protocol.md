<!--
Author: Jeff
Date: 2026-08-28
Description: The written manual QA protocol R4's exit evidence requires
Notes: Flow B. The automated half is `cargo test -p spectre-offline --test e2e_alpha`; this
  document is the half a human performs, and its output is one numbered block in
  `r4-qa-records.md`. A record with a blank Linux column is valid; one that omits the column is not
-->

# R4 QA protocol

- **Status:** accepted
- **Last verified:** 2026-08-28
- **Scope:** the manual half of R4's end-to-end exit evidence
- **Decision authority:** Jeff
- **Upstream sources:** `../06-plans/current-milestone.md` §"Exit evidence"; `../01-requirements/decision-gates.md` rows 1, 17, 23; `../00-product/vision.md`
- **Downstream dependents:** `r4-qa-records.md`, R4 exit review
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** whether R4's exit narrows while Linux device qualification is undischarged — decision 23's shape, and Jeff's call
- **Known gaps:** no operator run has been performed on any platform; `r4-qa-records.md` carries one `INCONCLUSIVE` automated-half record (block Q-1)

## Standing constraints at the time of writing

Read this section before running, because several rows below cannot pass today and the record
must say so rather than record a failure against the operator.

- ~~**Linux device qualification has never run.**~~ **Discharged 2026-08-28** on Arch Linux
  across three rows in the milestone's qualification record, one of them on the raw ALSA path
  with no sound server. A macOS run still produces no Linux evidence and the reverse holds too:
  each platform's column is filled only by a run on that platform.
- ~~**The live bridge addresses exactly one instrument.**~~ **Lifted 2026-08-28** by the R4-9
  follow-up, which is the update that paragraph required of itself. `RenderBridge` now carries a
  fixed `Box<[ClipVoice]>` sized from `spectre_graph::MAX_FLAT_INPUTS`, so every instrument track
  sounds live. `the_bridge_can_only_deliver_notes_to_one_instrument` was removed;
  `the_live_bridge_plays_every_instrument_track` replaces it, with
  `one_voice_does_not_sound_like_three` as its control.
- ~~**The app does not attach a clip player to its audition path.**~~ **Lifted 2026-08-28.**
  `build_track_engine_parts` bakes each track's active placements before the stream opens and
  attaches them, so Play sends the transport command alone and the project's own material
  REPLACES the audition note. A project with **no** clip placements attaches no player and still
  auditions, so row 1 records which of the two it heard rather than assuming the audition voice.
- **The alpha's tracks are `Filament → Gloam` as of 2026-08-28**, and were `Filament` alone before that: `build_track_graph` constructed no effect, so row 3's "drag a `Gloam` parameter" had no live control to drag and row 9's bounce carried no effect. A run recorded before that date measured a different product.
- **Row 3 can pass as of 2026-08-28, and could not before it.** `./spectre` opens the track engine, whose lane targets are graph-node ObjectIds, while a Shape slider addresses the flat `AppModel::devices` list's own IDs. Every Shape edit was refused as an unknown target until `apply_parameter_edit` began resolving the destination by role on the selected track. Drag an **instrument level, a Gloam depth, or a gain** — those are the three roles a track hosts. `Saturator` is on no track, so its edit is still model-only and reports so; that is correct behaviour, not a row-3 failure.
- **`Gain` does not smooth**, though the accepted device contract says it does. Open as D-R3.
- **No autosave, journal, or recovery exists.** Row 8 checks replacement, not crash durability;
  crash injection is R5's.

## Preconditions

Fill every environment field before starting. Each comes from a named command on the host or
from direct observation — never from `docs/status/STATUS.md`, a sibling spec, or a previous run.

| Field | How to obtain it |
|---|---|
| E1 | Distribution/OS and release. macOS: `sw_vers`. Linux: `/etc/os-release` |
| E2 | Kernel or Darwin release and machine architecture: `uname -a` |
| E3 | Audio subsystem version. Linux: `cat /proc/asound/version`. macOS: CoreAudio with the OS build from `sw_vers -buildVersion` |
| E4 | The sound server mediating the device, if any, and **how that was determined** |
| E5 | Interface make/model and connection type (onboard / USB / PCI / Thunderbolt) |
| E6 | The device key **exactly as the app reports it** in the transport bar |
| E7 | Spectre commit SHA (`git rev-parse HEAD`), `rustc --version`, cargo features in effect |
| E8 | Requested geometry and granted geometry, or `granted: not recorded` with the reason |
| E9 | The workspace gate's verbatim summary line, run immediately before this session |
| E10 | Verbatim panic message or driver error string, or `none` |

## Procedure

1. Run the workspace gate and record its summary line as E9:
   `cargo fmt --all -- --check && cargo clippy --locked --workspace --all-targets -- -D warnings && cargo test --locked --workspace`
2. Run Flow A and record its verbatim libtest summary as Q-A:
   `cargo test --locked -p spectre-offline --test e2e_alpha -- --nocapture`
   Transcribe the `e2e frames=… channels=… peak=… hash=… blocks=…` line into Q-A as well. **That
   hash is a per-platform observation, not a golden constant** — cross-platform bit equality is
   undecided, because device math routes through the platform's libm.
3. Write the fixture project to a scratch path so rows 7–9 have a file to work with:
   `cargo run --locked -p spectre-offline -- --write-alpha-fixture --out /tmp/r4-alpha.json`
4. Launch the app **with system volume low**: `./spectre`. The audition voice is a held saw with
   no amplitude envelope and it clicks at note edges.

   **On a host with no ALSA default, `./spectre` will report the engine unavailable.** Check with
   `grep -rl 'pcm.!default' /etc/alsa/conf.d/`; an empty result means ALSA falls back to
   `defaults.pcm.card 0`, which may be an HDMI-only card with no device 0. Two fixes, either is
   valid and **the record's E4 must say which was used**:

   - Install the distribution's default wiring — on Arch, `pipewire-alsa`, which ships
     `99-pipewire-default.conf`. This is the ordinary desktop configuration.
   - Or supply one for the run without touching the system:

     ```sh
     cat > /tmp/spectre-alsa.conf <<'EOF'
     </usr/share/alsa/alsa.conf>
     pcm.!default { type pipewire }
     ctl.!default { type pipewire }
     EOF
     ALSA_CONFIG_PATH=/tmp/spectre-alsa.conf ./spectre
     ```

     Replace both `pipewire` lines with `plug`/`hw` on a host with no sound server — e.g.
     `pcm.!default { type plug; slave.pcm "hw:1,0" }` — which is what the raw-ALSA qualification
     row used. `/tmp` is cleared on reboot, so this is per-session by design.
5. Work rows 1–15 of §Manual checks in order. Record a verdict for **every** row. `NOT RUN` is a
   valid verdict; a blank is not.
6. Write one numbered block into `r4-qa-records.md` with all of E1–E10 and Q-A…Q-D.
7. Apply §Outcome rules **per platform**. Do not compute an aggregate word.

## Manual checks

| # | Row | What the operator does and records |
|---|---|---|
| 1 | **Audibility** | Press Play. Record **what was heard** — pitch, character, whether it matched what was expected — not that "audio worked". Record whether the onset felt immediate or delayed as an operator judgement; **no latency threshold is defined**, so record the judgement and no number. Note explicitly that this is the audition voice, not the fixture's clips |
| 2 | **Silence at rest and after Stop** | No sound before Play. After Stop, **exact** silence — not a fading tail and not a low hum |
| 3 | **A slider drag reaches live audio** | With sound playing, drag a device parameter in Shape. The sound changes, and the transport counters read `params pending 0`. This row is the only coverage the app's binding rule has, because it lives in `main.rs` |
| 4 | **Transport honesty** | The position readout does not show a frozen or fabricated position when the engine is not running. **Changed 2026-08-28:** it now reads bars.beats.sixteenths from the render thread's own published playhead. Confirm it reads `—` before any block has rendered — not `1.1.1` — and that it advances while rolling. A project with no clips does not advance the playhead, so a still readout there is correct, not frozen |
| 5 | **Engine-unavailable state** | Launch with the default output device disabled or unplugged. The app opens, states that the engine is unavailable, produces no sound, does not crash, and does not claim to be playing |
| 6 | **Device loss mid-playback** | Unplug the interface while playing. No crash; the engine state or counters reflect the loss; reconnecting and pressing Retry recovers. `NOT RUN` if the host has no detachable interface |
| 7 | **Save / quit / reopen** | Type a path in the PROJECT block, add two tracks, save, quit, relaunch, open. Same tracks, same order, same names, same selected track, same lens |
| 8 | **Save over an existing file** | Save onto an existing project file. The previous file is either fully replaced or fully intact — never half-written. Also confirm the directory holds exactly one file afterward, with no temporary left behind |
| 9 | **Bounce the fixture and listen to it** | Press `Bounce…`, type a destination, start. Open the written WAV in any player. It sounds like what was heard live and its duration matches the requested length. "Sounds like" is a recorded judgement, not a measurement |
| 10 | **Screen size extremes** | At the 1060×680 minimum and the 1420×860 default, every panel stays inside the window and no control is clipped, with the bounce panel open and closed |
| 11 | **Motion** | Note any animated transition with no reduced-motion path. Recorded as a finding, not a gate |
| 12 | **Keyboard-only pass** | Attempt rows 1–9 without a pointer. Record **which steps were impossible**. Record **no key assignments** — the accepted research corpus does not support a default shortcut map, and proposing one here would ratify it by the back door |
| 13 | **Screen-reader pass** | Repeat rows 1–9 with VoiceOver or Orca. Record which controls announced no label. `eframe`'s accessibility feature is not enabled, so an empty result is a build-configuration finding, not a labeling audit |
| 14 | **Theme variants** | **N/A — the shell hard-codes a single dark palette.** There is no light variant to check and introducing one is not this feature's work |
| 15 | **Traceability and status** | Read `../01-requirements/traceability.md` and `../status/STATUS.md` against this run's findings. Record every claim in them that this run contradicts |

**Two things the operator must not do**, stated because both are natural and both destroy the
record's value.

- **Do not use audibility to infer a passing hash, or a passing hash to infer audibility.** They
  are independent evidence classes. The case where every hash agrees and nothing is heard is the
  highest-value failure this protocol exists to catch.
- **Do not re-run a failing row until it passes and record only that.** Record the failure, then
  record the re-run as a separate numbered block.

## Outcome rules

Applied **per platform**, never across platforms.

| Outcome | Definition | Effect on R4's exit |
|---|---|---|
| `PASS` | Flow A passes in full, **and** every manual row is `PASS` or the structural `N/A` of row 14 | The end-to-end exit row is satisfied **for this platform only**, subject to §What a PASS authorizes |
| `FAIL` | Flow A fails any assertion, **or** any manual row is `FAIL` | The row stays open. The record is written and kept; the finding is routed to the owning slice |
| `INCONCLUSIVE` | Flow A passes but one or more manual rows are `NOT RUN` for host reasons, or the workspace gate did not run clean immediately before | The row stays open. Nothing adverse was learned |
| `BLOCKED` | One or more sibling features are unimplemented, so the rows depending on them have no surface to exercise | The row stays open |
| `REFUSED` | The host's audio device declined the requested geometry, so the engine never opened | The row stays open. This is a finding about the seam, not about the operator |

**A green `cargo test` is necessary and not sufficient.** A run may be `BLOCKED` while every test
that exists passes. Evaluate these rules against the record, not against the exit code.

**There is no aggregate word.** A macOS `PASS` beside an empty Linux column is a macOS `PASS`
and nothing more.

## What a PASS authorizes

A `PASS` on one platform authorizes exactly one sentence, of this shape and no broader:

> On {date}, on {OS} {release} with {audio subsystem}, at commit {SHA}, Spectre's end-to-end
> fixture rendered, round-tripped, and matched the live path in {N} automated assertions, and an
> operator worked the {M} manual rows of the R4 QA protocol on {interface}, with the results
> recorded in `r4-qa-records.md` block {Q-n}.

It does **not** authorize, and the record's Q-D field states each of these that applies:

1. **"R4 has exited."** R4's exit is a ten-row conjunction. This produces evidence for one row
   and contributes to several others; it produces **none** for Linux device qualification.
2. **"Spectre works on Linux."** The *device seam* is qualified on Linux as of 2026-08-28; the
   *shell* is not, because no operator has worked these rows on any platform. A combined verdict
   across a filled column and an empty one is still exactly the unauthorized claim decision 23
   forbids — it just is no longer the device layer that is missing.
3. **Any other machine, interface, or driver configuration.** One host, one interface, one
   configuration.
4. **Crash durability or recovery.** CORE-004's crash-injection evidence is R5's.
5. **That multi-track live playback holds anywhere but this host.** The bridge now addresses
   every instrument track, so the R4-9 limit is gone; what a run establishes is still one host,
   one interface, one configuration.
