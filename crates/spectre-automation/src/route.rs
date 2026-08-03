// =============================================================================
// File: crates/spectre-automation/src/route.rs
// Layer: automation
// Purpose: ModRoute { src: PortId, dst: PortId, depth, bipolar }
// Status: Implemented; one modulation connection with depth and polarity.
// Notes: A route scales a CV source (PortId) into a parameter target (ParamId).
//        Bipolar passes the source through; unipolar rectifies its negative half.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use spectre_core::ids::{ParamId, PortId};

// One modulation connection: source CV scaled into a parameter target
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModRoute {
    pub source: PortId,
    pub target: ParamId,
    pub depth: f32,
    // True passes the source through; false rectifies to its positive half
    pub bipolar: bool,
}

impl ModRoute {
    // Build a bipolar route (full-range source)
    pub fn bipolar(source: PortId, target: ParamId, depth: f32) -> Self {
        Self {
            source,
            target,
            depth,
            bipolar: true,
        }
    }

    // Build a unipolar route (only the positive part of the source contributes)
    pub fn unipolar(source: PortId, target: ParamId, depth: f32) -> Self {
        Self {
            source,
            target,
            depth,
            bipolar: false,
        }
    }

    // Modulation contributed for a given source value
    #[inline]
    pub fn contribution(&self, source_value: f32) -> f32 {
        let value = if self.bipolar {
            source_value
        } else {
            source_value.max(0.0)
        };
        value * self.depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: PortId = PortId::new(10);
    const DST: ParamId = ParamId::new(20);

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn bipolar_passes_full_range() {
        let r = ModRoute::bipolar(SRC, DST, 0.5);
        assert!(close(r.contribution(1.0), 0.5));
        assert!(close(r.contribution(-1.0), -0.5));
        assert!(close(r.contribution(0.0), 0.0));
    }

    #[test]
    fn unipolar_rectifies_negative_sources() {
        let r = ModRoute::unipolar(SRC, DST, 2.0);
        assert!(close(r.contribution(0.5), 1.0));
        assert!(close(r.contribution(-0.5), 0.0)); // negative half removed
    }

    #[test]
    fn depth_scales_contribution() {
        let r = ModRoute::bipolar(SRC, DST, 0.25);
        assert!(close(r.contribution(0.8), 0.2));
    }

    #[test]
    fn stores_source_and_target() {
        let r = ModRoute::bipolar(SRC, DST, 1.0);
        assert_eq!(r.source, SRC);
        assert_eq!(r.target, DST);
        assert!(r.bipolar);
    }
}
