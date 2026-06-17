// Author: Jeff
// Date: 2026-06-15
// Description: Arrange lens: surface model plus the timeline/arrangement drawing.
// Notes: Empty actions stay visible and come from workflow/frame planning. draw()
//        renders lanes, clips as musical objects, a beat ruler, the playhead, and
//        the loop region; clicking a clip emits a select intent.

use egui::{pos2, vec2, Align2, FontId, Rect, Sense, Stroke, StrokeKind};
use geist_config::commands::CommandIntent;

use crate::model::{TimelineModel, Transport};
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

// Draw the arrangement: lane gutter, beat ruler, clips, loop region, playhead
pub fn draw(
    ui: &mut egui::Ui,
    timeline: &TimelineModel,
    transport: &Transport,
    intents: &mut Vec<CommandIntent>,
) {
    let lanes = timeline.lanes.len().max(1) as f32;
    let content = vec2(
        LABEL_W + timeline.length_beats * PX_PER_BEAT,
        RULER_H + lanes * LANE_H,
    );

    egui::ScrollArea::horizontal().show(ui, |ui| {
        let (rect, _) = ui.allocate_exact_size(content, Sense::hover());
        let painter = ui.painter_at(rect);
        let grid_left = rect.left() + LABEL_W;
        let beat_x = |beat: f32| grid_left + beat * PX_PER_BEAT;

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

        // Lanes and clips
        for (lane_index, lane) in timeline.lanes.iter().enumerate() {
            let top = rect.top() + RULER_H + lane_index as f32 * LANE_H;
            let lane_rect = Rect::from_min_size(pos2(rect.left(), top), vec2(rect.width(), LANE_H));
            if lane_index % 2 == 1 {
                painter.rect_filled(lane_rect, 0.0, theme::FAINT);
            }
            // Lane name gutter
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

        // Clip blocks
        for clip in &timeline.clips {
            if clip.lane >= timeline.lanes.len() {
                continue;
            }
            let top = rect.top() + RULER_H + clip.lane as f32 * LANE_H + 4.0;
            let clip_rect = Rect::from_min_size(
                pos2(beat_x(clip.start_beats), top),
                vec2(clip.len_beats * PX_PER_BEAT, LANE_H - 8.0),
            );
            let color = clip.kind.color();
            painter.rect_filled(clip_rect, theme::RADIUS_CONTROL, color.linear_multiply(0.30));
            painter.rect_stroke(
                clip_rect,
                theme::RADIUS_CONTROL,
                Stroke::new(1.0, color),
                StrokeKind::Inside,
            );
            painter.text(
                clip_rect.left_top() + vec2(6.0, 3.0),
                Align2::LEFT_TOP,
                &clip.name,
                FontId::new(11.0, egui::FontFamily::Proportional),
                theme::TEXT,
            );

            let resp = ui.interact(
                clip_rect,
                ui.id().with(("clip", clip.lane, clip.name.as_str())),
                Sense::click(),
            );
            if resp.clicked() {
                intents.push(CommandIntent::new(format!("select_clip:{}", clip.name)));
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
