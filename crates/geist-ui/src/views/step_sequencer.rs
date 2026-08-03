// Author: Jeff
// Date: 2026-06-15
// Description: Step-sequencer editor: per-track gate grid with note rows.
// Notes: Mirrors the engine's tempo-synced step pattern. draw() renders a track
//        selector and a clickable gate grid for the selected track; toggles and
//        clears mutate the model and emit an intent the app diffs back to the
//        engine. Row 0 is the lowest note, drawn at the bottom like a keyboard.

use egui::{pos2, vec2, Align2, FontId, Rect, RichText, Sense, Stroke, StrokeKind};
use geist_config::commands::CommandIntent;

use crate::model::StepSequencerModel;
use crate::theme;

// Gate cell footprint and spacing, plus the note-label gutter width
const CELL: f32 = 22.0;
const GAP: f32 = 3.0;
const LABEL_W: f32 = 40.0;
// Sixteenth-note steps: four steps per beat, matching the engine sequencer
const STEPS_PER_BEAT: f32 = 4.0;

// Draw the step sequencer and apply track selection, clears, and gate toggles.
// `playhead_beats` highlights the column under the transport when set.
pub fn draw(
    ui: &mut egui::Ui,
    seq: &mut StepSequencerModel,
    playhead_beats: Option<f32>,
    intents: &mut Vec<CommandIntent>,
) {
    if seq.tracks.is_empty() {
        ui.label(RichText::new("No tracks").color(theme::TEXT_MUTED));
        return;
    }

    // Track selector and a clear for the selected pattern
    ui.horizontal(|ui| {
        ui.label(RichText::new("Track").small().color(theme::TEXT_MUTED));
        for track in 0..seq.tracks.len() {
            ui.selectable_value(&mut seq.selected, track, format!("{}", track + 1));
        }
        if ui.button("Clear").clicked() {
            let selected = seq.selected.min(seq.tracks.len() - 1);
            seq.tracks[selected].clear();
            intents.push(CommandIntent::new("clear_pattern"));
        }
    });
    ui.add_space(6.0);

    let selected = seq.selected.min(seq.tracks.len() - 1);
    let pattern = &mut seq.tracks[selected];
    let stride = CELL + GAP;
    let grid_size = vec2(
        LABEL_W + pattern.steps as f32 * stride,
        pattern.rows as f32 * stride,
    );

    // Column under the transport, if any, for the moving highlight
    let play_step =
        playhead_beats.map(|ph| ((ph * STEPS_PER_BEAT) as usize).rem_euclid(pattern.steps.max(1)));

    egui::ScrollArea::both().show(ui, |ui| {
        let (area, _resp) = ui.allocate_exact_size(grid_size, Sense::hover());
        let painter = ui.painter_at(area);

        // Tint the playing column behind the gates
        if let Some(step) = play_step {
            let x = area.left() + LABEL_W + step as f32 * stride;
            let column = Rect::from_min_size(
                pos2(x - GAP * 0.5, area.top()),
                vec2(CELL + GAP, pattern.rows as f32 * stride),
            );
            painter.rect_filled(column, 3.0, theme::ACCENT.linear_multiply(0.14));
        }

        for row in 0..pattern.rows {
            // Row 0 is the lowest note, so draw it at the bottom
            let display_row = pattern.rows - 1 - row;
            let y = area.top() + display_row as f32 * stride;

            let midi = pattern.base_midi as i32 + row as i32;
            painter.text(
                pos2(area.left() + LABEL_W - 5.0, y + CELL * 0.5),
                Align2::RIGHT_CENTER,
                note_name(midi),
                FontId::new(9.0, egui::FontFamily::Proportional),
                if is_black_key(midi) {
                    theme::TEXT_MUTED
                } else {
                    theme::TEXT
                },
            );

            for step in 0..pattern.steps {
                let x = area.left() + LABEL_W + step as f32 * stride;
                let cell_rect = Rect::from_min_size(pos2(x, y), vec2(CELL, CELL));
                let id = ui.id().with(("step_cell", selected, row, step));
                let resp = ui.interact(cell_rect, id, Sense::click());
                if resp.clicked() {
                    let on = !pattern.cell(row, step);
                    pattern.set(row, step, on);
                    intents.push(CommandIntent::new("set_cell"));
                }
                let color = if pattern.cell(row, step) {
                    theme::ACCENT
                } else if step % 4 == 0 {
                    theme::PANEL_RAISED
                } else {
                    theme::INSET
                };
                painter.rect_filled(cell_rect, 3.0, color);
                if resp.hovered() {
                    painter.rect_stroke(
                        cell_rect,
                        3.0,
                        Stroke::new(1.0, theme::STROKE_STRONG),
                        StrokeKind::Inside,
                    );
                }
            }
        }
    });
}

// MIDI note number to a short name like "C4" or "F#3"
fn note_name(midi: i32) -> String {
    const NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let octave = midi.div_euclid(12) - 1;
    format!("{}{}", NAMES[midi.rem_euclid(12) as usize], octave)
}

// Whether a MIDI pitch class is a black key
fn is_black_key(midi: i32) -> bool {
    matches!(midi.rem_euclid(12), 1 | 3 | 6 | 8 | 10)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_names_are_conventional() {
        assert_eq!(note_name(60), "C4");
        assert_eq!(note_name(61), "C#4");
        assert_eq!(note_name(48), "C3");
    }

    #[test]
    fn black_keys_are_the_accidentals() {
        for white in [60, 62, 64, 65, 67, 69, 71] {
            assert!(!is_black_key(white));
        }
        for black in [61, 63, 66, 68, 70] {
            assert!(is_black_key(black));
        }
    }
}
