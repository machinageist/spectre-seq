// Author: Jeff
// Date: 2026-05-27
// Description: Parameter identity, ranges, normalization, display metadata, and clamping rules.
// Notes: Parameter math must be deterministic across DAW automation, modulation, and plugins.

use crate::ids::ParamId;

// How a plain parameter value maps onto the normalized 0..1 control range
// Normalized space is what automation, modulation, and UI knobs operate in
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ParamRange {
    // Even mapping across the interval
    Linear { min: f32, max: f32 },
    // Perceptual mapping for frequency/gain; requires 0 < min < max
    Logarithmic { min: f32, max: f32 },
    // Integer-valued parameter inclusive of both ends
    Stepped { min: i32, max: i32 },
    // Two-state parameter; plain value is 0.0 or 1.0
    Boolean,
}

impl ParamRange {
    // Build a validated linear range
    pub fn linear(min: f32, max: f32) -> Option<Self> {
        (min.is_finite() && max.is_finite() && max > min).then_some(Self::Linear { min, max })
    }

    // Build a validated logarithmic range
    pub fn logarithmic(min: f32, max: f32) -> Option<Self> {
        (min.is_finite() && max.is_finite() && min > 0.0 && max > min)
            .then_some(Self::Logarithmic { min, max })
    }

    // Build a validated stepped range
    pub fn stepped(min: i32, max: i32) -> Option<Self> {
        (max > min).then_some(Self::Stepped { min, max })
    }

    // Map a plain value into normalized 0..1 space
    // Defensive against degenerate ranges; never divides by zero
    pub fn normalize(self, plain: f32) -> f32 {
        match self {
            ParamRange::Linear { min, max } => safe_ratio(plain - min, max - min),
            ParamRange::Logarithmic { min, max } => {
                let plain = plain.max(min);
                safe_ratio((plain / min).ln(), (max / min).ln())
            }
            ParamRange::Stepped { min, max } => safe_ratio(plain - min as f32, (max - min) as f32),
            ParamRange::Boolean => {
                if plain >= 0.5 {
                    1.0
                } else {
                    0.0
                }
            }
        }
        .clamp(0.0, 1.0)
    }

    // Map a normalized 0..1 value back into plain space
    pub fn denormalize(self, norm: f32) -> f32 {
        let norm = norm.clamp(0.0, 1.0);
        match self {
            ParamRange::Linear { min, max } => min + norm * (max - min),
            ParamRange::Logarithmic { min, max } => min * (max / min).powf(norm),
            ParamRange::Stepped { min, max } => (min as f32 + norm * (max - min) as f32).round(),
            ParamRange::Boolean => {
                if norm >= 0.5 {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }

    // Clamp a plain value to the legal range, quantizing where required
    pub fn clamp(self, plain: f32) -> f32 {
        match self {
            ParamRange::Linear { min, max } => plain.clamp(min, max),
            ParamRange::Logarithmic { min, max } => plain.clamp(min, max),
            ParamRange::Stepped { min, max } => (plain.round() as i32).clamp(min, max) as f32,
            ParamRange::Boolean => {
                if plain >= 0.5 {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }
}

// Divide while treating a zero denominator as a collapsed range
#[inline]
fn safe_ratio(num: f32, den: f32) -> f32 {
    if den == 0.0 {
        0.0
    } else {
        num / den
    }
}

// Static description of one automatable/modulatable parameter
// Display formatting stays off the audio thread; this struct is plain data
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct ParamInfo {
    pub id: ParamId,
    pub name: &'static str,
    pub unit: &'static str,
    pub range: ParamRange,
    pub default: f32,
    pub automatable: bool,
    pub modulatable: bool,
}

impl ParamInfo {
    // Clamp a plain value to this parameter's range
    #[inline]
    pub fn clamp(&self, plain: f32) -> f32 {
        self.range.clamp(plain)
    }

    // Normalized position of the default value
    #[inline]
    pub fn default_normalized(&self) -> f32 {
        self.range.normalize(self.default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-4;

    #[test]
    fn linear_round_trips() {
        let r = ParamRange::linear(-1.0, 1.0).unwrap();
        assert!((r.normalize(0.0) - 0.5).abs() < EPS);
        assert!((r.denormalize(0.5) - 0.0).abs() < EPS);
        assert!((r.denormalize(r.normalize(0.25)) - 0.25).abs() < EPS);
    }

    #[test]
    fn logarithmic_is_monotonic_and_round_trips() {
        let r = ParamRange::logarithmic(20.0, 20_000.0).unwrap();
        assert!((r.normalize(20.0) - 0.0).abs() < EPS);
        assert!((r.normalize(20_000.0) - 1.0).abs() < EPS);
        // 632.45 Hz is the geometric midpoint of 20..20000
        assert!((r.normalize(632.456) - 0.5).abs() < 1e-3);
        assert!((r.denormalize(r.normalize(1000.0)) - 1000.0).abs() < 0.5);
    }

    #[test]
    fn logarithmic_rejects_nonpositive_min() {
        assert!(ParamRange::logarithmic(0.0, 1.0).is_none());
        assert!(ParamRange::logarithmic(-1.0, 1.0).is_none());
    }

    #[test]
    fn stepped_quantizes_on_denormalize_and_clamp() {
        let r = ParamRange::stepped(0, 4).unwrap();
        assert_eq!(r.denormalize(0.5), 2.0);
        assert_eq!(r.clamp(2.4), 2.0);
        assert_eq!(r.clamp(99.0), 4.0);
        assert_eq!(r.clamp(-99.0), 0.0);
    }

    #[test]
    fn boolean_snaps_to_extremes() {
        let r = ParamRange::Boolean;
        assert_eq!(r.normalize(0.7), 1.0);
        assert_eq!(r.normalize(0.2), 0.0);
        assert_eq!(r.denormalize(1.0), 1.0);
        assert_eq!(r.clamp(0.9), 1.0);
    }

    #[test]
    fn clamp_holds_range_bounds() {
        let r = ParamRange::linear(0.0, 10.0).unwrap();
        assert_eq!(r.clamp(-5.0), 0.0);
        assert_eq!(r.clamp(15.0), 10.0);
        assert_eq!(r.normalize(20.0), 1.0);
    }

    #[test]
    fn degenerate_range_does_not_divide_by_zero() {
        let r = ParamRange::Linear { min: 1.0, max: 1.0 };
        assert_eq!(r.normalize(1.0), 0.0);
    }

    #[test]
    fn param_info_reports_default_position() {
        let info = ParamInfo {
            id: ParamId::new(0),
            name: "cutoff",
            unit: "Hz",
            range: ParamRange::logarithmic(20.0, 20_000.0).unwrap(),
            default: 20.0,
            automatable: true,
            modulatable: true,
        };
        assert!((info.default_normalized() - 0.0).abs() < EPS);
        assert_eq!(info.clamp(50_000.0), 20_000.0);
    }
}
