// Author: Jeff
// Date: 2026-06-15
// Description: Vertical fader with scale ticks, fill, and a grabbable handle.
// Notes: Drag the handle or track to set level; double-click resets to default.
//        Mutates a borrowed f32 and reports change via egui::Response.

use std::ops::RangeInclusive;

use egui::{vec2, Color32, Rect, Response, Sense, Stroke, StrokeKind, TextStyle, Ui, Vec2};

use crate::theme;

// Default strip footprint; height dominates so the throw is readable
const DEFAULT_SIZE: Vec2 = Vec2::new(40.0, 160.0);
// Handle thickness as a fraction of track height
const HANDLE_H: f32 = 10.0;
// Number of scale ticks drawn along the track
const TICKS: usize = 5;

// A vertical level control bound to a borrowed value
pub struct Fader<'a> {
    value: &'a mut f32,
    range: RangeInclusive<f32>,
    default: f32,
    label: &'a str,
    size: Vec2,
    fill_color: Color32,
}

impl<'a> Fader<'a> {
    // Bind to a value and its range; defaults to the range minimum
    pub fn new(value: &'a mut f32, range: RangeInclusive<f32>) -> Self {
        let default = *range.start();
        Self {
            value,
            range,
            default,
            label: "",
            size: DEFAULT_SIZE,
            fill_color: theme::ACCENT,
        }
    }

    pub fn default(mut self, default: f32) -> Self {
        self.default = default;
        self
    }

    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    pub fn fill_color(mut self, color: Color32) -> Self {
        self.fill_color = color;
        self
    }

    // Draw the fader, apply interaction, and return the track's response
    pub fn show(self, ui: &mut Ui) -> Response {
        let (lo, hi) = (*self.range.start(), *self.range.end());
        ui.vertical_centered(|ui| {
            ui.spacing_mut().item_spacing.y = 4.0;

            let (rect, mut resp) = ui.allocate_exact_size(self.size, Sense::click_and_drag());

            // Drag or click maps the pointer's y to a value; double-click resets
            let original = *self.value;
            let mut value = original;
            if resp.double_clicked() {
                value = self.default;
            } else if resp.dragged() || resp.is_pointer_button_down_on() {
                if let Some(pos) = resp.interact_pointer_pos() {
                    value = value_of(norm_for_y(pos.y, rect.top(), rect.bottom()), lo, hi);
                }
            }
            if value != original {
                *self.value = value;
                resp.mark_changed();
            }

            paint_fader(ui, rect, norm_of(value, lo, hi), self.fill_color);

            if !self.label.is_empty() {
                ui.label(
                    egui::RichText::new(self.label)
                        .text_style(TextStyle::Small)
                        .color(theme::TEXT_MUTED),
                );
            }
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

// Screen y for a normalized position: 1.0 sits at the top, 0.0 at the bottom
fn y_for_norm(norm: f32, top: f32, bottom: f32) -> f32 {
    bottom - norm.clamp(0.0, 1.0) * (bottom - top)
}

// Normalized position for a screen y, inverse of y_for_norm
fn norm_for_y(y: f32, top: f32, bottom: f32) -> f32 {
    if bottom <= top {
        0.0
    } else {
        ((bottom - y) / (bottom - top)).clamp(0.0, 1.0)
    }
}

// Paint the inset track, scale ticks, level fill, and the handle
fn paint_fader(ui: &Ui, rect: Rect, norm: f32, fill_color: Color32) {
    let p = ui.painter();

    // Narrow track centered in the strip
    let track_w = 6.0;
    let track = Rect::from_min_max(
        egui::pos2(rect.center().x - track_w * 0.5, rect.top()),
        egui::pos2(rect.center().x + track_w * 0.5, rect.bottom()),
    );
    p.rect_filled(track, track_w * 0.5, theme::INSET);

    // Scale ticks down the right edge
    for i in 0..TICKS {
        let t = i as f32 / (TICKS - 1) as f32;
        let y = y_for_norm(t, rect.top(), rect.bottom());
        p.line_segment(
            [egui::pos2(track.right() + 3.0, y), egui::pos2(track.right() + 8.0, y)],
            Stroke::new(1.0, theme::STROKE_STRONG),
        );
    }

    // Filled portion from the handle down to the bottom
    let handle_y = y_for_norm(norm, rect.top(), rect.bottom());
    let fill = Rect::from_min_max(
        egui::pos2(track.left(), handle_y),
        egui::pos2(track.right(), rect.bottom()),
    );
    p.rect_filled(fill, track_w * 0.5, fill_color);

    // Handle cap straddling the current level
    let handle = Rect::from_center_size(
        egui::pos2(rect.center().x, handle_y),
        vec2(rect.width() * 0.8, HANDLE_H),
    );
    p.rect_filled(handle, theme::RADIUS_CONTROL, theme::PANEL_HOVER);
    p.rect_stroke(
        handle,
        theme::RADIUS_CONTROL,
        Stroke::new(1.0, theme::STROKE_STRONG),
        StrokeKind::Inside,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn norm_value_round_trip() {
        assert!((value_of(norm_of(0.8, 0.0, 1.5), 0.0, 1.5) - 0.8).abs() < 1e-6);
        assert_eq!(norm_of(0.0, 0.0, 1.5), 0.0);
        assert_eq!(norm_of(1.5, 0.0, 1.5), 1.0);
    }

    #[test]
    fn top_of_track_is_max_value() {
        // y at the top maps to norm 1.0, the bottom to 0.0
        assert!((norm_for_y(0.0, 0.0, 100.0) - 1.0).abs() < 1e-6);
        assert!((norm_for_y(100.0, 0.0, 100.0)).abs() < 1e-6);
        // y_for_norm is the exact inverse
        assert!((y_for_norm(1.0, 0.0, 100.0)).abs() < 1e-6);
        assert!((y_for_norm(0.0, 0.0, 100.0) - 100.0).abs() < 1e-6);
    }

    #[test]
    fn y_mapping_clamps_outside_the_track() {
        assert_eq!(norm_for_y(-50.0, 0.0, 100.0), 1.0);
        assert_eq!(norm_for_y(150.0, 0.0, 100.0), 0.0);
    }
}
