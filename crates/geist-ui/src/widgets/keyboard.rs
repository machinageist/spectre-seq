// Author: Jeff
// Date: 2026-06-15
// Description: Playable on-screen piano keyboard with mouse press/release events.
// Notes: Owns no audio truth. It edge-detects the key under the pointer against a
//        borrowed held-state vector and returns the note on/off events that
//        crossed this frame; the app maps those to engine commands. One key
//        sounds at a time from the mouse (black keys win over the white below).

use egui::{pos2, vec2, Align2, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Stroke, StrokeKind, Ui};

use crate::theme;

// On-screen keyboard spans two octaves from C3 by default
const DEFAULT_BASE_MIDI: u8 = 48;
const DEFAULT_KEYS: usize = 25;
// Strip height and the minimum width before keys get unreadably thin
const DEFAULT_HEIGHT: f32 = 108.0;
const MIN_WIDTH: f32 = 220.0;
// Black keys are narrower and shorter than the white keys they sit between
const BLACK_W_FRAC: f32 = 0.62;
const BLACK_H_FRAC: f32 = 0.6;

// Light key faces tuned to read against the dark panel without glare
const WHITE_KEY: Color32 = Color32::from_rgb(206, 211, 220);
const WHITE_KEY_HOVER: Color32 = Color32::from_rgb(224, 228, 236);
const BLACK_KEY: Color32 = Color32::from_rgb(26, 28, 34);
const BLACK_KEY_HOVER: Color32 = Color32::from_rgb(40, 44, 52);

// One note transition emitted by the keyboard this frame
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    pub midi: u8,
    pub down: bool,
}

// A clickable piano keyboard bound to a borrowed held-state vector
pub struct Keyboard<'a> {
    held: &'a mut Vec<bool>,
    base_midi: u8,
    keys: usize,
    height: f32,
}

impl<'a> Keyboard<'a> {
    // Bind to the caller's per-key held state, persisted across frames
    pub fn new(held: &'a mut Vec<bool>) -> Self {
        Self {
            held,
            base_midi: DEFAULT_BASE_MIDI,
            keys: DEFAULT_KEYS,
            height: DEFAULT_HEIGHT,
        }
    }

    pub fn base_midi(mut self, base_midi: u8) -> Self {
        self.base_midi = base_midi;
        self
    }

    pub fn keys(mut self, keys: usize) -> Self {
        self.keys = keys;
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    // Draw the keyboard, edge-detect the pressed key, and return the transitions
    pub fn show(self, ui: &mut Ui) -> Vec<KeyEvent> {
        let Keyboard { held, base_midi, keys, height } = self;
        // Keep the held vector aligned with the key count across reconfigurations
        if held.len() != keys {
            held.clear();
            held.resize(keys, false);
        }

        let width = ui.available_width().max(MIN_WIDTH);
        let (rect, resp) = ui.allocate_exact_size(vec2(width, height), Sense::click_and_drag());
        let painter = ui.painter_at(rect);

        // Backboard behind the keys
        painter.rect_filled(rect, theme::RADIUS_PANEL, theme::PANEL);

        let white_total = count_white_keys(base_midi, keys).max(1);
        let white_w = rect.width() / white_total as f32;
        let black_w = white_w * BLACK_W_FRAC;
        let black_h = rect.height() * BLACK_H_FRAC;

        // Lay out each semitone's hit rect; black keys straddle white boundaries
        let mut white: Vec<(usize, Rect)> = Vec::new();
        let mut black: Vec<(usize, Rect)> = Vec::new();
        let mut wi = 0usize;
        for s in 0..keys {
            if is_black(base_midi, s) {
                let cx = rect.left() + wi as f32 * white_w;
                let r = Rect::from_min_size(pos2(cx - black_w * 0.5, rect.top()), vec2(black_w, black_h));
                black.push((s, r));
            } else {
                let x = rect.left() + wi as f32 * white_w;
                let r = Rect::from_min_size(pos2(x, rect.top()), vec2(white_w, rect.height()));
                white.push((s, r));
                wi += 1;
            }
        }

        // Single active key while pressed (topmost wins); hover for idle highlight
        let active = if resp.is_pointer_button_down_on() {
            resp.interact_pointer_pos().and_then(|p| key_at(p, &black, &white))
        } else {
            None
        };
        let hovered = resp.hover_pos().and_then(|p| key_at(p, &black, &white));

        // White keys first, black keys painted on top
        let white_cr = CornerRadius { nw: 0, ne: 0, sw: 4, se: 4 };
        for (s, r) in &white {
            let down = active == Some(*s);
            let fill = if down {
                theme::ACCENT
            } else if hovered == Some(*s) {
                WHITE_KEY_HOVER
            } else {
                WHITE_KEY
            };
            painter.rect_filled(r.shrink(0.5), white_cr, fill);
            painter.rect_stroke(r.shrink(0.5), white_cr, Stroke::new(1.0, theme::STROKE_STRONG), StrokeKind::Inside);
            let midi = base_midi + *s as u8;
            if midi.is_multiple_of(12) {
                painter.text(
                    pos2(r.center().x, r.bottom() - 7.0),
                    Align2::CENTER_BOTTOM,
                    format!("C{}", midi as i32 / 12 - 1),
                    FontId::new(9.0, egui::FontFamily::Proportional),
                    if down { theme::BG } else { theme::TEXT_MUTED },
                );
            }
        }
        let black_cr = CornerRadius { nw: 0, ne: 0, sw: 3, se: 3 };
        for (s, r) in &black {
            let down = active == Some(*s);
            let fill = if down {
                theme::ACCENT_DIM
            } else if hovered == Some(*s) {
                BLACK_KEY_HOVER
            } else {
                BLACK_KEY
            };
            painter.rect_filled(*r, black_cr, fill);
            painter.rect_stroke(*r, black_cr, Stroke::new(1.0, theme::BG), StrokeKind::Inside);
        }

        // Emit the on/off transitions that crossed since last frame
        // held was resized to `keys` above, so this visits every semitone
        let mut events = Vec::new();
        for (s, slot) in held.iter_mut().enumerate() {
            let now = active == Some(s);
            if now != *slot {
                events.push(KeyEvent { midi: base_midi + s as u8, down: now });
                *slot = now;
            }
        }
        events
    }
}

// Whether the semitone at offset `s` from `base_midi` is a black key
fn is_black(base_midi: u8, s: usize) -> bool {
    matches!((base_midi as usize + s) % 12, 1 | 3 | 6 | 8 | 10)
}

// White keys in the range, which sets the equal white-key width
fn count_white_keys(base_midi: u8, keys: usize) -> usize {
    (0..keys).filter(|&s| !is_black(base_midi, s)).count()
}

// Topmost key under a point: black keys checked before the white keys below
fn key_at(p: Pos2, black: &[(usize, Rect)], white: &[(usize, Rect)]) -> Option<usize> {
    for (s, r) in black {
        if r.contains(p) {
            return Some(*s);
        }
    }
    for (s, r) in white {
        if r.contains(p) {
            return Some(*s);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_keys_are_the_accidentals() {
        // From C (base 48): offsets 1,3,6,8,10 are the sharps
        for white in [0usize, 2, 4, 5, 7, 9, 11] {
            assert!(!is_black(48, white));
        }
        for black in [1usize, 3, 6, 8, 10] {
            assert!(is_black(48, black));
        }
    }

    #[test]
    fn two_octaves_from_c3_have_fifteen_white_keys() {
        // 25 semitones C3..C5: 14 across two octaves plus the closing C
        assert_eq!(count_white_keys(48, 25), 15);
        // One octave is seven white keys
        assert_eq!(count_white_keys(48, 12), 7);
    }

    #[test]
    fn black_keys_win_over_the_white_below() {
        let black = vec![(1usize, Rect::from_min_size(pos2(10.0, 0.0), vec2(10.0, 30.0)))];
        let white = vec![(0usize, Rect::from_min_size(pos2(0.0, 0.0), vec2(20.0, 50.0)))];
        // A point inside the overlap resolves to the black key
        assert_eq!(key_at(pos2(15.0, 10.0), &black, &white), Some(1));
        // A point only under the white key resolves to white
        assert_eq!(key_at(pos2(2.0, 40.0), &black, &white), Some(0));
        // Outside both is None
        assert_eq!(key_at(pos2(99.0, 99.0), &black, &white), None);
    }
}
