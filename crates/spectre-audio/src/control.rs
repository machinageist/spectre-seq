// Author: Jeff
// Date: 2026-08-09
// Description: RT-002 control→render transport with the decision-21 split-lane overflow policy
// Notes: Three lanes with deliberately different policies. Parameters are latest-wins per target,
//   so an arbitrarily fast knob sweep occupies one slot and can never starve the queue. Notes
//   and transport are strict FIFO and are never dropped; overflow on those lanes is a counted
//   defect, surfaced off-thread, not silent loss. Retired render state travels back to the app
//   thread on a reclaim lane so the audio thread never runs a destructor.

use crate::spsc::{bounded, Consumer, Producer};
use spectre_core::{ObjectId, TransportCommand};
use spectre_dsp::NoteEvent;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;

// Default lane depths; notes are deepest because they are the lane that must never drop
pub const DEFAULT_NOTE_CAPACITY: usize = 1_024;
pub const DEFAULT_TRANSPORT_CAPACITY: usize = 64;
pub const DEFAULT_RECLAIM_CAPACITY: usize = 32;

// Largest parameter-target set a control channel will register.
//
// Rationale (decision 16, PROD-003 — this bound is Spectre's own and is not taken from any
// reference product). It exists because ParameterReader::drain walks every registered slot on
// every block, so without a cap the per-block cost of the callback path grows without limit as a
// project grows, which RT-001's "bounded" clause does not permit. The value is deliberately
// generous rather than tuned, because Spectre has no measurement that would justify a tuned
// value and a generous bound still discharges the obligation that the number be bounded at all.
// What is computable from source is the memory: one target costs a 16-byte ParameterTarget, an
// 8-byte ParameterSlot, a 4-byte `seen` entry, and a 40-byte ParameterRoute, so 1_024 targets
// reserve 69,632 B — negligible against the plan's own channel pool. R4's complete accepted
// device scope registers four. Re-open when a real project approaches the cap, or when a
// hardware run produces the first drain-cost measurement
pub const MAX_PARAMETER_TARGETS: usize = 1_024;

// Identity of one automatable parameter on one device instance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ParameterTarget {
    pub device: ObjectId,
    pub parameter: ObjectId,
}

// App-thread control failure; never constructed on the render path
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlError {
    UnknownTarget(ParameterTarget),
    TargetIndexOutOfRange(usize),
    DuplicateTarget(ParameterTarget),
    // Registered target set exceeds the callback path's bounded-work budget
    TooManyTargets(usize),
    // Strict-FIFO lane overflowed; this is a defect, not a normal outcome
    NoteLaneFull,
    TransportLaneFull,
}

impl std::fmt::Display for ControlError {
    // Render an actionable app-thread diagnostic
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownTarget(target) => write!(
                f,
                "unknown parameter target device {} parameter {}",
                target.device.raw(),
                target.parameter.raw()
            ),
            Self::TargetIndexOutOfRange(index) => {
                write!(f, "parameter target index {index} out of range")
            }
            Self::DuplicateTarget(target) => write!(
                f,
                "duplicate parameter target device {} parameter {}",
                target.device.raw(),
                target.parameter.raw()
            ),
            Self::TooManyTargets(count) => write!(
                f,
                "{count} parameter targets exceeds the bounded budget of {MAX_PARAMETER_TARGETS}"
            ),
            Self::NoteLaneFull => write!(f, "note lane overflowed"),
            Self::TransportLaneFull => write!(f, "transport lane overflowed"),
        }
    }
}

impl std::error::Error for ControlError {}

// One latest-wins parameter cell; an f32 fits an AtomicU32 exactly, so writes never tear
struct ParameterSlot {
    bits: AtomicU32,
    version: AtomicU32,
}

// Fixed set of parameter targets with one latest-wins slot each
struct ParameterLane {
    targets: Box<[ParameterTarget]>,
    slots: Box<[ParameterSlot]>,
}

// App-thread half of the parameter lane
pub struct ParameterWriter {
    lane: Arc<ParameterLane>,
}

// Render-thread half of the parameter lane
pub struct ParameterReader {
    lane: Arc<ParameterLane>,
    // Last version applied per slot; allocated once, never resized
    seen: Box<[u32]>,
}

impl ParameterWriter {
    // Publish a value, overwriting any value the render thread has not yet observed
    pub fn set(&self, index: usize, value: f32) -> Result<(), ControlError> {
        let slot = self
            .lane
            .slots
            .get(index)
            .ok_or(ControlError::TargetIndexOutOfRange(index))?;
        // Bits first, then the version bump that publishes them
        slot.bits.store(value.to_bits(), Ordering::Release);
        slot.version.fetch_add(1, Ordering::Release);
        Ok(())
    }

    // Publish a value by target identity
    pub fn set_target(&self, target: ParameterTarget, value: f32) -> Result<(), ControlError> {
        let index = self
            .index_of(target)
            .ok_or(ControlError::UnknownTarget(target))?;
        self.set(index, value)
    }

    // Resolve a target to its dense slot index
    pub fn index_of(&self, target: ParameterTarget) -> Option<usize> {
        self.lane.targets.iter().position(|known| *known == target)
    }

    // Report the registered target set
    pub fn targets(&self) -> &[ParameterTarget] {
        &self.lane.targets
    }
}

impl ParameterReader {
    // Apply every target whose value changed since the last drain
    pub fn drain(&mut self, mut apply: impl FnMut(ParameterTarget, f32)) -> usize {
        let mut applied = 0;
        for (index, slot) in self.lane.slots.iter().enumerate() {
            // Version is read before bits, so a racing write is delivered next drain, never lost
            let version = slot.version.load(Ordering::Acquire);
            if version == self.seen[index] {
                continue;
            }
            let value = f32::from_bits(slot.bits.load(Ordering::Acquire));
            self.seen[index] = version;
            apply(self.lane.targets[index], value);
            applied += 1;
        }
        applied
    }

    // Report the registered target set
    pub fn targets(&self) -> &[ParameterTarget] {
        &self.lane.targets
    }
}

// Counters the render thread increments and the app thread reads
#[derive(Debug, Default)]
pub struct ControlTelemetry {
    note_overflows: AtomicU64,
    transport_overflows: AtomicU64,
    reclaim_overflows: AtomicU64,
}

impl ControlTelemetry {
    // Count note-lane overflows observed by the producer
    pub fn note_overflows(&self) -> u64 {
        self.note_overflows.load(Ordering::Relaxed)
    }

    // Count transport-lane overflows observed by the producer
    pub fn transport_overflows(&self) -> u64 {
        self.transport_overflows.load(Ordering::Relaxed)
    }

    // Count retired-state handoffs the app thread failed to accept
    pub fn reclaim_overflows(&self) -> u64 {
        self.reclaim_overflows.load(Ordering::Relaxed)
    }
}

// App-thread sending half of the whole control transport
pub struct ControlSender {
    parameters: ParameterWriter,
    notes: Producer<NoteEvent>,
    transport: Producer<TransportCommand>,
    // Retired render state arrives here and is dropped on the app thread
    reclaim: Consumer<RetiredState>,
    telemetry: Arc<ControlTelemetry>,
}

// Render-thread receiving half of the whole control transport
pub struct ControlReceiver {
    parameters: ParameterReader,
    notes: Consumer<NoteEvent>,
    transport: Consumer<TransportCommand>,
    reclaim: Producer<RetiredState>,
    telemetry: Arc<ControlTelemetry>,
}

// Owned state the render thread has retired and must not drop itself
pub type RetiredState = Box<dyn Send>;

// Build a control transport over a fixed parameter-target set
pub fn control_channel(
    targets: &[ParameterTarget],
    note_capacity: usize,
    transport_capacity: usize,
) -> Result<(ControlSender, ControlReceiver), ControlError> {
    if targets.len() > MAX_PARAMETER_TARGETS {
        return Err(ControlError::TooManyTargets(targets.len()));
    }
    for (index, target) in targets.iter().enumerate() {
        if targets[..index].contains(target) {
            return Err(ControlError::DuplicateTarget(*target));
        }
    }
    let mut slots = Vec::with_capacity(targets.len());
    slots.resize_with(targets.len(), || ParameterSlot {
        bits: AtomicU32::new(0.0_f32.to_bits()),
        version: AtomicU32::new(0),
    });
    let lane = Arc::new(ParameterLane {
        targets: targets.to_vec().into_boxed_slice(),
        slots: slots.into_boxed_slice(),
    });
    let telemetry = Arc::new(ControlTelemetry::default());
    let (note_tx, note_rx) = bounded::<NoteEvent>(note_capacity);
    let (transport_tx, transport_rx) = bounded::<TransportCommand>(transport_capacity);
    let (reclaim_tx, reclaim_rx) = bounded::<RetiredState>(DEFAULT_RECLAIM_CAPACITY);

    let sender = ControlSender {
        parameters: ParameterWriter {
            lane: Arc::clone(&lane),
        },
        notes: note_tx,
        transport: transport_tx,
        reclaim: reclaim_rx,
        telemetry: Arc::clone(&telemetry),
    };
    let receiver = ControlReceiver {
        parameters: ParameterReader {
            seen: vec![0; lane.slots.len()].into_boxed_slice(),
            lane,
        },
        notes: note_rx,
        transport: transport_rx,
        reclaim: reclaim_tx,
        telemetry,
    };
    Ok((sender, receiver))
}

impl ControlSender {
    // Borrow the latest-wins parameter writer
    pub fn parameters(&self) -> &ParameterWriter {
        &self.parameters
    }

    // Enqueue a note event; overflow is a counted defect, never a silent drop
    pub fn send_note(&mut self, event: NoteEvent) -> Result<(), ControlError> {
        match self.notes.push(event) {
            Ok(()) => Ok(()),
            Err(_rejected) => {
                self.telemetry
                    .note_overflows
                    .fetch_add(1, Ordering::Relaxed);
                Err(ControlError::NoteLaneFull)
            }
        }
    }

    // Enqueue a transport command; overflow is a counted defect, never a silent drop
    pub fn send_transport(&mut self, command: TransportCommand) -> Result<(), ControlError> {
        match self.transport.push(command) {
            Ok(()) => Ok(()),
            Err(_rejected) => {
                self.telemetry
                    .transport_overflows
                    .fetch_add(1, Ordering::Relaxed);
                Err(ControlError::TransportLaneFull)
            }
        }
    }

    // Drop retired render state on the app thread, returning how many were reclaimed
    pub fn reclaim(&mut self) -> usize {
        let mut count = 0;
        while let Some(retired) = self.reclaim.pop() {
            drop(retired);
            count += 1;
        }
        count
    }

    // Read the shared overflow counters
    pub fn telemetry(&self) -> &ControlTelemetry {
        &self.telemetry
    }
}

impl ControlReceiver {
    // Apply changed parameters for this block
    pub fn drain_parameters(&mut self, apply: impl FnMut(ParameterTarget, f32)) -> usize {
        self.parameters.drain(apply)
    }

    // Take the next queued note event
    pub fn next_note(&mut self) -> Option<NoteEvent> {
        self.notes.pop()
    }

    // Report whether any note remains queued
    pub fn notes_is_empty(&self) -> bool {
        self.notes.is_empty()
    }

    // Take the next queued transport command
    pub fn next_transport(&mut self) -> Option<TransportCommand> {
        self.transport.pop()
    }

    // Hand retired state to the app thread instead of dropping it here.
    // On overflow the value comes back to the caller rather than being dropped: running a
    // destructor here would deallocate on the audio thread and violate RT-001. The caller
    // must hold it until the next block, never drop it.
    pub fn retire(&mut self, state: RetiredState) -> Result<(), RetiredState> {
        match self.reclaim.push(state) {
            Ok(()) => Ok(()),
            Err(rejected) => {
                self.telemetry
                    .reclaim_overflows
                    .fetch_add(1, Ordering::Relaxed);
                Err(rejected)
            }
        }
    }

    // Read the shared overflow counters
    pub fn telemetry(&self) -> &ControlTelemetry {
        &self.telemetry
    }
}
