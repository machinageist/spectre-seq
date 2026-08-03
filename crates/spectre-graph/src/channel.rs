// =============================================================================
// File: crates/spectre-graph/src/channel.rs
// Layer: audio graph
// Purpose: rtrb ring buffer for UI-to-audio parameter changes
// Status: Implemented; metering uses the atomic MeterCell, not a ring.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use spectre_core::events::ParameterChange;
use rtrb::{Consumer, Producer, RingBuffer};

// App-thread end of the parameter channel
pub struct ParamProducer {
    inner: Producer<ParameterChange>,
}

// Audio-thread end of the parameter channel
pub struct ParamConsumer {
    inner: Consumer<ParameterChange>,
}

// Build a bounded lock-free SPSC parameter channel from the UI to the audio thread
pub fn param_channel(capacity: usize) -> (ParamProducer, ParamConsumer) {
    let (producer, consumer) = RingBuffer::new(capacity);
    (
        ParamProducer { inner: producer },
        ParamConsumer { inner: consumer },
    )
}

impl ParamProducer {
    // Enqueue a change; returns false when the queue is full and the change is dropped
    pub fn push(&mut self, change: ParameterChange) -> bool {
        self.inner.push(change).is_ok()
    }

    // Writable slots remaining before the queue is full
    pub fn free(&self) -> usize {
        self.inner.slots()
    }
}

impl ParamConsumer {
    // Pop the next pending change, oldest first; never blocks
    pub fn pop(&mut self) -> Option<ParameterChange> {
        self.inner.pop().ok()
    }

    // Count of changes waiting to be consumed
    pub fn len(&self) -> usize {
        self.inner.slots()
    }

    // Whether any change is pending
    pub fn is_empty(&self) -> bool {
        self.inner.slots() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use spectre_core::ids::ParamId;

    fn change(offset: u32, param: u64, value: f32) -> ParameterChange {
        ParameterChange {
            sample_offset: offset,
            param: ParamId::new(param),
            value,
        }
    }

    #[test]
    fn pushes_and_pops_in_order() {
        let (mut producer, mut consumer) = param_channel(4);
        assert!(consumer.is_empty());
        assert!(producer.push(change(0, 1, 0.1)));
        assert!(producer.push(change(1, 2, 0.2)));
        assert_eq!(consumer.len(), 2);
        assert_eq!(consumer.pop().unwrap().param, ParamId::new(1));
        assert_eq!(consumer.pop().unwrap().param, ParamId::new(2));
        assert!(consumer.pop().is_none());
        assert!(consumer.is_empty());
    }

    #[test]
    fn full_channel_drops_excess() {
        let (mut producer, consumer) = param_channel(2);
        assert!(producer.push(change(0, 1, 0.0)));
        assert!(producer.push(change(0, 2, 0.0)));
        // Capacity reached; further pushes are rejected, not blocked
        assert!(!producer.push(change(0, 3, 0.0)));
        assert_eq!(consumer.len(), 2);
        assert_eq!(producer.free(), 0);
    }
}
