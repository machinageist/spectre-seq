<!--
Author: Jeff
Date: 2026-08-01
Description: Product and architecture contract for the canonical app-thread ProjectDocument.
Notes: Fixes ownership, transaction, projection, and publication boundaries; per-domain content stays in sub-specs.
-->

# Canonical ProjectDocument Authority

## Status

Accepted. Four design decisions were taken at the 2026-08-01 interview:

1. Migration is a strangler, one domain at a time, with the app runnable throughout.
2. This spec fixes the whole document contract; each domain's internal content model stays in its own sub-spec.
3. Realtime publication is hybrid: immutable versioned render generations plus a bounded acknowledged control stream.
4. `ProjectDocument` lives in a new dependency-low `geist-document` crate.

This spec satisfies roadmap Milestone 2 and immediate work order item 2. It unblocks items 3 through 6 and supersedes the paused Slice B2 ownership questions in `docs/changes/canonical-clip-commands/SPEC.md`.

## Problem

Five owners currently hold overlapping durable arrangement and project truth:

- `geist_timeline::Timeline` (`crates/geist-timeline/src/track.rs:91`) — legacy arena handles, sample placement.
- `geist_timeline::Arrangement` (`crates/geist-timeline/src/arrangement.rs`) — canonical entities, zero consumers.
- `app::engine::Arrangement` (`app/geist-daw/src/engine.rs:259`) — audio-thread copy, fixed `NUM_TRACKS`.
- `geist_ui::model::TimelineModel` (`crates/geist-ui/src/model.rs:295`) — renderer-facing, directly mutated.
- `app::session::StudioSession` (`app/geist-daw/src/session.rs:121`) — de-facto persistence model in float beats.

Consequences visible in the code today:

- Durable references depend on vector position (`track: u8`) and raw untyped ids (`id: u64`).
- Per-track device state is encoded as magic parameter-id arithmetic on a synthetic macros node (`PARAM_TRACK_CUTOFF_BASE + t`).
- `EngineControl::send` returns a saturation bool that callers discard, so app and audio state diverge silently.
- No transaction boundary exists, so a multi-part edit can partially apply.
- Undo exists only for the legacy timeline and cannot prove exact identity restoration.
- Persistence advertises more state than the app round-trips.

## Ownership decision

One app-thread-owned `ProjectDocument` owns every durable project fact. UI models, the on-disk file, and compiled realtime state are projections derived from it. No projection may be written to directly, and no projection may be read as authority.

`ProjectDocument` owns these aggregates:

| Aggregate | Owns | Content spec |
| --- | --- | --- |
| `identity` | Every durable ID allocator | this spec |
| `meta` | Format version, dirty state, save/recovery metadata; project package layout | D6 |
| `tracks` | Ordered hybrid track records and track order | Milestone 4 |
| `clips` | Clip records: identity, kind, name, colour, typed content payload | `canonical-clip-content/SPEC.md` |
| `arrangement` | Arrangement placements: owning `TrackId`, start, window, lane membership | `canonical-clip-content/SPEC.md` |
| `launcher` | Scenes, ordered scene list, track/scene slot placements | Milestone 5 |
| `graph` | Devices, chains, routing, sends, returns — **the durable graph editor** | `typed-realtime-graph/SPEC.md` |
| `assets` | Managed asset registry, verified/offline state | `canonical-clip-content/PLAN.md` slice CC2a |
| `conductor` | Tempo map, time signatures, metronome, count-in, groove, loop region, punch points | Milestone 5 |
| `automation` | Durable targets and curves | Milestone 5 |
| `mappings` | Controller mappings, macros, remote pages | Milestone 4 |

An aggregate may be an empty typed owner until its content sub-spec lands. An empty aggregate is still the named authority for its domain; no other type may hold that domain's durable state once its slice completes.

**Amended 2026-08-03** by two accepted sub-specs:

- `canonical-clip-content/SPEC.md` decision 1a splits clip content from clip placement. `clips` is a new sibling aggregate owning content; `arrangement` is re-scoped to placement only. The reason is the launcher: content welded to arrangement placement would force a parallel clip type with duplicated editors and persistence. It also makes cross-track move provably unable to touch content.
- `typed-realtime-graph/SPEC.md` decision 17 confirms `graph` as the durable graph *editor*, with `geist-graph` reduced to compile-and-execute. There is one routing authority, not two.

### The named arrangement aggregate

`geist_document::arrangement::Arrangement` is the final arrangement authority. It is the entity model already implemented in `crates/geist-timeline/src/arrangement.rs`, relocated. Every other arrangement representation is demoted:

- `geist_timeline::Timeline` — legacy, frozen, deleted in slice 8.
- `app::engine::Arrangement` — becomes render-generation content, not an authority.
- `geist_ui::model::TimelineModel` — becomes a read-only projection.
- `app::session::StudioSession` — becomes a persistence adapter, then deleted.

## Crate and dependency boundaries

New crate `crates/geist-document`, `#![deny(unsafe_code)]`, no serde, no renderer, no audio-backend dependency.

Dependency direction:

- `spectre-core` owns shared domain vocabulary: the durable ID family, `MusicalTime`, `TICKS_PER_QUARTER`, sample coordinates, normalized values, and curve-shape vocabulary.
- `geist-document` depends on `spectre-core` only.
- `spectre-project` depends on `geist-document` and owns schema, serialization, migration, and the project package.
- `geist-ui` depends on `geist-document` for read-only projections and typed intent types.
- `geist-document` never depends on `geist-timeline`, `spectre-project`, `geist-ui`, or the app.

`MusicalTime`, `TICKS_PER_QUARTER`, and the canonical identity allocator relocate out of `geist-timeline`. `geist-timeline` re-exports them during the compatibility window so no consumer breaks in the same slice as the move. `PROPOSED_FILE_TREE.md` is revised to match once slice 1 lands.

## Transaction contract

Every durable mutation runs inside one transaction on the app thread.

A transaction:

- carries a human-readable label for history UIs;
- validates every precondition across every affected aggregate before mutating anything;
- either applies completely or leaves the document byte-identical;
- reserves all required identities as one checked batch at the enforcing aggregate;
- returns a typed result, never a silent no-op.

```
TransactionResult =
    Accepted { revision: DocumentRevision, effects: EffectSet }
  | Rejected { reason: TransactionError }
```

`DocumentRevision` is a monotonic counter. An accepted transaction advances it by exactly one. A rejected transaction does not advance it, does not enter history, does not clear redo history, and does not advance any allocator.

`EffectSet` names which aggregates changed. Projections use it to rebuild only what moved.

Rejection reasons must distinguish stale identity, missing owner, duplicate identity, invalid coordinate, invalid duration, unresolved reference, and identity exhaustion — distinctly enough for tests and for user-facing messaging.

## History contract

One project-level history owns every durable edit. Workspace panes do not own competing histories.

- A failed initial apply is not recorded.
- A successful new transaction clears redo history.
- Undo failure does not move the transaction to redo history; redo failure does not move it to done history.
- Undo restores exact identities, ordering, and state. It never allocates a replacement identity.
- Redo reapplies the same accepted mutation and restores original identities rather than allocating again.

Outside history, and therefore transient: transport playback position and run state, live audition, launcher performance state, and an in-progress recording. A successfully stopped recording commits as one atomic take transaction.

Durable transport settings are not transient. Loop region, punch points, metronome and count-in configuration are conductor state, edited through transactions and persisted. Only the moving playhead and live performance state sit outside the document.

Dirty state belongs to `meta` and is derived from the revision last saved. It is never tracked independently by a pane or a projection.

History is session state in this milestone and is not persisted.

## Identity contract

The document owns a non-cloneable allocator per durable domain. Domains are independent: exhausting one does not exhaust another.

- Every durable ID is an opaque nonzero newtype defined once in `spectre-core`.
- Allocation starts at 1, advances monotonically, and never reuses a value.
- Loading observes every stored ID and advances the allocator past it.
- Batch reservation is atomic; a partially available batch fails before any allocator advances.
- Abandoning a fully validated operation may leave monotonic gaps; it may never reuse an ID.
- Runtime handles (arena indices, compiled graph handles, generation ids) never appear in persistence or in the public edit boundary.

Domains required by this milestone: `TrackId`, `ClipId`, `AssetId`, `SceneId`, `DeviceId`, `ParamKey`, `RouteId`, `NoteId`, `MappingId`, `AutomationTargetId`. A domain may be declared and allocated before its aggregate content exists.

## Load and unresolved-reference contract

Loading builds a candidate document and validates it completely before the live document is touched.

Validation covers every identity domain, every asset reference, every clip, every graph and automation target, and every unresolved placeholder. It runs against the candidate, not against live state. A load either replaces the live document wholesale or leaves it untouched and reports exactly what failed. There is no partial load and no half-migrated document.

A reference whose subject is absent is unresolved, not invalid. The document holds unresolved assets, devices, plugins, modules, mappings, and automation targets losslessly:

- the complete original descriptor is preserved, never substituted with a default or an empty stand-in;
- unresolved entities remain visible, inspectable, and relinkable;
- the project saves with them intact and round-trips them exactly;
- resolved versus unresolved is derived from the current registry at read time, never persisted as a second source of truth.

An unresolved reference blocks only the behavior that actually needs its subject. It never blocks loading, editing, or saving the rest of the project.

## Projection contract

A projection is a read-only derived view stamped with the `DocumentRevision` it was built from.

- Projections are rebuilt from the document; they are never mutated in place as a source of truth.
- A projection stamped with a stale revision is either rebuilt or visibly marked stale; it is never silently trusted.
- UI emits typed intents that become transactions. UI never mutates durable state.
- Persistence serializes a projection of the document, not a renderer-facing mirror.
- Every projection has a parity test proving it reproduces the document facts it claims to show.

Provisional UI feedback during a drag remains disposable local state and never becomes a second authority. Pointer release submits one validated transaction.

A control reflects acknowledged canonical or realtime state. Behavior with no command behind it is disabled or visibly explained; it is never simulated locally so the surface appears to work. This applies to transport, routing, recording, and scene controls alike.

## Realtime publication contract

Publication is hybrid. This spec fixes the protocol; the *content* of a render generation stays whatever the engine needs today and is replaced by typed multi-rate routes in Milestone 3 without changing this protocol.

### Render generations

- A `RenderGeneration` is immutable, built entirely on the app thread, and identified by a monotonic `GenerationId`.
- Structural change — topology, device chain, clip set, track set — produces a new generation.
- Publication is a pointer swap the audio thread cannot block on.
- The audio thread publishes the `GenerationId` it is actually executing.
- The app treats a generation as live only once that acknowledgement is observed.
- A replaced generation, plugin instance, or audio asset is moved to a non-realtime reclaim queue and dropped there. Nothing is deallocated on the callback.

### Control stream

- High-rate change — parameter values, note events, transport commands — travels a bounded timestamped stream carrying a monotonic sequence number and a sample offset within the block.
- The audio thread publishes the highest sequence number it has applied.
- Every send result is consumed. There is no ignored saturation bool anywhere in the path.

### Saturation and reconciliation

When the stream saturates or a publication is rejected:

- the app marks the affected state unacknowledged;
- the app does not advance UI or project mirrors past the acknowledged point;
- reconciliation is explicit — the app republishes, escalates to a new generation, or surfaces the failure;
- silent divergence between document state and audible state is a defect, not a degraded mode.

### Callback rules

No allocation, deallocation, locking, logging, file I/O, UI work, or mutable document traversal occurs on the audio callback. This is enforced by allocator guards in debug builds, overload tests, graph-swap stress tests, and callback benchmarks against the 48 kHz / 128-frame baseline and the 64-frame stress mode.

## Compatibility window and deletion criteria

Each legacy owner is deleted only when all four hold for its domain:

1. `ProjectDocument` owns the domain and every mutation runs through a transaction.
2. Projection parity tests pass against the document.
3. Persistence round-trips the domain through the document with fixtures.
4. No writer to the legacy type remains outside the document.

Until all four hold, the legacy type may be read as a projection but must not be written except through the document. Pre-v1 prototype format breaks are permitted with explicit diagnostics and fixture-tested conversion. Loads never report success after discarding durable state.

## Non-goals

Deferred to their own specs and interviews: launcher content and launch quantization, warp and tempo interpretation, MPE and tuning, automation content and evaluation precedence, hybrid track internals, typed multi-rate route compilation, UI gesture routing, take lanes and comping, VST3 hosting.

## Acceptance criteria

1. `ProjectDocument` is the only type that owns durable project truth; every other holder is a projection or is deleted.
2. Durable mutation is impossible outside a transaction.
3. A rejected transaction leaves the document, history, revision, and every allocator unchanged.
4. An accepted transaction advances `DocumentRevision` by exactly one and reports which aggregates changed.
5. Undo and redo restore exact identities, ordering, and state without allocating replacements.
6. No durable reference depends on vector position, arena handle, or raw untyped integer.
7. Every projection is revision-stamped and covered by a parity test.
8. The audio thread acknowledges both the executing generation and the applied control sequence, and the app never advances mirrors past acknowledgement.
9. Stream saturation and rejected publication produce explicit reconciliation, never silent divergence.
10. Retired generations, plugin instances, and assets are dropped off the callback.
11. `geist-document` depends only on `spectre-core`.
12. Each legacy authority is deleted only after its four deletion criteria hold.
13. A load validates the complete candidate document before touching live state; a failed load leaves the live document unchanged and reports exactly what failed.
14. Unresolved assets, devices, plugins, modules, mappings, and targets round-trip losslessly, stay relinkable, and never block editing or saving the rest of the project.
15. Controls reflect acknowledged canonical or realtime state; behavior with no command behind it is disabled or explained rather than simulated locally.

## Slice boundaries

- **D1 — Crate skeleton and vocabulary relocation.** Create `geist-document`; move shared vocabulary to `spectre-core`; re-export from `geist-timeline`. No behavior change.
- **D2 — Document skeleton, revision, transactions, history.** Typed empty aggregates, transaction and result types, history with its failure semantics.
- **D3 — Arrangement domain onto the document.** Canonical arrangement under document ownership; create, delete, move, cross-track move, right resize as transactions. This un-pauses canonical-clip-commands Slice C with the correct owner.
- **D4 — Projection contract and arrangement UI projection.** Revision-stamped projections; UI reads projections and emits intents; parity tests.
- **D5 — Publication protocol.** Render generations with acknowledged `GenerationId`, sequenced control stream with applied-sequence acknowledgement, saturation reconciliation, reclaim queue, callback guards and benchmarks.
- **D6 — Persistence projection.** `spectre-project` serializes the document; versioned schema; atomic candidate-then-replace load validation; project package layout; migration fixtures.
- **D7 — Remaining domains.** Tracks and devices, assets, conductor, mappings absorbed as sub-slices, each with its own content sub-spec. Each sub-slice enforces the unresolved-reference contract for its own domain.
- **D8 — Delete legacy authorities.** Remove `StudioSession`, `app::engine::Arrangement` as authority, `geist_timeline::Timeline`, and UI-owned durable state once deletion criteria hold.

## Stop conditions

Stop before the next slice when a domain's content sub-spec is missing, a migration would change persistence without fixtures, publication would allocate or lock on the callback, undo cannot restore exact identity, or independent review has unresolved findings.
