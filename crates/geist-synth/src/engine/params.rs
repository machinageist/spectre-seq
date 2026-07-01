// =============================================================================
// File: crates/geist-synth/src/engine/params.rs
// Layer: internal synth device
// Purpose: all parameter definitions + ranges
// Status: Pseudocode scaffold; implementation intentionally pending.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// Pseudocode plan:
// - Declare responsibility: all parameter definitions + ranges
// - Define public types before behavior.
// - Separate real-time-safe paths from UI/app paths.
// - Prefer explicit errors over implicit panics.
// - Add tests beside behavior once implementation begins.
// - Operate on slices; allocate never in hot path.
// - Reset state deterministically.
// - Benchmark hot loops before optimizing.

// Module items land below this line.
