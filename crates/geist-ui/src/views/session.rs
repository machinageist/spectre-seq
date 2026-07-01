// Author: Jeff
// Date: 2026-06-19
// Description: Session clip-launch grid (Ableton session view): scenes by tracks.
// Notes: draw() renders a grid of clip slots, `scenes` rows by `tracks` columns.
//        Click a filled slot to select + launch it; double-click an empty slot to
//        create a clip; the scene column on the left launches a whole row. Slot
//        playing/queued state is drawn from the model; the app's launcher (engine)
//        owns the real timing and updates the model. All actions emit intents the
//        app diffs back to the engine.

use egui::{pos2, vec2, Align2, FontId, Rect, RichText, Sense, Stroke, StrokeKind};
use geist_config::commands::CommandIntent;

use crate::model::SessionGrid;
use crate::theme;

// Slot footprint and spacing, plus the scene-launch gutter width
const SLOT_W: f32 = 96.0;
const SLOT_H: f32 = 26.0;
const GAP: f32 = 4.0;
const SCENE_W: f32 = 40.0;
const HEADER_H: f32 = 20.0;

// Launch quantization options, as (label, beats); 0 = immediate
const QUANT_OPTIONS: [(&str, f32); 6] = [
    ("Off", 0.0),
    ("1/4", 1.0),
    ("1/2", 2.0),
    ("1 Bar", 4.0),
    ("2 Bar", 8.0),
    ("4 Bar", 16.0),
];

// Draw the session grid and apply selection, launch, stop, and create actions
pub fn draw(ui: &mut egui::Ui, grid: &mut SessionGrid, intents: &mut Vec<CommandIntent>) {
    if grid.tracks == 0 || grid.scenes == 0 {
        ui.label(RichText::new("No session slots").color(theme::TEXT_MUTED));
        return;
    }

    ui.horizontal(|ui| {
        ui.label(RichText::new("Session").small().color(theme::TEXT_MUTED));
        if ui.button("Stop All").clicked() {
            for slot in &mut grid.slots {
                slot.playing = false;
                slot.queued = false;
            }
            intents.push(CommandIntent::new("session_stop_all"));
        }
        ui.separator();
        ui.label(RichText::new("Quant").small().color(theme::TEXT_MUTED));
        for (label, beats) in QUANT_OPTIONS {
            ui.selectable_value(&mut grid.launch_quant, beats, label);
        }
    });
    ui.add_space(6.0);

    let tracks = grid.tracks;
    let scenes = grid.scenes;
    let stride_x = SLOT_W + GAP;
    let stride_y = SLOT_H + GAP;
    let size = vec2(
        SCENE_W + GAP + tracks as f32 * stride_x,
        HEADER_H + GAP + scenes as f32 * stride_y,
    );

    egui::ScrollArea::both().show(ui, |ui| {
        let (area, _resp) = ui.allocate_exact_size(size, Sense::hover());
        let painter = ui.painter_at(area);
        let grid_left = area.left() + SCENE_W + GAP;
        let grid_top = area.top() + HEADER_H + GAP;

        // Track headers
        for track in 0..tracks {
            let x = grid_left + track as f32 * stride_x;
            painter.text(
                pos2(x + SLOT_W * 0.5, area.top() + HEADER_H * 0.5),
                Align2::CENTER_CENTER,
                format!("Track {}", track + 1),
                FontId::new(10.0, egui::FontFamily::Proportional),
                theme::TEXT_MUTED,
            );
        }

        for scene in 0..scenes {
            let y = grid_top + scene as f32 * stride_y;

            // Scene-launch button in the left gutter
            let scene_rect = Rect::from_min_size(pos2(area.left(), y), vec2(SCENE_W, SLOT_H));
            let scene_id = ui.id().with(("scene_launch", scene));
            let scene_resp = ui.interact(scene_rect, scene_id, Sense::click());
            if scene_resp.clicked() {
                for track in 0..tracks {
                    if let Some(slot) = grid.slot_mut(track, scene) {
                        if slot.filled {
                            slot.queued = true;
                        }
                    }
                }
                intents.push(CommandIntent::new(format!("session_launch_scene:{scene}")));
            }
            painter.rect_filled(scene_rect, 3.0, theme::PANEL_RAISED);
            painter.text(
                scene_rect.center(),
                Align2::CENTER_CENTER,
                format!("{}", scene + 1),
                FontId::new(10.0, egui::FontFamily::Proportional),
                if scene_resp.hovered() { theme::TEXT } else { theme::TEXT_MUTED },
            );

            for track in 0..tracks {
                let x = grid_left + track as f32 * stride_x;
                let cell = Rect::from_min_size(pos2(x, y), vec2(SLOT_W, SLOT_H));
                let id = ui.id().with(("session_slot", scene, track));
                let resp = ui.interact(cell, id, Sense::click());
                let index = grid.index(track, scene);
                let (filled, playing, queued, name) = grid
                    .slot(track, scene)
                    .map(|s| (s.filled, s.playing, s.queued, s.name.clone()))
                    .unwrap_or((false, false, false, String::new()));

                // Double-click empty -> create a clip; click filled -> select + launch
                if resp.double_clicked() && !filled {
                    if let Some(slot) = grid.slot_mut(track, scene) {
                        slot.filled = true;
                        slot.name = format!("Clip {}", scene + 1);
                    }
                    grid.selected = Some(index);
                    intents.push(CommandIntent::new(format!("session_create:{track}:{scene}")));
                } else if resp.clicked() && filled {
                    grid.selected = Some(index);
                    if let Some(slot) = grid.slot_mut(track, scene) {
                        slot.queued = true;
                    }
                    intents.push(CommandIntent::new(format!("session_launch:{track}:{scene}")));
                } else if resp.clicked() {
                    grid.selected = Some(index);
                }

                let selected = grid.selected == Some(index);
                let fill = if !filled {
                    theme::INSET
                } else if playing {
                    theme::ACCENT
                } else if queued {
                    theme::ACCENT_DIM
                } else {
                    theme::PANEL_RAISED
                };
                painter.rect_filled(cell, 3.0, fill);
                if filled {
                    // A play triangle, then the clip name
                    let tri_x = cell.left() + 8.0;
                    let cy = cell.center().y;
                    painter.add(egui::Shape::convex_polygon(
                        vec![
                            pos2(tri_x, cy - 4.0),
                            pos2(tri_x, cy + 4.0),
                            pos2(tri_x + 6.0, cy),
                        ],
                        if playing { theme::BG } else { theme::TEXT },
                        Stroke::NONE,
                    ));
                    painter.text(
                        pos2(cell.left() + 20.0, cy),
                        Align2::LEFT_CENTER,
                        name,
                        FontId::new(10.0, egui::FontFamily::Proportional),
                        if playing { theme::BG } else { theme::TEXT },
                    );
                }
                let stroke = if selected {
                    Stroke::new(1.5, theme::ACCENT)
                } else if resp.hovered() {
                    Stroke::new(1.0, theme::STROKE_STRONG)
                } else {
                    Stroke::new(1.0, theme::STROKE)
                };
                painter.rect_stroke(cell, 3.0, stroke, StrokeKind::Inside);
            }
        }
    });
}
