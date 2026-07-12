<!--
Author: Jeff
Date: 2026-07-11
Description: Atomic clean-room behavioral observations from the Ableton Live 12 manual
Notes: Observed public behavior only; no Geist requirement or implementation decision lives here
-->

# Ableton Live 12 — Atomic Observations

- **Status:** draft
- **Last verified:** 2026-07-11
- **Scope:** source-anchored behavioral observations from official manual chapters 6, 7, 8, 9, 16, 17, 18, 19, 25, 41
- **Decision authority:** Jeff
- **Upstream sources:** `SRC-ABLETON-LIVE12-MANUAL-CHAPTERS`; `docs/02-reference-research/ableton-live.md`
- **Downstream dependents:** requirements ledger, command ontology, sequencing/automation/routing specs
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** none
- **Known gaps:** extraction used assisted summarization of rendered chapter pages; individual claims carry transcription risk until spot-verified against the page; chapters 2, 10–15, 20–24, 26, 33, 36–40 remain unextracted

Every observation is `observed public behavior`. IDs are stable. Chapter/section anchors refer to `https://www.ableton.com/en/live-manual/12/<chapter-slug>/`. Nothing here is adopted; adoption happens only in the requirements ledger with a `PROD-*`/`SEQ-*`/etc. ID citing these observations.

## Chapter 6 — Arrangement View

- `OBS-AB12-ARR-001` (6.5): Valid time signatures use a one- or two-digit numerator and a denominator of 1, 2, 4, 8, or 16.
- `OBS-AB12-ARR-002` (6.5): Time-signature markers are not launch-quantized; they are constrained only by the editing grid, and "impossible" placements create a fragmentary (incomplete) bar shown as a crosshatched region. Two repair commands exist: delete the fragmentary time (moves clips closer, affects all tracks) or complete the bar (inserts time, affects all tracks).
- `OBS-AB12-ARR-003` (6.6): If nothing is selected, enabling the arrangement loop covers the entire arrangement. Loop Selection (Ctrl/Cmd+L) sets the loop from the time selection and toggles it. Arrow keys nudge the brace by grid; Ctrl/Cmd+arrows resize by grid or double/halve length.
- `OBS-AB12-ARR-004` (6.7): Clips snap to the editing grid, other clip edges, locators, and time-signature changes. Content can be slid under a stationary clip (Ctrl+Shift / Shift+Option drag). Grid snapping is bypassed with a held modifier.
- `OBS-AB12-ARR-005` (6.8): Audio clip fades are clip properties independent of automation. With "Create Fades on Clip Edges" enabled, adjacent audio clips receive automatic 4 ms crossfades, and deleting a fade returns it to a 4 ms default. Fades cannot cross clip loop boundaries; start/end fades cannot overlap.
- `OBS-AB12-ARR-006` (6.10): The editing grid is either zoom-adaptive or fixed. Grid commands: halve width (Ctrl/Cmd+1), double width (Ctrl/Cmd+2), triplets toggle (Ctrl/Cmd+3), snap toggle (Ctrl/Cmd+4), fixed/adaptive toggle (Ctrl/Cmd+5).
- `OBS-AB12-ARR-007` (6.11): The "…Time" commands (Cut/Paste/Duplicate/Delete Time, Insert Silence) operate on all tracks and move time-signature markers in the region; they change total arrangement length, unlike clip-scope cut/copy/paste.
- `OBS-AB12-ARR-008` (6.12–6.13): Split (Ctrl/Cmd+E) divides at a point or isolates a time selection. Consolidate (Ctrl/Cmd+J) renders selected material to one new sample per track, incorporating clip-level gain/warp/pitch/envelopes but not track effects; samples land in `Samples/Processed/Consolidate` in the project folder (temporary folder if unsaved).
- `OBS-AB12-ARR-009` (6.14): Linked-track editing applies moves, selections, time commands, split/consolidate, fade edits (co-timed), arm state, and take-lane operations across all linked tracks; each track may belong to only one linked instance; multiple instances may exist per set.
- `OBS-AB12-ARR-010` (6.3): Playback chases MIDI notes by default, so notes sound even when playback starts mid-note (Options menu toggle).
- `OBS-AB12-ARR-011` (6.4): Locators are launchable (quantized to global launch quantization), navigable (previous/next), renameable, mappable to MIDI/keys, and can set the song start time; double-click launches from a locator while stopped.
- `OBS-AB12-ARR-012` (6.2): Zoom follows the selection (`Z`), and `X` reverts zoom step-by-step; the display "Follow" mode pauses on edit or manual scroll and resumes on restart or scrub click.

## Chapter 7 — Session View

- `OBS-AB12-SES-001` (7.1): Each track column plays at most one Session clip at a time; a clip can be launched with the pointer or pre-selected and launched with Enter.
- `OBS-AB12-SES-002` (7.2.1): Scenes may carry tempo and time-signature values; valid scene tempo is 20–999 BPM; numerator 1–99; denominator 1, 2, 4, 8, or 16. Scenes with values show colored launch buttons; values can be deleted or reset to default.
- `OBS-AB12-SES-003` (7.2): After launching a scene, the next scene is auto-selected by default ("Select Next Scene on Launch" on).
- `OBS-AB12-SES-004` (7.4.2): Clip stop buttons can be added/removed per slot (Ctrl/Cmd+E), controlling whether launching that slot stops the track's running clip.
- `OBS-AB12-SES-005` (7.5): Arrangement recording from Session logs launched clips, clip property changes, mixer/device automation, and tempo/time-signature changes into the arrangement at correct song time. Per track, Session and Arrangement playback are mutually exclusive; Back to Arrangement returns authority to the arrangement.
- `OBS-AB12-SES-006` (7.4): Dropping multiple clips with Ctrl/Cmd held arranges them across tracks instead of down one track.

## Chapter 8 — Clip View

- `OBS-AB12-CLIP-001` (8.1.1): Deactivated clips (key `0`) do not play when launched or during arrangement playback; renaming a clip does not rename the referenced sample file; new clips inherit their track color.
- `OBS-AB12-CLIP-002` (8.1.1): Default clip settings for a sample can be saved into the sample's analysis (`.asd`) file and apply to new clips using the same sample.
- `OBS-AB12-CLIP-003` (8.2.1): Unwarped audio clips cannot loop; Warp must be enabled first. Warped clip positions display in bars-beats-sixteenths; unwarped in minutes-seconds-milliseconds. Set Start/End/Loop buttons capture positions live, quantized to global quantization.
- `OBS-AB12-CLIP-004` (8.2.2): Clip time signature is display-only, independent of the set's signature, and does not affect playback.
- `OBS-AB12-CLIP-005` (8.2.3): Committing a groove writes its timing to the clip and disables the assigned groove; committing positive velocity grooves on audio creates a volume clip envelope, overwriting any existing one.
- `OBS-AB12-CLIP-006` (8.4.2): Reversing audio creates a new processed sample (`Samples/Processed/Reverse`); warp markers stay fixed to sample positions (reordering with reversal) while clip envelopes stay fixed to time positions.
- `OBS-AB12-CLIP-007` (8.4.4): Session clip fade is a signal-dependent 0–4 ms edge fade to avoid clicks; arrangement clips use envelope-based fades instead.
- `OBS-AB12-CLIP-008` (8.4.6): With high-quality interpolation, samples can be transposed roughly ±19 semitones before aliasing is likely audible.
- `OBS-AB12-CLIP-009` (8.4.7): Clip pitch is expressed in semitones plus cents (100 cents = 1 semitone); clip gain is a dB-calibrated slider; multi-clip selections show a value range with split handles preserving relative differences.
- `OBS-AB12-CLIP-010` (8.3.2): MIDI clips can emit bank/sub-bank/program changes; the space is 128 × 128 × 128, disabled with a "—" setting.
- `OBS-AB12-CLIP-011` (8.5.2): MIDI time tools include stretch, ×2 / ÷2 duration scaling, fixed-length set, humanize (random start variation up to half a grid division), reverse, and legato (extend each note to the next note's start).
- `OBS-AB12-CLIP-012` (8.6): Applied MIDI transformations replace the original notes; with Scale Mode on, generated pitches are constrained to the selected scale.

## Chapter 9 — Audio Clips, Tempo, and Warping

- `OBS-AB12-WARP-001` (9.1.1): Set tempo has coarse BPM and hundredths-of-BPM fields, individually mappable; external sync sources include MIDI Clock, Link, and Tempo Follower.
- `OBS-AB12-WARP-002` (9.1.4): A warped arrangement clip may be a tempo Leader: the set follows the clip's warp-derived tempo, the clip plays unwarped, the tempo field deactivates, and only the bottom-most leader clip governs when several exist. Non-editable tempo automation is created on the main track until "Unfollow Tempo Automation" converts it to editable automation.
- `OBS-AB12-WARP-003` (9.2.1): Import warping defaults: short samples "Auto" (choose unwarped one-shot / warped one-shot / warped loop), long samples auto-warp enabled with a marker at each bar's first beat, default warp mode Beats.
- `OBS-AB12-WARP-004` (9.2.3): Warp markers pin sample positions to timeline positions; transients are auto-analyzed and shown as gray markers; pseudo-warp markers appear on transient hover and become real markers when dragged; Shift+drag moves the waveform under a marker.
- `OBS-AB12-WARP-005` (9.2.3.2): Warp markers save with the set, and optionally into the sample's own analysis data so they persist across imports (user samples only, not factory content); samples with saved markers bypass auto-warp.
- `OBS-AB12-WARP-006` (9.2.4.1): Even-length loop import assumes 1, 2, 4, 8, or 16 bars and derives tempo; ×2 / ÷2 correct octave-of-tempo errors.
- `OBS-AB12-WARP-007` (9.2.4.4): Adjusting warp markers on one of several equal-length selected clips applies to all — the documented multitrack timing-correction path.
- `OBS-AB12-WARP-008` (9.2.7): Audio quantization moves the nearest transient's warp marker to the closest grid line, with an Amount percentage for partial quantization (Ctrl/Cmd+U).
- `OBS-AB12-WARP-009` (9.3): Warp modes are granular strategies: Beats (transient-preserving; loop off/forward/back-and-forth per segment; transient envelope 0–100 controlling inter-segment fades), Tones (grain size tracks pitch clarity), Texture (grain size + fluctuation randomness), Re-Pitch (rate change couples pitch and tempo; transpose controls deactivate), Complex and Complex Pro (whole-song material; Pro adds formant preservation 0–100% and an envelope control defaulting to 128). Complex modes are documented as more CPU-intensive.
- `OBS-AB12-WARP-010` (9.2.5.1): Section-wise auto-warp repair commands exist: Warp From Here, Warp From Here (Start At …), Warp From Here (Straight), Warp … BPM From Here, and Warp Selection As a suggested loop length.

## Chapter 16 — Launching Clips

- `OBS-AB12-LAUNCH-001` (16.2): Four launch modes govern button response: Trigger (down starts, up ignored), Gate (down starts, up stops), Toggle (down starts, next down stops), Repeat (retriggers at quantization rate while held).
- `OBS-AB12-LAUNCH-002` (16.3): Legato mode hands the play position from the previously playing clip in the track to the newly launched clip, enabling seamless switching.
- `OBS-AB12-LAUNCH-003` (16.4): Clip quantization can be None, Global, or a fixed value; global quantization has dedicated shortcuts (Ctrl/Cmd+6…0); non-None values also quantize follow-action launches.
- `OBS-AB12-LAUNCH-004` (16.5): Launch velocity sensitivity is 0–100%: 0% ignores note velocity; 100% maps softest notes to silence.
- `OBS-AB12-LAUNCH-005` (16.7): Each clip has two follow actions A/B with independent chance weights; ten action types (No Action, Stop, Play Again, Previous, Next, First, Last, Any, Other, Jump). Follow Action Time defaults to one bar; linked mode triggers at clip end/loop multiples, unlinked after the set duration. A global toggle disables all follow actions.
- `OBS-AB12-LAUNCH-006` (16.7): Scene follow actions take precedence over clip follow actions once triggered.
- `OBS-AB12-LAUNCH-007` (16.7): Follow-action groups are formed by successive clip slots and delimited by empty slots; a track may contain any number of groups.

## Chapter 17 — Routing and I/O

- `OBS-AB12-ROUTE-001` (17.1): Monitor modes are In (always monitor, suppresses clip output), Auto (monitor only while armed and not playing clips; default for MIDI), and Off (default for audio; for through-the-air or external monitoring).
- `OBS-AB12-ROUTE-002` (17.1/18.8): "Keep Monitoring Latency in Recorded Audio" is enabled by default for In/Auto and aligns the recording with what was heard through software monitoring; the manual recommends disabling it for acoustic sources monitored externally.
- `OBS-AB12-ROUTE-003` (17.2.1): Track device chains are always stereo even with mono input; mono inputs record mono files; stereo-to-mono output sums the channels and attenuates by 6 dB to avoid clipping.
- `OBS-AB12-ROUTE-004` (17.3.1): MIDI ports have three independent capability toggles — Track (notes/CC), Sync (clock/timecode; SPP+Continue in song mode, Start in pattern mode; adjustable millisecond sync delay; resync policies Stop-and-Start / Start Only / Don't Resync), and Remote (parameter mapping with feedback). Remote-mapped messages are consumed and not passed to tracks.
- `OBS-AB12-ROUTE-005` (17.3.2): The computer-keyboard MIDI device maps the home row to white keys from C3, the row above to black keys, Z/X octave, C/V velocity in steps of 20.
- `OBS-AB12-ROUTE-006` (17.4): Resampling records the main output into an audio track while suppressing that track's own output from the capture; files land in `Samples/Recorded`.
- `OBS-AB12-ROUTE-007` (17.5.1): Internal taps from another track offer Pre FX, Post FX, and Post Mixer points; Pre/Post FX allow hearing the tap when soloing the receiving track, Post Mixer does not. Racks expose per-chain tap points.
- `OBS-AB12-ROUTE-008` (17.5.2): Documented internal-routing workflows include post-effects recording, rendering MIDI to audio, submixing, several MIDI tracks driving one instrument, per-output taps of multi-out instruments (tapping removes the signal from the instrument's internal mix), multi-timbral plugin feeds, sidechain feeds via output routing, and instrument layering by tapping post-MIDI-effects.

## Chapter 18 — Mixing

- `OBS-AB12-MIX-001` (18.1): Track meters show peak and RMS; while monitoring they show input level. Multi-selected tracks adjust together, preserving relative differences.
- `OBS-AB12-MIX-002` (18.1.1): The 32-bit floating-point engine tolerates over-0 dB levels between tracks without clipping; clipping matters only at physical outputs, the main output, or file export.
- `OBS-AB12-MIX-003` (18.1): Solo and arm are exclusive by default (one track at a time) with modifier or preference overrides; with exclusive arm, inserting an instrument into an empty MIDI track auto-arms it.
- `OBS-AB12-MIX-004` (18.3): Group tracks cannot contain clips but have mixer controls and host audio effects; grouping re-targets members' output routing to the group unless custom-routed; deleting a group deletes its contents (ungroup reverts).
- `OBS-AB12-MIX-005` (18.4): Return sends on return tracks are disabled by default and can be enabled per-send, permitting feedback; sends can tap pre or post the mixer stage, with Pre enabling independent monitor/aux mixes.
- `OBS-AB12-MIX-006` (18.5): The crossfader spans any number of tracks including returns, has seven curve options, A/B assignment per track, and affects only gain — not signal routing; it is automatable and MIDI-mappable with three mapping positions.
- `OBS-AB12-MIX-007` (18.6): Cue requires ≥4 hardware outputs; in Cue mode, solo buttons become headphone cue switches, browser preview also routes to the cue output, and the track activator still governs main-out audibility.
- `OBS-AB12-MIX-008` (18.7): Track delays offset outputs in milliseconds to compensate real-world latencies; unavailable when device delay compensation is off; on-stage changes are warned against (clicks/pops).
- `OBS-AB12-MIX-009` (18.9): Per-track performance-impact indicators (six-step CPU meter) identify freeze/optimize candidates.

## Chapter 19 — Recording New Clips

- `OBS-AB12-REC-001` (19.1): Audio tracks default to mono input from external inputs 1/2; MIDI tracks default to All Ins/All Channels.
- `OBS-AB12-REC-002` (19.3.1): Arrangement recording honors a "Start Playback with Record" preference (Shift+click reverses it); punch-in/out switches suppress recording outside the arrangement loop; loop recording retains audio from every pass, recoverable via undo or the sample editor.
- `OBS-AB12-REC-003` (19.3.2): Session recording requires global quantization ≠ None for clean cuts; Session Record captures into the selected scene of all armed tracks; launching a recording clip transitions it to loop playback; scene launch does not start recording in empty slots unless a preference enables it.
- `OBS-AB12-REC-004` (19.3.3): MIDI overdub layers loop passes into the same clip; Session Record toggles pause/resume of capture while playback continues; Alt/Option+double-click an empty slot creates, arms, and launches a clip in one gesture.
- `OBS-AB12-REC-005` (19.3.4): Step recording: with the editor Preview enabled, held notes are written at the insert marker and the right arrow advances by grid (extending held notes); the left arrow deletes just-recorded notes; navigators are MIDI-mappable.
- `OBS-AB12-REC-006` (19.4.1): Metronome features count-in length, tick sound and beat-division settings ("Auto" follows the signature denominator; impossible divisions revert to Auto and restore when valid), and an "only while recording" mode; with punch-in, the metronome sounds only after the punch point.
- `OBS-AB12-REC-007` (19.5): Record quantization applies at capture; in arrangement recording it is a separately undoable step; it cannot change mid-recording, except during loop overdub where changes apply immediately and are not separately undoable.
- `OBS-AB12-REC-008` (19.6): Count-in displays negative bars-beats-sixteenths (e.g., −2.1.1) counting to 1.1.1 when recording begins.
- `OBS-AB12-REC-009` (19.8): Recorded samples live in the project's `Samples/Recorded`; before the set is saved they live in the temporary folder, whose disk space is the documented exhaustion risk.
- `OBS-AB12-REC-010` (19.10): Capture MIDI continuously buffers input on armed or input-monitored MIDI tracks and retrieves it after the fact. Empty-set capture detects tempo (80–160 BPM), sets loop boundaries, and starts playback; with existing material it uses the current tempo and can overdub into the playing clip. Pre-phrase notes are preserved before the clip start marker.
- `OBS-AB12-REC-011` (19.7): Recorded file type and bit depth are user settings; recommended default warp mode should match expected material.

## Chapter 25 — Automation and Editing Envelopes

- `OBS-AB12-AUTO-001` (25.1): With Automation Arm on, any control change during arrangement recording becomes arrangement automation; automated controls carry an LED indicator.
- `OBS-AB12-AUTO-002` (25.2): Session automation records into playing clips (optionally regardless of arm state), enabling automation overdub without note capture; session automation becomes track automation when clips move to the arrangement.
- `OBS-AB12-AUTO-003` (25.2.1): Recording gesture semantics differ by input: mouse edits behave as Touch (punch out on release), MIDI controller input behaves as Latch (hold until clip loop end).
- `OBS-AB12-AUTO-004` (25.4): Manually changing an automated control while not recording overrides its automation (LED dims); the Re-Enable Automation button, a per-parameter context command, or relaunching the clip restores it.
- `OBS-AB12-AUTO-005` (25.5.1): Drawing creates grid-width steps; a held modifier draws freehand ignoring the grid; Shift while dragging gives fine resolution.
- `OBS-AB12-AUTO-006` (25.5.2): Breakpoint editing includes click-to-create/delete, exact keyboard value entry, multi-breakpoint relative moves, segment dragging with insertion at selection edges, axis-locking with Shift, and curved segments via modifier-drag (double-click to straighten).
- `OBS-AB12-AUTO-007` (25.5.3): Time-selection handles stretch envelopes vertically/horizontally and skew via corner handles, with grid snapping, fine adjustment, and mirrored dragging via modifiers.
- `OBS-AB12-AUTO-008` (25.5.4–25.5.5): Simplify Envelope reduces breakpoints to an optimal set; insertable shapes include sine/triangle/saw/inverse-saw/square scaled to the selection, plus ramps and an ADSR that link to surrounding values.
- `OBS-AB12-AUTO-009` (25.5.6): Lock Envelopes pins automation to song position instead of clips, so moving clips leaves automation behind.
- `OBS-AB12-AUTO-010` (25.5.8): Tempo automation is edited on the main track with a user-set min/max BPM display range, which also scales any MIDI controller mapped to tempo.

## Chapter 41 — Keyboard Shortcuts (bindings evidence)

The chapter defines 29 shortcut categories. Captured binding families (Win / Mac):

- Views: Tab toggles Session/Arrangement; Shift+Tab or F12 toggles Device/Clip View; Ctrl/Cmd+Alt/Option+B browser; …+M mixer; …+3 Clip View; …+4 Device View; F11 / Cmd+F full screen; Ctrl/Cmd+, settings.
- Transport: Space play/stop from start marker; Shift+Space continue from stop point; F9 record; Shift+F9 arrangement record-arm; Ctrl/Cmd+Shift+F9 session record; F10 back to arrangement; `O` metronome; F1–F8 activate/deactivate tracks 1–8.
- Editing: platform-conventional cut/copy/paste/duplicate/delete/undo/redo/rename/select-all (redo is Ctrl+Y on Windows, Cmd+Shift+Z on macOS).
- Loop/markers: Ctrl/Cmd+F9…F12 set start marker, loop start, loop end, end marker; Ctrl/Cmd+Shift+L select material in loop.
- Session: Enter launches selected slot; arrows navigate; Ctrl/Cmd+E add/remove stop button; Ctrl/Cmd+Shift+M insert MIDI clip; Ctrl/Cmd+I insert scene.
- Arrangement: Ctrl/Cmd+E split; Ctrl/Cmd+J consolidate; Ctrl/Cmd+Shift+J crop; Ctrl/Cmd+Alt/Option+F fades; Ctrl/Cmd+L loop toggle; Ctrl/Cmd+I insert silence; Ctrl/Cmd+Shift+X cut time.
- MIDI editor: Ctrl/Cmd+U quantize; Ctrl/Cmd+J join notes; `K` highlight scale; Ctrl/Cmd+G / +Shift+G group/ungroup notes.
- Browser: Ctrl/Cmd+F search; Enter load; Shift+Enter preview; Ctrl/Cmd+[ ] history.
- Mapping: Ctrl/Cmd+M MIDI map; Ctrl/Cmd+K key map; `M` computer MIDI keyboard.

Notable cross-context reuse: Ctrl/Cmd+E is split in Arrangement but add/remove stop button in Session; Ctrl/Cmd+I is insert scene in Session but insert silence in Arrangement; Ctrl/Cmd+J is consolidate in Arrangement but join notes in the MIDI editor. Context-scoped command resolution is therefore load-bearing in Live's binding model.

## Cross-cutting patterns worth carrying into requirements work

1. Context-sensitive binding reuse (same chord, different command per focused surface) is pervasive and deliberate.
2. Modifier grammar is consistent: Shift = fine/extend, Alt/Option = alternate scope or curve, Ctrl/Cmd = bypass grid or exclusive-mode override.
3. Time semantics separate clip-scope edits from all-track "…Time" edits.
4. Latency policy is user-visible at exactly two places: monitoring-latency keep/discard per track and millisecond track delays; everything else is automatic compensation.
5. Destructive operations are funneled into new files under `Samples/Processed/<Operation>` rather than mutating source samples.
6. Defaults encode safety: return sends off, exclusive arm/solo on, fades on clip edges, 4 ms anti-click floors.
7. Session→Arrangement capture is a one-way authoritative recording of performed intent, with per-track exclusivity between the two surfaces.
