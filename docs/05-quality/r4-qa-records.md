<!--
Author: Jeff
Date: 2026-08-28
Description: Append-only record of R4 QA runs performed against `r4-qa-protocol.md`
Notes: One numbered block per run, per platform. A failing run is recorded and kept; a re-run is
  a new block, never an edit to an old one. No cell is sourced from status documents or from a
  previous record — a record that quotes documentation is a document reviewing itself
-->

# R4 QA run records

- **Status:** accepted
- **Last verified:** 2026-08-28
- **Scope:** completed runs of `r4-qa-protocol.md`
- **Decision authority:** Jeff
- **Upstream sources:** `r4-qa-protocol.md`; the host each run was performed on
- **Downstream dependents:** R4 exit review; `../01-requirements/traceability.md`
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** none
- **Known gaps:** only the automated half has been performed (block Q-1); no operator pass exists on any platform

Block Q-1 is the automated half only. R4's end-to-end exit row stays open.

## Q-1 — 2026-08-28, Linux, automated half only

> **Superseded in scope the same day.** Q-1 was recorded against an alpha whose tracks were
> `Filament` alone, because `build_track_graph` constructed no effect. The insert slot landed
> after this block was written, so the fixture Q-1 describes is not the fixture a later run will
> load: its plan was 8 nodes, not 11, and its Flow A hash was taken on a render with no `Gloam`
> in it. The environment fields E1–E8 and E10 still describe this host; Q-A does not describe the
> current fixture.

**Performed by an agent session, not an operator.** Rows 1–13 of §Manual checks require a human
at the keyboard — hearing pitch and onset, dragging a slider, unplugging an interface, running a
screen reader. They are recorded `NOT RUN` below rather than inferred from tests, which is the
distinction §"Two things the operator must not do" exists to protect. This block is written so
the environment and the automated evidence are not lost, not to stand in for an operator pass.

| Field | Value |
|---|---|
| E1 | Arch Linux, `BUILD_ID=rolling` (`/etc/os-release`) |
| E2 | `Linux mg-arch 7.1.9-arch1-2 #1 SMP PREEMPT_DYNAMIC Fri, 21 Aug 2026 22:18:59 +0000 x86_64 GNU/Linux` |
| E3 | ALSA `k7.1.9-arch1-2` (`/proc/asound/version`) |
| E4 | PipeWire 1.6.8 with `pipewire-pulse`, from `pgrep -a pipewire` and `pactl info` reporting `PulseAudio (on PipeWire 1.6.8)`. **The ALSA `default` PCM was not mediated by anything:** the host has no `pcm.!default` — `pipewire-alsa`'s `99-pipewire-default.conf` is absent — so ALSA fell back to `defaults.pcm.card 0`, an HDMI-only NVidia card with no device 0. A default was supplied for the run through `ALSA_CONFIG_PATH` at a temporary file; nothing on the host was installed or changed |
| E5 | C-Media Electronics Inc. USB Audio Device, USB, card 2 (PipeWire's default sink); Realtek ALC285 Analog, onboard, card 1, used for the raw-ALSA row |
| E6 | **NOT RECORDED** — requires the app's transport bar in a GUI session. The backend drill reported its key as `default`; `./spectre --smoke-test` reports `engine=not-started` and never opens a device |
| E7 | commit `a157846` **plus uncommitted fixes** (`eframe` x11/wayland features, `CpalBackend::find_device`, two stale comments); `rustc 1.98.0 (88d9e12ae 2026-08-18)`; features: workspace default, `live-audio` → `spectre-audio/cpal-backend` |
| E8 | requested `StreamConfig::stereo(48_000, 64)`; **granted: not recorded directly** — the drill reports no granted geometry, but `frame_capacity_rejections=0` across all three rows means no block ever exceeded the requested capacity |
| E9 | **CLEAN**, after the remediation described below. `cargo fmt --all -- --check` clean; `cargo clippy --locked --workspace --all-targets -- -D warnings` clean; `cargo test --locked --workspace --no-fail-fast` → **443 passed, 0 failed, 2 ignored**; `cargo run --locked -p spectre-offline -- --self-test` exits 0. The gate first ran **441 passed, 2 failed**: both failures were `LIVE_PATH_HASH`, a macOS-derived literal in `bounce_equivalence.rs` and `bounce_cli.rs`. Live/offline equivalence passed on this host throughout; only the literal failed |
| E10 | Three real errors, all resolved before the passing rows: `UnknownDevice(DeviceId("default"))` from `find_device`; `OpenFailed("... 'snd_pcm_open' failed with error 'No such file or directory (2)'")` from the absent host default; `OpenFailed("The requested device is no longer available...")` on raw `hw:2,0`, because PipeWire holds that interface |

### Q-A — Flow A

`cargo test --locked -p spectre-offline --test e2e_alpha -- --nocapture`

```
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s
e2e frames=20480 channels=2 peak=0.26873624 hash=0x9d201ccb77e43b65 blocks=80
```

That hash is this host's observation, not a golden constant — QUAL-006 is why `e2e_alpha` has
none, and it is why Flow A passes here while the two `LIVE_PATH_HASH` assertions do not.

### Manual checks

| # | Row | Verdict |
|---|---|---|
| 1 | Audibility | `NOT RUN` — requires an operator to hear it |
| 2 | Silence at rest and after Stop | `NOT RUN` — requires an operator to hear it |
| 3 | A slider drag reaches live audio | `NOT RUN` — requires a pointer in a GUI session |
| 4 | Transport honesty | `NOT RUN` |
| 5 | Engine-unavailable state | `NOT RUN` |
| 6 | Device loss mid-playback | `NOT RUN` — the host has a detachable USB interface, so this is runnable, just not by this session |
| 7 | Save / quit / reopen | `NOT RUN` |
| 8 | Save over an existing file | `NOT RUN` |
| 9 | Bounce the fixture and listen to it | `NOT RUN` — requires an operator to hear it |
| 10 | Screen size extremes | `NOT RUN` |
| 11 | Motion | `NOT RUN` |
| 12 | Keyboard-only pass | `NOT RUN` |
| 13 | Screen-reader pass | `NOT RUN` — Orca is the Linux equivalent; `eframe`'s accessibility feature is off |
| 14 | Theme variants | `N/A` — structural; the shell hard-codes one dark palette |
| 15 | Traceability and status | `FAIL` — see Q-D |

### Q-D — what this run does not authorize, and what it contradicts

This block authorizes **no** sentence of the §"What a PASS authorizes" shape. There was no
operator, so there is no `PASS` on any platform.

Claims in the documentation this run contradicts, all corrected in the same slice:

1. `current-milestone.md` said Linux build qualification established that "the workspace
   compiles" on 2026-08-09. It did not and had not since the rebuild foundation: `spectre-app`
   named neither `x11` nor `wayland` with `eframe`'s default features off, so winit matched no
   platform. CI caught this on `ubuntu-latest` on 2026-08-06 and 2026-08-16 and the failures went
   unread, because `ci.yml` triggers only on `main` and every R4 slice landed on
   `gauntlet/r4-spec-phase`.
2. Decision 23 rated discharging the Linux debt "High — the drill exists and runs unchanged on
   Linux". It did not run unchanged; it exposed a real `find_device` defect on the app's own open
   path that had to be fixed first.
3. `r4-qa-protocol.md`'s standing constraints still described the one-instrument bridge limit and
   the audition-note path, both lifted on 2026-08-28 before the protocol was ever run. The
   protocol required its own update on that event; it has now been made.

### Outcome

`INCONCLUSIVE`, per §Outcome rules on one ground: thirteen manual rows are `NOT RUN`. The
workspace gate ran clean immediately before (E9), so the other ground does not apply. Nothing
adverse was learned about the product from the automated half — Flow A passed in full, and the
three Linux device rows in the milestone's qualification table passed with no xruns. R4's
end-to-end exit row stays open and needs an operator pass on this host.

A fourth documentation contradiction was found and closed while clearing E9: R4-8 checked a
platform-dependent golden hash into two tests, which its own slice notes disclaim
("cross-machine bit-reproducibility is not claimed") and which QUAL-006 had already refused for
`e2e_alpha`. Both assertions now derive the expected hash in-process. The round trip survives:
`bounce_cli` proves the CLI subprocess equals an in-process bounce, `bounce_equivalence` proves
that bounce equals the live callback path. The derived assertion was falsified on 2,048 frames
and on 44,100 Hz before being trusted; `block_frames` does not move a clean render's hash, which
is the render's documented property rather than a gap in the test.
