// Author: Jeff
// Date: 2026-08-09
// Description: Callback bridge driving the existing CompiledPlan from a backend render callback
// Notes: This is the only render path. It executes the same immutable CompiledPlan the offline
//   harness renders, so live output and offline output are the same computation. Every method
//   reachable from `render` inherits RT-001: no allocation, no locks, no I/O, no logging, no
//   panics. Note scratch is preallocated and pushes stay inside its capacity, so notes that do
//   not fit stay queued for the next block rather than being dropped.

use crate::clip::{contract_order, ClipBlockOutcome, ClipPlayer, CLIP_SEQUENCE_BAND};
use crate::control::ControlReceiver;
use crate::RenderBlock;
use spectre_core::{ObjectId, SampleDuration, SampleTime, Transport};
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
    // R4-5 clip playback
    clip_events_refused: AtomicU64,
    loop_segments_refused: AtomicU64,
    schedules_installed: AtomicU64,
    schedules_held: AtomicU64,
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
            clip_events_refused: AtomicU64::new(0),
            loop_segments_refused: AtomicU64::new(0),
            schedules_installed: AtomicU64::new(0),
            schedules_held: AtomicU64::new(0),
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

    // Count blocks whose clip events would have exceeded CLIP_EVENT_RESERVE
    pub fn clip_events_refused(&self) -> u64 {
        self.clip_events_refused.load(Ordering::Relaxed)
    }

    // Count blocks that spanned more than MAX_BLOCK_SEGMENTS
    pub fn loop_segments_refused(&self) -> u64 {
        self.loop_segments_refused.load(Ordering::Relaxed)
    }

    // Count schedules the render thread installed
    pub fn schedules_installed(&self) -> u64 {
        self.schedules_installed.load(Ordering::Relaxed)
    }

    // Count retired schedules the reclaim lane refused, which the render thread must keep holding
    pub fn schedules_held(&self) -> u64 {
        self.schedules_held.load(Ordering::Relaxed)
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
    // Clip playback for the single note node. None means this bridge behaves exactly as it did
    // before R4-5, which is what keeps every existing caller of `new` correct
    player: Option<ClipPlayer>,
    // A retired schedule the reclaim lane refused. Held rather than dropped: running its
    // destructor here would deallocate on the audio thread and violate RT-001. Typed as the
    // lane's own opaque box, because the render thread never needs to look inside it again
    held: Option<crate::control::RetiredState>,
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
            player: None,
            held: None,
        }
    }

    // Attach clip playback to the bridge's single note node. Consuming rather than mutating, so a
    // bridge without a player cannot acquire one after the stream has started
    pub fn with_clip_player(mut self, player: ClipPlayer) -> Self {
        self.player = Some(player);
        self
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

        // Step 3: install a pending schedule before the transport moves, so the block that
        // installs it is also the block that releases what the outgoing one left sounding
        self.install_pending_schedule();
        // Step 4: the position before commands are applied, so a Seek is detectable as a
        // discontinuity rather than inferred after the fact
        let position_before = self.transport.position;
        self.apply_transport();

        // Steps 6 and 7: clip events first, into the cleared scratch
        self.notes.clear();
        self.emit_clip_block(position_before, frames);
        // Step 8: lane events append to what the player wrote; clearing here would discard it
        self.collect_notes();
        // Steps 9 and 10 apply only when a player is attached, so a bridge without one behaves
        // exactly as it did before this slice. That is not a convenience: R4-1's accepted
        // evidence pins that a Play and a Stop landing in one block are refused into counted
        // silence, and that refusal comes from the lane's own order. Sorting them would put the
        // all-notes-off before the note-on by rank and leave the note sounding after Stop — a
        // stuck note where there was a clean refusal. The merge rule exists for a second
        // producer, and with no second producer there is nothing to merge
        if self.player.is_some() {
            // Step 9: one array, sorted once, by the same tuple ProcessContext::new validates.
            // sort_unstable_by does not allocate, and the key is a total order because `sequence`
            // is unique within the block, so an unstable sort is still deterministic
            self.notes.sort_unstable_by(contract_order);
            // Step 10: the playhead moves before the plan runs, so a plan error costs that
            // block's material with the playhead already past it — a gap, not a re-emitted stutter
            self.transport.advance(SampleDuration::new(frames as u64));
        }

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

    // Take one queued schedule, if any, and hand the retired one to the reclaim lane.
    // Callback-safe: a pop, a pointer swap, and a push. Nothing is dropped here
    fn install_pending_schedule(&mut self) {
        let Some(player) = self.player.as_mut() else {
            return;
        };
        // A schedule held from a previous block goes first: the reclaim lane may have room now,
        // and holding two would need a second slot this bridge does not have
        if let Some(held) = self.held.take() {
            if let Err(returned) = self.control.retire(held) {
                self.held = Some(returned);
                return;
            }
        }
        let Some(next) = self.control.next_schedule() else {
            return;
        };
        let retired = player.install(next);
        self.telemetry
            .schedules_installed
            .fetch_add(1, Ordering::Relaxed);
        if let Err(returned) = self.control.retire(retired) {
            self.telemetry
                .schedules_held
                .fetch_add(1, Ordering::Relaxed);
            self.held = Some(returned);
        }
    }

    // Emit this block's clip events into the scratch, releasing everything first when the
    // playhead moved discontinuously
    fn emit_clip_block(&mut self, position_before: SampleTime, frames: usize) {
        let Some(player) = self.player.as_mut() else {
            return;
        };
        // Step 6: a Seek moves the position without the block having advanced it, so anything
        // sounding belongs to material the playhead has left
        if self.transport.position != position_before {
            player.release_all();
        }
        let outcome =
            player.emit_block(&self.transport, frames, &mut self.notes, CLIP_SEQUENCE_BAND);
        match outcome {
            ClipBlockOutcome::Complete => {}
            ClipBlockOutcome::EventReserveExceeded => {
                self.telemetry
                    .clip_events_refused
                    .fetch_add(1, Ordering::Relaxed);
            }
            ClipBlockOutcome::SegmentLimitExceeded => {
                self.telemetry
                    .loop_segments_refused
                    .fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    // Move queued notes into the block scratch, leaving any excess queued.
    // Appends rather than clears: the clip player has already written this block's events, and
    // clearing here would discard every one of them
    fn collect_notes(&mut self) {
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
