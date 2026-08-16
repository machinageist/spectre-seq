<!--
Author: Jeff
Date: 2026-08-15
Description: R4-3 spec — the protocol and record format for qualifying a Linux ALSA audio device against the existing lifecycle drill, discharging decision 23
Notes: Execution is hardware-blocked; authoring is not. This spec changes no Rust. It fixes what
  counts as a pass, what is an observation rather than a threshold, what a refusal or a failure
  records, and what claim a PASS does and does not authorize. No Linux support claim is asserted
  anywhere in this document. As of 2026-08-15 no Linux audio device has ever been opened.
-->

# Spec: Linux Device Qualification

**Feature ID:** `R4-3` (`linux-device-qualification`)
**Parent feature:** `R4` Credible Alpha (root)
**Spec author agent:** gauntlet spec agent, R4-3 leaf
**Date:** 2026-08-15
**Iteration:** 1

- **Status:** proposed
- **Last verified:** 2026-08-15 (every source path below opened at branch `rename/geist-to-spectre`, commit `2e005e5`)
- **Scope:** the qualification protocol and its record format for running `hardware_lifecycle_drill` on a Linux host with a real ALSA device, and the evidence rule that closes decision 23. Out of scope: any change to the drill itself, any change to `spectre-audio`, and the execution of the run
- **Decision authority:** Jeff
- **Upstream sources:** `docs/01-requirements/decision-gates.md` rows 1, 16, 19, 20, 23; `docs/01-requirements/requirements-ledger.md` RT-001/RT-002/RT-003 and PROD-003; `docs/06-plans/current-milestone.md` §"Inherited debt" item 1, §"Hardware qualification record", §"Single-platform exit and the debt it creates"; `docs/status/NEXT.md` slice 3; `docs/status/STATUS.md` known gaps; `docs/README.md` conflict precedence and status vocabulary
- **Downstream dependents:** `docs/06-plans/current-milestone.md` (the qualification table is this spec's output artifact), `docs/01-requirements/decision-gates.md` row 23 (closable only by this protocol), `docs/status/STATUS.md`, `docs/status/NEXT.md` slice 3, R4-1 §4.6 (whose two named ALSA risks this protocol converts into recorded data), R4-9 (manual QA protocol, which should reuse the record format rather than invent a second one)
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** §8 Q1–Q10
- **Known gaps:** (a) **execution is hardware-blocked** — no Linux host with a real ALSA device is available to this spec, so the protocol is authorable and reviewable but not runnable, see §7.4; (b) the accepted research corpus contains **no citable observation** about how any benchmark product qualifies an audio device, states its own driver-conformance criteria, or records a device test result — recorded as a research need in §Appendix A and §8 Q8, not filled in from recollection; (c) the existing macOS row's block count is arithmetically inconsistent with the geometry the drill requests and the record cannot resolve why (§4.6, §8 Q1).

This spec is subordinate to the conflict precedence in `docs/README.md`. It proposes a
protocol and a record format. It does not amend an accepted requirement, decision row, or
architecture contract; where it proposes new normative rows it says so and routes them to
§8 rather than asserting them.

**Standing constraint, stated once and binding on every section below.** Nothing in this
document claims that Spectre works on Linux, that cpal's ALSA backend opens a stream, or
that any Linux audio device has been driven. What is established as of 2026-08-15 is a
build-and-link result only: on 2026-08-09 the workspace compiled and linked against ALSA
in a Linux aarch64 container and all 34 non-hardware `spectre-audio` tests passed, and the
drill failed closed on a machine with no device rather than reporting a false pass
(`docs/06-plans/current-milestone.md:116`). No Linux audio device has ever been opened
(`docs/status/STATUS.md` known gaps; `docs/06-plans/current-milestone.md:127`). Every
outcome this spec describes is conditional on a run that has not happened.

---

## 1. Purpose

### 1.1 One-sentence job

As the person accountable for what Spectre claims, I need one Linux run of the existing
hardware drill to produce a record precise enough that a reader six months from now can
tell exactly what was proven, on what hardware, through which sound server, and what
remains unproven — so that decision 23's debt is discharged by evidence rather than closed
by assertion.

### 1.2 Why it matters

Decision 1 makes macOS and Linux co-first-class
(`docs/01-requirements/decision-gates.md:25`, **Accepted**). Decision 23 narrowed R3's exit
bar to macOS hardware alone and states in its own text that "no Linux support claim is
authorized until the drill runs on real Linux hardware"
(`docs/01-requirements/decision-gates.md:49`). The commitment in decision 1 is therefore
currently **undischarged**, and `docs/status/STATUS.md` records that in its known-gaps
line. `gauntlet-output/criteria.md` §3E names this "the live risk in R4, not a
hypothetical."

The debt is one command (`docs/06-plans/current-milestone.md:131–133`,
`docs/status/NEXT.md:25`). That is exactly why it is dangerous: a one-command task invites
a one-line record. The macOS row already shows the cost of an under-specified record. It
reports 173 blocks, 0 xruns, worst headroom 0.990
(`docs/06-plans/current-milestone.md:113`) — and the record does not say which of those
numbers were pass conditions and which were bystanders, does not say what buffer size the
driver actually granted, and does not say what the run failed to cover. §4.6 shows that
the block count cannot be reconciled with the geometry the drill requests, and the record
has no field that could resolve it. Repeating that shape on Linux would produce a second
number that looks like a claim and is not one.

The user pain this addresses is not a musician's. It is the pain of a maintainer who ships
a Linux build on the strength of a table row that turns out to have meant less than it
looked like it meant.

### 1.3 Success signal

**Observable outcome:** `docs/06-plans/current-milestone.md`'s hardware qualification table
contains a Linux row whose every cell is populated with a measured value or an explicit
outcome word, accompanied by a numbered environment record (§3.3, fields E1–E10) that
another person could use to reproduce the run, and `docs/01-requirements/decision-gates.md`
row 23 carries a dated disposition citing that row.

**The signal is not "the row says PASS."** A `FAIL`, `INCONCLUSIVE`, or `REFUSED` row with a
complete environment record is a successful application of this protocol; it discharges the
*protocol* obligation and leaves the *support claim* correctly unauthorized. The failure
mode this spec exists to prevent is an empty row, a discarded run, or a row whose numbers
cannot be interpreted.

---

## 2. User Stories

> As the maintainer, I want to run one documented command on a Linux box with a real sound
> card and get a printed line I can transcribe into a fixed table format, so that closing
> decision 23 is mechanical rather than a judgement call made months later from memory.

> As the maintainer, I want the protocol to tell me before I start which printed numbers
> are pass conditions and which are observations, so that I do not silently promote a
> bystander number into a threshold or ignore a number that should have failed the run.

> As the maintainer whose ALSA device refuses the drill's fixed 256-frame buffer or runs at
> 44 100 Hz, I want the refusal recorded as data with the driver's own error string, so
> that the run produces a finding about cpal-on-ALSA instead of being thrown away as "it
> didn't work."

> As the maintainer running on a desktop distribution, I want to be required to record
> whether PipeWire or PulseAudio was mediating the device, so that a result obtained
> through a compatibility layer is never mistaken for a result on raw ALSA hardware.

> As a reviewer reading the table long after the run, I want the row to state what the run
> did **not** cover — one device, one geometry, one kernel, one machine, silence rather
> than a sounding voice — so that I cannot reasonably read it as "Linux is supported."

> As an operator using a screen reader or a monochrome terminal, I want the drill's output
> and the recorded row to be plain text with no colour-carried meaning and no box-drawing
> characters, so that the evidence is readable in the same way by everyone who has to read
> it.

> As a contributor with no Linux audio hardware, I want the protocol to state plainly that
> CI cannot run this and that a container or VM without a real device cannot satisfy it, so
> that I do not spend time trying to automate a gate that is deliberately manual.

---

## 3. UX Specification

**Framing.** This feature introduces no application UI. Its two surfaces are real,
however, and are specified here rather than dismissed: a **terminal transcript** produced
by an existing test binary, and a **Markdown table row plus environment record** in an
accepted planning document. §3.1–§3.7 specify those two surfaces. Where a template
subsection genuinely has no referent, it is marked N/A with the reason.

### 3.1 Screen / view inventory

| Surface | Reached by | New vs. modified | Layout pattern |
|---|---|---|---|
| Drill transcript (stdout) | `cargo test -p spectre-audio --test lifecycle_health -- --ignored --nocapture` | **Existing, unmodified.** Emitted by `crates/spectre-audio/tests/lifecycle_health.rs:257` and `:281–288` | Two plain `key=value` lines plus libtest's own result line |
| Hardware qualification table | `docs/06-plans/current-milestone.md` §"Hardware qualification record" (`:109–118`) | **Modified** — the `Linux` row at `:114` is filled in; the column set is not changed | 8-column Markdown table |
| Qualification environment record | Immediately below the same table | **New** — a numbered block, one entry per non-macOS row | Definition list, one `Ex —` line per field |
| Durable protocol document | `docs/05-quality/device-qualification-protocol.md` | **New file** (§7.2). `docs/README.md:50` sanctions creating `05-quality/` when it holds a grounded contract | Prose contract with the §3.3 field list and §5.5 outcome rules |

No `./spectre` screen, panel, dialog, or menu is added or modified. `./spectre` does not
use `spectre-audio` at all today (`docs/status/STATUS.md` repository state), and this
feature does not change that.

### 3.2 Interaction flows

**Primary flow (operator, on the Linux host).**

1. Confirm the host has a real ALSA output device. The protocol's own precondition check is
   `aplay -l`, which must list at least one playback hardware device. A host where this
   prints "no soundcards found" cannot satisfy the run, and the drill will fail closed at
   `lifecycle_health.rs:253` with `no output device to qualify against` — the behavior
   already observed in the 2026-08-09 container run (`docs/06-plans/current-milestone.md:116`).
2. Record fields E1–E7 of §3.3 **before** running, because several of them (kernel, ALSA
   version, sound server) are properties of the host, not of the run.
3. Run the workspace gate first, unchanged and in full, so that the qualifying run is known
   to be against a green tree (§5.2).
4. Run the drill command (§5.1). It prints two lines and a libtest verdict.
5. Transcribe the printed values into the table row (§3.3) and the environment record.
6. Evaluate the outcome against §5.5's rules — **not** against libtest's pass/fail, which
   is necessary but not sufficient (§4.3).
7. Write the row whatever the outcome was. Then update the documents listed in §7.2.

**Branch A — no device.** The drill panics at `lifecycle_health.rs:253`. Outcome word
`INCONCLUSIVE`; environment record states the precondition that failed. Not a `FAIL`: the
drill proved nothing about the backend.

**Branch B — the open is refused.** `AudioBackend::open_output` returns
`BackendError::OpenFailed(String)` (`crates/spectre-audio/src/cpal_backend.rs:144`) and the
drill's `.expect("opening the default device must succeed")`
(`lifecycle_health.rs:271`) turns it into a panic carrying that string. Outcome word
`REFUSED`; the panic message is copied verbatim into environment field E10. This is the
branch R4-1 §4.6 predicts for `BufferSize::Fixed(256)` and for a device that will not run
at 48 000 Hz. It is a **finding**, not a wasted run (§4.4).

**Branch C — the run completes but a pass condition is not met.** libtest reports `ok`
while, for example, `xruns=3` was printed. Outcome word `FAIL`. §4.3 exists because this
branch is easy to miss.

**Branch D — the run completes and every pass condition is met.** Outcome word `PASS`, and
only then does §5.5's authorization clause take effect.

**Cues.** N/A — there is no haptic, sound, or animation surface. The drill is deliberately
**not** an audibility test (§4.4); the operator must not treat hearing nothing as a
failure, or hearing something as a pass.

### 3.3 Layout descriptions

**Table row format.** The column set at `docs/06-plans/current-milestone.md:111–112` is
**not changed**, because widening it would leave the existing macOS row with blanks in
columns it never measured, converting a complete record into an apparently incomplete one.
The Linux row fills the existing eight cells and gains a footnote marker in the first cell:

```markdown
| Platform | Date | Backend / device | Blocks | xruns | Worst headroom | Plan errors | Contaminated |
|---|---|---|---|---|---|---|---|
| Linux [L1] | YYYY-MM-DD | cpal / ALSA, {device key exactly as printed} | {blocks} | {xruns} | {worst_headroom} | {plan_errors} | {contaminated} |
```

- Every numeric cell is transcribed verbatim from `lifecycle_health.rs:281–288`. No
  rounding, no unit conversion, no "≈".
- `{device key exactly as printed}` is the `device=` value from `lifecycle_health.rs:257`.
  It matters that it is verbatim: cpal's device key **is** `cpal::Device::name()`
  (`cpal_backend.rs:17–22`), which on ALSA is the PCM name, so a key of `pipewire`,
  `pulse`, `default`, `sysdefault:CARD=…`, or `hw:CARD=…` is itself the strongest available
  evidence about what was actually opened (§4.6).
- A non-`PASS` row writes `—` in cells the run never produced and carries its outcome word
  in the footnote, never in a numeric cell.

**Environment record.** A new numbered block immediately below the table. One entry per
non-macOS row. Fields:

```
[L1] Linux qualification run, {ISO date} — outcome: PASS | FAIL | INCONCLUSIVE | REFUSED
  E1  — distribution and release (e.g. from /etc/os-release PRETTY_NAME)
  E2  — kernel release and machine architecture (uname -sri)
  E3  — ALSA runtime version (cat /proc/asound/version)
  E4  — sound server mediating the device, if any: none (raw ALSA) | PipeWire {version} |
        PulseAudio {version}, and how that was determined
  E5  — hardware: card and codec as reported by aplay -l, plus interface type
        (onboard / USB / PCI)
  E6  — cpal device key exactly as printed by the drill
  E7  — Spectre commit SHA, rustc version, cargo features in effect
        (default = ["cpal-backend"] unless stated)
  E8  — requested geometry: 48 000 Hz, 256 frames, 2 channels; granted geometry:
        {known / not recorded by the drill — see §8 Q2}
  E9  — workspace gate result immediately before the run (§5.2)
  E10 — verbatim panic message or driver error string, if any; otherwise "none"
```

**Empty state.** Before the run, the Linux row stays exactly as it is today —
`| Linux | not run | cpal / ALSA (decision 20 baseline) | — | — | — | — | — |`
(`docs/06-plans/current-milestone.md:114`). "not run" is the correct empty state and must
not be replaced by a placeholder that looks like a measurement.

**Data source.** Every cell is sourced from the drill's stdout or from a named shell
command on the host. No cell is sourced from inference, and no cell is sourced from
`docs/status/STATUS.md`.

### 3.4 Input & gestures

- **Keyboard:** one shell command (§5.1) plus the precondition commands named in §3.3. No
  interactive input; the drill takes no stdin.
- **Specialized input:** N/A — no stylus, controller, voice, or camera surface exists.
- **Keyboard shortcuts:** **none are defined, and none may be.** AF-5 prohibits a default
  shortcut map at the current evidence level, and this feature has no application surface
  that could carry one.
- **Responsive behavior:** the transcript is two short `key=value` lines and wraps
  acceptably at 80 columns; the table is a Markdown table read in an editor or on a code
  host. No responsive work is required.

### 3.5 Transitions & animation

N/A — the surfaces are a terminal transcript and a Markdown document. There is no
navigation transition, no in-view state change, and therefore no reduced-motion
alternative to specify. This is a real N/A rather than a deferral: introducing any
animation here would be inventing a surface.

### 3.6 Error states

| # | Trigger | Presentation | Recovery | Data loss risk |
|---|---|---|---|---|
| E-1 | No output device on the host | Panic at `lifecycle_health.rs:253`, message `no output device to qualify against` | Obtain a host with real audio hardware; a container or VM without a device cannot satisfy this row (`docs/06-plans/current-milestone.md:116`) | None — record as `INCONCLUSIVE` |
| E-2 | Enumeration fails | `BackendError::EnumerationFailed(String)` (`crates/spectre-audio/src/lib.rs:105`), surfaced through `.expect("enumeration must succeed")` at `lifecycle_health.rs:252` | Record the string in E10; investigate ALSA permissions and `audio` group membership | None — `INCONCLUSIVE` |
| E-3 | No default device although devices exist | `BackendError::NoDefaultDevice` (`lib.rs:98`) via `.expect("a default device is required")` at `lifecycle_health.rs:254–256` | Configure a default PCM; record what was configured in E4 | None — `INCONCLUSIVE` |
| E-4 | Device refuses the requested geometry | `BackendError::OpenFailed(String)` built at `cpal_backend.rs:144` from cpal's `BuildStreamError`, surfaced by `.expect(…)` at `lifecycle_health.rs:271` | **Do not retry with different values and record only the success.** Record the refusal as `REFUSED` with E10 verbatim, then optionally run a second, separately recorded attempt (§4.4) | None — the refusal is the finding |
| E-5 | `start()` or `stop()` fails mid-cycle | `BackendError::OpenFailed(String)` from `cpal_backend.rs:181` / `:196`, surfaced at `lifecycle_health.rs:275` / `:277` | Record as `FAIL` — the lifecycle is exactly what the drill exists to prove | None |
| E-6 | Run completes, `plan_errors > 0` or `contaminated_nodes > 0` | libtest failure from `lifecycle_health.rs:293` / `:294` | Record as `FAIL`. Under RT-003 this would mean non-finite output reached containment on this platform, which is a defect, not an environment quirk | None |
| E-7 | Run completes, libtest `ok`, but `xruns > 0` or blocks below the §4.5 floor | **No automated signal at all** — these are printed, not asserted (§4.3) | The operator applies §5.5. Record `FAIL` for xruns, `INCONCLUSIVE` for a short run | **This is the highest-risk error state in the feature**, because the tooling reports success |
| E-8 | Driver signalled stream errors during the run | **Not observable.** cpal's error callback increments an atomic (`cpal_backend.rs:138–141`) exposed only as `CpalStream::error_count` (`cpal_backend.rs:163–168`), which is inherent to the concrete type and absent from the `AudioStream` trait (`lib.rs:181–196`); the drill holds `Box<dyn AudioStream>` (`lib.rs:210–215`), so the counter is structurally unreachable from it | None available today. Recorded as a named limitation in §4.4 and routed to §8 Q3 | None, but the record must state that stream errors were **not** checked |

### 3.7 Accessibility

- **Screen reader:** the drill's output is plain ASCII `key=value` text
  (`lifecycle_health.rs:257`, `:281–288`) with no colour, no cursor addressing, and no
  box-drawing characters. It reads correctly through a screen reader as written. The
  protocol requires the recorded row to keep that property: no emoji status glyphs, no
  colour-only outcome marking. The outcome is carried by a word (`PASS` / `FAIL` /
  `INCONCLUSIVE` / `REFUSED`), which satisfies "color-independent state communication"
  by construction.
- **Custom actions:** N/A — no interactive element exists.
- **Text scaling / dynamic type:** N/A — the surfaces are terminal text and Markdown, both
  of which scale with the reader's own environment settings.
- **Focus order and keyboard navigability:** N/A — no focusable UI is introduced.
- **Trajectory (decision 17, criteria 3F):** this feature adds no surface, so it forecloses
  nothing about the beta accessibility gate. The one obligation it does carry is that
  `hw:CARD=…`-style device keys are recorded verbatim rather than prettified, because a
  future device picker will need to display the same identity the backend uses
  (`cpal_backend.rs:17–22`), and a record that paraphrased it would have destroyed the
  only evidence of what that identity looks like on ALSA.

---

## 4. Implementation Specification

### 4.1 Architecture placement

This feature adds **no code**. Its artifacts are documents:

- `docs/05-quality/device-qualification-protocol.md` — new; the durable protocol. Placed
  in `05-quality/` rather than in the milestone document because the milestone document is
  superseded at each milestone boundary (`docs/06-plans/current-milestone.md:16` records
  that it already superseded the R3 milestone), and a protocol that is superseded along
  with its milestone would have to be rediscovered at R5's crash qualification and again at
  R4-9's manual QA. `docs/README.md:50` permits creating `05-quality/` when it holds a
  grounded contract. Placement is routed to §8 Q7 as a proposal, not asserted as settled.
- `docs/06-plans/current-milestone.md` — modified; holds the **record**, per
  `docs/status/NEXT.md:25` which names the milestone's qualification table as the
  destination.
- `docs/01-requirements/requirements-ledger.md` — modified; holds the numeric-bound
  rationale rows (§4.5).

The code under test is unchanged and is named precisely so the review can check it:
`crates/spectre-audio/tests/lifecycle_health.rs:242–295` (the ignored drill),
`crates/spectre-audio/src/cpal_backend.rs` (the only file where cpal types appear),
`crates/spectre-audio/src/bridge.rs` (the telemetry the drill prints).

**The drill is not modified by this spec, and that is a load-bearing decision.** Decision
23 states the debt is discharged by running the drill (`decision-gates.md:49`;
`current-milestone.md:131–133`). If the drill changes before the Linux run, the Linux row
and the macOS row describe two different tests, and the pair no longer supports the
co-first-class comparison decision 1 requires. Any enrichment of the drill (§8 Q2, Q3)
therefore also requires a macOS re-run, and is deliberately deferred until after the Linux
row exists.

### 4.2 Data model

No Rust types are added or modified. The data model of this feature is the record schema,
which is defined once in §3.3 (table columns + fields E1–E10) and is the same schema in
both the durable protocol document and the milestone record.

Two existing types define what the schema can contain, and the schema does not exceed
them:

- `BridgeTelemetry` (`crates/spectre-audio/src/bridge.rs:24–38`) holds eleven counters:
  `blocks_rendered`, `plan_errors`, `frame_capacity_rejections`, `notes_deferred`,
  `parameters_pending`, `contaminated_nodes`, `denormals_flushed`,
  `last_contaminated_node`, `xruns`, `last_headroom_bits`, `worst_headroom_bits`, read
  through accessors at `:63`, `:68`, `:73`, `:78`, `:83`, `:88`, `:93`, `:98`, `:103`,
  `:108`, `:113`. The drill prints **five** of them (`lifecycle_health.rs:281–288`):
  `blocks_rendered`, `xruns`, `worst_headroom`, `plan_errors`, `contaminated_nodes`. The
  record therefore has five telemetry cells, exactly matching the table's existing five
  numeric columns. The other six are not in the record because the drill does not print
  them; adding them would require modifying the drill (§8 Q3).
- `StreamConfig` (`crates/spectre-audio/src/lib.rs:56–93`) carries `sample_rate`,
  `channels`, `buffer_frames` — the **requested** geometry only. There is no granted-config
  accessor anywhere in the seam: `AudioStream::config()` (`lib.rs:194–195`) returns the
  `StreamConfig` the stream was *opened with*, which `CpalStream` stores unchanged at
  `cpal_backend.rs:147`. Field E8 records "granted: not recorded by the drill" because
  that is the truth (§8 Q2).

No migrations. No schema versioning: the milestone table is prose-owned, and the durable
protocol document carries the metadata block `docs/README.md:64–66` requires.

### 4.3 API contracts

No function signatures change. The contract this feature defines is the **evaluation
contract** between what the tooling asserts and what qualification requires — and they are
not the same set. This is the most important table in the spec.

`hardware_lifecycle_drill` (`crates/spectre-audio/tests/lifecycle_health.rs:245–295`)
contains exactly five assertions and two `println!`s:

| Printed / asserted value | Source line | Asserted by the drill? | Role in qualification |
|---|---|---|---|
| `devices` non-empty | `lifecycle_health.rs:253` | **Yes** — `assert!(!devices.is_empty(), …)` | Precondition. Failure ⇒ `INCONCLUSIVE` |
| `backend=` / `device=` | `lifecycle_health.rs:257` (print) | No | **Recorded**, fields E6 and table column 3 |
| `blocks=` (`blocks_rendered`) | printed `:283`; asserted `> 0` at `:289–292` | **Yes, but only `> 0`** | Threshold, and the drill's threshold is far too weak — §4.5 raises it in the protocol |
| `xruns=` | printed `:284` | **No** | **Threshold in the protocol, unasserted in the drill.** §4.5 |
| `worst_headroom=` | printed `:285` | **No** | **Observation only. No threshold.** §4.5 |
| `plan_errors=` | printed `:286`; asserted `== 0` at `:293` | **Yes** | Threshold, already enforced |
| `contaminated=` | printed `:287`; asserted `== 0` at `:294` | **Yes** | Threshold, already enforced (RT-003) |
| `start()` / `stop()` / `close()` succeed | `lifecycle_health.rs:275`, `:277`, `:279` via `.expect` | **Yes** | Threshold — this is the lifecycle claim itself |
| cpal stream-error count | not printed, not reachable (§3.6 E-8) | No | **Not measured.** Must be stated as unmeasured in E10 |

**The consequence, stated plainly:** a Linux run can print `xruns=41` and libtest will
report `ok`. `cargo test` exiting 0 is a **necessary but not sufficient** condition for
`PASS`. The operator evaluates §5.5's rules against the printed line. Any protocol that
delegated the verdict to the exit code would silently accept a host that cannot meet its
callback budget.

**Auth / permissions.** The only permission surface is host-level: the operator's user must
be able to open the ALSA device (typically `audio` group membership or a seat-based
session). A permission failure surfaces as E-2 or E-4 and is recorded, not worked around
with elevated privileges — a root-only result would not describe how a user runs Spectre.
Recorded in field E4 if relevant.

**Pagination / rate limiting.** N/A — no service, no request stream.

### 4.4 State management

**Who owns the state.** The qualification record is owned by
`docs/06-plans/current-milestone.md`, whose decision authority is Jeff. No runtime state
container is introduced: `BridgeTelemetry` already owns the counters
(`bridge.rs:24–38`, constructed at `:46–54`) and the drill already holds the
`Arc<BridgeTelemetry>` handle it obtains at `lifecycle_health.rs:261–262` via
`RenderBridge::telemetry()` (`bridge.rs:152`).

**Local vs. synced.** N/A — nothing is server-synced; there is no server anywhere in
Spectre and none is proposed (Lens 3 non-goals).

**Draft / offline persistence.** The record is a committed Markdown file; the working
persistence strategy is git. A run's transcript should be pasted into the commit message or
the environment record rather than kept only in scrollback.

**Binding rules on the recorded state.** These are the rules that make the record trustable:

1. **A run is recorded whatever its outcome.** `FAIL`, `INCONCLUSIVE`, and `REFUSED` rows
   are written and kept. Deleting a failed run and recording only a later success would
   make the table a summary of successes rather than a record of evidence, which
   `docs/status/STATUS.md`'s own header prohibition on optimistic language rules out.
2. **A retry with different parameters is a new row, never an edit of the old one.** If a
   `REFUSED` run is followed by an attempt at a different geometry, both rows exist and the
   second states what changed. This is the specific failure mode E-4 warns about.
3. **The drill is not an audibility test, and hearing is never evidence.** The drill
   constructs its control channel at `lifecycle_health.rs:260` and never sends a note
   through it, so `PulseInstrument` has no active note and emits `0.0` for every frame
   (`crates/spectre-dsp/src/source.rs:202–209`, the `else { 0.0 }` branch). `Gain`
   multiplies zero by a finite factor, and `Saturator` maps a zero dry sample to
   `0 + (tanh(0)/normalization − 0) × mix = 0`
   (`crates/spectre-dsp/src/effect.rs:119–128`). The expected acoustic result is silence.
   An operator who treats silence as a failure will discard a valid run; one who expects
   sound will report a false problem.
4. **The measured cost is the not-sounding cost.** Because `active_note` is `None`, the
   instrument's per-frame work skips `self.sample()` and the phase update
   (`source.rs:202–206`). The headroom the drill reports therefore describes a chain
   rendering silence, not a chain rendering a voice. The macOS row's 0.990 carries the same
   caveat and the record does not currently say so. The Linux environment record must say
   so (§4.6 "What the run does not prove", item 5).

### 4.5 Dependencies

**No new Rust packages, no new assets, no infrastructure.** `spectre-audio` already depends
on `cpal = "0.15.3"` behind the default-on `cpal-backend` feature
(`crates/spectre-audio/Cargo.toml:12–18`).

**Host dependencies (Linux):** ALSA development headers to build (`libasound2-dev`, already
provisioned in CI at `.github/workflows/ci.yml:27–32`) and a real ALSA playback device to
run. `aplay -l`, `uname`, `/proc/asound/version`, and `/etc/os-release` are used only to
populate the environment record; all four are present on a stock Linux install.

**The feature gate is a live trap and the protocol must guard it.** The drill carries three
attributes at `lifecycle_health.rs:245–247`: `#[test]`, `#[ignore = …]`, and
`#[cfg(feature = "cpal-backend")]`. If the feature is off, the drill **does not exist** and
`cargo test … -- --ignored` reports zero tests run and exits 0. The default feature set
turns it on (`Cargo.toml:14–15`), so the plain command is correct — but field E7 records
the features in effect precisely so a `--no-default-features` invocation cannot be
mistaken for a passing run. A "0 tests" transcript is `INCONCLUSIVE`, never `PASS`.

**New numeric bounds, each requiring its own rationale row** (`docs/01-requirements/decision-gates.md:40`
row 16, standing rule; PROD-003 at `requirements-ledger.md:64`). These are proposed as new
ledger rows in a new `QUAL` family and are scheduled in §7.2. The family name itself is
routed to §8 Q6.

- **`QUAL-002` — a qualifying run MUST deliver at least 90 driver callbacks.**
  *Rationale, derived entirely from Spectre's own constants.* The drill runs two
  start/sleep/stop cycles of 250 ms each (`lifecycle_health.rs:274–278`), so 500 ms of
  nominal running time. The requested geometry is 48 000 Hz and 256 frames
  (`lifecycle_health.rs:22–23`, `:264`), giving a block period of
  256 / 48 000 = 5.333 ms and predicting 500 / 5.333 ≈ 93.75 callbacks. The drill's
  structure admits exactly four transition points at which a partial block can be lost —
  two `start()` and two `stop()` calls — so the floor is the prediction less four blocks:
  ⌊93.75⌋ − 4 = **90**. Nothing here is borrowed from a reference product; it is arithmetic
  over this repository's own values.
  *Why the drill's own `> 0` is insufficient:* one callback satisfies `blocks_rendered > 0`
  (`lifecycle_health.rs:289–292`), and "0 xruns over 1 callback" is not a claim about
  anything. The only honest content of "0 xruns over N callbacks" is "no xrun was observed
  in N observations" — the bound is 1/N with no confidence attached, so N must be stated
  and must be large enough that the observation is not trivially satisfiable.
  *If the granted geometry differs from the requested geometry,* the floor is recomputed as
  ⌊0.5 s × granted_rate ÷ granted_frames⌋ − 4 and both the recomputation and its inputs are
  written into E8. A run below the floor is `INCONCLUSIVE`, not `FAIL`: a short run failed
  to gather evidence rather than gathering adverse evidence.

- **`QUAL-003` — a qualifying run MUST report `xruns == 0`.**
  *Rationale.* An xrun is not a tunable tolerance in Spectre; it is a definition. The
  bridge counts an xrun when fractional headroom is `<= 0.0`
  (`crates/spectre-audio/src/bridge.rs:222`), i.e. when a block's render consumed the
  entire time budget for the audio it produced (`bridge.rs:211–217`). A nonzero count on a
  three-node chain rendering silence would mean the render path does not fit inside the
  callback on that host, which is exactly the property the qualification exists to
  establish. Zero is therefore the only value consistent with the claim; any other
  threshold would be inventing tolerance for a defect. This is stricter than the drill,
  which prints `xruns` and asserts nothing about it (§4.3).

- **No headroom threshold is proposed, and this is deliberate.** Spectre possesses exactly
  one worst-case headroom measurement — 0.990, macOS, one device, one fixture, silence
  (`docs/06-plans/current-milestone.md:113`, §4.4 rule 4). One measurement cannot establish
  a bound, and PROD-003 forbids asserting one without rationale. Adjacently, AF-5 prohibits
  a monitoring-latency threshold at the current evidence level; headroom is not monitoring
  latency, but the neighbourhood counsels restraint rather than a guessed number.
  `worst_headroom` is therefore **recorded as an observation with no pass condition**. The
  thresholded form of the same signal already exists and is `QUAL-003`, because headroom
  `<= 0` *is* an xrun by construction (`bridge.rs:222`). Proposing a headroom floor later
  is §8 Q5.

- **`QUAL-001`** is normative rather than numeric — see §7.2 for its text.

### 4.6 Platform-specific considerations

**Decision 20 and the ALSA/JACK question.** Decision 20
(`docs/01-requirements/decision-gates.md:44`, **Accepted**) makes ALSA the qualification
baseline "because it is present on every Linux host and PipeWire exposes an ALSA
compatibility layer," and keeps JACK "available behind a cargo feature for pro routing"
that "never becomes a hard dependency."

**Does JACK need its own row? No — and not because it is unimportant.** Two reasons, in
order:

1. Decision 23's debt is specified against the ALSA baseline; the Linux row in the table
   already reads `cpal / ALSA (decision 20 baseline)`
   (`docs/06-plans/current-milestone.md:114`). Qualifying JACK would not discharge it.
2. **JACK is not reachable from this workspace today.** `crates/spectre-audio/Cargo.toml`
   declares exactly two features — `default = ["cpal-backend"]` and
   `cpal-backend = ["dep:cpal"]` (`:12–15`) — and `cpal = { version = "0.15.3", optional =
   true }` (`:18`) with no feature list, so no Spectre feature enables cpal's JACK host. A
   JACK row cannot be produced without first adding that feature, which is a separate
   change with its own scope.

If a JACK row is ever added it is an **additional** row and never a substitute for the ALSA
row, because decision 20 makes ALSA the baseline that must hold on every Linux host while
JACK is optional. Routed to §8 Q4.

**PipeWire / PulseAudio mediation.** This is the single most consequential field in the
record. Decision 20's own justification cites PipeWire's ALSA compatibility layer, which
means a run may traverse ALSA's API without ever touching ALSA hardware directly. A result
obtained through PipeWire's compatibility layer supports the claim "Spectre opens and
drives a stream on a PipeWire desktop"; it does not support "cpal's ALSA backend drives
hardware." Those are different claims and the table's columns cannot tell them apart, so
field E4 carries the distinction and the row's footnote states which claim the run
supports. The cpal device key printed at `lifecycle_health.rs:257` is corroborating
evidence: a key of `pipewire` or `pulse` indicates mediation, a key of `hw:CARD=…`
indicates a direct hardware PCM, and `default` indicates whatever the host's ALSA
configuration routes it to — which is why E4 also records *how* the determination was made
rather than only its conclusion. **Recommendation, routed to §8 Q4:** if both are
available, run raw ALSA first, because it is the stronger claim and it is the one decision
20 names.

**The two ALSA risks R4-1 §4.6 flags, and what each records.** R4-1 names them as live and
unknowable without this hardware; this spec converts each into a defined record rather than
a blocked run:

- **`BufferSize::Fixed(256)` may be refused.** `cpal_backend.rs:126–130` always builds
  `cpal::StreamConfig { buffer_size: cpal::BufferSize::Fixed(config.buffer_frames as u32),
  … }`; there is no `BufferSize::Default` path anywhere in the file. A refusal surfaces as
  `BackendError::OpenFailed` from `cpal_backend.rs:144` and reaches the operator as a panic
  at `lifecycle_health.rs:271`. Outcome `REFUSED`, E10 verbatim. **That is a genuine
  finding about cpal-on-ALSA and is worth more than a green row**, because it would tell
  R4-1 that the seam needs a fallback before the alpha can produce sound on Linux.
- **The device may default to 44 100 Hz.** The drill requests 48 000 explicitly
  (`lifecycle_health.rs:264`), which `StreamConfig::stereo` validates against
  `MIN_SAMPLE_RATE`/`MAX_SAMPLE_RATE` = 8 000 / 768 000 (`lib.rs:25–26`, `:76–87`) and then
  hands to cpal as `cpal::SampleRate(48_000)` (`cpal_backend.rs:128`). If the device cannot
  run at 48 000, the outcome is the same `REFUSED` branch — not a silent resample. Record
  the device's supported rates from `aplay -l`/`/proc/asound` in E5.

**An unresolved arithmetic inconsistency in the existing macOS row, stated because the
protocol has to handle it.** The macOS row records 173 blocks
(`docs/06-plans/current-milestone.md:113`) for the same drill, whose running window is
500 ms (`lifecycle_health.rs:274–278`) at a requested 256 frames / 48 000 Hz
(`:22–23`, `:264`). 500 ms at 256 frames predicts ≈ 94 callbacks; 173 is ≈ 1.85× that, and
is closer to what a 128-frame granted buffer would predict (≈ 188). The record contains no
field that can resolve this, because nothing in the seam reports the granted geometry
(§4.2). This spec does **not** assert an explanation. It draws two consequences: (a) field
E8 exists and must be filled as completely as the host allows; (b) `QUAL-002`'s floor is
defined as recomputable from granted geometry rather than fixed at 90. Routed to §8 Q1.

**CI.** `.github/workflows/ci.yml` runs a single `ubuntu-latest` job (`:23`) whose own
header comment already states it is "a deliberate minimum, not a platform coverage claim"
(`:7`). It installs `libasound2-dev` (`:27–32`) — which is what makes the Linux **build**
result real — and runs `cargo test --locked --workspace` (`:47`), which does not include
ignored tests. **This protocol must not be added to CI.** A hosted runner has no audio
device, so the drill would fail at `lifecycle_health.rs:253`; and a green CI job that
somehow appeared to pass would be exactly the false claim decision 23 exists to prevent.
Qualification is manual by design.

**macOS.** Unchanged by this spec. The macOS row stays as written. If §8 Q2/Q3 lead to an
enriched drill, macOS must be re-run under the enriched drill before the two rows are
compared again (§4.1).

**Windows.** Out of scope. Decision 1 places Windows at beta
(`docs/01-requirements/decision-gates.md:25`) and no Windows row is proposed.

**Feature flags / gradual rollout:** N/A — no product code and therefore no rollout. The
only flag in play is `cpal-backend`, covered in §4.5.

### 4.7 Performance budget

- **Memory:** no change. This spec adds no code and no allocation. The drill's own
  footprint is unchanged and is dominated by the plan's channel pool and the bridge's
  256-event note scratch (`bridge.rs:19`), neither of which this spec touches.
- **CPU / render time:** no change to the render path. The measurement the drill takes is
  already implemented: `publish_headroom` computes `1.0 − spent/budget` and stores it in an
  atomic (`bridge.rs:211–231`), using `Instant::now` which the milestone record documents
  as callback-safe via the vDSO/commpage (`docs/06-plans/current-milestone.md:67`).
- **Operator wall-clock cost:** roughly 1 s of drill run time (two 250 ms sleeps plus
  enumeration and open), plus the workspace gate (§5.2), plus the environment record. The
  gate dominates.
- **Network payload:** N/A — no network I/O anywhere in this feature.
- **Storage:** a few dozen lines of Markdown per run. No binary artifact is produced or
  stored.
- **Startup time:** N/A — no product startup path is touched.

---

## 5. Test Specification

**Command honesty statement, stated before any command appears.** Of the commands below,
§5.2's workspace gate and §5.3's deterministic-half command run today on this macOS
workstation and on CI. **§5.1's command cannot produce a qualifying result today**: it
requires a Linux host with a real ALSA output device, and no such host is available to this
spec (§7.4). The command itself is real, is exactly the one decision 23 names
(`docs/06-plans/current-milestone.md:131–133`), and is unmodified — but this spec must not
be read as reporting that it has been run. It has not.

### 5.1 Unit tests

**The qualifying command.** Copied verbatim from `docs/06-plans/current-milestone.md:132`
and `docs/status/NEXT.md:25`:

```sh
cargo test -p spectre-audio --test lifecycle_health -- --ignored --nocapture
```

Three properties of this command are load-bearing and must not be altered:

- `--ignored` runs **only** ignored tests, so this invocation runs the drill alone and
  skips the six deterministic tests in the same file (`lifecycle_health.rs:68`, `:108`,
  `:133`, `:165`, `:207`, `:226`). §5.3 runs those separately and both results are
  recorded.
- `--nocapture` is mandatory, not stylistic: `blocks`, `xruns`, `worst_headroom`,
  `plan_errors`, and `contaminated` reach the operator **only** through
  `println!` at `lifecycle_health.rs:281–288`. Without it, libtest swallows the output on
  success and the run produces a verdict with no record.
- The default feature set must be in effect so that `#[cfg(feature = "cpal-backend")]`
  at `lifecycle_health.rs:247` compiles the drill in at all (§4.5).

**Assertions that fail if the behavior regresses** — these exist in the drill today and are
listed with the exact line that would fail:

| # | Assertion | Line | What its failure means |
|---|---|---|---|
| T-1 | at least one output device enumerates | `:253` | precondition unmet ⇒ `INCONCLUSIVE` |
| T-2 | a default output device exists | `:254–256` | precondition unmet ⇒ `INCONCLUSIVE` |
| T-3 | `open_output` on the default device succeeds at 48 000 Hz / 256 frames / 2 ch | `:265–271` | geometry or device refusal ⇒ `REFUSED` |
| T-4 | `start()` succeeds, twice | `:275` | the lifecycle claim fails ⇒ `FAIL` |
| T-5 | `stop()` succeeds, twice | `:277` | the lifecycle claim fails ⇒ `FAIL` |
| T-6 | `close()` succeeds | `:279` | device release fails ⇒ `FAIL` |
| T-7 | `blocks_rendered > 0` | `:289–292` | the driver never called back ⇒ `FAIL` |
| T-8 | `plan_errors == 0` | `:293` | a block failed to render ⇒ `FAIL` |
| T-9 | `contaminated_nodes == 0` | `:294` | non-finite output was contained on this platform ⇒ `FAIL` (RT-003) |

**Operator-evaluated conditions with no automated assertion** — these are the additions the
protocol makes, and each would be invisible to `cargo test`:

| # | Condition | Source of the value | Outcome if unmet |
|---|---|---|---|
| T-10 | `xruns == 0` (`QUAL-003`) | printed at `:284` | `FAIL` |
| T-11 | `blocks ≥ 90`, or ≥ the geometry-adjusted floor (`QUAL-002`) | printed at `:283` | `INCONCLUSIVE` |
| T-12 | environment fields E1–E10 all populated (`QUAL-001`) | host commands, §3.3 | `INCONCLUSIVE` — an unreproducible run is not evidence |
| T-13 | the transcript reports exactly `1 passed` for the ignored filter, not `0 passed` | libtest summary line | `INCONCLUSIVE` — the feature was off (§4.5) |

**No new test file is created.** A test that could not fail would score zero under criterion
1G; the honest position is that T-10 through T-13 *cannot* be automated without modifying
the drill, which §4.1 forbids before the Linux run. That constraint is stated rather than
papered over with a test that asserts nothing.

### 5.2 Integration tests

The run must be against a green tree, so the workspace gate is run first, in full,
unchanged, on the Linux host, and its result recorded in field E9:

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

These are exactly the commands `docs/status/STATUS.md` §Validation and
`.github/workflows/ci.yml:41–47` name. The expected shape of the third result on Linux is
the 2026-08-09 container result — all non-hardware `spectre-audio` tests passing with the
drill reported as ignored (`docs/06-plans/current-milestone.md:116`) — and any deviation is
itself a finding recorded in E9 before the drill is run at all.

### 5.3 UI / E2E tests

There is no UI, so the E2E analogue is the **deterministic half of the same file**, run on
the same Linux host, immediately before the drill:

```sh
cargo test -p spectre-audio --test lifecycle_health
```

This runs the six non-ignored tests (`lifecycle_health.rs:68`, `:108`, `:133`, `:165`,
`:207`, `:226`) against `NullBackend`, proving the lifecycle state machine, sample-rate
reopen, device-loss reporting, off-thread headroom publication, xrun counting, and
containment telemetry on this host with no hardware involved. Recording both results
separates "the state machine is sound on this host" from "the driver behaves" — which is
precisely the distinction the file's own header comment draws
(`lifecycle_health.rs:4–7`). This command runs today on any host, Linux included.

### 5.4 Visual / manual verification

- **Theme variants:** N/A — no rendered UI. The transcript inherits the operator's terminal
  theme and carries no colour-dependent meaning (§3.7).
- **Text size extremes / screen size extremes:** N/A for the same reason; the transcript is
  two lines under 80 columns and the record is Markdown.
- **Empty vs. populated states:** both are specified and both must be checked in the diff —
  the empty state is the current `| Linux | not run | … | — | … |` row
  (`docs/06-plans/current-milestone.md:114`) and the populated state is §3.3's format.
- **Manual checks the operator performs and records:**
  1. The transcript's `device=` value matches field E6 character for character.
  2. The libtest summary reports `1 passed`, not `0 passed` (T-13).
  3. Every numeric cell in the row appears verbatim in the transcript.
  4. The row's outcome word matches §5.5's evaluation, independently of libtest's verdict.
  5. **Do not** use audibility as a check — the drill renders silence by construction
     (§4.4 rule 3).

### 5.5 Outcome evaluation rules

*(Added beyond the template's four subsections because the record format is this feature's
deliverable and the outcome vocabulary has nowhere else to live.)*

| Outcome | Definition | Effect on decision 23 |
|---|---|---|
| `PASS` | T-1…T-9 pass **and** T-10…T-13 hold | Decision 23's debt is discharged for the recorded configuration only, subject to §5.6 |
| `FAIL` | T-4…T-9 or T-10 not met | Debt **not** discharged. The row is written and kept; the finding is a defect to route |
| `REFUSED` | T-3 not met — the driver declined the requested geometry | Debt **not** discharged. The row is written and kept; this is a finding about the seam, not about the host (§4.6) |
| `INCONCLUSIVE` | T-1, T-2, T-11, T-12, or T-13 not met | Debt **not** discharged. Nothing adverse was learned about the backend either |

### 5.6 What a PASS authorizes, and what it does not

A `PASS` authorizes exactly one sentence, of this shape and no broader:

> On {date}, on {distro} {kernel} with ALSA {version} {mediated by … | on raw ALSA},
> Spectre's cpal backend opened {device key}, ran {blocks} driver callbacks across two
> start/stop cycles with 0 xruns, 0 plan errors, and 0 contaminated nodes, and released the
> device cleanly.

**It does not authorize** — and the row's footnote states each of these:

1. **"Spectre supports Linux."** One machine is not a platform. Decision 23's own wording
   authorizes no support claim beyond what the drill establishes.
2. **Any other device.** One device was opened, identified by a cpal key that is just
   `Device::name()` (`cpal_backend.rs:17–22`); cpal reports no stable device identity, as
   that function's own comment says.
3. **Any other geometry.** One sample rate and one buffer size were requested
   (`lifecycle_health.rs:264`), and the granted geometry is not even recorded (§4.2).
4. **Any other kernel, distribution, or sound-server configuration.** E1–E4 exist precisely
   so the row cannot be read as covering configurations it did not touch.
5. **Any statement about audible output or about DSP cost under load.** The drill renders
   silence and the instrument takes its cheap branch (§4.4 rules 3 and 4), so the headroom
   figure describes an idle chain.
6. **Any statement about driver error behavior.** The cpal error-callback counter is
   structurally unreachable from the drill (§3.6 E-8), so a run with driver errors and a
   run without are indistinguishable in this record.
7. **Long-run stability.** The window is 500 ms of running time. Nothing about xruns over
   minutes or hours is measured.
8. **Device hot-plug or default-device change on a live stream.** The deterministic half
   models device loss by closing the stream (`lifecycle_health.rs:146–148`); no real device
   was removed underneath a running ALSA stream.
9. **Anything about `./spectre`.** The drill runs from a test binary. `./spectre` does not
   use `spectre-audio` (`docs/status/STATUS.md` repository state) and R4-1 is the slice that
   changes that.

---

## 6. Compliance & Safety Gate

### 6.1 Sensitive data classification

- [x] **No sensitive data involvement.**

The record contains host configuration facts — distribution, kernel release, ALSA version,
audio device model, and a commit SHA. None is personal or sensitive. One caution is worth
stating because it is easy to trip: `aplay -l` and cpal device keys can embed a machine's
hostname or a user-assigned device label, so field E5/E6 transcription should not
inadvertently commit a personal identifier into an accepted document. If a device key
contains one, record it verbatim in the working notes and note the redaction explicitly in
E6 rather than silently altering the key.

### 6.2 Asset provenance

- [x] **No third-party assets.**

No models, images, fonts, samples, or data files are added. The only third-party component
involved is the already-adopted `cpal 0.15.3` dependency
(`crates/spectre-audio/Cargo.toml:18`), whose adoption is decision 19
(`docs/01-requirements/decision-gates.md:43`, which records the dual MIT/Apache licence
match), and this spec neither adds nor changes it.

### 6.3 Language / claims audit

- [ ] Makes claims not supported by evidence — **no.** Every claim about current state
  cites a file and line, and the one claim that would be most tempting to make (that Linux
  works) is explicitly refused in the header, in the standing constraint, in §1.2, §4.6,
  §5.6, and §7.1.
- [ ] Promises capabilities not yet built — **no.** The protocol describes a run that has
  not happened and says so at every point where it could be misread, including a dedicated
  honesty statement opening §5.
- [ ] Uses language restricted by domain regulations — **no.** No regulated domain applies.

Two specific self-checks against AF-6: the words that would signal optimism here are
"should pass", "expected to pass", and "straightforward" — none is used about the Linux
outcome, and §5.5 gives failure outcomes equal standing with `PASS`. The word "qualified"
is used only about macOS, which has a dated record, and never about Linux.

### 6.4 Regulatory alignment

Against `gauntlet-output/criteria.md` Lens 3:

- **3A — Milestone fit.** R4's exit evidence list includes "Linux device qualification runs,
  discharging decision 23's debt" (`docs/06-plans/current-milestone.md:88`), and
  `docs/status/NEXT.md:25` is slice 3. This spec produces exactly that and nothing beyond
  it; every enrichment idea is deferred to §8 rather than smuggled in.
- **3B — Non-goal respect.** No CLAP/LV2/AU hosting, no plugin-format authoring, no
  cross-DAW compatibility, no cloud service, no content store, no video scoring. The
  feature adds no product surface at all, so it cannot propose one.
- **3C — Deliberately small first devices.** The fixture is the existing Pulse → Gain →
  Saturator chain built at `lifecycle_health.rs:27–66`, unchanged. No device grows.
- **3D — Originality.** The protocol, the outcome vocabulary, the environment field set, and
  both numeric bounds are derived from this repository's own constants and its own prior
  measurement. Nothing is transcribed from a reference product — and §Appendix A records
  that no reference product in the corpus documents anything of the kind.
- **3E — Platform commitment.** This is the spec whose entire purpose is decision 1's
  undischarged Linux commitment. It names the Linux path explicitly, refuses every Linux
  claim, and defines the exact evidence that would close the gap.
- **3F — Accessibility trajectory.** No surface is added, so nothing is foreclosed; §3.7
  states the one positive obligation (verbatim device keys, word-carried outcome) that
  keeps a future device picker and a future screen-reader label honest.

---

## 7. Gap Analysis vs. Current State

### 7.1 What exists today

Every statement below was checked by opening the file.

**Implemented.**

- `crates/spectre-audio/tests/lifecycle_health.rs` exists, 295 lines. Six deterministic
  tests run against `NullBackend` at `:68`, `:108`, `:133`, `:165`, `:207`, `:226`. Its
  header comment (`:1–7`) states the drill "must be run explicitly on macOS and on Linux;
  until it has, the exit row stays open and is documented as open."
- `hardware_lifecycle_drill` exists at `:248`, guarded by three attributes at `:245–247`:
  `#[test]`, `#[ignore = "requires a real audio device; run on macOS and Linux to close the
  slice 7 exit row"]`, and `#[cfg(feature = "cpal-backend")]`. Its body spans `:248–295`:
  enumeration `:252–253`, default device `:254–256`, identity print `:257`, fixture and
  bridge `:259–262`, config `:264`, open `:265–271`, two start/sleep(250 ms)/stop cycles
  `:274–278`, close `:279`, telemetry print `:281–288`, three assertions `:289–294`.
- `CpalBackend` (`crates/spectre-audio/src/cpal_backend.rs:34`) implements `AudioBackend`
  at `:68`. `open_output` (`:115–151`) validates the config, resolves the device by key,
  builds `cpal::StreamConfig` with `BufferSize::Fixed(config.buffer_frames as u32)`
  (`:126–130`), and maps a build failure to `BackendError::OpenFailed(String)` (`:144`).
  Device identity is `cpal::Device::name()` (`:17–22`), with the file's own comment stating
  "cpal reports no stable device identity, so the reported name is the key."
- The cpal error callback increments an `AtomicU64` and does nothing else (`:138–141`),
  exposed as `CpalStream::error_count` (`:163–168`) — an inherent method on the concrete
  type, absent from the `AudioStream` trait (`crates/spectre-audio/src/lib.rs:181–196`).
- `BridgeTelemetry` (`crates/spectre-audio/src/bridge.rs:24–38`) holds eleven counters with
  accessors at `:63`–`:115`. `publish_headroom` (`:211–231`) computes
  `1.0 − spent/budget` and counts an xrun when headroom is `<= 0.0` (`:222`).
- CI installs `libasound2-dev` at `.github/workflows/ci.yml:27–32` on a single
  `ubuntu-latest` job (`:23`), and its own header comment calls that "a deliberate minimum,
  not a platform coverage claim" (`:7`).

**Verified (evidence passing) — macOS only.**

- `docs/06-plans/current-milestone.md:113` records one row: macOS, 2026-08-09, cpal /
  CoreAudio, M-Audio AIR 192|6, 173 blocks, 0 xruns, worst headroom 0.990, 0 plan errors, 0
  contaminated.

**Gated / absent — Linux.**

- The Linux row is `| Linux | not run | cpal / ALSA (decision 20 baseline) | — | — | — | — |
  — |` (`docs/06-plans/current-milestone.md:114`). **No Linux audio device has ever been
  opened** (`:127`).
- What did happen on Linux on 2026-08-09 is a **build-and-link result only**: the workspace
  compiled and linked against ALSA in an aarch64 container with `libasound2-dev` and all 34
  non-hardware `spectre-audio` tests passed, and the drill failed closed on a machine with
  no device (`:116`). The same paragraph states this "is a build and portability result,
  **not** a device qualification" and that "the Linux row cannot be satisfied by a container
  or a VM without real audio."
- Decision 23 (`docs/01-requirements/decision-gates.md:49`) is **Accepted** and carries the
  debt to R4 with "no Linux support claim is authorized until the drill runs on real Linux
  hardware."
- Decision 1 (`:25`) is **Accepted** and its co-first-class commitment is currently
  undischarged (`docs/status/STATUS.md` known gaps; `current-milestone.md:127`).

**Absent — the protocol itself.**

- There is no `docs/05-quality/` directory; `docs/` contains `00-product`, `01-requirements`,
  `02-reference-research`, `03-architecture`, `06-plans`, `README.md`, and `status`. No
  document anywhere states what counts as a passing hardware qualification, what fields a
  run records, or what a `PASS` authorizes. The only pass conditions that exist are the
  three assertions at `lifecycle_health.rs:289–294`, and they omit `xruns` entirely.
- The requirements ledger has RT, TIME, CORE, GRAPH, and PROD families
  (`docs/01-requirements/requirements-ledger.md:24`, `:32`, `:42`, `:51`, `:58`). There is
  no qualification family and no row governing hardware evidence.

### 7.2 Delta to spec

**New files (2).**

1. `docs/05-quality/device-qualification-protocol.md` — the durable protocol: §3.3's field
   set, §4.3's threshold/observation split, §4.5's bounds, §5.5's outcome vocabulary, and
   §5.6's authorization limits. Carries Jeff's header and the full metadata block per
   `docs/README.md:64–66`. Status `proposed` until Jeff accepts. Placement routed to §8 Q7.
2. `docs/05-quality/README.md` — one short index page, only because `docs/README.md:39–48`
   lists a document class per directory and a bare directory with one file would break that
   map. If Jeff prefers no index, drop it; it carries no content of its own.

**Modified files (6).**

1. **`docs/01-requirements/requirements-ledger.md`** — new `QUAL — hardware qualification`
   family with three rows, each carrying its own Spectre rationale so that AF-4 and
   PROD-003 (`:64`) are satisfied by record rather than by assertion:
   - `QUAL-001` — a platform MUST NOT be recorded as device-qualified unless the drill
     completed with T-1…T-9 satisfied **and** the environment record fields E1–E10 are
     populated. *Provenance:* decision 23 (`decision-gates.md:49`), decision 1 (`:25`).
     *Acceptance evidence:* a complete row plus environment record in the milestone
     qualification table. *Rationale:* an unreproducible run is an anecdote; decision 1's
     co-first-class commitment requires evidence a second person could re-derive.
   - `QUAL-002` — a qualifying run MUST deliver at least **90** driver callbacks at the
     drill's requested geometry, or at least ⌊0.5 s × granted_rate ÷ granted_frames⌋ − 4 if
     the granted geometry differs. *Rationale as derived in §4.5*, from
     `lifecycle_health.rs:22–23`, `:264`, `:274–278` — this repository's own constants, not
     a vendor figure.
   - `QUAL-003` — a qualifying run MUST report `xruns == 0`. *Rationale as derived in §4.5*,
     from the xrun definition at `bridge.rs:222`.
   The row explaining why **no** headroom threshold is proposed belongs in the protocol
   document rather than the ledger, since the ledger records limits that exist.
2. **`docs/06-plans/current-milestone.md`** — fill the Linux row at `:114` per §3.3; add the
   `[L1]` environment record block below the table at `:115`; add a one-line pointer from
   §"Inherited debt" item 1 (`:24`) to the protocol document. Column headers at `:111–112`
   are **not** changed (§3.3).
3. **`docs/01-requirements/decision-gates.md`** — row 23 (`:49`) gains a dated disposition
   **only on `PASS`**, citing the milestone row. On `FAIL`, `REFUSED`, or `INCONCLUSIVE`
   the row stays open and gains a dated note naming the outcome and the finding. Decision 1
   (`:25`) is **not** edited by this spec.
4. **`docs/status/STATUS.md`** — the known-gaps line "no Linux audio device has ever been
   opened" and the §Validation paragraph are updated to match whichever outcome occurred.
   On a non-`PASS` outcome the gap line stays and gains the new finding.
5. **`docs/status/NEXT.md`** — slice 3 (`:25`) closes on `PASS`, or gains its outcome and a
   restated next action otherwise. The `Open decisions` line at `:18` is updated in step.
6. **`docs/01-requirements/traceability.md`** — RT-001/RT-002/RT-003 evidence gains the
   platform qualifier, so that "guards hold under a real driver" reads as "under a real
   CoreAudio driver, and under a real ALSA driver as of {date}" rather than as an unqualified
   claim.

**Migrations / schema changes:** none.

**New dependencies:** none in Cargo. Host-side, the run needs a Linux machine with a real
ALSA playback device (§4.5, §7.4).

**Explicitly not changed:** `crates/spectre-audio/tests/lifecycle_health.rs` and every file
under `crates/`. §4.1 gives the reason: changing the drill before the Linux run breaks
comparability with the macOS row it exists to match.

### 7.3 Estimated scope

**S** — for the authoring and recording work. Two new documents, six edited documents, no
code, no tests, no dependency changes. The largest single piece of work is the ledger family
with its three rationale rows, and even that is one table.

The **S** applies to the deliverable, not to the prerequisite. Obtaining a Linux host with
real audio hardware is not sized here because it is not an engineering task; it is the
gate described in §7.4. The run itself is ~1 s plus the workspace gate.

This size is credible precisely because the protocol refuses scope: no drill enrichment
(§8 Q2, Q3), no JACK row (§8 Q4), no CI automation (§4.6), no headroom threshold (§4.5).
Each of those would move this to **M** and each is deferred with a reason.

### 7.4 Blocking dependencies

**HARDWARE-BLOCKED — this is the feature's defining constraint and it is stated first.**

Execution requires a Linux host with a **real ALSA playback device**. This spec has no such
host. The blockage is documented in the repository, not asserted here:
`docs/06-plans/current-milestone.md:116` records that the drill fails closed on a machine
with no audio device, "so the Linux row cannot be satisfied by a container or a VM without
real audio," and `:127` records that no Linux audio device has ever been opened. The gate is
physical: it needs the box.

**What is not blocked.** Everything in §7.2 except the row's contents. The protocol
document, the three ledger rows, the outcome vocabulary, the field set, and the
authorization limits are all authorable, reviewable, and acceptable now — and authoring
them *before* the run is the point, because a protocol written after seeing the numbers can
be shaped to fit them.

**Other dependencies.**

| Dependency | Status | Effect |
|---|---|---|
| A Linux host with real ALSA hardware | **Blocking, unavailable** | Blocks §5.1 only |
| The drill, unmodified | Present (`lifecycle_health.rs:248`) | None — it exists and is not changed |
| `cpal-backend` feature on | Default (`Cargo.toml:14–15`) | None, but recorded in E7 |
| Decision 20's ALSA baseline | **Accepted** (`decision-gates.md:44`) | Fixes ALSA as the row's subject; JACK excluded (§4.6) |
| R4-1 (live audio wiring) | Not landed; spec passed review at 2.950 | **Not a dependency in either direction.** The drill runs from a test binary and needs nothing from `./spectre`. R4-3 can run before, during, or after R4-1 |
| R4-1's proposed `stream_errors` seam addition | Proposed only, in R4-1 §4.3 | If it lands, §3.6 E-8's blind spot becomes closable — but closing it means modifying the drill, so §4.1 defers it to after the Linux row |
| `docs/05-quality/` directory | Absent | Created by this spec, sanctioned by `docs/README.md:50` |

**Ordering recommendation.** Run R4-3 **early** in R4, before R4-1 lands. If ALSA refuses
`BufferSize::Fixed(256)` (§4.6), R4-1's engine would be silent on Linux and the seam would
need a fallback — and it is much cheaper to learn that before the app is wired to the
backend than after.

---

## 8. Open Questions

- **Q1 — Why does the macOS row record 173 blocks?** §4.6 shows 500 ms of running at the
  requested 256 frames / 48 000 Hz predicts ≈ 94, and 173 is closer to a 128-frame granted
  buffer (≈ 188). The record cannot resolve it. Options: (a) accept the discrepancy and let
  `QUAL-002`'s geometry-adjusted floor absorb it; (b) re-run macOS with a drill that prints
  the granted geometry, which then also requires the Linux run to use that drill. Does Jeff
  want this resolved before the Linux row is written, or after? — blocks §4.5's floor
  interpretation, §4.6, field E8.
- **Q2 — Should the drill print the granted geometry?** Nothing in the seam exposes it:
  `AudioStream::config()` (`lib.rs:194–195`) returns the requested `StreamConfig` that
  `CpalStream` stored at `cpal_backend.rs:147`. Reporting the granted values would need a
  new seam method and a cpal query, which is a real change to an accepted trait — a decision
  row, not a spec assertion. It would also invalidate macOS/Linux comparability until macOS
  is re-run (§4.1). — blocks §4.2, §5.6 item 3.
- **Q3 — Should the drill assert `xruns == 0` and read the cpal error counter?** Both are
  currently operator-evaluated or invisible (§4.3, §3.6 E-8). Making them assertions would
  convert T-10 and the error-count blind spot into automated gates, at the cost of modifying
  the drill and re-running macOS. Recommendation: after the Linux row exists, not before.
  — blocks §5.1's T-10, §3.6 E-8.
- **Q4 — Raw ALSA, PipeWire, or both, and does JACK ever get a row?** §4.6 recommends raw
  ALSA first because decision 20 names it as the baseline and it is the stronger claim, and
  argues JACK needs no row to discharge decision 23 (and is not reachable today —
  `Cargo.toml:12–18` enables no JACK feature). If Jeff's only available hardware is on a
  PipeWire desktop, the row is still worth writing, with E4 carrying the narrower claim.
  Which does Jeff want, and is a second row on the other configuration wanted? — blocks
  §3.3 field E4, §4.6.
- **Q5 — Should a worst-case headroom floor ever exist?** §4.5 proposes none, because
  Spectre has one measurement and PROD-003 forbids an unrationalized bound. A second data
  point (this Linux run) would make a floor arguable but still thin, and AF-5's prohibition
  on a monitoring-latency threshold sits next door. Recommendation: leave headroom as an
  observation through R4 and revisit when the alpha renders a sounding voice under load.
  — blocks §4.5, `QUAL` family completeness.
- **Q6 — Is `QUAL` the right ledger family name, and are three rows the right granularity?**
  The ledger currently has RT, TIME, CORE, GRAPH, PROD
  (`requirements-ledger.md:24`, `:32`, `:42`, `:51`, `:58`). A new family is a small but
  permanent taxonomy decision that belongs to Jeff, and the alternative is folding these
  into RT — which reads wrong, because they govern evidence rather than realtime behavior.
  — blocks §7.2 item 1.
- **Q7 — Should the protocol live in `docs/05-quality/` or inside the milestone document?**
  §4.1 argues for `05-quality/` because milestone documents are superseded at each boundary
  and the protocol will be needed again at R4-9's manual QA and R5's crash qualification.
  The cost is creating a new document class directory, which `docs/README.md:50` permits but
  frames as something done only when grounded. — blocks §7.2 items 1 and 2.
- **Q8 — Is a research need worth opening on how other products document device
  qualification?** Appendix A records that the corpus has nothing on this surface. It is
  plausible that public sources exist (driver conformance suites, host compatibility
  matrices) that would inform `QUAL-002`'s floor better than arithmetic over one drill. It
  is equally plausible this is simply not something products publish. Worth a ledger
  research-need entry, or drop it? — blocks Appendix A's disposition.
- **Q9 — What happens to R4's exit if the Linux run is `REFUSED` or unavailable?** R4's exit
  evidence list requires "Linux device qualification runs, discharging decision 23's debt"
  (`current-milestone.md:88`). If the hardware never materializes, or if ALSA refuses the
  geometry, R4 cannot exit on that row as written. The options are a scoped narrowing
  analogous to decision 23 itself, or holding R4 open. This spec deliberately does not
  choose. — blocks R4 exit.
- **Q10 — Should the run be repeated on a second Linux machine before the row is treated as
  settled?** §5.6 item 1 says one machine is not a platform, but the protocol as written
  discharges decision 23 on one row. A two-machine rule would be stronger and slower.
  Jeff's call on how much evidence "co-first-class" actually demands. — blocks §5.5's effect
  column, `QUAL-001`.

---

## Appendix A — Benchmark evidence used, and where it does not exist

**Cited from the accepted corpus:**

- `OBS-VCV-VOLT-006` (`docs/02-reference-research/synth-modular-observations.md:50`) — the
  0-on-NaN/infinity precedent, which is RT-003's provenance in the ledger
  (`requirements-ledger.md:30`) and therefore the reason the drill's
  `contaminated_nodes == 0` assertion (`lifecycle_health.rs:294`) is a threshold rather than
  an observation. Used only for that link, not extended.
- `OBS-AB12-MIX-009` (`docs/02-reference-research/ableton-live-observations.md:106`) — a
  benchmark exposes a per-track six-step CPU meter to identify freeze candidates. This is
  the **closest adjacent record in the corpus**, and it is genuinely adjacent rather than on
  point: it establishes that a AAA product surfaces per-element performance impact to the
  user at runtime, which corroborates that Spectre's headroom telemetry is a reasonable
  thing to have. It says nothing about qualification protocols, thresholds, or records, and
  is not used to justify any number in this spec.
- `OBS-SR2-CPU-001` (`synth-modular-observations.md:54`) — one of only two citable Serum 2
  records. Cited solely to note that its content (unison voice-count CPU guidance) is
  unrelated to device qualification. No Serum 2 claim beyond `OBS-SR2-CPU-001` and
  `OBS-SR2-KB-001` is made anywhere in this spec.

**Named gaps — no citable evidence exists, and none is invented:**

- **How any benchmark product qualifies an audio device against a driver.** No `OBS-`
  record in the corpus describes a device qualification protocol, a driver conformance
  criterion, a buffer-size negotiation policy, or a recorded device test result for
  **Ableton Live 12, Logic Pro, Serum 2, Phase Plant, or VCV Rack 2**. This is unsurprising
  — it is internal engineering practice, not user-facing documented behavior, and the
  corpus is built from manuals and support articles. Recorded as a research need (§8 Q8),
  not filled in from recollection.
- **Logic Pro.** Its dossier is `inventory-only` with **zero** behavioral records
  (`gauntlet-output/criteria.md` evidence inventory). No Logic Pro claim appears anywhere in
  this spec, on this surface or any other.
- **ALSA/PipeWire behavior under cpal.** This is not a benchmark-corpus question at all; it
  is exactly what the run would establish, and §4.6 states both risks as unknown rather than
  predicted.

**Convergent patterns (criterion 2E):** none apply. Device qualification has no cross-product
convergent pattern in the corpus, because the corpus contains no instance of it. Spectre
does not diverge from a pattern here; it operates where the evidence is silent, and says so.

**Differentiation (criterion 2F):** the differentiating claim is narrow and is not a parity
claim. Spectre publishes, in a committed document, exactly which platform-and-device
configuration its realtime evidence covers, which numbers were pass conditions and which
were bystanders, and an enumerated list of what the result does **not** authorize (§5.6). No
product in the benchmark set is documented in the corpus as publishing any of that — which
is a statement about the corpus's coverage, not a claim that they do not. Spectre's release
bar of "honest telemetry, no fake surfaces" is what makes this worth doing at all, and this
spec's entire content is that bar applied to its own evidence.

**Loop-first core loop, linked lenses, modulation visibility, keyboard-first UI (criteria
2A–2D):** N/A for this feature, and the N/A is structural rather than convenient — R4-3
introduces no user-facing surface, no view, no parameter display, and no command. It cannot
lose selection, zoom, or transport context because it never touches them; it cannot fork
state per lens because it owns no state; it cannot obscure a modulation contribution because
it renders no parameter. The one obligation it does carry toward those criteria is negative
and is discharged in §3.7: record the device key verbatim so a future picker has honest data
to work from.

---

**End of spec.**
