<!--
Author: Jeff
Date: 2026-07-17
Description: Contribution workflow for the Geist DAW repository
Notes: This repo is early-stage and specification-first; read docs/README.md before proposing changes
-->

# Contributing

Geist is in early, specification-first development. `docs/README.md` owns documentation authority — read it, `docs/status/STATUS.md`, and `docs/status/NEXT.md` before proposing a change.

## Workflow

- The repository root is the only implementation workspace (see `docs/06-plans/rebuild-roadmap.md`).
- Work in small, validated slices; each slice should compile, pass its tests, and update any docs it makes stale in the same change.
- Non-trivial features follow the spec hierarchy under `AGENTS/changes/<change-id>/SPEC.md` and `PLAN.md`: desired outcome and acceptance criteria first, atomic implementation tasks second.
- Reordering the roadmap requires documented dependency reasoning, not just preference.

## Code conventions

- Keep comments terse, information-dense, and declarative — state the non-obvious, not the obvious.
- Every new source or doc file carries the standard header block (author, date, description, notes).
- Don't claim behavior in docs or comments that the code doesn't yet implement.

## Before opening a pull request

Run the commands in README's `## Validate` section:

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo run --locked -p geist-offline -- --self-test
```

CI runs the same commands on every push and pull request against `main`.

## Reporting issues

Use GitHub Issues for bugs and gaps against documented, accepted behavior. For security vulnerabilities, see `SECURITY.md` instead of opening a public issue.
