// =============================================================================
// File: crates/spectre-automation/src/curve.rs
// Layer: automation
// Purpose: CurveSegment: linear, exponential, bezier
// Status: Implemented; shape interpolation between two breakpoint values.
// Notes: Each shape maps t in [0,1] from value a to value b. Step holds a until
//        the next point; the eased shapes bow below or above the linear chord.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

// Interpolation shape governing one segment between two breakpoints
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CurveShape {
    // Hold the left value until the next breakpoint
    Step,
    // Straight line from a to b
    #[default]
    Linear,
    // Ease-in: slow start, bows below the linear chord
    Exponential,
    // Ease-out: fast start, bows above the linear chord
    Logarithmic,
    // Smoothstep: eased at both ends, symmetric about the midpoint
    SCurve,
}

impl CurveShape {
    // Interpolate from a to b across the segment at fraction t
    #[inline]
    pub fn interpolate(&self, a: f32, b: f32, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        let shaped = match self {
            CurveShape::Step => {
                // Hold a across the segment; the next point owns its own value
                if t >= 1.0 {
                    1.0
                } else {
                    0.0
                }
            }
            CurveShape::Linear => t,
            CurveShape::Exponential => t * t,
            CurveShape::Logarithmic => {
                let inv = 1.0 - t;
                1.0 - inv * inv
            }
            CurveShape::SCurve => t * t * (3.0 - 2.0 * t),
        };
        a + (b - a) * shaped
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EASED: [CurveShape; 4] = [
        CurveShape::Linear,
        CurveShape::Exponential,
        CurveShape::Logarithmic,
        CurveShape::SCurve,
    ];

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn eased_shapes_hit_both_endpoints() {
        for shape in EASED {
            assert!(
                close(shape.interpolate(2.0, 6.0, 0.0), 2.0),
                "{shape:?} at 0"
            );
            assert!(
                close(shape.interpolate(2.0, 6.0, 1.0), 6.0),
                "{shape:?} at 1"
            );
        }
    }

    #[test]
    fn t_is_clamped_outside_unit_range() {
        let s = CurveShape::Linear;
        assert!(close(s.interpolate(0.0, 10.0, -1.0), 0.0));
        assert!(close(s.interpolate(0.0, 10.0, 2.0), 10.0));
    }

    #[test]
    fn step_holds_then_jumps() {
        let s = CurveShape::Step;
        assert!(close(s.interpolate(3.0, 9.0, 0.0), 3.0));
        assert!(close(s.interpolate(3.0, 9.0, 0.99), 3.0)); // still held
        assert!(close(s.interpolate(3.0, 9.0, 1.0), 9.0)); // jumps at the point
    }

    #[test]
    fn linear_and_scurve_pass_through_the_midpoint() {
        assert!(close(CurveShape::Linear.interpolate(0.0, 10.0, 0.5), 5.0));
        assert!(close(CurveShape::SCurve.interpolate(0.0, 10.0, 0.5), 5.0));
    }

    #[test]
    fn exponential_bows_below_and_logarithmic_above() {
        // For a rising segment, ease-in sits under the chord, ease-out over it
        let mid_exp = CurveShape::Exponential.interpolate(0.0, 10.0, 0.5);
        let mid_log = CurveShape::Logarithmic.interpolate(0.0, 10.0, 0.5);
        assert!(mid_exp < 5.0, "exp midpoint = {mid_exp}");
        assert!(mid_log > 5.0, "log midpoint = {mid_log}");
        // They are mirror images about the chord
        assert!(close(mid_exp + mid_log, 10.0));
    }

    #[test]
    fn eased_shapes_are_monotonic() {
        for shape in EASED {
            let mut prev = shape.interpolate(0.0, 1.0, 0.0);
            let mut t = 0.0;
            while t <= 1.0 {
                let v = shape.interpolate(0.0, 1.0, t);
                assert!(v >= prev - 1e-6, "{shape:?} not monotonic at {t}");
                prev = v;
                t += 0.05;
            }
        }
    }
}
