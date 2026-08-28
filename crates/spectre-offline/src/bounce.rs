// Author: Jeff
// Date: 2026-08-25
// Description: Deterministic multi-quantum offline render with a localizable equality proof
// Notes: Runs on the caller's thread and may allocate during setup, never inside the block loop.
//   Nothing here is reachable from an audio callback: the bounce builds its own plan, holds no
//   reference to the live one, and every write happens on this side of CompiledPlan::process.

use crate::fixture;
use crate::hash::{hash_block, SampleHasher};
use crate::wav;
use crate::DeviceValues;
use serde::Serialize;
use spectre_dsp::{DeviceParameterSnapshot, NoteEvent};
use spectre_graph::{GraphError, PlanError, PlanNoteInput};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

// Fallback bounce sample rate when no live engine exists to take it from.
// Rationale row BOUNCE-001 in docs/01-requirements/requirements-ledger.md. Deliberately NOT taken
// from the macOS qualification row, which has no sample-rate column and cannot justify it. What
// justifies it is Spectre's own render corpus: every render gate in this workspace runs at
// 48 000 Hz. The constant exists only so a bounce can run with no engine present, and it carries
// the corpus's value so a no-engine bounce and the workspace's own gates do not silently differ.
// Re-open now that R4-1 has landed AudioBackend::default_sample_rate, which supersedes it
// wherever an engine is actually open
pub const BOUNCE_FALLBACK_SAMPLE_RATE: f64 = 48_000.0;

// Fallback bounce quantum when no live engine exists to take it from.
// Rationale row BOUNCE-002. Not a free choice: RT-003 containment silences a whole render
// quantum, so live and offline agree on a contaminated render only at equal block size, which is
// why the live block size is a required input rather than a preference. The value is R4-1's own
// ENGINE_BUFFER_FRAMES, carried here so the two paths do not silently differ when both default.
// Re-open when R4-3's Linux qualification produces a second measurement
pub const BOUNCE_FALLBACK_BLOCK_FRAMES: usize = 256;

// Refusal ceiling on bounce length, in seconds.
// Rationale row BOUNCE-003. Derived from Spectre's own accepted time horizon, not from any
// product: TIME-003 commits the tempo map to projects of at least 24 hours, so 24 hours is the
// longest render the accepted requirements contemplate. The bound's real cost is the per-block
// hash log: 86 400 s x 48 000 Hz = 4 147 200 000 frames; / 256 = 16 200 000 blocks; x 8 B =
// 129.6 MB. Bounded and linear in render length, which is what makes the ceiling honest rather
// than arbitrary — but 129.6 MB is not free, so the log follows its only consumer rather than
// being unconditionally on. For scale it is 1/256 of what the same render writes to disk
pub const BOUNCE_MAX_SECONDS: u32 = 24 * 60 * 60;

// Channels the fixture chain produces; the plan's own output width, not a choice made here
const BOUNCE_CHANNELS: usize = 2;

// The ceiling in samples at a given rate. A derived quantity, not a fourth bound: it introduces
// no number of its own and gets no ledger row
pub fn max_frames(sample_rate: f64) -> usize {
    (f64::from(BOUNCE_MAX_SECONDS) * sample_rate).max(0.0) as usize
}

// What to render and how. Sample rate and block size are inputs, never defaults chosen here
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BounceConfig {
    pub sample_rate: f64,
    pub frames: usize,
    // Must equal the live path's block size when a comparison is intended
    pub block_frames: usize,
    // Retain a per-block hash so a mismatch localizes without holding samples. Follows the
    // live/offline comparison rather than defaulting on: that comparison is the log's only
    // consumer, and at this file's own ceiling the log is 129.6 MB
    pub log_block_hashes: bool,
}

// BounceConfig has no Default and must not gain one. Three of its four fields have no defensible
// default: frames comes from the project's length, and sample_rate and block_frames are taken
// from the live engine. A Default would contradict that and hide the one case where a fallback is
// legitimate. That case gets a named constructor instead, so the fallback is visible at the call site
pub fn fallback_config(frames: usize) -> BounceConfig {
    BounceConfig {
        sample_rate: BOUNCE_FALLBACK_SAMPLE_RATE,
        frames,
        block_frames: BOUNCE_FALLBACK_BLOCK_FRAMES,
        log_block_hashes: false,
    }
}

// Deterministic summary of one completed bounce
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BounceReport {
    pub frames: usize,
    pub channels: usize,
    pub sample_rate: f64,
    pub block_frames: usize,
    pub blocks: usize,
    pub peak: f32,
    // Frame-major streaming hash over the whole render. NOT comparable to RenderReport.hash,
    // which is channel-major over one quantum
    pub hash: u64,
    // One hash per block when BounceConfig::log_block_hashes; empty otherwise
    pub block_hashes: Vec<u64>,
    // RT-003 activity, copied from the plan's containment after the last block
    pub contaminated_nodes: u64,
    pub denormals_flushed: u64,
    pub last_contaminated: Option<u64>,
}

// Where a bounce and a live render first disagree, in the terms a defect report needs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Divergence {
    pub block: usize,
    pub frame_in_block: usize,
    pub absolute_frame: usize,
    pub channel: usize,
    // Compared as bits, never as f32: RT-003 flushes denormals to *signed* zero, so -0.0 against
    // +0.0 is real engine state, and NaN != NaN would make a float comparison report a false
    // match on equal payloads
    pub live_bits: u32,
    pub offline_bits: u32,
}

// Bounce failure, with the block index a mid-render failure needs
#[derive(Debug)]
pub enum BounceError {
    InvalidConfig(&'static str),
    LengthOutOfRange { frames: usize, max_frames: usize },
    Graph(GraphError),
    Plan { block: usize, error: PlanError },
    Write { block: usize, error: std::io::Error },
    Cancelled { blocks_written: usize },
}

impl std::fmt::Display for BounceError {
    // Render an actionable diagnostic; never called on a render path
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidConfig(reason) => write!(formatter, "invalid bounce config: {reason}"),
            Self::LengthOutOfRange { frames, max_frames } => write!(
                formatter,
                "{frames} frames exceeds the {max_frames}-frame ceiling"
            ),
            Self::Graph(error) => write!(formatter, "graph: {error}"),
            Self::Plan { block, error } => write!(formatter, "block {block}: {error}"),
            Self::Write { block, error } => {
                write!(formatter, "block {block}: write failed: {error}")
            }
            Self::Cancelled { blocks_written } => {
                write!(formatter, "cancelled after {blocks_written} blocks")
            }
        }
    }
}

impl std::error::Error for BounceError {}

// Two atomics the worker stores into and the app thread loads; no lock either way
#[derive(Debug, Default)]
pub struct BounceProgress {
    blocks_done: AtomicU64,
    blocks_total: AtomicU64,
}

impl BounceProgress {
    pub fn blocks_done(&self) -> u64 {
        self.blocks_done.load(Ordering::Relaxed)
    }

    pub fn blocks_total(&self) -> u64 {
        self.blocks_total.load(Ordering::Relaxed)
    }
}

// Render `config.frames` frames of the fixture chain to `sink`, block by block
pub fn bounce_into<W: std::io::Write>(
    config: BounceConfig,
    values: &[DeviceParameterSnapshot],
    events: &[NoteEvent],
    sink: &mut W,
    cancel: &AtomicBool,
    progress: &BounceProgress,
) -> Result<BounceReport, BounceError> {
    run(config, values, events, Some(sink), cancel, progress)
}

// Render without writing anywhere; the equivalence tests and the CLI's report-only mode use this
pub fn bounce_report(
    config: BounceConfig,
    values: &[DeviceParameterSnapshot],
    events: &[NoteEvent],
) -> Result<BounceReport, BounceError> {
    let cancel = AtomicBool::new(false);
    let progress = BounceProgress::default();
    run::<std::io::Sink>(config, values, events, None, &cancel, &progress)
}

// The one block loop. A sink of None renders without writing, so the two entry points cannot
// diverge in what they compute
fn run<W: std::io::Write>(
    config: BounceConfig,
    values: &[DeviceParameterSnapshot],
    events: &[NoteEvent],
    mut sink: Option<&mut W>,
    cancel: &AtomicBool,
    progress: &BounceProgress,
) -> Result<BounceReport, BounceError> {
    validate(config)?;
    let device_values = DeviceValues::from_snapshot(values).map_err(|_| {
        BounceError::InvalidConfig("bounce requires the complete four-parameter fixture snapshot")
    })?;

    // Compilation allocates, so it runs once, before the loop, and never inside it
    let (mut plan, note_node) =
        fixture::compile_fixture_plan_with(config.block_frames, device_values)
            .map_err(|_| BounceError::InvalidConfig("fixture chain failed to compile"))?;

    let blocks = config.frames.div_ceil(config.block_frames);
    progress
        .blocks_total
        .store(blocks as u64, Ordering::Relaxed);
    progress.blocks_done.store(0, Ordering::Relaxed);

    let mut hasher = SampleHasher::new();
    let mut block_hashes = if config.log_block_hashes {
        Vec::with_capacity(blocks)
    } else {
        Vec::new()
    };
    let mut interleaved = vec![0.0_f32; config.block_frames * BOUNCE_CHANNELS];
    let mut peak = 0.0_f32;
    let mut remaining_events = events;
    // Rebased copies of one block's events. Allocated once, cleared per block: the offline path
    // is not the callback, but a per-block allocation would still make the loop's cost depend on
    // how the timeline happens to be chopped
    let mut block_events: Vec<NoteEvent> = Vec::with_capacity(events.len());

    if let Some(sink) = sink.as_deref_mut() {
        wav::write_header(
            sink,
            BOUNCE_CHANNELS as u16,
            config.sample_rate as u32,
            config.frames,
        )
        .map_err(|error| BounceError::Write { block: 0, error })?;
    }

    for block in 0..blocks {
        if cancel.load(Ordering::Relaxed) {
            return Err(BounceError::Cancelled {
                blocks_written: block,
            });
        }
        // The last block is short when frames is not a multiple of the quantum. It renders at
        // the full quantum and only its tail is trimmed, so the plan's per-quantum containment
        // and the live path's see identical block geometry
        let produced = (config.frames - block * config.block_frames).min(config.block_frames);

        // Callers describe the timeline in absolute frames; a plan only ever sees offsets inside
        // the quantum it is rendering. Events are consumed in order, each block taking the prefix
        // that falls inside it and rebasing it onto that block's origin. Handing an un-rebased
        // offset to ProcessContext::new is refused, so a missed rebase surfaces as
        // BounceError::Plan rather than as audio that silently lands in the wrong place
        let block_start = block * config.block_frames;
        let block_end = block_start + config.block_frames;
        let split = remaining_events
            .iter()
            .position(|event| event.frame_offset >= block_end)
            .unwrap_or(remaining_events.len());
        let (this_block, rest) = remaining_events.split_at(split);
        remaining_events = rest;
        block_events.clear();
        block_events.extend(this_block.iter().map(|event| NoteEvent {
            frame_offset: event.frame_offset.saturating_sub(block_start),
            ..*event
        }));

        plan.process(
            config.sample_rate,
            config.block_frames,
            &[PlanNoteInput {
                node: note_node,
                events: &block_events,
            }],
        )
        .map_err(|error| BounceError::Plan { block, error })?;

        let output = plan
            .last_output()
            .ok_or(BounceError::InvalidConfig("the plan produced no quantum"))?;
        for frame in 0..produced {
            for (channel, plane) in output.iter().enumerate().take(BOUNCE_CHANNELS) {
                let sample = plane[frame];
                peak = peak.max(sample.abs());
                interleaved[frame * BOUNCE_CHANNELS + channel] = sample;
            }
        }

        let used = produced * BOUNCE_CHANNELS;
        hash_block(&mut hasher, &interleaved[..used], BOUNCE_CHANNELS, produced);
        if config.log_block_hashes {
            // Each entry is the fold of that block alone, so a mismatch names one block rather
            // than everything from the first divergence onward
            let mut per_block = SampleHasher::new();
            hash_block(
                &mut per_block,
                &interleaved[..used],
                BOUNCE_CHANNELS,
                produced,
            );
            block_hashes.push(per_block.finish());
        }

        if let Some(sink) = sink.as_deref_mut() {
            wav::write_block(sink, &interleaved[..used])
                .map_err(|error| BounceError::Write { block, error })?;
        }
        progress
            .blocks_done
            .store(block as u64 + 1, Ordering::Relaxed);
    }

    let containment = plan.containment();
    Ok(BounceReport {
        frames: config.frames,
        channels: BOUNCE_CHANNELS,
        sample_rate: config.sample_rate,
        block_frames: config.block_frames,
        blocks,
        peak,
        hash: hasher.finish(),
        block_hashes,
        contaminated_nodes: containment.contaminated_nodes,
        denormals_flushed: containment.denormals_flushed,
        last_contaminated: containment
            .last_contaminated
            .map(|node| node.object_id().raw()),
    })
}

// Refuse a config the loop could not execute, before anything is compiled or written
fn validate(config: BounceConfig) -> Result<(), BounceError> {
    if !config.sample_rate.is_finite() || config.sample_rate <= 0.0 {
        return Err(BounceError::InvalidConfig("sample rate must be positive"));
    }
    if config.block_frames == 0 {
        return Err(BounceError::InvalidConfig("block size must be positive"));
    }
    if config.frames == 0 {
        return Err(BounceError::InvalidConfig("length must be positive"));
    }
    let ceiling = max_frames(config.sample_rate);
    if config.frames > ceiling {
        return Err(BounceError::LengthOutOfRange {
            frames: config.frames,
            max_frames: ceiling,
        });
    }
    Ok(())
}

// Locate the first bit-level disagreement between two equal-length interleaved streams.
// Diagnostic only; both streams must already be materialized, which is why this is a tooling
// surface and not part of the shipping block loop
pub fn first_divergence(
    live: &[f32],
    offline: &[f32],
    channels: usize,
    block_frames: usize,
) -> Option<Divergence> {
    if channels == 0 || block_frames == 0 {
        return None;
    }
    let shared = live.len().min(offline.len());
    for index in 0..shared {
        // Bits, not floats: RT-003 flushes denormals to signed zero, so -0.0 against +0.0 is
        // real engine state, and NaN != NaN would report a false match on equal payloads
        let (live_bits, offline_bits) = (live[index].to_bits(), offline[index].to_bits());
        if live_bits == offline_bits {
            continue;
        }
        let absolute_frame = index / channels;
        return Some(Divergence {
            block: absolute_frame / block_frames,
            frame_in_block: absolute_frame % block_frames,
            absolute_frame,
            channel: index % channels,
            live_bits,
            offline_bits,
        });
    }
    None
}
