// Author: Jeff
// Date: 2026-06-17
// Description: Arrange lens: surface model plus the interactive timeline drawing.
// Notes: draw() renders lanes, clips as musical objects, a beat ruler, the
//        playhead, and the loop region. Clips are the source of playback truth:
//        double-click an empty lane to create one, drag the body to move it
//        (snapping to the beat on release), drag the right edge to resize, click
//        to select, Delete to remove. The app diffs the mutated model to the
//        engine. New clips carry id 0 until the app assigns a stable id.

use egui::{pos2, vec2, Align2, FontId, Rect, Sense, Stroke, StrokeKind};
use geist_config::commands::CommandIntent;

use crate::model::{snap_beat, Clip, TimelineModel, Transport};
use crate::renderer::ViewPlan;
use crate::theme;
use crate::views::{action_chips, LensSurface};

pub fn surface(plan: &ViewPlan) -> LensSurface {
    LensSurface {
        lens: plan.lens,
        title: plan.title.to_string(),
        purpose: "Compose musical time with tracks, clips, recording, and automation.",
        empty_actions: action_chips(&plan.empty_actions),
    }
}

const LABEL_W: f32 = 92.0;
const RULER_H: f32 = 20.0;
const LANE_H: f32 = 44.0;
const PX_PER_BEAT: f32 = 16.0;
const BEATS_PER_BAR: f32 = 4.0;
// Width of the draggable resize handle on a clip's right edge
const HANDLE_W: f32 = 6.0;
// Default length, in beats, of a clip created by double-click
const NEW_CLIP_BEATS: f32 = 4.0;

// Draw the arrangement and apply create/select/move/resize/delete editing.
pub fn draw(
    ui: &mut egui::Ui,
    timeline: &mut TimelineModel,
    transport: &mut Transport,
    intents: &mut Vec<CommandIntent>,
) {
    // Header: the arrangement grid selector
    ui.horizontal(|ui| {
        crate::views::grid_selector(ui, &mut timeline.grid_div);
    });
    ui.add_space(2.0);

    let lane_count = timeline.lanes.len();
    let lanes = lane_count.max(1) as f32;
    let content = vec2(
        LABEL_W + timeline.length_beats * PX_PER_BEAT,
        RULER_H + lanes * LANE_H,
    );

    egui::ScrollArea::horizontal().show(ui, |ui| {
        let grid = timeline.grid_div;
        let (rect, bg) = ui.allocate_exact_size(content, Sense::click());
        let painter = ui.painter_at(rect);
        let grid_left = rect.left() + LABEL_W;
        let beat_x = |beat: f32| grid_left + beat * PX_PER_BEAT;
        let x_beat = |x: f32| ((x - grid_left) / PX_PER_BEAT).max(0.0);
        let y_lane = |y: f32| (((y - (rect.top() + RULER_H)) / LANE_H).floor() as i64)
            .clamp(0, lane_count.saturating_sub(1) as i64) as usize;

        painter.rect_filled(rect, 0.0, theme::BG);

        // Loop region shading
        if transport.loop_enabled {
            let loop_rect = Rect::from_min_max(
                pos2(beat_x(transport.loop_start_beats as f32), rect.top()),
                pos2(beat_x(transport.loop_end_beats as f32), rect.bottom()),
            );
            painter.rect_filled(loop_rect, 0.0, theme::ACCENT.linear_multiply(0.06));
        }

        // Drag across the bar ruler to set the loop region; a bare click clears it.
        // The anchor beat where the drag began is stashed in egui temp state.
        let ruler_rect = Rect::from_min_max(
            pos2(grid_left, rect.top()),
            pos2(rect.right(), rect.top() + RULER_H),
        );
        let ruler_id = ui.id().with("loop_ruler");
        let ruler = ui.interact(ruler_rect, ruler_id, Sense::click_and_drag());
        if ruler.drag_started() {
            if let Some(p) = ruler.interact_pointer_pos() {
                ui.data_mut(|d| d.insert_temp(ruler_id, x_beat(p.x)));
            }
        }
        if ruler.dragged() {
            if let (Some(p), Some(anchor)) = (
                ruler.interact_pointer_pos(),
                ui.data(|d| d.get_temp::<f32>(ruler_id)),
            ) {
                let a = snap_beat(anchor, grid).max(0.0);
                let b = snap_beat(x_beat(p.x), grid).max(0.0);
                let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
                transport.loop_start_beats = lo as f64;
                transport.loop_end_beats = hi as f64;
                transport.loop_enabled = hi - lo > 0.01;
            }
        }
        if ruler.clicked() {
            transport.loop_enabled = false;
        }

        // Faint subdivision lines at the grid (beat lines a touch stronger)
        if grid > 0.0 {
            let mut g = 0.0;
            while g <= timeline.length_beats + 1e-3 {
                let x = beat_x(g);
                let on_beat = g.fract().abs() < 1e-3;
                let tint = if on_beat { 0.7 } else { 0.4 };
                painter.line_segment(
                    [pos2(x, rect.top() + RULER_H), pos2(x, rect.bottom())],
                    Stroke::new(1.0, theme::STROKE.linear_multiply(tint)),
                );
                g += grid;
            }
        }

        // Bar ruler
        let mut bar_beat = 0.0;
        let mut bar = 1;
        while bar_beat <= timeline.length_beats {
            let x = beat_x(bar_beat);
            painter.line_segment(
                [pos2(x, rect.top()), pos2(x, rect.bottom())],
                Stroke::new(1.0, theme::STROKE),
            );
            painter.text(
                pos2(x + 3.0, rect.top() + 2.0),
                Align2::LEFT_TOP,
                format!("{bar}"),
                FontId::new(10.0, egui::FontFamily::Proportional),
                theme::TEXT_MUTED,
            );
            bar_beat += BEATS_PER_BAR;
            bar += 1;
        }

        // Lanes
        for (lane_index, lane) in timeline.lanes.iter().enumerate() {
            let top = rect.top() + RULER_H + lane_index as f32 * LANE_H;
            let lane_rect = Rect::from_min_size(pos2(rect.left(), top), vec2(rect.width(), LANE_H));
            if lane_index % 2 == 1 {
                painter.rect_filled(lane_rect, 0.0, theme::FAINT);
            }
            painter.text(
                pos2(rect.left() + 8.0, top + LANE_H * 0.5),
                Align2::LEFT_CENTER,
                &lane.name,
                FontId::new(12.0, egui::FontFamily::Proportional),
                theme::TEXT,
            );
            painter.line_segment(
                [pos2(grid_left, top), pos2(rect.right(), top)],
                Stroke::new(1.0, theme::STROKE),
            );
        }

        // Clip blocks: draw and interact. Collect any removal for after the loop.
        let mut remove: Option<usize> = None;
        for index in 0..timeline.clips.len() {
            let clip = timeline.clips[index].clone();
            if clip.lane >= lane_count {
                continue;
            }
            let top = rect.top() + RULER_H + clip.lane as f32 * LANE_H + 4.0;
            let clip_rect = Rect::from_min_size(
                pos2(beat_x(clip.start_beats), top),
                vec2((clip.len_beats * PX_PER_BEAT).max(2.0), LANE_H - 8.0),
            );
            let selected = timeline.selected == Some(index);
            let color = clip.kind.color();
            let fill = if selected { 0.45 } else { 0.30 };
            painter.rect_filled(clip_rect, theme::RADIUS_CONTROL, color.linear_multiply(fill));
            painter.rect_stroke(
                clip_rect,
                theme::RADIUS_CONTROL,
                Stroke::new(if selected { 2.0 } else { 1.0 }, color),
                StrokeKind::Inside,
            );
            painter.text(
                clip_rect.left_top() + vec2(6.0, 3.0),
                Align2::LEFT_TOP,
                &clip.name,
                FontId::new(11.0, egui::FontFamily::Proportional),
                theme::TEXT,
            );

            // Right-edge resize handle
            let handle_rect = Rect::from_min_max(
                pos2(clip_rect.right() - HANDLE_W, clip_rect.top()),
                clip_rect.right_bottom(),
            );
            let handle = ui.interact(handle_rect, ui.id().with(("clip_handle", index)), Sense::drag());
            if handle.hovered() || handle.dragged() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
            }
            if handle.dragged() {
                let new_right = handle.interact_pointer_pos().map(|p| p.x).unwrap_or(clip_rect.right());
                timeline.selected = Some(index);
                timeline.clips[index].len_beats = (x_beat(new_right) - clip.start_beats).max(0.25);
            }
            if handle.drag_stopped() {
                let len = timeline.clips[index].len_beats;
                let snapped = if grid > 0.0 {
                    ((len / grid).round().max(1.0)) * grid
                } else {
                    len.round().max(1.0)
                };
                timeline.clips[index].len_beats = snapped;
            }

            // Body: select + move (the handle sits on top and wins its strip)
            let body = ui.interact(clip_rect, ui.id().with(("clip_body", index)), Sense::click_and_drag());
            if body.clicked() {
                timeline.selected = Some(index);
            }
            if body.dragged() {
                timeline.selected = Some(index);
                let dx = body.drag_delta().x / PX_PER_BEAT;
                timeline.clips[index].start_beats = (clip.start_beats + dx).max(0.0);
                if let Some(p) = body.interact_pointer_pos() {
                    timeline.clips[index].lane = y_lane(p.y);
                }
            }
            if body.drag_stopped() {
                let snapped = snap_beat(timeline.clips[index].start_beats, grid).max(0.0);
                timeline.clips[index].start_beats = snapped;
            }

            // Right-click menu: rename inline, duplicate after itself, or delete
            body.context_menu(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Name");
                    ui.text_edit_singleline(&mut timeline.clips[index].name);
                });
                if ui.button("Duplicate").clicked() {
                    let mut dup = timeline.clips[index].clone();
                    dup.id = 0;
                    dup.start_beats = clip.start_beats + clip.len_beats.max(1.0);
                    timeline.clips.push(dup);
                }
                if ui.button("Delete").clicked() {
                    remove = Some(index);
                }
            });

            // Focus gate: Backspace while typing in a text field must not
            // delete the selected clip
            if selected
                && ui.memory(|m| m.focused().is_none())
                && ui.input(|i| {
                    i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)
                })
            {
                remove = Some(index);
            }
        }
        if let Some(index) = remove {
            timeline.clips.remove(index);
            timeline.selected = None;
        }

        // Double-click an empty lane to create a clip; a bare click deselects
        if bg.double_clicked() {
            if let Some(p) = bg.interact_pointer_pos() {
                if p.x > grid_left {
                    let over_clip = timeline.clips.iter().enumerate().any(|(i, c)| {
                        if c.lane >= lane_count {
                            return false;
                        }
                        let top = rect.top() + RULER_H + c.lane as f32 * LANE_H + 4.0;
                        let r = Rect::from_min_size(
                            pos2(beat_x(c.start_beats), top),
                            vec2((c.len_beats * PX_PER_BEAT).max(2.0), LANE_H - 8.0),
                        );
                        let _ = i;
                        r.contains(p)
                    });
                    if !over_clip {
                        let lane = y_lane(p.y);
                        let start = snap_beat(x_beat(p.x), grid).max(0.0);
                        timeline.clips.push(Clip {
                            id: 0,
                            lane,
                            name: format!("Clip {}", timeline.clips.len() + 1),
                            start_beats: start,
                            len_beats: NEW_CLIP_BEATS,
                            kind: crate::theme::SignalKind::Note,
                        });
                        timeline.selected = Some(timeline.clips.len() - 1);
                    }
                }
            }
        } else if bg.clicked() {
            timeline.selected = None;
        }

        // A browser item dropped on a lane targets that lane's track: effects
        // insert into its chain, anything else runs its intent unchanged
        if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
            let lane = y_lane(pos.y);
            if bg.dnd_hover_payload::<CommandIntent>().is_some() && lane < lane_count {
                let top = rect.top() + RULER_H + lane as f32 * LANE_H;
                let lane_rect =
                    Rect::from_min_size(pos2(rect.left(), top), vec2(rect.width(), LANE_H));
                painter.rect_stroke(
                    lane_rect,
                    0.0,
                    Stroke::new(1.5, theme::ACCENT),
                    StrokeKind::Inside,
                );
            }
            if let Some(intent) = bg.dnd_release_payload::<CommandIntent>() {
                if lane < lane_count {
                    if let Some(name) = intent.command.strip_prefix("add_effect:") {
                        intents.push(CommandIntent::new(format!("add_effect_to:{lane}:{name}")));
                    } else {
                        intents.push((*intent).clone());
                    }
                }
            }
        }

        // Playhead
        let x = beat_x(transport.position_beats as f32);
        painter.line_segment(
            [pos2(x, rect.top()), pos2(x, rect.bottom())],
            Stroke::new(1.5, theme::ACCENT),
        );
    });
}
