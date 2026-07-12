<!--
Author: Jeff
Date: 2026-07-11
Description: Human-readable index of clean-room and workflow research sources for Geist DAW
Notes: The machine-readable source ledger is authoritative for individual source records
-->

# Reference and Workflow Source Index

- **Status:** draft
- **Last verified:** 2026-07-11
- **Scope:** research-source discovery, version state, coverage state, and unresolved source gaps
- **Decision authority:** Jeff
- **Upstream sources:** `docs/02-reference-research/source-ledger.json`; `docs/02-reference-research/methodology.md`
- **Downstream dependents:** product dossiers, coverage matrices, workflow corpus, shortcut corpus, requirements provenance
- **Supersedes:** source lists embedded in legacy clean-room specifications
- **Superseded by:** none
- **Open decisions:** archival strategy for mutable official sources; whether third-party snapshots can be redistributed
- **Known gaps:** official source entry points remain under verification for five substantive references and VST3; 8 Ableton, 10 Bitwig, and 8 FL Studio workflow candidates plus 12 REAPER candidates await extraction review; Ableton evidence is interview-only and needs visible-session corroboration

## Authority and interpretation

`docs/02-reference-research/source-ledger.json` owns source-record facts. This document summarizes research progress. A source listed here is discovered, not exhaustively reviewed. No entry grants a Geist requirement or implementation decision.

## Official reference-source status

| Product | Research role | Verified source state | Version evidence | Coverage | Immediate gap |
|---|---|---|---|---|---|
| Ableton Live | substantive DAW reference | official versioned manual welcome page and rendered top-level navigation inspected | manual identifies Version 12 and URL is versioned `/12/` | inventory-only | inspect all top-level chapters and capture section-level matrix |
| Bitwig Studio | substantive DAW reference | official mutable `latest` guide welcome page inspected twice; two official artist interviews fully inspected with documented extraction shortfall | rendered guide navigation and change section both identify v5.3 (2026-07-11) | inventory-only | verify 5.3 against an official release notice; obtain visible-session video evidence — interviews yielded preference/friction statements without action sequences |
| REAPER | substantive DAW reference | official guide landing page, direct PDF identity, bounded dossier, rendered official-video index, and initial workflow candidates recorded | guide filename identifies 7.75b; site advertised application 7.77 on 2026-07-11; individual video versions vary or are unknown | inventory-only | review/timestamp candidate videos and inventory PDF TOC when source processing is available |
| VCV Rack | substantive modular reference | official manual index inspected | official navigation identifies Rack 2; no manual revision exposed | inventory-only | inspect user chapters and distinguish Rack behavior from plugin-development material |
| FL Studio | substantive DAW reference | official online manual entry, navigation, title page, title artwork, and bounded dossier recorded | title artwork identifies FL Studio 26; live paths are unversioned | inventory-only | capture a deterministic TOC and direct shortcut-source identity |
| Logic Pro | substantive DAW reference | official online guide, selected version, PDF links, main 1324-page PDF identity/TOC, and bounded dossier recorded | online guide selects 12.3; PDF does not independently print 12.3 | inventory-only | classify complete TOC and separately inventory instruments/effects/control-surfaces guides |
| Cubase Pro | substantive DAW reference | official Steinberg Webhelp metadata, TOC, New Features entry, footer, and bounded dossier recorded | version branch 15.0; rendered content includes 15.0.20 | inventory-only | capture deterministic TOC; branch URL is not patch-pinned |
| Phase Plant | substantive sound-design reference | complete rendered official documentation page inspected; bounded dossier created | no product/manual version or revision date shown | inventory-only | resolve version scope before completeness claims and classify sections |
| Serum 2 | substantive sound-design reference | official product page, change-guide PDF endpoint, full displayed support-category list, and source-gap dossier recorded | version 2 only; build/date unknown | blocked-source-gap | no complete public user manual was exposed by inspected official entry points |
| VST3 | compatibility target | official developer portal, 3.8.x interface docs, SDK tag 3.8.0 build 66, and licensing guidance inspected | tag v3.8.0_build_66; full commit/submodule identities not yet pinned | inventory-only | audit exact tagged tree, submodule licenses, host contracts, and selected Rust binding before decision |

## Workflow-source status

Four FL Studio sources have passed extraction: two official tutorials, one independent educator tutorial, and professional long-form session `WF-FL-NICK-MIRA-003`. `WF-FL-RECORD-004` adds an interface-input/mixer-recording success path but no measured latency or failure/recovery evidence.

Two Ableton Live sources have passed extraction: official Input/Output interviews supporting `WF-ABLETON-LUSTWERK-005` and `WF-ABLETON-TSURUTA-006`. Both are first-person self-reports admitted at `low` confidence; they establish stated habits, one concrete automation-drawing binding, and live-performance latency constraints, but no visible action sequences. No command-frequency, ergonomic, convergence, or priority claim follows from the current corpus.

Direct-source discovery includes 10 Ableton Live candidates, 10 Bitwig Studio candidates, 12 FL Studio candidates, and 12 REAPER candidates. These counts describe review queues, not evidence saturation: only four FL sources have admitted timestamped action sequences. Candidate details and limitations live in `workflow-corpus.md`.

The next source expansion MUST include both complete-task evidence and friction evidence. Official manuals establish documented commands but do not establish repeated real-world use. Artist marketing material alone is insufficient, and isolated community complaints are not representative.

Explicit product rationale now prioritizes hypnotic techno, forest psytrance, deep dubstep, modern synthesis/arrangement, and realtime audio-interface input through the mixer. The next expansion must therefore include genre-relevant professional sessions plus device/input selection, software monitoring, mixer/effect routing, latency, recording, dropout/disconnect, and captured-media recovery evidence. Vendor names such as M-Audio and Focusrite denote the expected interface class, not an unverified model-compatibility promise.

## Source gaps requiring explicit handling

### Mutable official pages

A mutable `latest` URL MUST carry a dated access record and explicit rendered-version evidence. If rendered version evidence conflicts with the current product release, the source remains inventory-only until resolved.

### Guide and application version divergence

A guide may lag its application. Claims MUST identify the guide version actually inspected rather than inheriting the application version from a nearby download banner.

### Installed or licensed manuals

If a complete manual is available only inside a licensed installation, research MUST record that limitation and restrict claims to legitimately accessible official public material. Product pages and “what’s new” documents do not become exhaustive manuals by substitution.

### Open-source documentation

Public source availability does not authorize copying source code, artwork, panels, patch files, or distinctive interaction composition. Geist research remains behavioral and attribution-preserving.

## Promotion checklist

A source advances from discovery only when:

1. direct URL or document identity is recorded;
2. publisher and source class are known;
3. product/manual version evidence is captured or explicitly unavailable;
4. access date and limitations are recorded;
5. inspected sections are named;
6. mutable-source risk is classified;
7. the source is assigned to a declared dossier scope.

A dossier advances to product-planning acceptance only through the stricter gate in `docs/02-reference-research/methodology.md`.
