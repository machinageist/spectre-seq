// =============================================================================
// File: crates/geist-graph/src/swap.rs
// Layer: audio graph
// Purpose: lock-free executor handoff between the app and audio threads
// Status: Implemented; SPSC handoff. The executor is mutable, so an rtrb handoff
//         replaces the planned ArcSwap: ArcSwap only lends shared refs.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use rtrb::{Consumer, Producer, RingBuffer};

use crate::process_list::Executor;

// In-flight executors tolerated before the app thread must reclaim
const SWAP_CAPACITY: usize = 8;

// App-thread side: publishes new executors and reclaims retired ones for drop
pub struct GraphPublisher {
    to_audio: Producer<Executor>,
    from_audio: Consumer<Executor>,
}

// Audio-thread side: owns the active executor and adopts published ones
pub struct ActiveGraph {
    current: Option<Executor>,
    from_app: Consumer<Executor>,
    to_app: Producer<Executor>,
}

// Build a publisher/active-graph pair sharing two lock-free SPSC rings
pub fn graph_swap(initial: Option<Executor>) -> (GraphPublisher, ActiveGraph) {
    let (to_audio, from_app) = RingBuffer::new(SWAP_CAPACITY);
    let (to_app, from_audio) = RingBuffer::new(SWAP_CAPACITY);
    (
        GraphPublisher {
            to_audio,
            from_audio,
        },
        ActiveGraph {
            current: initial,
            from_app,
            to_app,
        },
    )
}

impl GraphPublisher {
    // Publish a freshly compiled executor; false when the audio inbox is full
    pub fn publish(&mut self, executor: Executor) -> bool {
        self.to_audio.push(executor).is_ok()
    }

    // Drop retired executors on the app thread; returns how many were freed
    pub fn reclaim(&mut self) -> usize {
        let mut freed = 0;
        while let Ok(retired) = self.from_audio.pop() {
            drop(retired);
            freed += 1;
        }
        freed
    }
}

impl ActiveGraph {
    // Adopt a published executor when one waits and a retire slot is free
    // Never drops on the audio thread: the old executor is shipped back for the app to free
    pub fn poll_swap(&mut self) -> bool {
        if self.to_app.slots() == 0 {
            return false;
        }
        if let Ok(next) = self.from_app.pop() {
            if let Some(old) = self.current.replace(next) {
                let _ = self.to_app.push(old);
            }
            true
        } else {
            false
        }
    }

    // Borrow the active executor for one block, if a graph is installed
    pub fn current_mut(&mut self) -> Option<&mut Executor> {
        self.current.as_mut()
    }

    // Whether an executor is currently installed
    pub fn has_graph(&self) -> bool {
        self.current.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::Graph;
    use crate::process_list::compile;
    use spectre_core::config::AudioConfig;

    // Build a runnable executor from an empty graph
    fn empty_executor() -> Executor {
        let graph = Graph::new();
        let config = AudioConfig::new(48_000, 8, 0, 2).unwrap();
        let plan = compile(&graph, &config).unwrap();
        Executor::new(graph, plan, &config).unwrap()
    }

    #[test]
    fn publishes_adopts_and_reclaims() {
        let (mut publisher, mut active) = graph_swap(Some(empty_executor()));
        assert!(active.has_graph());

        assert!(publisher.publish(empty_executor()));
        assert!(active.poll_swap());
        assert!(active.has_graph());

        // The retired executor is freed on the app thread, not the audio thread
        assert_eq!(publisher.reclaim(), 1);
    }

    #[test]
    fn poll_swap_is_noop_without_pending() {
        let (_publisher, mut active) = graph_swap(Some(empty_executor()));
        assert!(!active.poll_swap());
        assert!(active.has_graph());
    }

    #[test]
    fn starts_empty_then_adopts_first_graph() {
        let (mut publisher, mut active) = graph_swap(None);
        assert!(!active.has_graph());
        assert!(publisher.publish(empty_executor()));
        assert!(active.poll_swap());
        assert!(active.has_graph());
        // No previous executor existed, so nothing to reclaim
        assert_eq!(publisher.reclaim(), 0);
    }
}
