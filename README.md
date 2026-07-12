<!--
Author: Jeff
Date: 2026-07-11
Description: Root orientation for the Geist DAW repository during the specification-first rebuild
Notes: docs/README.md owns documentation authority; this file only orients
-->

# Geist DAW

Geist is an original, Rust-first digital audio workstation in early development. The goal is a serious open-source tool for composing, recording, sound design, mixing, and live performance — with its own audio engine, render graph, project model, native devices, and VST3 hosting behind an isolated host boundary.

## Honest state

This repository is mid-way through a specification-first rebuild. Read `docs/status/STATUS.md` for the current verified state before believing any other status claim.

In short:

- The legacy prototype (`app/geist-daw` plus `crates/`) compiles and passes its automated tests on macOS, but its live audio path is a fixed-track engine; the compiled graph, automation, modular-rack, and stacksynth crates are unit-tested in isolation and not wired into the running application. No subsystem is release-qualified.
- A forensic audit of the legacy code and documentation lives in `docs/audits/`.
- Clean-room reference research and a musician workflow field study are in progress under `docs/02-reference-research/`.
- The ground-up rebuild (new requirements-traced foundation) has **not** begun; the specification gate in the rebuild mandate is not yet satisfied.

## Where things live

| Path | What it is |
|---|---|
| `docs/README.md` | Documentation authority map — start here for any doc question |
| `docs/status/` | Current verified state, next slices, validation evidence, subsystem ledger |
| `docs/audits/` | Forensic audits of the legacy repository |
| `docs/02-reference-research/` | Clean-room source ledger, product dossiers, workflow field study |
| `docs/06-plans/current-milestone.md` | The single active rebuild milestone |
| `app/geist-daw`, `crates/` | Legacy prototype workspace (active code, not the target architecture) |
| `INITIAL_PLAN.md`, `PRODUCTION_PLAN.md`, `PROPOSED_FILE_TREE.md`, `HANDOFF.md` | Legacy planning documents — historical evidence, superseded for rebuild-lane authority by `docs/` |

## Building the legacy prototype

```sh
cargo check --locked --workspace --all-targets --all-features
cargo test  --locked --workspace --all-features
```

Baseline results and known gate failures (formatting drift, strict-clippy diagnostics) are recorded in `docs/status/VALIDATION.md`.

## Working lanes

Two lanes are deliberately kept separate:

1. **Rebuild documentation lane** — audits, research, and specification work under `docs/`.
2. **Legacy feature lanes** — modular rack (`AGENTS/changes/modular-rack/`) and stacksynth, continuing in the prototype workspace.

Do not mix commits across lanes.

## Legal posture

Geist uses original code, DSP, design, names, and content. Public documentation for other products informs behavioral requirements only, under the clean-room methodology in `docs/02-reference-research/methodology.md`. Trademarks appear only in research and compatibility contexts.
