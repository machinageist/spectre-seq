<!--
Author: Jeff
Date: 2026-08-01
Description: The audio-thread contract as implemented, plus the mechanisms that keep it true.
Notes: Contributor law. Every rule cites the code that satisfies it or the gap where it does not.
-->

# Realtime Rules

This document is law for code that runs on the audio callback, and a map of where
that law is enforced today. It states current architecture, not aspiration; the
gaps are listed as gaps.

## What "the audio thread" means here

There is exactly one callback path in the shipping app:

```
cpal output callback closure           crates/spectre-audio-backend/src/cpal_backend.rs:182
  |- BlockBridge::render                crates/spectre-audio-backend/src/bridge.rs:49
      \- BlockProcessor::process_block   crates/spectre-audio-backend/src/bridge.rs:17
          \- SynthProcessor::process_block   app/geist-daw/src/engine.rs:575
```

The bridge exists because backends honor a requested buffer size only sometimes.
It owns a one-block carry so the processor always sees a fixed channel-major block
regardless of what the driver asks for.

Everything reachable from `process_block` is audio-thread code and is bound by
this document. Everything else is app-thread code. There is no third category.

The `BlockProcessor` trait states the contract at its definition
(`bridge.rs:11-13`): *must not allocate, lock, block, or panic*. That comment is
the short form of what follows.

## The prohibitions

On the callback, never:

1. **Allocate.** No `Vec::push` past capacity, no `Box::new`, no `String`, no
   `format!`, no `collect`, no `clone` of an owning container.
2. **Deallocate.** Dropping a value that owns heap memory is a `free` call. This
   includes dropping a retired graph, a plugin instance, or an `Arc` whose
   refcount reaches zero on the callback.
3. **Lock.** No `Mutex`, no `RwLock`, not even `try_lock`. A failed `try_lock`
   still needs a fallback path, and that path is a silent glitch.
4. **Block or wait.** No condvars, no channel `recv`, no sleeping, no spinning on
   another thread's progress.
5. **Perform I/O.** No file access, no sockets, no `println!`, no logging
   framework, no `dbg!`.
6. **Panic.** A panic on the audio thread unwinds through a C callback. Prefer a
   bounds-checked `get`/`get_mut` returning `None` over indexing.
7. **Run unbounded work.** Every loop on the callback must have a bound that does
   not depend on how long the app thread has been away.
8. **Traverse app-owned mutable state.** The document, the UI model, and the
   project file are app-thread property. The callback reads published snapshots.

Rule 8 is the one that generalizes: the audio thread does not share the app
thread's data structures. It receives ownership of things, or it reads immutable
published state.

## How the code satisfies these

### Preallocate on the app thread, cap on the audio thread

Every audio-thread buffer is sized once during construction and never grows:

| Bound | Value | Where |
| --- | --- | --- |
| `MAX_BLOCK_EVENTS` | 64 | `app/geist-daw/src/engine.rs:174` |
| `MAX_CLIPS_PER_TRACK` | 64 | `app/geist-daw/src/engine.rs:199` |
| `MAX_CLIP_NOTES` | 256 | `app/geist-daw/src/engine.rs:197` |
| `MAX_AUDIO_ASSETS` | 64 | `app/geist-daw/src/engine.rs:27` |
| `CAPTURE_RING_CAPACITY` | 65536 | `crates/spectre-audio-backend/src/stream.rs:15` |

Each has a matching `Vec::with_capacity` at construction, and each write site
checks the bound *before* pushing. `push_capped` (`engine.rs:167`) is the pattern:

```rust
fn push_capped(out: &mut Vec<NoteEvent>, event: NoteEvent) {
    if out.len() < out.capacity() {
        out.push(event);
    }
}
```

A push that would reallocate is dropped instead. Dropping an event is a bug you
can hear and then fix; a reallocation is a glitch you cannot reproduce.

The same discipline covers arrangement mutation, which *does* run on the callback
today because clip commands are drained there: `add_clip` (`engine.rs:322`),
`add_note` (`engine.rs:362`), and `add_audio_clip` (`engine.rs:274`) each test
their capacity bound before pushing, and the backing `Vec`s were built with
exactly that capacity at `engine.rs:230,268-269`.

### Move ownership across SPSC rings; do not share

All app-to-audio traffic crosses an `rtrb` single-producer/single-consumer ring.
Ownership moves; nothing is shared and nothing is refcounted on the callback.

- **Control** — `EngineCommand` (`app/geist-daw/src/control.rs:30`), drained at
  the top of `process_block`. `EngineCommand` is `Copy`, so draining it cannot
  allocate or drop.
- **Assets** — recorded audio arrives as an `AudioAsset` whose `Arc<[f32]>` was
  built on the app thread. The callback stores the pointer into a fixed slot; it
  never constructs or drops the buffer.
- **Capture** — the input ring drops its tail rather than blocking when full
  (`stream.rs:25`). A full capture ring loses samples; it never stalls the driver.
- **Graph** — `crates/spectre-graph/src/swap.rs` moves a compiled `Executor`,
  node state and buffer pool included, to the audio thread and moves the retired
  one back. See ADR 002.

### Never drop on the callback

The graph swap is the reference implementation of rule 2. `ActiveGraph::poll_swap`
(`swap.rs:67`) refuses to adopt a new executor unless the return ring already has
room for the one it would displace:

```rust
if self.to_app.slots() == 0 {
    return false;
}
```

Check capacity, *then* swap. The retired executor rides the return ring to the app
thread and is freed there by `GraphPublisher::reclaim()`. A full return ring
degrades to "run the current graph one more block," which is always safe. It never
degrades to a `free` on the callback.

Every future owner of heap state on the audio path — plugin instances, render
generations, streamed assets — uses this shape. The reclaim queue in
`docs/changes/project-document/SPEC.md` slice D5 generalizes it rather than
replacing it.

### Count errors, do not report them

The cpal error callback increments an atomic and returns (`cpal_backend.rs:185`,
`XrunCounter` at `stream.rs:65`). Meters, the beat clock, and scope samples
publish the same way: an atomic store the UI polls. Nothing on the audio path
formats a string or takes a logging lock.

### Deny unsafe by default

Every workspace crate carries `#![deny(unsafe_code)]` except the three plugin-host
crates — `spectre-clap-host`, `spectre-lv2-host`, `geist-vst-host` — where FFI makes
it unavoidable. Unsafe in those crates belongs behind a wrapper with its invariants
written down, not at the call site.

## Known gaps

These are real and currently unenforced. Do not read the section above as a claim
that the rules are mechanically guaranteed.

1. **No allocator guard.** Nothing detects an allocation on the audio thread in a
   debug build. The rules are enforced by review, not by tooling. A guarding
   allocator is scoped in `docs/changes/project-document/SPEC.md` slice D5.
2. **Saturation results are discarded.** `EngineControl::send`
   (`app/geist-daw/src/control.rs:225`) returns `false` when the ring is full, and
   every one of its 103 call sites ignores it. The app therefore believes commands
   landed that the engine never received. The acknowledged-publication protocol in
   slice D5 exists to make this divergence impossible; until then it is a known
   defect, not a tolerated design.
3. **No realtime performance fixtures.** There is no reproducible measurement
   against the 48 kHz / 128-frame baseline or the 64-frame stress mode. The
   criterion benches (`crates/spectre-graph/benches/graph_bench.rs`) cover graph
   compile and swap, not end-to-end callback headroom. Roadmap Milestone 3.
4. **The compiled graph is not on the audio path.** `app/geist-daw/src/engine.rs`
   imports only `spectre_graph::node::AudioNode` and runs a hand-wired fixed
   three-track chain. The executor and swap are exercised only by their own tests
   and bench. Rules 1-8 apply to the hand-wired path today and to the compiled
   path when Milestone 3 wires it in.
5. **Arrangement edits mutate engine state on the callback.** Bounded and
   capacity-checked, so it is not a rule violation, but it is the wrong shape: the
   audio thread should adopt a published arrangement, not apply edits. Slice D5
   replaces the mutation path with generation publication.

## Reviewing a change

Ask, in order:

1. Is any new code reachable from `SynthProcessor::process_block`?
2. If yes, does it allocate, drop, lock, block, do I/O, or panic — including
   inside anything it calls?
3. Does every new loop have a bound independent of app-thread scheduling?
4. Does every new buffer have a `with_capacity` at construction and a capacity
   check at its write site?
5. Does anything that owns heap memory cross to the audio thread without a return
   path for its destructor?

A "no" to 4 or a "yes" to 5 blocks the change.

## See also

- ADR 002 — why graph publication moves ownership instead of sharing it
- `docs/architecture.md` — the full audio path as built and what is not wired
- `docs/changes/project-document/SPEC.md` — the publication and reclaim contract
