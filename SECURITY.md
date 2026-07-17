<!--
Author: Jeff
Date: 2026-07-17
Description: Vulnerability reporting policy for the Geist DAW repository
Notes: Early-stage software; no versioned releases or formal support window yet
-->

# Security Policy

Geist is early-stage, pre-release software. There are no tagged releases and no formal support window yet — `main` is the only supported line.

## Reporting a vulnerability

Do not open a public GitHub issue for a suspected vulnerability. Instead, use this repository's [private security advisory](../../security/advisories/new) form on GitHub, which reaches the maintainer directly and keeps details confidential until a fix is available.

Include:

- Affected path or crate (e.g. `crates/geist-project`, project file decoding, VST3 host once it exists).
- Steps to reproduce, and the impact if exploited (e.g. crash, memory unsafety, arbitrary code execution, malicious project/plugin files).
- Any relevant environment details (OS, Rust toolchain version).

## Scope notes

Geist currently has no live audio backend, network I/O, or plugin hosting — the highest-value targets today are the project file codec (`crates/geist-project`) and future VST3 host isolation (`docs/06-plans/rebuild-roadmap.md`, milestone R8). Reports on either are especially welcome.
