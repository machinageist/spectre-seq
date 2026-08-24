// Author: Jeff
// Date: 2026-08-09
// Description: Callback bridge driving the existing CompiledPlan from a backend render callback
// Notes: This is the only render path. It executes the same immutable CompiledPlan the offline
//   harness renders, so live output and offline output are the same computation. Every method
//   reachable from `render` inherits RT-001: no allocation, no locks, no I/O, no logging, no
//   panics. Note scratch is preallocated and pushes stay inside its capacity, so notes that do
//   not fit stay queued for the next block rather than being dropped.

use crate::control::ControlReceiver;
use crate::RenderBlock;
use spectre_core::{ObjectId, Transport};
use spectre_dsp::NoteEvent;
use spectre_graph::{CompiledPlan, NodeId, PlanNoteInput};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

// Default depth of the per-block note scratch
pub const DEFAULT_NOTE_SCRATCH: usize = 256;

// Counters the render thread increments and the app thread reads
#[derive(Debug)]
pub struct BridgeTelemetry {
    blocks_rendered: AtomicU64,
    plan_errors: AtomicU64,
    frame_capacity_rejections: AtomicU64,
    notes_deferred: AtomicU64,
    parameters_pending: AtomicU64,
    parameters_applied: AtomicU64,
    // RT-003 containment republished from the plan so the app thread can read it
    contaminated_nodes: AtomicU64,
    denormals_flushed: AtomicU64,
    last_contaminated_node: AtomicU64,
    // Blocks whose render took at least as long as the audio they produced
    xruns: AtomicU64,
    // Headroom as f32 bits: 1.0 means the block cost nothing, 0.0 means it consumed its budget
    last_headroom_bits: AtomicU32,
    worst_headroom_bits: AtomicU32,
}

impl Default for BridgeTelemetry {
    // Worst-case headroom starts at infinity so the first block establishes the real minimum;
    // a zero default would silently look like a saturated callback
    fn default() -> Self {
        Self {
            blocks_rendered: AtomicU64::new(0),
            plan_errors: AtomicU64::new(0),
            frame_capacity_rejections: AtomicU64::new(0),
            notes_deferred: AtomicU64::new(0),
            parameters_pending: AtomicU64::new(0),
            parameters_applied: AtomicU64::new(0),
            contaminated_nodes: AtomicU64::new(0),
            denormals_flushed: AtomicU64::new(0),
            last_contaminated_node: AtomicU64::new(0),
            xruns: AtomicU64::new(0),
            last_headroom_bits: AtomicU32::new(f32::INFINITY.to_bits()),
            worst_headroom_bits: AtomicU32::new(f32::INFINITY.to_bits()),
        }
    }
}

impl BridgeTelemetry {
    // Count blocks the bridge has rendered
    pub fn blocks_rendered(&self) -> u64 {
        self.blocks_rendered.load(Ordering::Relaxed)
    }

    // Count blocks the plan refused to render
    pub fn plan_errors(&self) -> u64 {
        self.plan_errors.load(Ordering::Relaxed)
    }

    // Count blocks whose frame count fell outside the plan's preallocated capacity
    pub fn frame_capacity_rejections(&self) -> u64 {
        self.frame_capacity_rejections.load(Ordering::Relaxed)
    }

    // Count notes left queued because the block's scratch was full
    pub fn notes_deferred(&self) -> u64 {
        self.notes_deferred.load(Ordering::Relaxed)
    }

    // Count parameter changes observed but not delivered to a processor. Since R4-2 the plan
    // can accept them, so in a correctly wired build this stays zero for the life of the process
    pub fn parameters_pending(&self) -> u64 {
        self.parameters_pending.load(Ordering::Relaxed)
    }

    // Count parameter changes applied to a live processor
    pub fn parameters_applied(&self) -> u64 {
        self.parameters_applied.load(Ordering::Relaxed)
    }

    // Count node-quanta silenced by RT-003 containment
    pub fn contaminated_nodes(&self) -> u64 {
        self.contaminated_nodes.load(Ordering::Relaxed)
    }

    // Count samples flushed from denormal to signed zero
    pub fn denormals_flushed(&self) -> u64 {
        self.denormals_flushed.load(Ordering::Relaxed)
    }

    // Identify the most recent contaminated node, or None if containment never fired
    pub fn last_contaminated_node(&self) -> Option<ObjectId> {
        ObjectId::from_raw(self.last_contaminated_node.load(Ordering::Relaxed))
    }

    // Count blocks whose render consumed the whole time budget for the audio it produced
    pub fn xruns(&self) -> u64 {
        self.xruns.load(Ordering::Relaxed)
    }

    // Fractional headroom on the most recent block
    pub fn last_headroom(&self) -> f32 {
        f32::from_bits(self.last_headroom_bits.load(Ordering::Relaxed))
    }

    // Worst fractional headroom observed; negative means a block overran its budget
    pub fn worst_headroom(&self) -> f32 {
        f32::from_bits(self.worst_headroom_bits.load(Ordering::Relaxed))
    }
}

// Owns the compiled plan and drives it from the audio callback
pub struct RenderBridge {
    plan: CompiledPlan,
    control: ControlReceiver,
    // The one node that accepts note input in the v1 fixture chain
    note_node: NodeId,
    // Preallocated; pushes never exceed capacity, so rendering never allocates
    notes: Vec<NoteEvent>,
    sample_rate: f64,
    transport: Transport,
    telemetry: Arc<BridgeTelemetry>,
    // Built on the app thread before the stream opens; immutable for the bridge's life, so no
    // retired route table is ever handed to the reclaim lane
    routes: crate::route::ParameterRoutes,
}

impl RenderBridge {
    // Build a bridge over an already-compiled plan, with no parameter routing. Kept so the R3
    // evidence that calls it compiles unchanged rather than being rewritten for this slice
    pub fn new(
        plan: CompiledPlan,
        control: ControlReceiver,
        note_node: NodeId,
        sample_rate: f64,
        note_scratch: usize,
    ) -> Self {
        Self::with_parameter_routes(
            plan,
            control,
            note_node,
            sample_rate,
            note_scratch,
            crate::route::ParameterRoutes::empty(),
        )
    }

    // Build a bridge with a live parameter route table
    pub fn with_parameter_routes(
        plan: CompiledPlan,
        control: ControlReceiver,
        note_node: NodeId,
        sample_rate: f64,
        note_scratch: usize,
        routes: crate::route::ParameterRoutes,
    ) -> Self {
        Self {
            plan,
            control,
            note_node,
            notes: Vec::with_capacity(note_scratch.max(1)),
            sample_rate,
            transport: Transport::default(),
            telemetry: Arc::new(BridgeTelemetry::default()),
            routes,
        }
    }

    // Share the telemetry handle with the app thread
    pub fn telemetry(&self) -> Arc<BridgeTelemetry> {
        Arc::clone(&self.telemetry)
    }

    // Report the transport state the render thread has applied
    pub fn transport(&self) -> Transport {
        self.transport
    }

    // Render one block: drain control, execute the plan, interleave the result
    pub fn render(&mut self, block: &mut RenderBlock<'_>) {
        // Instant::now reads a monotonic clock through the vDSO/commpage: no syscall,
        // no allocation, no lock, so it is safe on the callback path
        let started = Instant::now();
        let frames = block.frames();
        if frames == 0 || frames > self.plan.max_frames() {
            block.fill_silence();
            self.telemetry
                .frame_capacity_rejections
                .fetch_add(1, Ordering::Relaxed);
            return;
        }

        self.apply_transport();
        self.collect_notes();

        // Parameter application runs once per block, before `process`, so the whole block sees
        // one coherent parameter set. Borrows are split by field before the call, so the closure
        // captures `plan` and `routes` rather than `self`, leaving `&mut self.control` free
        let plan = &mut self.plan;
        let routes = &self.routes;
        let mut applied = 0_u64;
        let mut unapplied = 0_u64;
        self.control.drain_parameters(|target, value| {
            match routes.resolve(target) {
                Some((node, key)) if plan.set_parameter(node, key, value).is_ok() => applied += 1,
                // Fail-closed: the previous value keeps rendering, nothing is logged, nothing panics
                _ => unapplied += 1,
            }
        });
        if applied > 0 {
            self.telemetry
                .parameters_applied
                .fetch_add(applied, Ordering::Relaxed);
        }
        if unapplied > 0 {
            self.telemetry
                .parameters_pending
                .fetch_add(unapplied, Ordering::Relaxed);
        }

        let inputs = [PlanNoteInput {
            node: self.note_node,
            events: &self.notes,
        }];
        if self
            .plan
            .process(self.sample_rate, frames, &inputs)
            .is_err()
        {
            block.fill_silence();
            self.telemetry.plan_errors.fetch_add(1, Ordering::Relaxed);
            return;
        }

        self.publish_containment();
        self.interleave(block, frames);
        self.telemetry
            .blocks_rendered
            .fetch_add(1, Ordering::Relaxed);
        self.publish_headroom(started, frames);
    }

    // Publish how much of this block's time budget the render left unused
    fn publish_headroom(&self, started: Instant, frames: usize) {
        let budget = frames as f64 / self.sample_rate;
        if budget <= 0.0 {
            return;
        }
        let spent = started.elapsed().as_secs_f64();
        let headroom = (1.0 - spent / budget) as f32;
        self.telemetry
            .last_headroom_bits
            .store(headroom.to_bits(), Ordering::Relaxed);
        // An xrun is a block that used its entire budget; the driver had nothing in reserve
        if headroom <= 0.0 {
            self.telemetry.xruns.fetch_add(1, Ordering::Relaxed);
        }
        let worst = f32::from_bits(self.telemetry.worst_headroom_bits.load(Ordering::Relaxed));
        if headroom < worst {
            self.telemetry
                .worst_headroom_bits
                .store(headroom.to_bits(), Ordering::Relaxed);
        }
    }

    // Republish the plan's RT-003 counters so the app thread can read them without a lock
    fn publish_containment(&self) {
        let stats = self.plan.containment();
        self.telemetry
            .contaminated_nodes
            .store(stats.contaminated_nodes, Ordering::Relaxed);
        self.telemetry
            .denormals_flushed
            .store(stats.denormals_flushed, Ordering::Relaxed);
        if let Some(node) = stats.last_contaminated {
            self.telemetry
                .last_contaminated_node
                .store(node.object_id().raw(), Ordering::Relaxed);
        }
    }

    // Apply every queued transport command to the render-thread transport
    fn apply_transport(&mut self) {
        while let Some(command) = self.control.next_transport() {
            self.transport.apply(command);
        }
    }

    // Move queued notes into the block scratch, leaving any excess queued
    fn collect_notes(&mut self) {
        self.notes.clear();
        while self.notes.len() < self.notes.capacity() {
            match self.control.next_note() {
                Some(event) => self.notes.push(event),
                None => return,
            }
        }
        // Scratch is full; whatever remains stays queued and is counted, never dropped
        if !self.control.notes_is_empty() {
            self.telemetry
                .notes_deferred
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    // Copy the plan's stereo output into the driver's interleaved buffer
    fn interleave(&self, block: &mut RenderBlock<'_>, frames: usize) {
        let Some(output) = self.plan.last_output() else {
            block.fill_silence();
            return;
        };
        let channels = block.channels() as usize;
        let samples = block.samples_mut();
        for frame in 0..frames {
            let base = frame * channels;
            for (channel, slot) in (0..channels).zip(base..base + channels) {
                // Channels beyond the plan's stereo pair repeat the last plan channel
                let source = output[channel.min(1)];
                samples[slot] = source[frame];
            }
        }
    }
}
