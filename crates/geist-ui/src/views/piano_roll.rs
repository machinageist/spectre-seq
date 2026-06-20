// Author: Jeff
// Date: 2026-06-15
// Description: Piano roll surface helpers plus the note-editing grid drawing.
// Notes: Piano roll is the focused note editor under Arrange. draw() renders a
//        keyboard gutter and beat grid; click empty cells to add a one-beat note,
//        click a note to remove it. Notes live in the model; edits emit intents.

use egui::{pos2, vec2, Align2, FontId, Pos2, Rect, Sense, Stroke, StrokeKind};
use geist_config::commands::CommandIntent;

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
        let (rect, resp) = ui.allocate_exact_size(content, Sense::click_and_drag());
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
                let row = Rect::from_min_size(pos2(grid_left, y), vec2(rect.right() - grid_left, ROW_H));
                painter.rect_filled(row, 0.0, theme::FAINT);
            }
            painter.line_segment(
                [pos2(grid_left, y), pos2(rect.right(), y)],
                Stroke::new(1.0, theme::STROKE),
            );

            let key = Rect::from_min_size(pos2(rect.left(), y), vec2(KEY_W, ROW_H));
            let fill = if is_black_key(pitch) { theme::INSET } else { theme::PANEL_RAISED };
            painter.rect_filled(key, 0.0, fill);
            painter.rect_stroke(key, 0.0, Stroke::new(1.0, theme::STROKE), StrokeKind::Inside);
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
                Stroke::new(1.0, if strong { theme::STROKE_STRONG } else { theme::STROKE }),
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
            painter.rect_stroke(note_rect, 3.0, Stroke::new(1.0, theme::ACCENT), StrokeKind::Inside);
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

        // --- Mouse editing ---
        // Pointer -> grid coordinates (float beat, MIDI pitch)
        let pointer_beat = |x: f32| ((x - grid_left) / PX_PER_BEAT).max(0.0);
        let pointer_pitch = |y: f32| LOW_PITCH as i32 + ((rect.bottom() - y) / ROW_H).floor() as i32;
        let in_range = |pitch: i32| (LOW_PITCH as i32..HIGH_PITCH as i32).contains(&pitch);
        let edit_id = ui.id().with("pr_edit");

        // Right-click a note to delete it
        if resp.secondary_clicked() {
            if let Some(p) = resp.interact_pointer_pos() {
                if let Some((i, _)) = hit_note(roll, p, rect, grid_left) {
                    roll.notes.remove(i);
                    intents.push(CommandIntent::new("remove_note"));
                }
            }
        }

        // Begin a drag: resize the right edge, move the body, or draw on empty
        if resp.drag_started() {
            if let Some(p) = resp.interact_pointer_pos() {
                let edit = match hit_note(roll, p, rect, grid_left) {
                    Some((i, true)) => Some(PrEdit::Resize { index: i }),
                    Some((i, false)) => Some(PrEdit::Move {
                        index: i,
                        grab_beat: pointer_beat(p.x) - roll.notes[i].start_beats,
                    }),
                    None if p.x > grid_left && in_range(pointer_pitch(p.y)) => {
                        roll.add(Note {
                            pitch: pointer_pitch(p.y) as u8,
                            start_beats: pointer_beat(p.x).floor(),
                            len_beats: 1.0,
                            velocity: 0.9,
                        });
                        Some(PrEdit::Create { index: roll.notes.len() - 1 })
                    }
                    _ => None,
                };
                if let Some(edit) = edit {
                    ui.data_mut(|d| d.insert_temp(edit_id, edit));
                }
            }
        }

        // Continue a drag
        if resp.dragged() {
            if let (Some(edit), Some(p)) =
                (ui.data(|d| d.get_temp::<PrEdit>(edit_id)), resp.interact_pointer_pos())
            {
                match edit {
                    PrEdit::Resize { index } | PrEdit::Create { index } => {
                        if let Some(n) = roll.notes.get_mut(index) {
                            n.len_beats = (pointer_beat(p.x) - n.start_beats).max(0.25);
                        }
                    }
                    PrEdit::Move { index, grab_beat } => {
                        let pitch = pointer_pitch(p.y);
                        if let Some(n) = roll.notes.get_mut(index) {
                            n.start_beats = (pointer_beat(p.x) - grab_beat).max(0.0);
                            if in_range(pitch) {
                                n.pitch = pitch as u8;
                            }
                        }
                    }
                }
            }
        }

        // Commit a drag: snap to the beat grid, then clear the edit state
        if resp.drag_stopped() {
            if let Some(edit) = ui.data(|d| d.get_temp::<PrEdit>(edit_id)) {
                let index = match edit {
                    PrEdit::Resize { index } | PrEdit::Create { index } | PrEdit::Move { index, .. } => index,
                };
                if let Some(n) = roll.notes.get_mut(index) {
                    n.len_beats = n.len_beats.round().max(1.0);
                    n.start_beats = n.start_beats.round().max(0.0);
                }
                ui.data_mut(|d| d.remove::<PrEdit>(edit_id));
                intents.push(CommandIntent::new("edit_note"));
            }
        }

        // A plain click on an empty cell adds a one-beat note (quick entry)
        if resp.clicked() {
            if let Some(p) = resp.interact_pointer_pos() {
                if p.x > grid_left
                    && in_range(pointer_pitch(p.y))
                    && hit_note(roll, p, rect, grid_left).is_none()
                {
                    roll.add(Note {
                        pitch: pointer_pitch(p.y) as u8,
                        start_beats: pointer_beat(p.x).floor(),
                        len_beats: 1.0,
                        velocity: 0.9,
                    });
                    intents.push(CommandIntent::new("add_note"));
                }
            }
        }
    });
}

// One in-progress piano-roll mouse edit, stashed in egui temp data across frames
#[derive(Clone, Copy)]
enum PrEdit {
    Move { index: usize, grab_beat: f32 },
    Resize { index: usize },
    Create { index: usize },
}

// Note under a point, with whether the point is on its right-edge resize handle
fn hit_note(roll: &PianoRollModel, p: Pos2, rect: Rect, grid_left: f32) -> Option<(usize, bool)> {
    for (i, n) in roll.notes.iter().enumerate() {
        let y = rect.bottom() - (n.pitch - LOW_PITCH + 1) as f32 * ROW_H;
        let left = grid_left + n.start_beats * PX_PER_BEAT;
        let nr = Rect::from_min_size(pos2(left, y + 1.0), vec2((n.len_beats * PX_PER_BEAT).max(2.0), ROW_H - 2.0));
        if nr.contains(p) {
            return Some((i, p.x >= nr.right() - 6.0));
        }
    }
    None
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
