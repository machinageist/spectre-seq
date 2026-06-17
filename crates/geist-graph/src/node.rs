// =============================================================================
// File: crates/geist-graph/src/node.rs
// Layer: audio graph
// Purpose: AudioNode trait definition
// Status: Implemented.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use geist_core::config::AudioConfig;
use geist_core::context::ProcessContext;

// Unit of audio processing in the graph
// Send so a compiled graph can move to the audio thread; never required to be Sync
// process runs on the audio thread and must not allocate, lock, or block
pub trait AudioNode: Send {
    // Process one block in place through the borrowed context
    fn process(&mut self, ctx: &mut ProcessContext);

    // Configure for a sample rate and block size before streaming starts
    // Runs on the app thread; allocation is permitted here, not in process
    fn prepare(&mut self, _config: &AudioConfig) {}

    // Clear internal state back to silence
    fn reset(&mut self) {}
}
