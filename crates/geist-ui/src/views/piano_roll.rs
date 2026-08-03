// Author: Jeff
// Date: 2026-06-15
// Description: Piano roll surface helpers plus the note-editing grid drawing.
// Notes: Piano roll is the focused note editor under Arrange. draw() renders a
//        keyboard gutter and beat grid; click empty cells to add a one-beat note,
//        click a note to remove it. Notes live in the model; edits emit intents.

use egui::{pos2, vec2, Align2, FontId, Rect, Sense, Stroke, StrokeKind};
use spectre_config::commands::CommandIntent;

use crate::model::{Note, PianoRollModel};
use crate::theme;
use crate::views::{label_from_command, ActionChip};

pub fn default_piano_roll_actions() -> Vec<ActionChip> {
    ["draw_note", "quantize", "humanize"]
        .into_iter()
        .map(|command| ActionChip {
            label: label_from_command(command),
            command: command.to_string(),
        })
        .collect()
}

const KEY_W: f32 = 42.0;
const ROW_H: f32 = 12.0;
const PX_PER_BEAT: f32 = 28.0;
const LOW_PITCH: u8 = 36; // C2
const HIGH_PITCH: u8 = 84; // C6

// Draw the piano roll and apply click-to-add / click-to-remove note editing.
// `playhead_beats` draws a moving cursor at that loop position when set.
pub fn draw(
    ui: &mut egui::Ui,
    roll: &mut PianoRollModel,
    playhead_beats: Option<f32>,
    intents: &mut Vec<CommandIntent>,
) {
    let rows = (HIGH_PITCH - LOW_PITCH) as f32;
    let content = vec2(KEY_W + roll.length_beats * PX_PER_BEAT, rows * ROW_H);

    egui::ScrollArea::both().show(ui, |ui| {
        let (rect, resp) = ui.allocate_exact_size(content, Sense::click());
        let painter = ui.painter_at(rect);
        let grid_left = rect.left() + KEY_W;
        let beat_x = |beat: f32| grid_left + beat * PX_PER_BEAT;
        // Top y of the row for a given pitch (low pitches at the bottom)
        let pitch_top = |pitch: u8| rect.bottom() - (pitch - LOW_PITCH + 1) as f32 * ROW_H;

        painter.rect_filled(rect, 0.0, theme::BG);

        // Rows: black-key shading, row separators, and the keyboard gutter
        for pitch in LOW_PITCH..HIGH_PITCH {
            let y = pitch_top(pitch);
            if is_black_key(pitch) {
                let row =
                    Rect::from_min_size(pos2(grid_left, y), vec2(rect.right() - grid_left, ROW_H));
                painter.rect_filled(row, 0.0, theme::FAINT);
            }
            painter.line_segment(
                [pos2(grid_left, y), pos2(rect.right(), y)],
                Stroke::new(1.0, theme::STROKE),
            );

            let key = Rect::from_min_size(pos2(rect.left(), y), vec2(KEY_W, ROW_H));
            let fill = if is_black_key(pitch) {
                theme::INSET
            } else {
                theme::PANEL_RAISED
            };
            painter.rect_filled(key, 0.0, fill);
            painter.rect_stroke(
                key,
                0.0,
                Stroke::new(1.0, theme::STROKE),
                StrokeKind::Inside,
            );
            if pitch % 12 == 0 {
                painter.text(
                    key.right_center() - vec2(3.0, 0.0),
                    Align2::RIGHT_CENTER,
                    format!("C{}", pitch as i32 / 12 - 1),
                    FontId::new(9.0, egui::FontFamily::Proportional),
                    theme::TEXT_MUTED,
                );
            }
        }

        // Beat grid lines, bars emphasized
        let mut beat = 0.0;
        while beat <= roll.length_beats {
            let x = beat_x(beat);
            let strong = (beat as i32) % 4 == 0;
            painter.line_segment(
                [pos2(x, rect.top()), pos2(x, rect.bottom())],
                Stroke::new(
                    1.0,
                    if strong {
                        theme::STROKE_STRONG
                    } else {
                        theme::STROKE
                    },
                ),
            );
            beat += 1.0;
        }

        // Notes colored by velocity
        for note in &roll.notes {
            let y = pitch_top(note.pitch);
            let note_rect = Rect::from_min_size(
                pos2(beat_x(note.start_beats), y + 1.0),
                vec2(note.len_beats * PX_PER_BEAT, ROW_H - 2.0),
            );
            painter.rect_filled(
                note_rect,
                3.0,
                theme::ACCENT.linear_multiply(0.4 + 0.6 * note.velocity.clamp(0.0, 1.0)),
            );
            painter.rect_stroke(
                note_rect,
                3.0,
                Stroke::new(1.0, theme::ACCENT),
                StrokeKind::Inside,
            );
        }

        // Playhead at the current loop position
        if let Some(ph) = playhead_beats {
            if roll.length_beats > 0.0 {
                let x = beat_x(ph.rem_euclid(roll.length_beats));
                painter.line_segment(
                    [pos2(x, rect.top()), pos2(x, rect.bottom())],
                    Stroke::new(1.5, theme::ACCENT),
                );
            }
        }

        // Click to toggle a note at the pointed cell
        if resp.clicked() {
            if let Some(p) = resp.interact_pointer_pos() {
                if p.x > grid_left {
                    let beat = ((p.x - grid_left) / PX_PER_BEAT).floor().max(0.0);
                    let row = ((rect.bottom() - p.y) / ROW_H).floor() as i32;
                    let pitch = LOW_PITCH as i32 + row;
                    if (LOW_PITCH as i32..HIGH_PITCH as i32).contains(&pitch) {
                        let pitch = pitch as u8;
                        if roll.remove_at(pitch, beat) {
                            intents.push(CommandIntent::new("remove_note"));
                        } else {
                            roll.add(Note {
                                pitch,
                                start_beats: beat,
                                len_beats: 1.0,
                                velocity: 0.9,
                            });
                            intents.push(CommandIntent::new("add_note"));
                        }
                    }
                }
            }
        }
    });
}

// Whether a MIDI pitch class is a black key
fn is_black_key(pitch: u8) -> bool {
    matches!(pitch % 12, 1 | 3 | 6 | 8 | 10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn black_keys_are_the_accidentals() {
        // C, D, E, F, G, A, B are white
        for white in [0u8, 2, 4, 5, 7, 9, 11] {
            assert!(!is_black_key(60 + white % 12));
        }
        for black in [1u8, 3, 6, 8, 10] {
            assert!(is_black_key(60 + black));
        }
    }
}
