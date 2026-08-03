// Author: Jeff
// Date: 2026-06-15
// Description: Vertical peak/level meter with dB scaling, peak-hold, clip cap.
// Notes: Read-only: consumes a level (and optional held peak) published by the
//        audio side and draws it. Amplitude is mapped to a dB scale for a
//        musically useful response rather than a raw linear bar.

use egui::{Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};

use crate::theme;

// Footprint of a single meter column
const DEFAULT_SIZE: Vec2 = Vec2::new(14.0, 160.0);
// Bottom of the visible scale; quieter than this reads as silence
const MIN_DB: f32 = -60.0;
// Amplitude at or above this lights the clip cap (0 dBFS)
const CLIP_AMP: f32 = 1.0;
// Height of the clip cap at the top of the column
const CLIP_CAP_H: f32 = 3.0;

// A read-only level meter driven by audio-side amplitude
pub struct Meter {
    level: f32,
    peak: Option<f32>,
    size: Vec2,
}

impl Meter {
    // Bind to a current amplitude (0..~1.5, linear)
    pub fn new(level: f32) -> Self {
        Self {
            level,
            peak: None,
            size: DEFAULT_SIZE,
        }
    }

    // Held peak amplitude drawn as a marker line
    pub fn peak(mut self, peak: f32) -> Self {
        self.peak = Some(peak);
        self
    }

    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    // Draw the meter and return its (hover-only) response
    pub fn show(self, ui: &mut Ui) -> Response {
        let (rect, resp) = ui.allocate_exact_size(self.size, Sense::hover());
        let p = ui.painter();

        // Inset well
        p.rect_filled(rect, theme::RADIUS_CONTROL, theme::INSET);

        // Level fill from the bottom up
        let frac = meter_fraction(self.level);
        if frac > 0.0 {
            let fill = Rect::from_min_max(
                egui::pos2(rect.left(), rect.bottom() - frac * rect.height()),
                rect.right_bottom(),
            );
            p.rect_filled(fill, theme::RADIUS_CONTROL, theme::meter_color(frac));
        }

        // Peak-hold marker
        if let Some(peak) = self.peak {
            let pf = meter_fraction(peak);
            let y = rect.bottom() - pf * rect.height();
            p.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                Stroke::new(1.5, theme::TEXT),
            );
        }

        // Clip cap when the signal reaches full scale
        if self.level >= CLIP_AMP {
            let cap = Rect::from_min_max(
                rect.left_top(),
                egui::pos2(rect.right(), rect.top() + CLIP_CAP_H),
            );
            p.rect_filled(cap, 0.0, theme::METER_HIGH);
        }

        // Border
        p.rect_stroke(
            rect,
            theme::RADIUS_CONTROL,
            Stroke::new(1.0, theme::STROKE),
            StrokeKind::Inside,
        );

        resp
    }
}

// Map a linear amplitude to a 0..1 fill fraction on a dB scale
fn meter_fraction(amp: f32) -> f32 {
    if amp <= 0.0 {
        return 0.0;
    }
    let db = 20.0 * amp.log10();
    ((db - MIN_DB) / (0.0 - MIN_DB)).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_scale_fills_the_meter() {
        assert!((meter_fraction(1.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn silence_is_empty() {
        assert_eq!(meter_fraction(0.0), 0.0);
        assert_eq!(meter_fraction(-0.5), 0.0);
    }

    #[test]
    fn minus_six_db_is_about_ninety_percent() {
        // 0.5 amplitude is roughly -6 dB; on a -60..0 scale that is ~0.9
        let f = meter_fraction(0.5);
        assert!((f - 0.8997).abs() < 0.01, "got {f}");
    }

    #[test]
    fn over_unity_clamps_to_full() {
        assert_eq!(meter_fraction(2.0), 1.0);
    }

    #[test]
    fn fill_color_matches_zone() {
        assert_eq!(theme::meter_color(meter_fraction(0.5)), theme::METER_MID);
        assert_eq!(theme::meter_color(meter_fraction(1.0)), theme::METER_HIGH);
    }
}
