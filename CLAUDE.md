<!--
Author: Jeff
Date: 2026-05-27
Description: Claude working instructions for Geist DAW
Notes: Project-local agent entrypoint; load before any implementation or review
-->

# Geist DAW Claude Instructions

## Load order

1. Read this file.
2. Load `.claude/skills/geist-daw-working-context.md`.
3. Read `docs/README.md`, `docs/status/STATUS.md`, and `docs/status/NEXT.md`.
4. Load `geist-realtime-rust` for callback-adjacent Rust and `geist-validation-gates` before review.

## Non-negotiables

- Follow the accepted authority order in `docs/README.md`.
- Follow `docs/06-plans/rebuild-roadmap.md`; the repository root is the only implementation workspace.
- Work in small validated slices.
- Keep comments terse, information dense, and declarative.
- Preserve Jeff's standard header block in every new source/doc file.
- Run targeted validation before claiming completion.

## Local skills

- `.claude/skills/geist-daw-working-context.md`
- `.claude/skills/geist-realtime-rust.md`
- `.claude/skills/geist-validation-gates.md`
