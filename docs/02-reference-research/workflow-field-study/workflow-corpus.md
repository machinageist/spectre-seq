<!--
Author: Jeff
Date: 2026-07-11
Description: Reviewed and candidate source corpus for musician workflow field research
Notes: Index discovery is not a reviewed workflow observation
-->

# Workflow Corpus

- **Status:** draft
- **Last verified:** 2026-07-11
- **Scope:** direct sources selected for end-to-end musician-workflow observation across Geist reference products
- **Decision authority:** Jeff
- **Upstream sources:** `docs/02-reference-research/workflow-field-study/methodology.md`; `docs/02-reference-research/source-ledger.json`
- **Downstream dependents:** workflow observations, archetypes, command ontology, friction analysis, product implications
- **Supersedes:** none
- **Superseded by:** none
- **Open decisions:** scoring/long-timeline cohort; interview and usability-study plan
- **Known gaps:** four FL Studio sources have completed timestamped action-sequence
  extraction; Ableton evidence is interview self-report only; product and population
  coverage remain far below saturation targets

## Corpus states

- `discovered`: direct source identity is known.
- `candidate`: source appears substantive enough for review from its official title/series context.
- `in-review`: content is being watched/read and timestamped.
- `reviewed`: structured extraction has passed a second consistency check.
- `excluded`: source was inspected and rejected with a reason.

A title or index entry can reach only `candidate`. It cannot produce a workflow observation without content review.

## Coverage floor

| Product family | Starting floor | Current reviewed | State |
|---|---:|---:|---|
| Ableton Live | 8–12 accounts across at least three workflow categories | 2 | below floor |
| Bitwig Studio | 8–12 accounts across at least three workflow categories | 0 | below floor |
| FL Studio | 8–12 accounts across at least three workflow categories | 4 | below floor |
| REAPER | 8–12 accounts across at least three workflow categories | 0 | below floor |
| Logic Pro | 8–12 accounts across at least three workflow categories | 0 | below floor |
| Cubase | 8–12 accounts across at least three workflow categories | 0 | below floor |
| VCV Rack | 5–8 sound-design/patching accounts | 0 | below floor |
| Phase Plant | 5–8 patch-design accounts | 0 | below floor |
| Serum 2 | 5–8 patch-design accounts | 0 | below floor |

These are qualitative sampling floors, not statistical sample sizes.

## Ableton Live official-workflow candidate set

These ten direct Ableton pages were inspected as source candidates. Several provide downloadable Live Sets, but a finished Set establishes inspectable state rather than the historical order used to create it. Two are `reviewed`; the rest remain `candidate`.

| Candidate ID | Direct source | Workflow categories | Version/date evidence | Review gap |
|---|---|---|---|---|
| `WF-SRC-ABLETON-001` | `https://www.ableton.com/en/blog/input-output-galcher-lustwerk/` | Session loops; vocal recording | 2026-07-03; download requires Live 12 Suite | reviewed as `WF-ABLETON-LUSTWERK-005`; interview self-report only — no visible action sequence, template treated as finished state |
| `WF-SRC-ABLETON-002` | `https://www.ableton.com/en/blog/laura-misch-microphones-stones-saxophones/` | field/acoustic recording; arrangement | 2026-06-17; version unknown | Extract boundaries between experimentation, recording, and arrangement |
| `WF-SRC-ABLETON-003` | `https://www.ableton.com/en/blog/april-vista-making-traditional-noise/` | linear arrangement; performance recording | 2026-06-11; shared project requires Live 12 Suite | Review article and substituted-plugin project separately |
| `WF-SRC-ABLETON-004` | `https://www.ableton.com/en/blog/nathan-fake-live-reshape-repeat/` | performance-driven composition; hybrid production | 2026-05-21; version unknown | Extract only demonstrated or explicit actions; loop download is not project state |
| `WF-SRC-ABLETON-005` | `https://www.ableton.com/en/blog/download-the-live-set-of-artefakts-new-track/` | hardware; re-amping; sequencing; arrangement | 2026-05-14; Live 12/12.3 references require disambiguation | Review external-state gaps and timestamp explicit variation/modulation actions |
| `WF-SRC-ABLETON-006` | `https://www.ableton.com/en/blog/clipping-push-into-the-red/` | collaborative sound design; live monitoring | 2026-05-07; version unknown | Bound observations by task; downloadable sounds do not preserve routing |
| `WF-SRC-ABLETON-007` | `https://www.ableton.com/en/blog/circle-of-live/` | collaborative live improvisation; synchronization | 2026-04-30; version unknown | Timestamp visible performance actions; exclude Link Audio from filmed setup |
| `WF-SRC-ABLETON-008` | `https://www.ableton.com/en/blog/grand-river-symphony-for-endangered-birds/` | field recording; seven-channel spatial composition | 2026-04-22; version unknown | Establish routing evidence and distinguish editorial summary from shown action |
| `WF-SRC-ABLETON-009` | `https://www.ableton.com/en/blog/the-black-dog-rothko-to-roland-to-ableton/` | hybrid hardware; Session arrangement; automation | 2026-04-09; shared Set requires Live 12 Suite | Separate current Set evidence from historical practices and external hardware |
| `WF-SRC-ABLETON-010` | `https://www.ableton.com/en/blog/input-output-sakura-tsuruta/` | MIDI editing; automation; production | 2026-03-19; shared Set requires Live 12 Suite | reviewed as `WF-ABLETON-TSURUTA-006`; interview self-report with one concrete automation-drawing binding; shared Set is an explicit recreation |

## Bitwig Studio workflow candidate set

These ten direct Bitwig pages include artist interviews, official learning pages, and curated practitioner videos. Roundups must be split by embedded video during observation review. All remain `candidate`.

| Candidate ID | Direct source | Workflow categories | Version/date evidence | Review gap |
|---|---|---|---|---|
| `WF-SRC-BITWIG-001` | `https://www.bitwig.com/artists/electrically-alive-inside-richie-hawtins-workflow-40/` | hardware/hybrid; Grid/modulation; live | adoption around v3; page date/version unknown | Review embedded demos and controller scripts separately |
| `WF-SRC-BITWIG-002` | `https://www.bitwig.com/artists/hamilton-13/` | sound design; routing; arrangement | date/version unknown | inspected 2026-07-11 (`SRC-BITWIG-ARTIST-HAMILTON`): preference/friction statements only, no admissible action sequence; retained as corroborating evidence |
| `WF-SRC-BITWIG-003` | `https://www.bitwig.com/artists/polarity-85/` | Grid; sound design; composition; scripting | retrospective across versions | inspected 2026-07-11 (`SRC-BITWIG-ARTIST-POLARITY`): habit statements only; the five referenced tutorials are not embedded and need separate video review |
| `WF-SRC-BITWIG-004` | `https://www.bitwig.com/learnings/go-from-loop-to-arrangement-using-automation-239/` | loop-to-arrangement; automation | 2023-02-23; version unknown | Timestamp the embedded tutorial; synopsis alone is not procedural evidence |
| `WF-SRC-BITWIG-005` | `https://www.bitwig.com/learnings/stepwise-spotlight-3-video-tutorials-on-bitwigs-step-sequencer-347/` | sequencing; Grid; sound design | 2024-12-04; Bitwig 5.3 | Split and review all three practitioner videos |
| `WF-SRC-BITWIG-006` | `https://www.bitwig.com/learnings/note-grid-audio-fx-spotlight-tutorials-on-bitwig-studio-42-189/` | Grid; sound design; audio-to-note | 2022-03-23; Bitwig 4.2 | Select bounded videos; do not count roundup as one coherent workflow |
| `WF-SRC-BITWIG-007` | `https://www.bitwig.com/learnings/polarity-tip-how-to-use-a-transient-shaper-186/` | dynamics; contextual sound design | 2022-02-23; 4.2 context | Extract embedded-video actions and preserve genre limitation |
| `WF-SRC-BITWIG-008` | `https://www.bitwig.com/learnings/learn-bitwig-studio-5s-live-performance-features-245/` | Clip Launcher; live performance | 2023-04-17; Bitwig 5 | Distinguish feature overview from complete performance workflow |
| `WF-SRC-BITWIG-009` | `https://www.bitwig.com/learnings/watch-our-superbooth-demo-on-live-sets-in-bitwig-studio-249/` | live-set construction; hybrid performance | 2023-06-01; v5 context | Review lecture and concert; record off-camera setup gaps |
| `WF-SRC-BITWIG-010` | `https://www.bitwig.com/artists/trovarsi-51/` | modular/CV; hybrid live performance | date/version unknown | Timestamp embedded performance and establish hardware configuration limits |

## FL Studio workflow candidate set

The set combines official education, an official Power User chain, and independent creator demonstrations. Runtime, chapters, and descriptions support candidacy only; they are not substitutes for watching the source. All remain `candidate`.

| Candidate ID | Direct source | Workflow categories | Version/date evidence | Review gap |
|---|---|---|---|---|
| `WF-SRC-FL-001` | `https://www.youtube.com/watch?v=3oKVpTHC-0M` | broad beginner path; cross-surface transitions | 2026-01-28; version unknown | Split chapters; do not count broad instruction as authentic professional session |
| `WF-SRC-FL-002` | `https://www.youtube.com/watch?v=uDVny0ruaUc` | pattern beatmaking; arrangement | 2025-06-27; `flstudio2024` hashtag only | Timestamp the compact introductory loop and Song-mode sequence |
| `WF-SRC-FL-003` | `https://www.youtube.com/watch?v=qTh4COvhIs0` | interface setup; audio recording | 2025-09-26; version unknown | reviewed as `WF-FL-RECORD-004`; success-path tutorial without measured latency or failure/media-lifecycle coverage |
| `WF-SRC-FL-004` | `https://www.youtube.com/watch?v=dpYGB6EUSPI` | Playlist arrangement; shortcuts | 2025-06-27; `flstudio2024` hashtag only | reviewed as `WF-FL-ARRANGE-001`; remains insufficient for frequency, preference, or professional-workflow claims |
| `WF-SRC-FL-005` | `https://www.youtube.com/watch?v=Mx7AnMUCDic` | levels; buses; sends; mixing | 2021-06-22; version unknown | Establish demonstrated UI version and customer-only fixture limits |
| `WF-SRC-FL-006` | `https://www.youtube.com/watch?v=H8soRtrlgF8` via `https://www.image-line.com/artists/murda-beatz` | professional beat deconstruction | 2018-04-12; version unknown | Confirm actual FL-specific actions; short retrospective may be too compressed |
| `WF-SRC-FL-007` | `https://www.youtube.com/watch?v=AwJGKMRhn-A` | extended beatmaking tutorial | 2024-10-16; version unknown | Watch full sequence and record project-download access constraints |
| `WF-SRC-FL-008` | `https://www.youtube.com/watch?v=t8DLBltLa6A` | vocal recording; routing | 2024-05-10; FL Studio 21 | Separate general stock workflow from promoted preset ecosystem |
| `WF-SRC-FL-009` | `https://www.youtube.com/watch?v=yRzMby4PXzc` | start-to-finish mix; automation; sidechain | 2021-03-20; FL Studio 20 | Record third-party plugin/stem dependencies and cross-DAW framing |
| `WF-SRC-FL-010` | `https://www.youtube.com/watch?v=aBzQ5KV4glE` | templates; shortcuts; organization | 2018-05-08; version unknown | Focus on repeated workflow changes; verify legacy bindings |
| `WF-SRC-FL-011` | `https://www.youtube.com/watch?v=TkTZLblecPM` | Playlist arrangement; shortcuts | 2018-06-12; FL Studio 20 | reviewed as `WF-FL-PLAYLIST-002`; focused surface tutorial, not a complete arrangement account |
| `WF-SRC-FL-012` | `https://www.youtube.com/watch?v=yTZTmZrNdnw` | extended melody/beat session | 2025-01-21; version unknown | reviewed as `WF-FL-NICK-MIRA-003`; long-form professional session, but transcript is sparse during music-only action |

## REAPER official-video candidate set

Source index: `SRC-REAPER7.77-OFFICIAL-VIDEOS`.

The official index displayed a REAPER 7.77 site banner, but individual videos vary in age and demonstrated version. Version must be established per video when possible.

### End-to-end candidates

| Candidate ID | Direct source | Visible sequence | Workflow categories | State | Review gap |
|---|---|---|---|---|---|
| `WF-SRC-REAPER-001` | `https://www.reaper.fm/videos.php#Jj2kpds_GgI` and linked numbered continuation | First Loop → drum/percussion loops → bass/keys/synths → vocals/effects → arrangement → final mix/rendering | electronic composition; loop-based arrangement; mixing; export | candidate | watch all 11 parts; capture dates, versions, actions, timestamps, repeated loops, friction, completion state |
| `WF-SRC-REAPER-002` | `https://www.reaper.fm/videos.php#8H7Wa3bmMmM` and linked numbered continuation | finding tempo → programmed drums/percussion → piano/bass → loops/synths → effects/fills → final arrangement | beatmaking; pattern/loop composition; arrangement | candidate | watch all 7 parts; establish version and complete action sequence |
| `WF-SRC-REAPER-003` | `https://www.reaper.fm/videos.php#RjMtnx134SI` and linked numbered continuation | setup/scratch tracks → live drums → bass/acoustic guitar → piano/electric guitars → solo guitars → vocals/strings | band multitrack recording; overdub; arrangement | candidate | watch all 6 parts; identify monitoring, takes, editing, routing, and file lifecycle |
| `WF-SRC-REAPER-004` | `https://www.reaper.fm/videos.php#0kEm36n8TKQ` and linked numbered continuation | new project → tracks → audio/MIDI recording → editing → effects → sends/buses → folders/groups → markers → automation → actions → rendering | beginner breadth; recording; editing; routing; command system; export | candidate | tutorial breadth may not represent one authentic project; review and split observations carefully |

### Focused workflow candidates

| Candidate ID | Direct source | Research use | State | Review gap |
|---|---|---|---|---|
| `WF-SRC-REAPER-005` | `https://www.reaper.fm/videos.php#jHgu633GoRY` | track lanes and comping | candidate | capture full comping state transitions and selection/focus behavior |
| `WF-SRC-REAPER-006` | `https://www.reaper.fm/videos.php#cX5Cq_Roa9E` | FX containers and parallel routing | candidate | capture route construction, visibility, and error recovery |
| `WF-SRC-REAPER-007` | `https://www.reaper.fm/videos.php#TLhIHNp1zWU` | mouse overrides | candidate | distinguish customization capability from actual repeated workflow use |
| `WF-SRC-REAPER-008` | `https://www.reaper.fm/videos.php#eaFPpoOr0DQ` | keyboard overrides | candidate | capture contexts, conflicts, discoverability, and safety |
| `WF-SRC-REAPER-009` | `https://www.reaper.fm/videos.php#p6PrKmhYlxY` | Actions List | candidate | distinguish built-in actions, custom actions, scripts, and extension actions |
| `WF-SRC-REAPER-010` | `https://www.reaper.fm/videos.php#HZcXZ9kEJbY` | recording latency adjustment | candidate | capture prerequisites, measurement, correction, and failure cases |
| `WF-SRC-REAPER-011` | `https://www.reaper.fm/videos.php#r5seIzlWzuQ` | project backup | candidate | capture project state, media scope, recovery path, and verification |
| `WF-SRC-REAPER-012` | `https://www.reaper.fm/videos.php#huhMxZEb5Uc` | sample audition and insertion | candidate | capture preview context, selection, transport, and placement continuity |

## Exclusions and cautions

- Plugin-buying and third-party-plugin catalog videos do not establish core DAW workflows unless a specific host interaction is under review.
- “What's new” material is release-delta evidence, not an end-to-end workflow account.
- Theme/layout customization is relevant only when it materially changes repeated task execution or accessibility.
- Beginner tutorial sequences can reveal discoverability but must not be counted as independent professional accounts without evidence about the performer and task context.
- An official tutorial may omit friction and failures; independent practitioner evidence is required for triangulation.

## Observation admission gate

Before adding a line to `workflow-observations.jsonl`, a reviewer must verify:

1. direct source URL and title;
2. source type and evidence class;
3. publication date and product version, or explicit unknowns;
4. musician role, genre, and environment, or explicit unknowns;
5. ordered action sequence with timestamps/pages for demonstrated workflows; interview
   self-reports MUST instead preserve thematic statements without implying observed order;
6. repeated inner loop and completion condition;
7. commands, gestures, focus/view transitions, and acted-on objects;
8. observed friction versus reviewer inference;
9. confidence and corroboration;
10. no extrapolation from tutorial title or product capability alone.

## Current evidence state

Four FL Studio observations have passed extraction, including professional long-form session `WF-FL-NICK-MIRA-003` and official audio-interface recording tutorial `WF-FL-RECORD-004`. The recording source establishes a documented success path from device/permission setup through input, monitoring, arming, capture, and mixer routing, while leaving latency, effect monitoring, failure, compatibility, and media recovery unproven.

Two Ableton Live interview self-reports have passed thematic extraction:
`WF-ABLETON-LUSTWERK-005` (Session-loop production, minimal-take vocal workflow,
Stream-Deck-driven zero-latency live template) and `WF-ABLETON-TSURUTA-006`
(color-coded groups, +6 dB master Utility deactivated at export, Command-modifier
free-hand automation drawing). They are not action-sequence observations and MUST NOT
be used for sequence, gesture, frequency, or priority analysis. Video or session
evidence is still required for Ableton Live before any gesture-level claim.

This remains far below saturation; command frequency, product priority, and usability targets remain prohibited.
