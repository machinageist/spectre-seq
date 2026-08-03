<!--
Author: Jeff
Date: 2026-05-27
Description: Claude working instructions for Spectre Seq
Notes: Project-local agent entrypoint; load before any implementation or review
-->

# Spectre Seq Claude Instructions

## Load order

1. Read this file.
2. Load `.claude/skills/spectre-seq-working-context.md`.
3. Load the domain skill that matches the files being touched.
4. Use the narrowest `.claude/agents/*.md` agent for implementation or review.

## Non-negotiables

- Follow `INITIAL_PLAN.md` phase order unless Jeff explicitly changes it.
- Treat `PROPOSED_FILE_TREE.md` as intended architecture until implementation proves otherwise.
- Work in small validated slices.
- Keep comments terse, information dense, and declarative.
- Preserve Jeff's standard header block in every new source/doc file.
- Run targeted validation before claiming completion.

## Local indexes

- Skills: `.claude/skills/README.md`
- Agents: `.claude/agents/README.md`
