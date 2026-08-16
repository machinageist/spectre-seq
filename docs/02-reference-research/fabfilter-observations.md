<!--
Author: Jeff
Date: 2026-08-15
Description: Atomic clean-room observations from FabFilter's official public help files for the mixing and mastering plug-ins
Notes: Documented public behavior only; no algorithm reconstruction, no vendor content reuse, no adopted Spectre requirement
-->

# FabFilter Mixing and Mastering Plug-Ins — Atomic Observations

- **Status:** draft
- **Research state:** in-review
- **Last verified:** 2026-08-15
- **Scope:** publicly documented control semantics, metering, dynamic behavior, latency, stereo/immersive handling, state handling, and interaction model of FabFilter Pro-Q 4, Pro-C 3, Pro-L 2, Pro-MB, Pro-DS, Saturn 2, and Pro-R 2
- **Decision authority:** Jeff
- **Upstream sources:** `SRC-FABFILTER-HELP-INDEX`; `SRC-FABFILTER-NEWS-INDEX`; `SRC-FABFILTER-PROQ4-HELP`; `SRC-FABFILTER-PROC3-HELP`; `SRC-FABFILTER-PROL2-HELP`; `SRC-FABFILTER-PROMB-HELP`; `SRC-FABFILTER-PRODS-HELP`; `SRC-FABFILTER-SATURN2-HELP`; `SRC-FABFILTER-PROR2-HELP`; `docs/02-reference-research/methodology.md`
- **Downstream dependents:** future Spectre mixing/mastering device suite requirements; metering and analyzer contracts; device UI interaction contract; requirements ledger
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** whether FabFilter enters the external reference register as a substantive behavioral reference or a bounded subsystem reference (proposal in `docs/02-reference-research/fabfilter.md`)
- **Known gaps:** the online help exposes no point version or revision date for any plug-in; several plug-ins expose no generation number at all; numerous ranges, defaults, and time constants are qualitative in the help and remain `GAP-FABFILTER-*`

## Coverage note

This file is **not** source-complete and MUST NOT be read as such.

| Plug-in | Declared source | Pages `claims-extracted` | Pages `unreviewed` |
|---|---|---|---|
| Pro-Q 4 | `SRC-FABFILTER-PROQ4-HELP` | 19 of 37 | Quick start, Overview, Piano display, Spectrum Grab, Using on iOS, Loading presets, Saving presets, 2 Purchasing pages, 8 Support pages |
| Pro-C 3 | `SRC-FABFILTER-PROC3-HELP` | 8 of ~30 | remaining Using pages, Presets, Purchasing, Support |
| Pro-L 2 | `SRC-FABFILTER-PROL2-HELP` | 6 of ~28 | remaining Using pages, Presets, Purchasing, Support |
| Pro-MB | `SRC-FABFILTER-PROMB-HELP` | 5 of ~28 | remaining Using pages, Presets, Purchasing, Support |
| Pro-DS | `SRC-FABFILTER-PRODS-HELP` | 4 of 25 | Quick start, Overview, Knobs, Oversampling, Full Screen/resize, Instance list, Input and output options, MIDI Learn, Undo/redo, Using on iOS, 3 Presets pages, 2 Purchasing pages, 6 Support pages |
| Saturn 2 | `SRC-FABFILTER-SATURN2-HELP` | 6 of 32 | Quick start, Overview, Knobs, Modulation visualization, **all five per-source pages** (XLFO, EG, EF, MIDI source, XY controller), Full Screen/resize, MIDI Learn, Undo/redo, Using on iOS, 4 Presets pages, 2 Purchasing pages, 7 Support pages |
| Pro-R 2 | `SRC-FABFILTER-PROR2-HELP` | 5 of 26 | Quick start, Overview, Knobs, **Surround and Dolby Atmos**, Spectrum analyzer, Full Screen/resize, Input and output options, MIDI Learn, Undo/redo, Using on iOS, 3 Presets pages, 2 Purchasing pages, 7 Support pages |

Page counts for Pro-C 3, Pro-L 2, and Pro-MB are approximate because only the rendered navigation was counted, not a vendor-declared page inventory; the Pro-Q 4, Pro-DS, Saturn 2, and Pro-R 2 rows are exact counts from the rendered navigation. Every count above is a coverage claim about *pages read*, never about *behavior understood*.

Two unreviewed pages are called out in bold because their absence bounds a claim someone might otherwise assume: Saturn 2's five per-source modulation pages were not read, so **no XLFO waveform set, envelope-generator stage range, or envelope-follower timing is established here**; and Pro-R 2's Surround page was not read, so **no immersive claim is recorded for Pro-R 2** even though Pro-Q 4's and Pro-L 2's immersive behavior is.

Two further limitations apply to every record below.

1. **Transcription risk.** Extraction used assisted page summarization over the official HTML help. Exact numerals are reproduced as read but have not been spot-verified against the official PDF manuals. Any numeral promoted toward a Spectre decision must be re-read directly first.
2. **No point-version anchor.** No FabFilter help page rendered a point version, build number, or revision date. All claims are scoped to "online help as rendered 2026-08-15" (`GAP-FABFILTER-0001`).

## Layer legend

`OBSERVED` — explicitly supported by the cited page. `SOURCE-GAP` — relevant and absent or ambiguous in the inspected page. `SPECTRE-CANDIDATE` — inference or product implication, carries no authority. No record here is `SPECTRE-REQ`.

---

## Pro-Q 4 — equalizer

All records cite `SRC-FABFILTER-PROQ4-HELP`, page path given per record.

### Band model and filter shapes

- `OBS-FF-PROQ-001` `OBSERVED` (Band controls): Ten filter shapes are offered — Bell, Low Shelf, Low Cut, High Shelf, High Cut, Notch, Band Pass, Tilt Shelf, Flat Tilt, All Pass. The gain parameter applies only to Bell, Shelving, and Flat Tilt shapes; Notch and Band Pass are documented as having no gain setting.
- `OBS-FF-PROQ-002` `OBSERVED` (Band controls): Slope is a continuous parameter from 0 dB/oct to 96 dB/oct with fractional values permitted, plus a discrete `Brickwall` setting available only on Low Cut and High Cut. Minimum slope is shape-dependent: 0 dB/oct for Low Cut, High Cut, and Band Pass; 12 dB/oct for Bell and Notch; 6 dB/oct for the remaining shapes.
- `OBS-FF-PROQ-003` `OBSERVED` (About FabFilter Pro-Q 4; Band controls): Up to 24 EQ bands are available, and slope is documented as universal across EQ shapes rather than a low/high-cut-only parameter.
- `OBS-FF-PROQ-004` `OBSERVED` (Band controls): Frequency spans 10 Hz to 30 kHz. Value entry accepts plain numerals, an abbreviated form such as `2k`, and note names with cent offsets such as `A4` or `C#2+13`.
- `OBS-FF-PROQ-005` `OBSERVED` (Band controls): Gain spans −30 dB to +30 dB. Q is documented relative to a reference — value 1 corresponds to the default bandwidth — and cannot be adjusted while a 6 dB/oct slope is in use. No numeric Q range is stated.
- `OBS-FF-PROQ-006` `OBSERVED` (Band controls): A Gain-Q interaction control sits between the gain and Q knobs for Bell bands only; when engaged, Q narrows automatically as gain increases. The plug-in remembers the last Gain-Q interaction setting across bands.
- `OBS-FF-PROQ-007` `OBSERVED` (Band controls): Bands are numbered sequentially at creation and are deliberately **not** renumbered after a deletion, so that existing host automation assignments remain valid.
- `OBS-FF-PROQ-008` `OBSERVED` (Display and workflow): Display range is a user choice of ±3 dB, ±6 dB, ±12 dB, or ±30 dB, and the frequency axis extends to 30 kHz even though the audible range ends near 20 kHz.

### Dynamic EQ

- `OBS-FF-PROQ-009` `OBSERVED` (Dynamic EQ): Dynamic behavior is expressed as a **dynamic range** ring around the gain knob spanning −30 dB to +30 dB, further constrained by the gain limits. A positive dynamic range produces upward/expanding behavior and a negative range produces downward/compressing behavior. Dynamic range is offered only on Bell, Shelving, and Flat Tilt shapes.
- `OBS-FF-PROQ-010` `OBSERVED` (Dynamic EQ): The ring is also the meter — a yellow bar inside the ring shows the dynamic gain change currently applied, against a red indication of the configured dynamic range. Gain, dynamic depth, and gain reduction therefore occupy a single control affordance.
- `OBS-FF-PROQ-011` `OBSERVED` (Dynamic EQ): Threshold defaults to an automatic mode shown as `A` in the threshold slider button; the automatic threshold continuously tracks the band-limited trigger signal. Expanding the dynamics panel exposes a manual threshold slider, and collapsing the panel reverts all dynamics behavior to automatic.
- `OBS-FF-PROQ-012` `OBSERVED` (Dynamic EQ): A soft knee is applied internally, so triggering begins slightly below the selected threshold value.
- `OBS-FF-PROQ-013` `OBSERVED` (Dynamic EQ): Attack and release are normalized knobs whose centre position of 50 % **is** the automatic setting; moving off centre makes the corresponding stage faster or slower. No time units, no millisecond ranges, and no automatic-mode time constants are stated.
- `OBS-FF-PROQ-014` `OBSERVED` (Dynamic EQ): The trigger source defaults to `Band` — the band's own frequency range — and can be switched to `Free`, which exposes independent low-cut and high-cut controls over the trigger signal. An audition button monitors the trigger signal.
- `OBS-FF-PROQ-015` `OBSERVED` (Dynamic EQ): When external side chain is enabled for a band, the side-chain signal receives the same band-limiting filtering that the internal trigger receives. The threshold slider itself displays the live trigger level.

### Spectral dynamics

- `OBS-FF-PROQ-016` `OBSERVED` (Spectral dynamics): A spectral band processes only those frequencies **within** the band that exceed the threshold, in contrast to a dynamic band, which changes the gain of the whole band in response to level. Spectral mode is available only on Bell and Shelving bands.
- `OBS-FF-PROQ-017` `OBSERVED` (Spectral dynamics): Spectral mode is reachable three ways — the Spectral icon above the gain/dynamic-range knobs, `Alt`+`Shift`+click in the display, and a `Make Spectral` item in the band's right-click menu.
- `OBS-FF-PROQ-018` `OBSERVED` (Spectral dynamics): A **Spectral Density** control sets selectivity: lower values trigger over wider frequency ranges, higher values over narrower, more specific areas. No numeric range or unit is stated.
- `OBS-FF-PROQ-019` `OBSERVED` (Spectral dynamics): A **Spectral Tilt** toggle, on by default for new spectral bands, applies a 3 dB/oct tilt to the input spectrum before triggering so that higher frequencies trigger more readily.
- `OBS-FF-PROQ-020` `OBSERVED` (Spectral dynamics; Processing mode): Spectral processing requires linear-phase processing. When spectral bands are active, a Processing Resolution control appears and is restricted to Low, Medium, and High; Very High and Maximum are unavailable. Latency is stated to depend on the chosen resolution.

### Processing modes, phase, and latency

- `OBS-FF-PROQ-021` `OBSERVED` (Processing mode): Three processing modes exist. **Zero Latency** matches the magnitude response of analog EQ with no added latency and is documented as the most efficient mode. **Natural Phase** matches both analog magnitude and analog phase response without noticeable pre-ring or long latency. **Linear Phase** alters magnitude only and leaves phase untouched.
- `OBS-FF-PROQ-022` `OBSERVED` (Processing mode): Linear-phase latency at 44.1 kHz is documented per resolution: Low 3072 samples (~70 ms), Medium 5120 samples (~116 ms), High 9216 samples (~209 ms), Very High 17408 samples (~395 ms), Maximum 66560 samples (~1509 ms). *Transcription risk applies; re-read before use.*
- `OBS-FF-PROQ-023` `OBSERVED` (Processing mode): CPU cost of linear-phase processing is characterized as very low even with all 24 bands active, and as not changing much across the linear-phase resolutions. No CPU figure is given for Zero Latency or Natural Phase, and no oversampling behavior is stated for any mode.
- `OBS-FF-PROQ-024` `OBSERVED` (Stereo options): The help explicitly advises linear-phase processing when filtering left, right, mid, or side channels differently, to avoid introducing unwanted phase changes.

### Stereo, mid/side, and immersive

- `OBS-FF-PROQ-025` `OBSERVED` (Stereo options): Stereo placement is **per band**, chosen from Stereo (default), Left, Right, Mid, and Side. There is no global L/R-versus-M/S mode switch for the band set.
- `OBS-FF-PROQ-026` `OBSERVED` (Stereo options): Placement is encoded in curve color — white for left, red for right, green for mid, blue for side, yellow for all-channel. Channel routing is therefore legible from the curve itself without selecting the band.
- `OBS-FF-PROQ-027` `OBSERVED` (Stereo options; Band controls): A split control duplicates a band into two identical copies bound to the two halves of a pair (left and right, or mid and side), turning one shared band into two independently editable ones.
- `OBS-FF-PROQ-028` `OBSERVED` (Surround and Dolby Atmos): Channel layouts up to 9.1.6 Dolby Atmos are supported, subject to DAW and plug-in format. Per-band speaker selection allows clicking a whole speaker row (for example Lss/Rss, or Center) or an individual speaker icon; with L/R selected the Center speaker may be added to form an L/C/R selection. Stereo placement options apply within the selected speaker set.
- `OBS-FF-PROQ-029` `OBSERVED` (Surround and Dolby Atmos): Output panning is unavailable in a surround layout, and the output level meter shows all channels with identifying labels. Loading a preset whose surround settings are unavailable in the current layout may disable the affected bands; a `Reset Placement/Speakers` menu item recovers from this.

### Analysis, metering, and matching

- `OBS-FF-PROQ-030` `OBSERVED` (Spectrum analyzer): Three independent spectrum overlays can be shown — Pre, Post, and SC/Ext. The external overlay can display either the side-chain input or the post-EQ spectrum of another Pro-Q 4 instance, with the contributing instance marked by a light red outline.
- `OBS-FF-PROQ-031` `OBSERVED` (Spectrum analyzer): Analyzer range is a choice of 60 dB, 90 dB (default), or 120 dB. Resolution is a choice of Low 1024, Medium 2048, High 4096, or Maximum 8192 points, and higher resolutions are documented to reduce the update rate and therefore slow the analyzer's effective attack.
- `OBS-FF-PROQ-032` `OBSERVED` (Spectrum analyzer): A tilt setting expressed in dB per octave shapes the displayed spectrum; 4.5 dB/oct is documented as the default and as the value that best resembles human loudness perception.
- `OBS-FF-PROQ-033` `OBSERVED` (Spectrum analyzer): Analyzer speed is described only qualitatively as a release behavior — fast release shows dynamic change more clearly, slow release leaves the spectrum readable for longer. No numeric time constants are stated.
- `OBS-FF-PROQ-034` `OBSERVED` (Spectrum analyzer): A Freeze mode stops spectrum fallback and accumulates a running maximum, indicated by a blue marker on the analyzer button.
- `OBS-FF-PROQ-035` `OBSERVED` (Spectrum analyzer): A `Show Collisions` mode highlights overlapping frequency areas in red and is explicitly documented as an indication rather than a scientifically precise measurement.
- `OBS-FF-PROQ-036` `OBSERVED` (Spectrum analyzer; Display and workflow): Spectrum Grab arms itself automatically after the pointer hovers the display for several seconds; on arming it dims the existing bands and freezes the spectrum so a peak can be grabbed directly.
- `OBS-FF-PROQ-037` `OBSERVED` (EQ Match): EQ Match analyses the plug-in input continuously and compares it against a reference chosen from four sources — a saved reference spectrum, the main plug-in input with record/pause control, an external spectrum (another Pro-Q instance or the side chain), or an audio file via `Load File…`. Analysis averages over time and is documented as normally taking no more than 30 seconds.
- `OBS-FF-PROQ-038` `OBSERVED` (EQ Match): On pressing Match, the plug-in chooses how many and what kind of bands are needed; a slider then trades band count against fidelity, with more bands capturing the smallest differences and fewer bands covering only the main shape. A white line shows the spectral difference. The help advises raising analyzer resolution to High or Maximum to improve low-frequency match resolution.

### Gain staging and output

- `OBS-FF-PROQ-039` `OBSERVED` (Output options): Output gain spans minus infinity to +36 dB. Output pan is available only on stereo tracks and has its own mode control selecting normal left/right panning or mid/side panning.
- `OBS-FF-PROQ-040` `OBSERVED` (Output options): Auto Gain compensates for level change caused by the EQ curve, and is documented explicitly as an *educated guess based on the current EQ settings*, **not** a dynamic process driven by measured levels.
- `OBS-FF-PROQ-041` `OBSERVED` (Output options): A Gain Scale slider scales the gain of all curves at once and affects only the shapes that have a gain setting — Bell, Shelving, and Flat Tilt.
- `OBS-FF-PROQ-042` `OBSERVED` (Output options): The plug-in is documented as having unlimited internal headroom and as never clipping itself. Phase invert is a discrete output toggle with a blue active state.

### Interaction model

- `OBS-FF-PROQ-043` `OBSERVED` (Knobs): Knobs support three input methods: vertical drag with speed-sensitive resolution, mouse wheel while hovering, and double-click text entry. Text entry accepts unit shorthand including `1k` for 1000 Hz, `A4` for 440 Hz, and `2x` for +6 dB.
- `OBS-FF-PROQ-044` `OBSERVED` (Knobs): Modifier conventions are uniform across the plug-in and are explicitly remapped for Pro Tools. Reset to default is `Ctrl`/`Cmd`+click, or `Alt`+click in Pro Tools. Fine-tune is `Shift`+drag or `Shift`+wheel, or `Ctrl`/`Cmd`+drag in Pro Tools. Linked adjustment of paired knobs is `Alt`+drag, or `Shift` in Pro Tools.
- `OBS-FF-PROQ-045` `OBSERVED` (Display and workflow): Band creation is gestural and layered: drag the yellow curve, click or double-click the background after a hover preview, `Ctrl`/`Cmd`+click to add a band while others are selected, `Alt`+create for a dynamic band, and `Alt`+`Shift`+create for a spectral band. The band *type* is therefore selected by the creation gesture, not by a subsequent menu.
- `OBS-FF-PROQ-046` `OBSERVED` (Display and workflow): The mouse wheel is overloaded by modifier on the display: bare wheel adjusts Q (or slope on LP/HP bands), `Shift`+wheel fine-tunes Q, `Ctrl`/`Cmd`+wheel adjusts gain, `Alt`+wheel adjusts dynamic range, and `Alt`+`Ctrl`/`Cmd`+wheel trades gain against dynamic range in a linked move.
- `OBS-FF-PROQ-047` `OBSERVED` (Display and workflow): Drag is modified as well: `Shift`+drag fine-tunes, and `Alt`+drag constrains the drag to a single axis (frequency, or gain/Q).
- `OBS-FF-PROQ-048` `OBSERVED` (Display and workflow; Band controls): Direct manipulation of a band dot covers bypass (`Alt`+click), shape change (`Ctrl`/`Cmd`+`Alt`+click), slope change on LP/HP (`Alt`+`Shift`+click), numeric entry (double-click), and a full pop-up menu (right-click). Copy is on the band's right-click menu and paste is on the display's right-click menu.
- `OBS-FF-PROQ-049` `OBSERVED` (Display and workflow): Multi-band selection supports rubber-band rectangle selection on the background, `Ctrl`/`Cmd`+click to extend, and `Shift`+click to select a consecutive range; selected bands are then adjusted in parallel. Clicking the background deselects.
- `OBS-FF-PROQ-050` `OBSERVED` (Display and workflow): The frequency axis is zoomable by dragging the frequency scale vertically, pannable by dragging it horizontally while zoomed, and resets to full range on double-click.
- `OBS-FF-PROQ-051` `OBSERVED` (Solo): Solo is a press-and-hold gesture on a headphones button and is a live modal state, not a latched toggle. While held, other bands and the overall curve dim.
- `OBS-FF-PROQ-052` `OBSERVED` (Solo): Solo content is shape-aware: for Bell and Shelving bands the soloed signal is the affected part of the spectrum as defined by frequency and Q; for Low Cut and High Cut bands the soloed signal is the frequencies being *removed*, not the ones passing. During solo, horizontal drag moves frequency, vertical drag sets the solo listening level, and `Ctrl`/`Cmd`+drag changes Q for Bell and Shelving.
- `OBS-FF-PROQ-053` `OBSERVED` (EQ Sketch): EQ Sketch converts a single left-to-right freehand drag into a set of bands, choosing LP/HP filters, bells, and shelves as it goes; the steepness of the drawing motion sets slope or Q. A new band is created when the stroke moves far enough away from the 0 dB line after approaching it, and moving backward within the same stroke removes bands just created. It is documented as a starting-point tool rather than a precise one.
- `OBS-FF-PROQ-054` `OBSERVED` (Full Screen mode, resizing and scaling): Five predefined interface sizes exist — Mini (matching the iOS AUv3 default), Small, Medium (default), Large, Extra Large — and the chosen size becomes the default for new instances. Sizes unavailable on the current display are greyed out. Under VST3 the window may additionally be resized freely by dragging its edges and then returned to a predefined size.
- `OBS-FF-PROQ-055` `OBSERVED` (Full Screen mode, resizing and scaling): A separate scaling submenu adjusts size relative to the system default (examples given are 150 % and 300 % on Retina displays). Full Screen mode automatically selects a larger scaling. Scaling choices are remembered separately for normal and Full Screen mode and separately per monitor type (Retina/High-DPI versus regular).

### Multi-instance control

- `OBS-FF-PROQ-056` `OBSERVED` (Instance list): The instance list spans plug-ins, not just Pro-Q — it controls Pro-Q 4, Pro-C 3, Pro-DS, and Pro-G instances in the session, with more stated to follow. Instances are grouped per track in DAW track order, with DAW-derived track names (renameable by double-click) and track color dots where the host exposes them.
- `OBS-FF-PROQ-057` `OBSERVED` (Instance list): Actions available without leaving the list include bypassing an instance, adjusting its output level and panning, zooming an instance to maximum for precise editing, editing Pro-Q 4 curves directly (create, adjust, right-click), loading presets by menu or drag-and-drop, copying and pasting settings between instances, starting EQ Match, and designating a collision-reference track.
- `OBS-FF-PROQ-058` `OBSERVED` (Instance list): Cross-instance analysis is first-class: a designated collision-reference track's spectrum is displayed in the main interface of other instances and drives collision highlighting. Tracks can be pinned and the list filtered to pinned tracks.
- `OBS-FF-PROQ-059` `OBSERVED` (Instance list): Host-specific degradations are documented rather than hidden — Pro Tools does not supply track colors, Audio Units instances can only be ordered alphabetically, and FL Studio may order latency-introducing instances incorrectly.

### State, automation, and persistence

- `OBS-FF-PROQ-060` `OBSERVED` (How presets are stored): Presets are individual files with the `.ffp` extension, stored by default at `Documents/FabFilter/Presets/Pro-Q 4` on both Windows and macOS, with a legacy macOS location at `~/Library/Audio/Presets/FabFilter/FabFilter Pro-Q 4`. Subfolders become preset-menu categories and may be nested.
- `OBS-FF-PROQ-061` `OBSERVED` (How presets are stored): The preset folder is user-relocatable via `Change Preset Folder` in the preset browser's Options menu, and the browser reloads itself when the dialog closes. The `.ffp` format is identical on Windows and macOS, so preset files are cross-platform. `Restore Factory Presets` recovers deleted factory content.
- `OBS-FF-PROQ-062` `OBSERVED` (Undo, redo, A/B switch): Every change made *through the plug-in interface*, including loading a preset, pushes a new undo state. Changes arriving by MIDI or host automation deliberately record **no** undo state. Undo and redo buttons disable themselves when their stack is empty.
- `OBS-FF-PROQ-063` `OBSERVED` (Undo, redo, A/B switch): The A/B switch saves the current state before switching, so pressing it twice returns to the original state. A Copy button copies the active state onto the inactive one and then disables itself to signal that the two states are equal.
- `OBS-FF-PROQ-064` `OBSERVED` (MIDI learn): MIDI Learn is a modal binding flow — enter the mode, the interface dims and controllable parameters highlight, touch a parameter (marked with a red square), then move a hardware control. Bindings are per plug-in instance, are saved automatically when the plug-in closes, and can be saved manually to allow reverting. A Clear submenu lists all associations for individual or wholesale removal.

---

## Pro-C 3 — compressor

All records cite `SRC-FABFILTER-PROC3-HELP`, page path given per record.

### Dynamics controls

- `OBS-FF-PROC-001` `OBSERVED` (Dynamics controls): Threshold is defined as the side-chain level above which gain reduction begins. No numeric range, unit, or default is stated on the control-reference page.
- `OBS-FF-PROC-002` `OBSERVED` (Dynamics controls): Ratio spans 1:1 to infinity and is explained in input/output terms — at 10:1, one dB of output above threshold remains for every 10 dB of input above threshold.
- `OBS-FF-PROC-003` `OBSERVED` (Dynamics controls): Knee spans 0 dB to 72 dB and is described as the roundness of compression around the threshold. A 72 dB knee is an unusually wide stated maximum and is recorded as-read.
- `OBS-FF-PROC-004` `OBSERVED` (Dynamics controls): A **Range** control caps the maximum applied gain change, decoupling "how hard" from "how much" — the compressor can be aggressive at the knee yet bounded in total reduction. No numeric range is stated.
- `OBS-FF-PROC-005` `OBSERVED` (Dynamics controls; About FabFilter Pro-C 3): **Auto Threshold** makes the threshold work independently of input level. It is presented as a headline Pro-C 3 addition. No adaptation time constant, target, or measurement basis is stated.
- `OBS-FF-PROC-006` `OBSERVED` (Time controls): **Auto Gain** applies automatic make-up gain derived from the Threshold, Ratio, Knee, and Attack settings, and is documented as aware of mid/side processing. It is therefore a settings-derived estimate rather than a measured-level correction — the same design stance as Pro-Q 4's Auto Gain (`OBS-FF-PROQ-040`).

### Time controls

- `OBS-FF-PROC-007` `OBSERVED` (Time controls): Attack spans 0.005 ms to 250 ms. The stated 5 µs minimum is well below one sample period at common host rates, which the help does not reconcile.
- `OBS-FF-PROC-008` `OBSERVED` (Time controls): Release is documented as style-dependent and strongly program-dependent — quick recovery after transients, slower after sustained gain reduction. No numeric range is stated on the inspected page.
- `OBS-FF-PROC-009` `OBSERVED` (Time controls): **Auto Release** adjusts release time according to the current amount of gain reduction, and the Release knob then scales the overall effect rather than setting an absolute time. Auto mode is a modifier on the manual control, not a replacement for it.
- `OBS-FF-PROC-010` `OBSERVED` (Time controls; About FabFilter Pro-C 3): Lookahead is `Off` plus a set of settings up to a maximum of 20 ms; the About page states the maximum lookahead is settable between 1 and 20 ms. Lookahead is documented to cause additional latency and can be controlled globally so that the combined latency of a processing chain can be minimized.
- `OBS-FF-PROC-011` `OBSERVED` (Time controls): **Hold** prolongs peaks in gain reduction; short hold values are documented as increasing transparency and long values as producing pumping. No numeric range is stated.

### Styles and character

- `OBS-FF-PROC-012` `OBSERVED` (Style and character): Fourteen compression styles are offered in three named groups. Modern: Clean, Versatile, Smooth, Punch, Upward, TTM. Classic: Op-El, Vari-Mu, Classic, Opto. Utility: Vocal, Mastering, Bus, Pumping.
- `OBS-FF-PROC-013` `OBSERVED` (Style and character): Style descriptions attach topology vocabulary to presets of behavior — Clean is described as feedforward and program dependent, Classic as feedback and very program dependent, Vari-Mu as a variable-mu feedback topology, Opto as slow with a very soft knee. These are documentation characterizations of the resulting behavior; no algorithm is described and none is reconstructed here.
- `OBS-FF-PROC-014` `OBSERVED` (Style and character): Upward is documented as increasing level when it drops below the threshold, and TTM as combining upward and downward compression across multiple bands. A single style selector therefore changes the *direction* and *band count* of the dynamics, not only its flavor.
- `OBS-FF-PROC-015` `OBSERVED` (Style and character): Vocal is documented as using automatic knee and ratio settings, meaning a style choice can also take over otherwise user-facing parameters.
- `OBS-FF-PROC-016` `OBSERVED` (Style and character): Character modes are Off, Tube, Diode, and Bright, with a **Drive** control and a routing option placing the character stage Pre or Post compression, defaulting to Post. No Drive range or unit is stated.

### Side chain

- `OBS-FF-PROC-017` `OBSERVED` (Side chain section): Four trigger sources are offered — Internal (the plug-in's own main input), External (side-chain input), Host Sync (a generated pulse locked to host tempo), and MIDI (incoming note-on messages). A compressor trigger can therefore be rhythmic or note-driven with no audio side chain present.
- `OBS-FF-PROC-018` `OBSERVED` (Side chain section): The side-chain EQ offers up to six bands using the same filter shapes as Pro-Q, including All Pass, with high/low-pass slopes up to 96 dB/oct plus Brickwall.
- `OBS-FF-PROC-019` `OBSERVED` (Side chain section): An Audition button monitors the filtered *and stereo-linked* trigger signal, and supports both a latching click and a momentary click-and-hold. An Audition Level slider sets its monitoring level.
- `OBS-FF-PROC-020` `OBSERVED` (Side chain section): Stereo Link is a continuous 0 % (channels fully independent) to 100 % (fully linked) control, and extends *past* 100 % into modes that process mid or side exclusively: Mid, Side, M>S (trigger on mid, compress side) and S>M (trigger on side, compress mid). Linking and channel-routing therefore share one continuous control.
- `OBS-FF-PROC-021` `OBSERVED` (Side chain section): Host Sync exposes a Sync button setting pulse speed relative to song tempo, an Offset slider adjusting the sync speed by a factor between 50 % and 200 %, and a Length slider setting pulse duration as a percentage of the current sync setting.

### Metering and displays

- `OBS-FF-PROC-022` `OBSERVED` (Displays and metering): The level display superimposes input (dark grey), output (light grey with stroke), and gain reduction (red line) on one time-scrolling plot, so gain reduction is read against the signal that caused it.
- `OBS-FF-PROC-023` `OBSERVED` (Displays and metering): The knee display draws the static transfer curve in white and turns the curve green at the point corresponding to the current input level while audio is playing, making the operating point on the curve continuously visible.
- `OBS-FF-PROC-024` `OBSERVED` (Displays and metering): A Compact layout hides both the level and knee displays and leaves only horizontal level meters, explicitly to resemble a traditional compressor.
- `OBS-FF-PROC-025` `OBSERVED` (Displays and metering): Loudness metering on the input and output meters is documented as complying with the Momentary mode of the EBU R128 / ITU-R 1770 standards, shown per channel, with a peak indication above the loudness readout representing combined Momentary loudness of all channels together.
- `OBS-FF-PROC-026` `OBSERVED` (Displays and metering): One meter-scale setting spans 9 dB (stated for precise mastering) to 90 dB (stated for general mixing and bus processing), and the same scale applies simultaneously to the knee display, the level display, and the meters. The setting persists for new instances.
- `OBS-FF-PROC-027` `OBSERVED` (Displays and metering): The input level meter is optional via the Help menu and is omitted entirely in surround/immersive mode. Clipping indication is documented as not implying that distortion occurred.

### Oversampling, gain staging, and mix

- `OBS-FF-PROC-028` `OBSERVED` (Oversampling): Oversampling runs the internal process at a multiple of the host sample rate to reduce aliasing, is documented to add a small latency on top of lookahead latency, and to increase CPU use significantly at higher factors. The Oversampling page enumerates **no** factor list; it only advises starting at 2x or 4x.
- `OBS-FF-PROC-029` `OBSERVED` (About FabFilter Pro-C 3): The help set's own About page states up to 32x oversampling and full immersive / Dolby Atmos functionality up to 9.1.6. This is an in-help feature statement that the Oversampling control-reference page does not corroborate with a factor list.
- `OBS-FF-PROC-030` `OBSERVED` (Input and output options): Input and output are single knobs that combine level with L/R panning — input level/pan is applied before all processing, output level/pan after. No numeric ranges or units are stated for either.
- `OBS-FF-PROC-031` `OBSERVED` (Input and output options): The Mix slider spans 0 % to 200 %, and the help notes explicitly that values above 100 % increase rather than reduce the amount of processing — parallel compression and exaggerated processing share one control.

### Cross-plug-in integration

- `OBS-FF-PROC-032` `OBSERVED` (About FabFilter Pro-C 3; Pro-Q 4 Instance list): Pro-C 3 participates in the shared Instance List alongside Pro-Q 4, Pro-DS, and Pro-G, and the About page lists a preset browser with search, tags, and metadata.

---

## Pro-L 2 — limiter

All records cite `SRC-FABFILTER-PROL2-HELP`, page path given per record.

### Gain structure and limiting styles

- `OBS-FF-PROL-001` `OBSERVED` (Overview; Knobs and gain slider): The Gain slider is documented as the single most important control and moves two things at once — it raises the threshold of limiting and the output gain together, making audio louder while respecting the current maximum output level. Limiting depth and loudness are one gesture, not two.
- `OBS-FF-PROL-002` `OBSERVED` (Knobs and gain slider): The Output Level control can be reverse-linked to the Gain slider using the `Alt` modifier, so raising gain lowers the ceiling in one move.
- `OBS-FF-PROL-003` `OBSERVED` (Advanced settings): Eight limiting styles are offered: Transparent, Punchy, Dynamic, Allround, Aggressive, Modern, Bus, Safe. Documented characterizations are behavioral only — Transparent stays true to the original sound and feel; Punchy is the most apparent and introduces some pumping; Dynamic preserves original punch and clarity; Allround works on almost any material; Aggressive uses an aggressive near-clipping style; Modern is presented as the new all-purpose transparent standard; Bus is explicitly *not* transparent and aims at glue, pump, or squash; Safe is designed never to distort.
- `OBS-FF-PROL-004` `OBSERVED` (About FabFilter Pro-L 2): The About page groups the newer four styles as Modern (the default best-for-all algorithm), Aggressive (for EDM/dance), Bus (for drums and individual tracks), and Safe, which implies the other four are carried-forward styles. The help does not label any style as legacy explicitly.
- `OBS-FF-PROL-005` `OBSERVED` (Advanced settings): Lookahead sets the look-ahead time for an initial *transient* stage, expressed in ms. Attack controls how quickly the release stage sets in — shorter attack lets release begin sooner — and longer release times increase the release stage's effect. The limiter is therefore documented as a staged transient-then-release structure, but no numeric range is given for any of the three.
- `OBS-FF-PROL-006` `OBSERVED` (Advanced settings): Look-ahead times shorter than 0.1 ms are documented as approximating hard clipping. This is the only numeric anchor the page gives for a time control.
- `OBS-FF-PROL-007` `OBSERVED` (Advanced settings): Channel linking is split into two independent percentages — a Transient link and a Release link. Documented guidance is that transient linking often works well below 100 % while release linking is best started at 100 %. Exact ranges are only implied, not stated.

### True peak and ceiling

- `OBS-FF-PROL-008` `OBSERVED` (True peak limiting): True peak limiting is documented as doing two distinct jobs — reducing true peaks already present in the input, and smoothly attenuating overshoot introduced by the limiting process itself. Compliance is claimed against ITU-R BS.1770 and EBU R128, and the mode is stated as suitable for Mastered for iTunes.
- `OBS-FF-PROL-009` `OBSERVED` (True peak limiting): The additional processing for true peak limiting is documented as introducing about 5 ms of extra latency. No true-peak ceiling range is stated on the page.
- `OBS-FF-PROL-010` `OBSERVED` (True peak limiting): Documented guidance is that 4x oversampling combined with a minimum look-ahead of 0.1 ms keeps inter-sample peaks within about 0.1 dB in most cases. True peak limiting is therefore presented as a stronger guarantee than oversampling alone, not as a substitute for it.
- `OBS-FF-PROL-011` `OBSERVED` (Input and output options): Documented ceiling recommendations are −1.0 dBTP for EBU R128, Spotify, and YouTube; −2.0 dBTP for ATSC A/85; and −0.1 dBTP for CD and iTunes. The factory-preset Output Level is documented as 0.0 dB, so the shipped default is deliberately *not* one of the recommended broadcast ceilings.

### Loudness metering

- `OBS-FF-PROL-012` `OBSERVED` (Loudness metering; About FabFilter Pro-L 2): Loudness metering implements ITU-R BS.1770-4 and EBU R128, and additionally supports ATSC A/85 and TR-B32. Three time scales are shown: Momentary (continuously changing current loudness), Short Term (slower, overall loudness of the current audio), and Integrated (long-term, combining all audio since the last reset).
- `OBS-FF-PROL-013` `OBSERVED` (Loudness metering): Target-level presets are −9 LUFS (audio CDs), −14 LUFS (streaming services, naming Spotify, iTunes, YouTube), −23 LUFS (EBU R128), −24 LUFS (ATSC A/85 and TR-B32), plus a Custom entry.
- `OBS-FF-PROL-014` `OBSERVED` (Loudness metering): The loudness scale can be shown as +9 LU or +18 LU above target for EBU-mode compliance, and readings can be absolute or relative with the target displayed as zero.
- `OBS-FF-PROL-015` `OBSERVED` (Loudness metering): Loudness Range (LRA) is shown when the Integrated scale is selected and is defined per EBU R128 as the variation of loudness over time. True peak metering displays a maximum true peak level and must be enabled separately.
- `OBS-FF-PROL-016` `OBSERVED` (Loudness metering): Integrated measurement has explicit pause and reset controls plus an Auto-Reset option that resets when the host starts playback. The help also notes LUFS and LKFS are effectively interchangeable since ITU-R BS.1770-2.

### Level metering and display

- `OBS-FF-PROL-017` `OBSERVED` (Metering): The real-time level display superimposes input level, output level, gain reduction, and a thin loudness line on one scrolling plot, with gain-reduction peaks called out by yellow peak readings.
- `OBS-FF-PROL-018` `OBSERVED` (Metering): Meter scale options are 16 dB, 32 dB, and 48 dB, plus K-System scales K-12, K-14, and K-20, plus a dedicated Loudness scale. The 16 dB setting is documented as showing the top 5 dB region of the input, output, and gain-reduction meters.
- `OBS-FF-PROL-019` `OBSERVED` (Metering): The display's time behavior is a named option set — Slow Down, Fast, Slow, Infinite, Off — with no stated seconds-per-screen values for any option.
- `OBS-FF-PROL-020` `OBSERVED` (Metering): The output meter shows an RMS level following the EBU R128 Momentary specification both per channel and combined, and shows peak level in green when True Peak Metering is enabled. Maximum peak output level and gain reduction are shown numerically above the meters.

### Oversampling, dither, and utility options

- `OBS-FF-PROL-021` `OBSERVED` (Oversampling; About FabFilter Pro-L 2): Up to 32x oversampling is offered. Documented guidance splits real-time from offline use: 4x or 8x drastically reduces aliasing without extreme CPU cost, while 16x and 32x may sound better but are described as too taxing for most systems in real time. No per-factor latency figure is given anywhere.
- `OBS-FF-PROL-022` `OBSERVED` (Oversampling): Oversampling is documented as addressing three named problems — aliasing from rapid limiting adjustments, spurious non-musical frequencies, and inter-sample peaks that can distort during D/A conversion or MP3 encoding.
- `OBS-FF-PROL-023` `OBSERVED` (Dithering and noise shaping): Dither bit depths are Off, 24, 22, 20, 18, and 16 bits.
- `OBS-FF-PROL-024` `OBSERVED` (Dithering and noise shaping): Three noise-shaping options are documented by their tradeoff, not their filter design. Basic lowers the overall noise floor a few dB at the cost of raising noise above 6 kHz. Optimized reaches an even lower overall noise level but boosts noise above 10 kHz more extremely. Weighted transforms the noise spectrum according to the ear's sensitivity at low listening levels and is stated to be designed for use at 44.1 kHz.
- `OBS-FF-PROL-025` `OBSERVED` (Dithering and noise shaping): The help states dithering should only be used as the final stage of audio processing/mastering. It does not state whether the dither stage sits before or after the limiter internally.
- `OBS-FF-PROL-026` `OBSERVED` (Input and output options): Four discrete option buttons are documented. **Filter DC Offset** applies gentle DC-offset filtering to remove DC bias in the input. **Side Chain Triggering** feeds the detection path from the external side chain instead of the normal input, so the limiter can duck to a signal it does not output. **Unity Gain** automatically sets Output Level to the inverse of the current Gain for level-matched comparison. **Audition Limiting** subtracts the processed output from the input to monitor the delta signal — the limiter's own contribution in isolation.
- `OBS-FF-PROL-027` `OBSERVED` (Input and output options): Global bypass compensates for plug-in latency and applies soft bypassing to avoid clicks, so bypass comparison stays time-aligned and click-free.

---

## Pro-MB — multiband dynamics

All records cite `SRC-FABFILTER-PROMB-HELP`, page path given per record.

### Band model

- `OBS-FF-PROMB-001` `OBSERVED` (About FabFilter Pro-MB): Up to six processing bands are offered, freely placed anywhere in the spectrum rather than fixed to preset crossover regions.
- `OBS-FF-PROMB-002` `OBSERVED` (About FabFilter Pro-MB; Display and workflow): Crossover slopes are variable between 6 dB/oct and 48 dB/oct, and the About page states this variability applies in Dynamic Phase and Linear Phase mode.
- `OBS-FF-PROMB-003` `OBSERVED` (About FabFilter Pro-MB): The plug-in is documented as covering the full dynamics span in one device — transparent compression, limiting, expansion, pumping upward compression, and punchy gating — rather than being a compressor with a band splitter in front.
- `OBS-FF-PROMB-004` `OBSERVED` (About FabFilter Pro-MB): 64-bit internal processing and up to four times oversampling are stated. The four-times ceiling is markedly lower than Pro-C 3's and Pro-L 2's stated 32x.

### Per-band dynamics controls

- `OBS-FF-PROMB-005` `OBSERVED` (Basic band controls): Threshold sets the level for compression *or* expansion — one control serves both directions.
- `OBS-FF-PROMB-006` `OBSERVED` (Basic band controls): **Range** does two jobs: it limits the maximum applied gain change and it chooses between downward and upward compression or expansion. The sign of the range parameter therefore selects processing direction, mirroring Pro-Q 4's dynamic-range ring (`OBS-FF-PROQ-009`). No numeric range is stated.
- `OBS-FF-PROMB-007` `OBSERVED` (Basic band controls): Attack and Release are expressed as 0 % to 100 % rather than in time units. The About page states the underlying curves are intelligent and highly program- and frequency-dependent, so the percentage is a position within an adaptive curve family, not a millisecond value.
- `OBS-FF-PROMB-008` `OBSERVED` (Basic band controls): Ratio adjusts the amount of compression or expansion and Knee sets the knee type; neither has a stated numeric range or unit on the inspected page.
- `OBS-FF-PROMB-009` `OBSERVED` (Basic band controls; About FabFilter Pro-MB): Lookahead is per band and lets a band begin reacting up to 20 ms before the gain change is detected. The same 20 ms ceiling appears in Pro-C 3 (`OBS-FF-PROC-010`).
- `OBS-FF-PROMB-010` `OBSERVED` (Basic band controls): Each band has its own output Level and a Pan that adjusts panning between mid and side levels — per-band make-up gain and per-band stereo placement are part of the band, not a global stage.

### Side chain and stereo linking

- `OBS-FF-PROMB-011` `OBSERVED` (Expert band controls): The trigger frequency range is selectable per band as either **Band** (the band's own input, the default and the behavior when Expert mode is off) or **Free** (any range in the spectrum). Detection band and processing band are therefore decoupled.
- `OBS-FF-PROMB-012` `OBSERVED` (Expert band controls): Each band independently selects internal (normal plug-in input) or external side-chain input as its trigger source. Side-chain source is per band, not per plug-in.
- `OBS-FF-PROMB-013` `OBSERVED` (Expert band controls): An Audition button monitors the filtered and stereo-linked trigger signal for that band, supporting both a latching click and a momentary click-and-hold — the same interaction Pro-C 3 uses (`OBS-FF-PROC-019`).
- `OBS-FF-PROMB-014` `OBSERVED` (Expert band controls): Stereo Link runs 0 % (channels fully independent) to 100 % (fully linked) on the trigger signal, and a separate Stereo Link Mode button selects mid-only (mono content) or side-only (stereo content) processing.

### Processing modes and phase

- `OBS-FF-PROMB-015` `OBSERVED` (Processing mode): **Dynamic Phase** is the default and is documented as producing a flat/linear phase response when no gain processing is applied, without introducing latency or pre-ringing; phase effects appear only when a band's gain actually changes. It is described as by far the most transparent mode, suitable for both mastering and mixing.
- `OBS-FF-PROMB-016` `OBSERVED` (Processing mode): **Linear Phase** guarantees a flat phase response for the split-and-resummed spectrum but is documented as introducing quite a bit of extra latency and possible pre-ringing. No latency value in samples or milliseconds is given.
- `OBS-FF-PROMB-017` `OBSERVED` (Processing mode): **Minimum Phase** adds no extra latency but introduces static phase changes at the crossover frequencies, and the help calls it virtually unusable for mastering except when using 6 dB/oct slopes throughout.
- `OBS-FF-PROMB-018` `OBSERVED` (Processing mode): Unlike Pro-Q 4, Pro-MB's processing-mode page states no latency figures, no linear-phase resolution options, and no CPU comparison between modes.

### Display and interaction

- `OBS-FF-PROMB-019` `OBSERVED` (Display and workflow): Band creation is layered like Pro-Q 4's — the first band is created by clicking anywhere in the display; further bands by hovering an empty area or an existing band and clicking a `+` button, by dragging the yellow overall curve, or by double-clicking an empty area. Band width during creation is set by the mouse wheel while hovering.
- `OBS-FF-PROMB-020` `OBSERVED` (Display and workflow): Band editing uses the same modifier vocabulary as Pro-Q 4: drag the band's peak dot vertically for gain and horizontally for centre frequency; mouse wheel or `Ctrl`/`Cmd`+vertical drag for width; `Shift`+drag to fine-tune; `Alt`+drag to escape the axis constraint.
- `OBS-FF-PROMB-021` `OBSERVED` (Display and workflow): Multi-band selection mirrors Pro-Q 4 — rubber-band rectangle on the background for adjacent bands, `Ctrl`/`Cmd`+click for non-adjacent, `Shift`+click for a consecutive range, click empty area to deselect. Slope buttons can themselves be multi-selected by dragging a rectangle around them.
- `OBS-FF-PROMB-022` `OBSERVED` (Display and workflow): Crossover lines are dragged independently. Solo and mute buttons support press-and-hold for a temporary state and `Ctrl`/`Cmd`+click for exclusive solo/mute.
- `OBS-FF-PROMB-023` `OBSERVED` (Display and workflow): A thick yellow curve displays the *overall dynamic frequency response at the present moment* — the live, moving result of all bands, not the static setting.

### Gain staging and metering

- `OBS-FF-PROMB-024` `OBSERVED` (Input and output options): Input and output are combined level/pan knobs, input applied before all processing and output after — the same shape as Pro-C 3 (`OBS-FF-PROC-030`). Neither has a stated numeric range.
- `OBS-FF-PROMB-025` `OBSERVED` (Input and output options; About FabFilter Pro-MB): The global Mix slider spans 0 % to 200 %, identical to Pro-C 3's (`OBS-FF-PROC-031`).
- `OBS-FF-PROMB-026` `OBSERVED` (Input and output options): The output level meter uses a grey display scale from −100 dB to 0 dB and shows the maximum output level with a clipping indicator. Global bypass compensates for plug-in latency and soft-bypasses to avoid clicks, matching Pro-L 2 (`OBS-FF-PROL-027`).

---

## Pro-DS — de-esser

All records cite `SRC-FABFILTER-PRODS-HELP`, page path given per record. Three `Using` pages were read: Basic controls, Advanced controls, Metering.

### Detection and threshold

- `OBS-FF-PRODS-001` `OBSERVED` (About FabFilter Pro-DS): The plug-in is scoped to sibilance reduction and states two detection modes — **Single Vocal**, described as an intelligent detection algorithm for a single vocal track, and **Allround**, described as high-frequency limiting for drums and full mixes. The mode is a discrete button, not a continuous control.
- `OBS-FF-PRODS-002` `OBSERVED` (Basic controls): Threshold sets the side-chain level above which gain reduction triggers. The knob carries a circular side-chain level meter around it, so the level being thresholded is read at the control that sets the threshold rather than on a separate meter.
- `OBS-FF-PRODS-003` `OBSERVED` (Basic controls): Range scales the *detected* gain reduction so it stays within a chosen bound. Detection and the amount applied are therefore separate stages, not one curve.
- `OBS-FF-PRODS-004` `OBSERVED` (Basic controls): The detection side chain is band-limited by a high-pass and a low-pass slider, each spanning **2 kHz to 20 kHz**. The page notes normal vocal s-sounds sit around 8–10 kHz as guidance, not as a default.
- `OBS-FF-PRODS-005` `OBSERVED` (Basic controls): The two modes differ in what the side chain means. Single Vocal splits sibilance out by algorithm; Allround depends on the filtered frequency range plus Threshold. The same two knobs therefore drive two different detection contracts.
- `GAP-FABFILTER-0060` `SOURCE-GAP`: No numeric range, unit, or default is stated for Threshold or Range on the Basic controls page. Only the two side-chain filter sliders carry numerals.

### Processing topology and latency

- `OBS-FF-PRODS-006` `OBSERVED` (Advanced controls): Two processing topologies are offered. **Full Band** reduces overall gain when sibilance is detected; **Split Band** attenuates only the high frequencies, and **the split point is the high-pass side-chain filter setting** — one control serves both detection and the processing split.
- `OBS-FF-PRODS-007` `OBSERVED` (Advanced controls): Lookahead is up to **15 ms**, with a separate enable button; the page suggests roughly 10 ms for vocals. Latency is stated as a consequence rather than a number per mode: disabled lookahead with Wide Band and no oversampling incurs **no latency**; enabling lookahead costs the full **15 ms** plus whatever oversampling and split-band processing add.
- `OBS-FF-PRODS-008` `OBSERVED` (About FabFilter Pro-DS): Up to **4× linear-phase oversampling** is available, and Split Band is described as linear phase.
- `GAP-FABFILTER-0061` `SOURCE-GAP`: The added latency of split-band processing and of each oversampling factor is never quantified, so total reported latency cannot be derived from the help.

### Stereo and side chain

- `OBS-FF-PRODS-009` `OBSERVED` (Advanced controls): Stereo Link runs **0 % to 100 %** and **past 100 %** into mid-only or side-only processing, selected by a Stereo Link Mode button — the same control vocabulary Pro-C 3 (`OBS-FF-PROC-019`) and Pro-MB (`OBS-FF-PROMB-014`) use, but with the mid/side selection folded into the far end of the same knob's travel rather than a separate mode.
- `OBS-FF-PRODS-010` `OBSERVED` (Advanced controls): An external side-chain input can replace the internal signal for detection, and an Audition Side-chain button monitors the stereo-linked and mid/side-processed trigger signal live.

### Metering

- `OBS-FF-PRODS-011` `OBSERVED` (Metering): The primary display is a **real-time waveform** of the incoming signal after input gain, with the processed portions highlighted in proportion to processing intensity. The plug-in's main visualization is therefore time-domain, unlike Pro-Q 4's and Pro-MB's frequency-domain displays.
- `OBS-FF-PRODS-012` `OBSERVED` (Metering): A spectrum display inside the side-chain filter section shows the strong frequencies of the **post-filter** side-chain signal, so the analyzer shows what the detector hears, not what the input contains.
- `OBS-FF-PRODS-013` `OBSERVED` (Metering): Output and gain-reduction meters sit at the right. A dB read-out holds the maximum detected level and is reset by clicking it; a clipping indicator fires above 0 dBFS, and the page states explicitly that **the audio is not clipped inside Pro-DS** — the indicator reports, it does not act.
- `OBS-FF-PRODS-014` `SPECTRE-CANDIDATE`: Two separations here are worth carrying into a Spectre metering contract as questions rather than answers: a clipping *indicator* that provably does not alter audio, and an analyzer scoped to the detector's post-filter signal rather than the device input. Both are honesty properties — the meter reports the thing it names — and both are cheap to specify and easy to get wrong.

---

## Saturn 2 — multiband saturation and distortion

All records cite `SRC-FABFILTER-SATURN2-HELP`, page path given per record. Five `Using` pages were read: Display, Band controls, Modulation, Modulation slots, Input and output.

### Band model

- `OBS-FF-SAT-001` `OBSERVED` (About FabFilter Saturn 2): The plug-in is a multiband distortion and saturation processor with up to **6 bands**.
- `OBS-FF-SAT-002` `OBSERVED` (Display): A band is created by clicking a `+` button that appears at the top of the display on hover; **the current band splits at the mouse position and the new band inherits its parent's settings.** Band creation is a split operation on an existing band, not an insertion into empty space — a different model from Pro-Q 4's and Pro-MB's.
- `OBS-FF-SAT-003` `OBSERVED` (Display): Crossover slope is selectable from **6 dB/oct to 48 dB/oct**, a narrower span than Pro-MB's per-crossover range and set by a single button.
- `OBS-FF-SAT-004` `OBSERVED` (Display): Crossovers move two ways — dragging the vertical split directly, or dragging a band's level button horizontally, which moves **both** of that band's edges at once. Double-clicking a split enters a frequency numerically.
- `OBS-FF-SAT-005` `OBSERVED` (Display): Solo and mute appear on hover when more than one band exists, support press-and-hold for a temporary state, and take `Ctrl`/`Cmd`+click for exclusivity — identical to Pro-MB (`OBS-FF-PROMB-022`).

### Per-band processing

- `OBS-FF-SAT-006` `OBSERVED` (Band controls): Each band carries Enabled, Drive, Mix, Feedback Amount, Feedback Frequency, Dynamics, Tone, and Level. Drive controls the clipping stage's input level **with automatic output compensation**, so drive and loudness are decoupled by design rather than by the user.
- `OBS-FF-SAT-007` `OBSERVED` (Band controls): Dynamics is a single bidirectional control — gating/expansion toward one end, compression toward the other — placed inside the distortion band rather than as a separate device.
- `OBS-FF-SAT-008` `OBSERVED` (Band controls): Level spans **minus infinity to +36 dB** and carries a pan ring that switches between stereo and mid/side. Level is the only per-band control on this page with a stated numeric range.
- `OBS-FF-SAT-009` `OBSERVED` (Band controls): Named distortion styles are Tube, Tape, Amplifier, Transformer, Smudge, Breakdown, Foldback, Rectify, and Destroy. The index page and About page describe multiple variants within the Amplifier and Transformer families.
- `OBS-FF-SAT-010` `OBSERVED` (Display): Editing gestures overload one drag: vertical drag or wheel sets **level**, `Alt`+drag or `Alt`+wheel sets **drive**, `Shift` fine-tunes (`Ctrl` in Pro Tools), `Ctrl`/`Cmd`+click resets level, and double-click enters a value. Two continuous parameters share one gesture surface, separated only by a modifier.
- `GAP-FABFILTER-0062` `SOURCE-GAP`: No range, unit, or default is stated for Drive, Mix, Feedback Amount, Feedback Frequency, Dynamics, or any Tone control. Only Level's range is given.

### Modulation

- `OBS-FF-SAT-011` `OBSERVED` (Modulation): Five modulation source types exist — XLFO, Envelope Generator (ADSR, audio- or MIDI-triggered), Envelope Follower, MIDI Source, and XY Controller. Each source carries **its own color scheme** used consistently across the interface.
- `OBS-FF-SAT-012` `OBSERVED` (Modulation slots): Routing is drag-and-drop: grab a source's drag button, drop it on a highlighted target. Every connection passes through a **modulation slot** that owns the depth, so depth is a property of the connection rather than of the source or the target.
- `OBS-FF-SAT-013` `OBSERVED` (Modulation slots): A slot carries a level slider (with `Shift` fine-tune and `Alt` to move every slot from the same source at once), a polarity `+`/`−` button, an on/off toggle, source and target dropdowns, and a remove button. The connection is fully editable after the fact from either end.
- `OBS-FF-SAT-014` `OBSERVED` (Modulation slots): Slots that modulate **another slot's level** are displayed indented and connected beneath their parent, so modulation-of-modulation is visible as nesting rather than inferred.
- `OBS-FF-SAT-015` `OBSERVED` (Modulation slots): A modulated control shows a small indicator dot colored to match its source; multiple sources produce multiple dots. Modulation presence and its origin are readable at the target without opening a routing view.
- `GAP-FABFILTER-0063` `SOURCE-GAP`: No maximum count is stated for modulation sources or slots. "Up to 6 XY controllers" appears on the index page, but no limit is given for XLFOs, envelope generators, followers, MIDI sources, or total slots.

### Quality modes, phase, and gain staging

- `OBS-FF-SAT-016` `OBSERVED` (Input and output): Input and output are each adjustable **−36 dB to +36 dB**.
- `OBS-FF-SAT-017` `OBSERVED` (Input and output): High Quality mode oversamples internally at two settings — **Good = 8×** and **Superb = 32×** — with the page warning the Superb CPU increase can be "quite drastic". These are the highest oversampling factors named anywhere in the inspected FabFilter corpus.
- `OBS-FF-SAT-018` `OBSERVED` (Input and output): Linear Phase mode applies linear-phase filters to **the crossover and the internal oversampling only**. The page states explicitly that other stages — tone EQ, amp modeling — do **not** switch to linear phase. A "linear phase" claim here is scoped to part of the signal path, and the help says which part.
- `OBS-FF-SAT-019` `OBSERVED` (Input and output): Global bypass soft-bypasses to avoid clicks and compensates latency, and the page states the compensation works correctly with Linear Phase and High Quality modes engaged — the two settings that change latency.
- `OBS-FF-SAT-020` `SPECTRE-CANDIDATE`: `OBS-FF-SAT-018` is the strongest documentation-honesty example in this corpus and is worth adopting as a Spectre *documentation* rule rather than a DSP one: when a device offers a phase or quality mode that applies to some stages and not others, the device's own surface must say which stages, because a global-sounding label over a partial guarantee is the failure mode.

---

## Pro-R 2 — reverb

All records cite `SRC-FABFILTER-PROR2-HELP`, page path given per record. Five `Using` pages were read: Main controls, Decay Rate EQ, Post EQ, Impulse Response import, plus the About page.

### Main controls

- `OBS-FF-PROR-001` `OBSERVED` (Main controls): **Space** is a single stepless control combining room model and decay time, spanning **200 ms to 10 seconds**, moving continuously across more than a dozen designed room models. The conventional two-parameter split — pick a room, then set a decay — is collapsed into one axis.
- `OBS-FF-PROR-002` `OBSERVED` (Main controls): **Decay Rate** then scales that decay from **25 % to 400 %** of what the current Space setting implies. Absolute decay is therefore expressed as a base plus a proportional trim, not as a time.
- `OBS-FF-PROR-003` `OBSERVED` (Main controls): **Predelay** is the gap between the dry sound and the reverb onset, spanning **0–500 ms**, and can sync to host tempo at quarter, 8th, 16th, and 32nd notes. When synced, a **Predelay Offset** knob scales the synchronized time **50 % to 200 %** — the sync is a base that stays musical while remaining tunable.
- `OBS-FF-PROR-004` `OBSERVED` (Main controls): **Character** spans **0–100 %**: 0 % is transparent, 50 % introduces modulation and more pronounced early reflections, 100 % reaches a chorus-like effect. The page gives behavior at three named points on the range rather than only at the ends.
- `OBS-FF-PROR-005` `OBSERVED` (Main controls): **Distance** trades brighter early reflections at 0 % against a longer build-up and more diffuse tail as it rises; **Brightness** shifts high/low decay balance to model spaces with more high-frequency absorption. Both are described by perceptual outcome, with no unit.
- `OBS-FF-PROR-006` `OBSERVED` (Main controls): **Stereo Width** starts at 0 % (mono), reaches maximum cross-feeding at **50 %**, full channel independence at **100 %**, and can exceed 100 %. The midpoint is a distinct behavior, not an interpolation between the ends.
- `OBS-FF-PROR-007` `OBSERVED` (Main controls): Pro-R 2 adds **Ducking** (separating wet from dry dynamically), **Thickness** (density combined with saturation), **Auto Gate** (automatic threshold, attack and release, tempo-syncable), and **Freeze** (eternal decay by reusing the current reverb tank contents, with some parameters restricted while engaged).
- `OBS-FF-PROR-008` `OBSERVED` (Main controls): **Mix** carries a Lock option and the page instructs setting it to 100 % on a send. The lock exists because the next observation makes Mix something the plug-in would otherwise adjust implicitly.
- `GAP-FABFILTER-0064` `SOURCE-GAP`: No range or unit is stated for Distance, Brightness, Thickness, or Ducking, and Auto Gate's automatic threshold, attack, and release values are not exposed.

### Decay Rate EQ and Post EQ

- `OBS-FF-PROR-009` `OBSERVED` (Decay Rate EQ): Up to **six** Bell, High/Low Shelf, or Notch curves shape **decay rate per frequency** rather than amplitude — extending decay at some frequencies while shortening it at others. The page motivates it physically: in real spaces high frequencies decay fast while bass persists.
- `OBS-FF-PROR-010` `OBSERVED` (Decay Rate EQ): The Decay Rate curve is drawn in blue and the Post EQ curve in a different color, so two EQ-shaped curves over the same frequency axis are distinguished by color while meaning entirely different quantities — one time, one level.
- `OBS-FF-PROR-011` `OBSERVED` (Post EQ): Post EQ offers up to **six** bands with bell, shelf, and low-cut/high-cut shapes, and the display range is switchable between **±30 dB, ±18 dB, and ±9 dB**.
- `OBS-FF-PROR-012` `OBSERVED` (Post EQ): **Reverb gain is automatically compensated for Post EQ changes**, so shaping the reverb does not require re-adjusting Mix or the send level. The device absorbs the gain consequence of an EQ edit rather than exposing it.
- `GAP-FABFILTER-0065` `SOURCE-GAP`: The Decay Rate EQ's own range and units are never stated — the page says what the curve means but not how far it goes in either direction.

### Impulse response import

- `OBS-FF-PROR-013` `OBSERVED` (Impulse Response import): Pro-R 2 accepts **WAV or AIFF** impulse responses and **does not convolve them**. It interprets the file and **converts it into Decay Rate EQ, Post EQ, and main control settings** — the import produces a parameter state, not a convolution kernel.
- `OBS-FF-PROR-014` `OBSERVED` (Impulse Response import): The conversion is shown in the interface as it happens, and the reverb style defaults to Modern, documented as the most natural style for matching real spaces.
- `OBS-FF-PROR-015` `OBSERVED` (Impulse Response import): The page states its own limit plainly: results "will never match perfectly, but will certainly be 'ballpark'", offering a fully customizable substitute for an IR reverb. It also warns when the user loads a sine-sweep response or a non-IR file, because those cannot be interpreted properly.
- `OBS-FF-PROR-016` `SPECTRE-CANDIDATE`: The import model — read an external artifact, convert it into ordinary editable device parameters, and state up front that the match is approximate — is a **general** pattern worth considering wherever Spectre imports anything, and it is the opposite of an opaque imported blob. The property that makes it work is that the result lands in the same parameter space the user already edits, so nothing about the device becomes unreachable after an import. Recorded as a candidate only; Spectre has no reverb, no convolution, and no import path.
- `GAP-FABFILTER-0066` `SOURCE-GAP`: No limit is stated on IR length, sample rate, or channel count, and the analysis method is not described beyond the outcome.

---
