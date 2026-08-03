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

use crate::model::{Clip, TimelineModel, Transport};
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
    transport: &Transport,
    _intents: &mut Vec<CommandIntent>,
) -> bool {
    let mut selection_interacted = false;
    let lane_count = timeline.lanes.len();
    let lanes = lane_count.max(1) as f32;
    let content = vec2(
        LABEL_W + timeline.length_beats * PX_PER_BEAT,
        RULER_H + lanes * LANE_H,
    );

    egui::ScrollArea::horizontal().show(ui, |ui| {
        let (rect, bg) = ui.allocate_exact_size(content, Sense::click());
        let painter = ui.painter_at(rect);
        let grid_left = rect.left() + LABEL_W;
        let beat_x = |beat: f32| grid_left + beat * PX_PER_BEAT;
        let x_beat = |x: f32| ((x - grid_left) / PX_PER_BEAT).max(0.0);
        let y_lane = |y: f32| {
            (((y - (rect.top() + RULER_H)) / LANE_H).floor() as i64)
                .clamp(0, lane_count.saturating_sub(1) as i64) as usize
        };

        painter.rect_filled(rect, 0.0, theme::BG);

        // Loop region shading
        if transport.loop_enabled {
            let loop_rect = Rect::from_min_max(
                pos2(beat_x(transport.loop_start_beats as f32), rect.top()),
                pos2(beat_x(transport.loop_end_beats as f32), rect.bottom()),
            );
            painter.rect_filled(loop_rect, 0.0, theme::ACCENT.linear_multiply(0.06));
        }

        // Bar ruler
        let mut bar_beat = 0.0;
        let mut bar = 1;
        while bar_beat <= timeline.length_beats {
            let x = beat_x(bar_beat);
            painter.line_segment(
                [pos2(x, rect.top()), pos2(x, rect.bottom())],
                Stroke::new(1.0_f32, theme::STROKE),
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
                Stroke::new(1.0_f32, theme::STROKE),
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
            painter.rect_filled(
                clip_rect,
                theme::RADIUS_CONTROL,
                color.linear_multiply(fill),
            );
            painter.rect_stroke(
                clip_rect,
                theme::RADIUS_CONTROL,
                Stroke::new(if selected { 2.0_f32 } else { 1.0_f32 }, color),
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
            let handle = ui.interact(
                handle_rect,
                ui.id().with(("clip_handle", index)),
                Sense::drag(),
            );
            if handle.hovered() || handle.dragged() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
            }
            if handle.dragged() {
                let new_right = handle
                    .interact_pointer_pos()
                    .map(|p| p.x)
                    .unwrap_or(clip_rect.right());
                timeline.selected = Some(index);
                selection_interacted = true;
                timeline.clips[index].len_beats = (x_beat(new_right) - clip.start_beats).max(0.25);
            }
            if handle.drag_stopped() {
                let snapped = timeline.clips[index].len_beats.round().max(1.0);
                timeline.clips[index].len_beats = snapped;
            }

            // Body: select + move (the handle sits on top and wins its strip)
            let body = ui.interact(
                clip_rect,
                ui.id().with(("clip_body", index)),
                Sense::click_and_drag(),
            );
            if body.clicked() {
                timeline.selected = Some(index);
                selection_interacted = true;
            }
            if body.dragged() {
                timeline.selected = Some(index);
                selection_interacted = true;
                let dx = body.drag_delta().x / PX_PER_BEAT;
                timeline.clips[index].start_beats = (clip.start_beats + dx).max(0.0);
                if let Some(p) = body.interact_pointer_pos() {
                    timeline.clips[index].lane = y_lane(p.y);
                }
            }
            if body.drag_stopped() {
                let snapped = timeline.clips[index].start_beats.round().max(0.0);
                timeline.clips[index].start_beats = snapped;
            }
            if selected
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
            selection_interacted = true;
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
                        let start = x_beat(p.x).round().max(0.0);
                        timeline.clips.push(Clip {
                            id: 0,
                            lane,
                            name: format!("Clip {}", timeline.clips.len() + 1),
                            start_beats: start,
                            len_beats: NEW_CLIP_BEATS,
                            kind: crate::theme::SignalKind::Note,
                        });
                        timeline.selected = Some(timeline.clips.len() - 1);
                        selection_interacted = true;
                    }
                }
            }
        } else if bg.clicked() {
            timeline.selected = None;
            selection_interacted = true;
        }

        // Playhead
        let x = beat_x(transport.position_beats as f32);
        painter.line_segment(
            [pos2(x, rect.top()), pos2(x, rect.bottom())],
            Stroke::new(1.5_f32, theme::ACCENT),
        );
    });
    selection_interacted
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Lane;
    use crate::theme::SignalKind;
    use egui::{Event, Modifiers, PointerButton, RawInput};

    fn timeline() -> TimelineModel {
        TimelineModel {
            lanes: vec![Lane {
                name: "Track 1".to_string(),
            }],
            clips: vec![Clip {
                id: 7,
                lane: 0,
                name: "Clip 1".to_string(),
                start_beats: 1.0,
                len_beats: 4.0,
                kind: SignalKind::Note,
            }],
            length_beats: 16.0,
            selected: Some(0),
        }
    }

    fn frame(ctx: &egui::Context, timeline: &mut TimelineModel, events: Vec<Event>) -> bool {
        let raw = RawInput {
            screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 200.0))),
            events,
            ..Default::default()
        };
        let mut changed = false;
        let _ = ctx.run_ui(raw, |ui| {
            changed = draw(ui, timeline, &Transport::default(), &mut Vec::new());
        });
        changed
    }

    #[test]
    fn passive_frame_does_not_report_selection_interaction() {
        let ctx = egui::Context::default();
        let mut timeline = timeline();

        assert!(!frame(&ctx, &mut timeline, Vec::new()));
        assert_eq!(timeline.selected, Some(0));
    }

    #[test]
    fn clicking_an_already_selected_clip_reports_selection_interaction() {
        let ctx = egui::Context::default();
        let mut timeline = timeline();
        let clip_center = pos2(LABEL_W + PX_PER_BEAT * 2.0, RULER_H + LANE_H * 0.5);

        assert!(!frame(
            &ctx,
            &mut timeline,
            vec![Event::PointerMoved(clip_center)],
        ));
        assert!(!frame(
            &ctx,
            &mut timeline,
            vec![
                Event::PointerMoved(clip_center),
                Event::PointerButton {
                    pos: clip_center,
                    button: PointerButton::Primary,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                },
            ],
        ));
        assert!(frame(
            &ctx,
            &mut timeline,
            vec![
                Event::PointerMoved(clip_center),
                Event::PointerButton {
                    pos: clip_center,
                    button: PointerButton::Primary,
                    pressed: false,
                    modifiers: Modifiers::NONE,
                },
            ],
        ));
        assert_eq!(timeline.selected, Some(0));
    }

    #[test]
    fn deleting_the_selected_clip_reports_selection_interaction() {
        let ctx = egui::Context::default();
        let mut timeline = timeline();

        assert!(frame(
            &ctx,
            &mut timeline,
            vec![Event::Key {
                key: egui::Key::Delete,
                physical_key: Some(egui::Key::Delete),
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
        ));
        assert!(timeline.clips.is_empty());
        assert_eq!(timeline.selected, None);
    }
}
