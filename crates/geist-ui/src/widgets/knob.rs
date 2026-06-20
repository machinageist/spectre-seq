// Author: Jeff
// Date: 2026-06-15
// Description: Tactile rotary knob with value arc, default tick, and mod ring.
// Notes: Drag vertically to change; double-click resets to default. Owns no
//        truth: it mutates a borrowed f32 and reports change via egui::Response.

use std::ops::RangeInclusive;

use egui::{Color32, Pos2, Response, Sense, Shape, Stroke, TextStyle, Ui, Vec2};

use crate::theme;

// Knob sweep: 270 degrees with a gap centered at the bottom
const SWEEP_DEG: f32 = 270.0;
const START_DEG: f32 = -135.0;
// Pixels of vertical drag that span the whole range
const DRAG_FULL_PX: f32 = 200.0;
// Drag is this much slower while Shift is held, for fine adjustment
const FINE_FACTOR: f32 = 6.0;
// Arc sampling resolution
const ARC_STEPS: usize = 24;

// How a 0..1 control position maps onto the value range
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum Taper {
    // Even value steps across the sweep
    #[default]
    Linear,
    // Even ratio steps across the sweep; for frequency and time controls where
    // perception is logarithmic. Requires a strictly positive range or it falls
    // back to linear.
    Logarithmic,
}

// A continuous rotary control bound to a borrowed value
pub struct Knob<'a> {
    value: &'a mut f32,
    range: RangeInclusive<f32>,
    default: f32,
    taper: Taper,
    label: &'a str,
    unit: &'a str,
    diameter: f32,
    modulation: Option<f32>,
    arc_color: Color32,
}

impl<'a> Knob<'a> {
    // Bind to a value and its range; defaults to the range minimum
    pub fn new(value: &'a mut f32, range: RangeInclusive<f32>) -> Self {
        let default = *range.start();
        Self {
            value,
            range,
            default,
            taper: Taper::Linear,
            label: "",
            unit: "",
            diameter: 48.0,
            modulation: None,
            arc_color: theme::CONTROL,
        }
    }

    // The value the knob snaps to on double-click
    pub fn default(mut self, default: f32) -> Self {
        self.default = default;
        self
    }

    // Position-to-value taper; logarithmic suits frequency and time controls
    pub fn taper(mut self, taper: Taper) -> Self {
        self.taper = taper;
        self
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    pub fn unit(mut self, unit: &'a str) -> Self {
        self.unit = unit;
        self
    }

    pub fn diameter(mut self, diameter: f32) -> Self {
        self.diameter = diameter;
        self
    }

    // Outer modulation ring amount (0..1 added on top of the base value)
    pub fn modulation(mut self, amount: f32) -> Self {
        self.modulation = Some(amount);
        self
    }

    // Color of the value arc; defaults to the control/accent color
    pub fn arc_color(mut self, color: Color32) -> Self {
        self.arc_color = color;
        self
    }

    // Draw the knob, apply interaction, and return the knob area's response
    pub fn show(self, ui: &mut Ui) -> Response {
        let (lo, hi) = (*self.range.start(), *self.range.end());
        ui.vertical_centered(|ui| {
            ui.spacing_mut().item_spacing.y = 3.0;
            if !self.label.is_empty() {
                ui.label(
                    egui::RichText::new(self.label)
                        .text_style(TextStyle::Small)
                        .color(theme::TEXT_MUTED),
                );
            }

            let (rect, mut resp) =
                ui.allocate_exact_size(Vec2::splat(self.diameter), Sense::click_and_drag());

            // Apply interaction before painting so the arc reflects this frame
            let original = *self.value;
            let mut value = original;
            if resp.double_clicked() {
                value = self.default;
            } else if resp.dragged() {
                // Shift slows the drag for fine adjustment
                let span = if ui.input(|i| i.modifiers.shift) {
                    DRAG_FULL_PX * FINE_FACTOR
                } else {
                    DRAG_FULL_PX
                };
                let pos = position_of(value, lo, hi, self.taper);
                let next = (pos - resp.drag_delta().y / span).clamp(0.0, 1.0);
                value = value_at(next, lo, hi, self.taper);
            }
            if value != original {
                *self.value = value;
                resp.mark_changed();
            }

            let norm = position_of(value, lo, hi, self.taper);
            paint_knob(
                ui,
                rect,
                norm,
                position_of(self.default, lo, hi, self.taper),
                self.modulation,
                self.arc_color,
            );

            ui.label(
                egui::RichText::new(format!("{}{}", fmt_value(value), unit_suffix(self.unit)))
                    .color(theme::TEXT),
            );

            resp
        })
        .inner
    }
}

// Normalized 0..1 position of a value within its range
fn norm_of(value: f32, lo: f32, hi: f32) -> f32 {
    if hi <= lo {
        0.0
    } else {
        ((value - lo) / (hi - lo)).clamp(0.0, 1.0)
    }
}

// Value at a normalized 0..1 position within its range
fn value_of(norm: f32, lo: f32, hi: f32) -> f32 {
    lo + norm.clamp(0.0, 1.0) * (hi - lo)
}

// Control position of a value under the given taper
// Logarithmic needs a strictly positive range; otherwise it degrades to linear
fn position_of(value: f32, lo: f32, hi: f32, taper: Taper) -> f32 {
    match taper {
        Taper::Logarithmic if lo > 0.0 && hi > lo => {
            (value.clamp(lo, hi) / lo).ln() / (hi / lo).ln()
        }
        _ => norm_of(value, lo, hi),
    }
}

// Value at a control position under the given taper
fn value_at(position: f32, lo: f32, hi: f32, taper: Taper) -> f32 {
    match taper {
        Taper::Logarithmic if lo > 0.0 && hi > lo => lo * (hi / lo).powf(position.clamp(0.0, 1.0)),
        _ => value_of(position, lo, hi),
    }
}

// Pointer angle in radians for a normalized position, clockwise from straight up
fn pointer_angle(norm: f32) -> f32 {
    (START_DEG + norm.clamp(0.0, 1.0) * SWEEP_DEG).to_radians()
}

// Unit direction for a pointer angle, in screen space (y grows downward)
fn dir(angle: f32) -> Vec2 {
    Vec2::new(angle.sin(), -angle.cos())
}

// Compact value text: coarser precision as magnitude grows
fn fmt_value(v: f32) -> String {
    let a = v.abs();
    if a >= 100.0 {
        format!("{v:.0}")
    } else if a >= 10.0 {
        format!("{v:.1}")
    } else {
        format!("{v:.2}")
    }
}

// Render a unit as a space-prefixed suffix, or nothing when empty
fn unit_suffix(unit: &str) -> String {
    if unit.is_empty() {
        String::new()
    } else {
        format!(" {unit}")
    }
}

// Paint the track arc, value arc, body, pointer, default tick, and mod ring
fn paint_knob(
    ui: &Ui,
    rect: egui::Rect,
    norm: f32,
    default_norm: f32,
    modulation: Option<f32>,
    arc_color: Color32,
) {
    let p = ui.painter();
    let center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.5 - 2.0;
    let arc_r = radius - 2.0;

    let a_min = pointer_angle(0.0);
    let a_max = pointer_angle(1.0);
    let a_val = pointer_angle(norm);

    // Track then filled value arc
    paint_arc(p, center, arc_r, a_min, a_max, Stroke::new(3.0, theme::STROKE_STRONG));
    paint_arc(p, center, arc_r, a_min, a_val, Stroke::new(3.0, arc_color));

    // Knob body
    let body_r = radius - 5.0;
    p.circle_filled(center, body_r, theme::PANEL_RAISED);
    p.circle_stroke(center, body_r, Stroke::new(1.0, theme::STROKE));

    // Pointer from hub to edge
    let d = dir(a_val);
    p.line_segment(
        [center + d * (body_r * 0.35), center + d * (body_r - 1.0)],
        Stroke::new(2.0, theme::TEXT),
    );

    // Default marker tick just outside the arc
    let dd = dir(pointer_angle(default_norm));
    p.line_segment(
        [center + dd * (arc_r + 1.0), center + dd * (arc_r + 4.0)],
        Stroke::new(1.5, theme::TEXT_MUTED),
    );

    // Outer modulation ring showing the modulated reach of the value
    if let Some(amount) = modulation {
        let a_mod = pointer_angle((norm + amount).clamp(0.0, 1.0));
        paint_arc(p, center, arc_r + 5.0, a_val, a_mod, Stroke::new(2.0, theme::CV));
    }
}

// Stroke a circular arc between two angles as a sampled polyline
fn paint_arc(
    painter: &egui::Painter,
    center: Pos2,
    radius: f32,
    a0: f32,
    a1: f32,
    stroke: Stroke,
) {
    let points: Vec<Pos2> = (0..=ARC_STEPS)
        .map(|i| {
            let t = i as f32 / ARC_STEPS as f32;
            let a = a0 + (a1 - a0) * t;
            center + dir(a) * radius
        })
        .collect();
    painter.add(Shape::line(points, stroke));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn norm_and_value_round_trip() {
        assert_eq!(norm_of(20.0, 20.0, 18_000.0), 0.0);
        assert_eq!(norm_of(18_000.0, 20.0, 18_000.0), 1.0);
        assert!((value_of(0.5, 0.0, 10.0) - 5.0).abs() < 1e-6);
        // Out-of-range values clamp to the ends
        assert_eq!(norm_of(-5.0, 0.0, 10.0), 0.0);
        assert_eq!(norm_of(99.0, 0.0, 10.0), 1.0);
    }

    #[test]
    fn degenerate_range_is_safe() {
        assert_eq!(norm_of(5.0, 5.0, 5.0), 0.0);
    }

    #[test]
    fn log_taper_endpoints_are_exact() {
        let (lo, hi) = (20.0, 18_000.0);
        assert!(position_of(lo, lo, hi, Taper::Logarithmic).abs() < 1e-6);
        assert!((position_of(hi, lo, hi, Taper::Logarithmic) - 1.0).abs() < 1e-6);
        assert!((value_at(0.0, lo, hi, Taper::Logarithmic) - lo).abs() < 1e-3);
        assert!((value_at(1.0, lo, hi, Taper::Logarithmic) - hi).abs() < 1.0);
    }

    #[test]
    fn log_taper_midpoint_is_the_geometric_mean() {
        let (lo, hi) = (20.0, 20_000.0);
        // Half travel lands on sqrt(lo*hi), not the arithmetic midpoint
        let mid = value_at(0.5, lo, hi, Taper::Logarithmic);
        assert!((mid - (lo * hi).sqrt()).abs() < 1.0, "mid={mid}");
        assert!((position_of(mid, lo, hi, Taper::Logarithmic) - 0.5).abs() < 1e-4);
    }

    #[test]
    fn log_taper_round_trips() {
        let (lo, hi) = (0.01, 1.5);
        for &p in &[0.0, 0.2, 0.5, 0.73, 1.0] {
            let v = value_at(p, lo, hi, Taper::Logarithmic);
            assert!((position_of(v, lo, hi, Taper::Logarithmic) - p).abs() < 1e-4);
        }
    }

    #[test]
    fn log_taper_falls_back_to_linear_on_nonpositive_range() {
        // A range touching zero can't be logarithmic; behave linearly instead
        assert!((value_at(0.5, 0.0, 10.0, Taper::Logarithmic) - 5.0).abs() < 1e-6);
        assert!((position_of(5.0, 0.0, 10.0, Taper::Logarithmic) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn pointer_angle_spans_the_sweep_centered_up() {
        // Mid value points straight up (0 radians in this convention)
        assert!((pointer_angle(0.5)).abs() < 1e-6);
        assert!((pointer_angle(0.0) - START_DEG.to_radians()).abs() < 1e-6);
        assert!((pointer_angle(1.0) - (START_DEG + SWEEP_DEG).to_radians()).abs() < 1e-6);
    }

    #[test]
    fn straight_up_points_negative_y() {
        let d = dir(pointer_angle(0.5));
        assert!(d.x.abs() < 1e-6);
        assert!(d.y < 0.0, "up is negative y in screen space");
    }

    #[test]
    fn value_text_precision_tracks_magnitude() {
        assert_eq!(fmt_value(1500.0), "1500");
        assert_eq!(fmt_value(12.34), "12.3");
        assert_eq!(fmt_value(0.91), "0.91");
        assert_eq!(unit_suffix("Hz"), " Hz");
        assert_eq!(unit_suffix(""), "");
    }
}
