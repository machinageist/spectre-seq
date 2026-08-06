<!--
Author: Jeff
Date: 2026-07-11
Description: Root orientation for the Geist DAW repository during the specification-first rebuild
Notes: docs/README.md owns documentation authority; this file only orients
-->

# Geist DAW

Geist is an original, Rust-first digital audio workstation in early development. It targets electronic composition, recording, sound design, mixing, and live performance with an original engine, project model, native devices, and isolated VST3 hosting.

## Current state

The repository contains one clean implementation workspace. The earlier prototype and its plans, assets, CI, audits, and feature lanes were removed on 2026-07-12; committed history remains available through Git.

R0/R1 provides a tested musical kernel and versioned project envelope. A native interaction prototype makes the workspace launchable and reviewable. For deterministic offline R2, the editable graph compiles to an immutable validated render plan, app-model parameter snapshots feed the compiled-plan Pulse → Gain → Saturator fixture, and Build can focus an existing device in Shape without losing track or device selection. `geist-offline` is the current deterministic offline render and project-inspection harness. Live audio I/O and its callback bridge, plugin hosting, and recording are not implemented and remain later scope. Read `docs/status/STATUS.md` before relying on any maturity claim.

## Repository map

| Path | Purpose |
|---|---|
| `crates/geist-app/` | Native interaction prototype and renderer-neutral app model |
| `crates/geist-core/` | IDs, time, tempo, transport, events, and parameter contracts |
| `crates/geist-dsp/` | Realtime-safe native sources, instruments, effects, and process contracts |
| `crates/geist-graph/` | Editable device graph and immutable compiled render plan |
| `crates/geist-offline/` | Current deterministic offline render and project-inspection harness |
| `crates/geist-project/` | Versioned project envelope and validated decoding |
| `docs/README.md` | Documentation authority and status vocabulary |
| `docs/00-product/` | Accepted product direction |
| `docs/01-requirements/` | Requirements, decisions, and traceability |
| `docs/02-reference-research/` | Clean-room evidence and workflow research |
| `docs/06-plans/` | Roadmap and active milestone |
| `docs/status/` | Verified state and next slices |

## Launch

From the repository root:

```sh
./geist
```

The first build downloads and compiles the native UI dependencies. Later launches reuse Cargo's build cache.

The prototype exposes Arrange, Build, Shape, and Mix lenses, track selection and creation, transport interaction, a context shelf, and an in-app feedback box. Each Build device has a labeled **Open in Shape** action; Shape edits only that stable project-instance selection through backend-owned descriptors. Enter observations in **Prototype Feedback**, click **Copy feedback report**, and paste the resulting state-rich report, including the selected device, into the next development conversation.

Run the launch target without opening a window:

```sh
./geist --smoke-test
```

## Validate

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

Run the deterministic project harness:

```sh
cargo run --locked -p geist-offline -- --self-test
```

## Legal posture

Geist uses original code, DSP, design, names, and content. Public documentation for other products informs behavioral requirements only, under the clean-room methodology in `docs/02-reference-research/methodology.md`. Trademarks appear only in research and compatibility contexts.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option. See `CONTRIBUTING.md` for the contribution workflow and `SECURITY.md` for vulnerability reporting.
