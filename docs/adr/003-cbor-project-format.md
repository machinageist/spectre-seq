<!--
Author: Jeff
Date: 2026-08-01
Description: Record the project-format decision — a project-directory package with a canonical manifest and atomic load validation.
Notes: The binary encoding is deliberately still open; the filename's "cbor" is a prototype fact, not a decided outcome.
-->

# ADR 003 — Project directory package with canonical manifest

- Status: **Accepted for package shape, versioning, and load semantics** (2026-08-01). **Encoding is open.**
- Source of the decision: `docs/changes/project-document/SPEC.md` (accepted 2026-08-01) and the "Project package and recovery" section of `docs/product/PRODUCT_VISION.md`. This ADR records those decisions; it does not add new ones.
- Filename: `003-cbor-project-format.md` is retained because ADR numbers and paths are stable. **The slug is wrong.** It asserts an encoding this ADR explicitly leaves open. Read the title, not the filename.
- Delivery: roadmap Milestone 2, slice D6 in `docs/changes/project-document/PLAN.md`.

## Context

`INITIAL_PLAN.md:128` deferred the choice: "`CBOR` (compact binary) or
`MessagePack` for the main project file; human-readable `TOML` for settings." No
decision followed. The prototype picked CBOR and the scaffold ADR was named after
that pick, which is how an undecided question acquired a decided-looking filename.

What the prototype actually does today — all of it pre-v1, none of it binding:

| Behavior | Where |
| --- | --- |
| Single-file CBOR encode/decode over `ciborium` | `crates/spectre-project/src/serialize.rs:57` |
| `SCHEMA_VERSION: u32 = 1`; newer files rejected | `crates/spectre-project/src/schema.rs:15`, `migrate.rs` |
| Forward-migration step table, currently empty | `crates/spectre-project/src/migrate.rs:23` |
| Global settings as human-readable TOML | `crates/spectre-project/src/settings.rs` |
| blake3 content-addressed asset references, never embedded | `crates/spectre-project/src/asset_map.rs` |
| Temp-sibling + rename atomic write | `crates/spectre-project/src/autosave.rs:29` |
| 60 s background autosave to a `.geist-autosave` sidecar | `crates/spectre-project/src/autosave.rs:23` |
| Flat `<name>.gproj` plus sibling `<stem>.assets/Takes/<blake3>.wav` | `app/spectre-seq/src/session.rs:425-438` |
| Fixed session slot at `$HOME/spectre-studio.gproj` | `app/spectre-seq/src/session.rs:24,468` |

Three of those contradict the accepted contract and are recorded here as
implementation debt, not as precedent:

1. **Manual save is not atomic.** `save_to_path` (`serialize.rs:69`) calls
   `std::fs::write` directly. Only the autosaver uses `atomic_write_cbor`. The
   product contract requires the opposite pairing — atomic manual save, and an
   autosave that never overwrites the last known-good manual save.
2. **Load mutates live state incrementally.** `app/spectre-seq/src/studio.rs:484-494`
   tears down the engine's clips and resets the asset budget while reading the
   file. A failed load leaves a half-replaced session.
3. **The package is a flat file with an ad-hoc sibling directory**, not a
   directory package with a canonical manifest.

## Decision

A Spectre project is a **directory package**, not a single file.

- **Canonical manifest.** One manifest inside the package holds the project
  document. Everything else in the package is managed content the manifest
  references.
- **Managed subdirectories**, each with a defined owner: recordings and media;
  renders and freezes; autosaves and backups; disposable cache. Disposable cache
  is safe to delete without data loss; the others are not.
- **External media stays external.** Referenced files are not rewritten in place.
  Collect All and Save imports external dependencies into managed storage using
  verified asset identity — size plus content hash, which the blake3 `AssetMap`
  already models.
- **Versioned schema.** The manifest carries a schema version. A file newer than
  the binary supports is rejected, never best-effort read. Spectre never silently
  writes an older format.
- **Atomic candidate-then-replace load.** A load builds a complete candidate
  document and validates it — every identity domain, asset reference, clip, graph
  and automation target, and unresolved placeholder — against the candidate,
  before the live document is touched. It then replaces the live document
  wholesale, or leaves it untouched and reports exactly what failed. There is no
  partial load and no half-migrated document.
- **Unresolved is not invalid.** Missing assets, devices, plugins, modules,
  mappings, and automation targets round-trip losslessly with their complete
  original descriptor preserved. They stay visible, inspectable, and relinkable,
  and they never block loading, editing, or saving the rest of the project.
  Resolved-versus-unresolved is derived from the current registry at read time,
  never persisted as a second source of truth.
- **Atomic manual save.** Manual saves replace the manifest atomically. Autosave
  and startup recovery write elsewhere in the package and never overwrite the last
  known-good manual save.
- **Persistence is a projection.** `spectre-project` serializes a projection of the
  app-thread `ProjectDocument`, not a renderer-facing mirror. The on-disk contract
  carries no vector indices and no runtime handles.
- **Pre-v1 breaks are allowed.** Until stable format v1 is declared, prototype
  schemas may take intentional breaking cleanup with explicit diagnostics and
  fixture-tested conversion. Stable v1 begins the supported forward-migration
  contract. A load never reports success after discarding durable state.

## Open: the binary encoding

**Neither `docs/changes/project-document/SPEC.md` nor its `PLAN.md` settles the
manifest's binary encoding.** The D6 slice requires a "versioned schema" and a
"project package layout with canonical manifest" and says nothing about bytes.
`docs/product/PRODUCT_VISION.md` is likewise silent. The word CBOR appears in
neither document.

The encoding is therefore **open**, and this ADR does not close it.
CBOR-via-`ciborium` is what the prototype happens to use — a fact about
`crates/spectre-project/src/serialize.rs`, not a ratified outcome. This ADR's
filename is not evidence of a decision.

Candidates still live: CBOR, MessagePack, or a human-diffable text manifest with
binary sidecars. The choice is made in slice D6 and recorded by amending this ADR
or by adding a superseding one. Criteria to decide against:

- unknown-field tolerance for forward compatibility;
- self-describing enough to inspect a damaged file without the exact binary;
- opaque-blob support for plugin state (`serde_bytes` is already a dependency);
- diffability and merge behavior for users under version control;
- encode and decode cost for large projects.

Until then, treat CBOR as the prototype's incumbent, not as settled.

## Consequences

- **`spectre-project` gains a dependency on `spectre-document`** and loses its role as
  a de-facto data model. The `ProjectFile` DTO tree becomes a serialization
  projection of the document rather than a parallel schema.
- **`app/spectre-seq/src/session.rs` becomes a persistence adapter and is then
  deleted** (SPEC slice D8). `StudioSession` is not a durable authority.
- **The flat `.gproj` file becomes a package directory.** That is a breaking
  format change, permitted pre-v1, and it needs diagnostics plus fixture-tested
  conversion for existing prototype files.
- **Migration must be exercised before v1.** `migrate.rs` has a step table with no
  steps in it. The first real schema break is where that engine gets proven.
- **Load failure becomes reportable rather than partial**, which requires the
  candidate document to exist before the live one is replaced — so this ADR cannot
  land ahead of the `ProjectDocument` slices it depends on.
- **ADR 001's claim still holds.** It cites "the project format already stores
  opaque plugin state blobs (ADR 003)" in support of VST3 `IComponent`
  get/setState. That stays true under a package format: opaque blob storage is an
  encoding-independent requirement, and it is listed above as a criterion for
  whichever encoding wins.

## Alternatives considered

- **Keep the single-file project.** Simplest, and it is what exists. Rejected:
  recordings, renders, freezes, autosaves, and cache have different lifetimes and
  different delete-safety. One opaque file cannot express that, and it forces the
  app to invent sibling directories anyway — which
  `app/spectre-seq/src/session.rs:425` already does.
- **Bundle everything inside one archive (zip-style).** Portable and atomic to
  copy. Rejected for now: it makes large recorded media expensive to write
  incrementally and hostile to external tooling. Revisit as an export format, not
  the working format.
- **Fully human-readable manifest with no binary at all.** Attractive for
  diffability. Not rejected — it is one of the live encoding candidates above.
