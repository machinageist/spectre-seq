// Author: Jeff
// Date: 2026-06-15
// Description: Build lens: surface model plus the spatial node-graph drawing.
// Notes: Graph empty actions are workflow-configured so modular profiles can lead.
//        draw() renders semantic node blocks with signal-colored typed ports and
//        bezier cables; nodes are draggable. Drag-to-connect lands in a follow-up;
//        this slice establishes the spatial map and color language.

use egui::{pos2, vec2, Align2, FontId, Pos2, Rect, Sense, Shape, Stroke, StrokeKind};
use geist_config::commands::CommandIntent;

use crate::model::{GraphModel, GraphNode};
use crate::renderer::ViewPlan;
use crate::theme;
use crate::views::{action_chips, LensSurface};

pub fn surface(plan: &ViewPlan) -> LensSurface {
    LensSurface {
        lens: plan.lens,
        title: plan.title.to_string(),
        purpose: "Build and understand sound flow.",
        empty_actions: action_chips(&plan.empty_actions),
    }
}

const NODE_W: f32 = 124.0;
const HEADER_H: f32 = 24.0;
const PORT_ROW_H: f32 = 20.0;
const PORT_R: f32 = 5.0;
const GRID_STEP: f32 = 36.0;
const BEZIER_STEPS: usize = 24;

// Draw the graph canvas: grid, cables, then draggable node blocks
pub fn draw(ui: &mut egui::Ui, graph: &mut GraphModel, _intents: &mut [CommandIntent]) {
    let (canvas, _) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
    let origin = canvas.min.to_vec2();
    let painter = ui.painter_at(canvas);

    // Backdrop and faint grid for depth
    painter.rect_filled(canvas, 0.0, theme::BG);
    paint_grid(&painter, canvas);

    // Move nodes first so cables and bodies render at the new positions
    for index in 0..graph.nodes.len() {
        let rect = node_rect(&graph.nodes[index], origin);
        let resp = ui.interact(
            rect,
            ui.id().with(("gnode", graph.nodes[index].id)),
            Sense::drag(),
        );
        if resp.dragged() {
            graph.nodes[index].pos.0 += resp.drag_delta().x;
            graph.nodes[index].pos.1 += resp.drag_delta().y;
        }
    }

    // Cables under the nodes
    for cable in &graph.cables {
        let (Some(from), Some(to)) = (graph.node(cable.from_node), graph.node(cable.to_node))
        else {
            continue;
        };
        let p0 = output_port_pos(from, cable.from_port, origin);
        let p3 = input_port_pos(to, cable.to_port, origin);
        painter.add(Shape::line(
            bezier(p0, p3),
            Stroke::new(2.0, cable.kind.color()),
        ));
    }

    // Node bodies, headers, ports, labels
    for node in &graph.nodes {
        paint_node(&painter, node, origin);
    }
}

// Faint dotted grid across the canvas
fn paint_grid(painter: &egui::Painter, canvas: Rect) {
    let mut x = canvas.left();
    while x < canvas.right() {
        let mut y = canvas.top();
        while y < canvas.bottom() {
            painter.circle_filled(pos2(x, y), 1.0, theme::STROKE);
            y += GRID_STEP;
        }
        x += GRID_STEP;
    }
}

// The rectangle a node occupies on screen
fn node_rect(node: &GraphNode, origin: egui::Vec2) -> Rect {
    let rows = node.inputs.len().max(node.outputs.len()).max(1) as f32;
    let height = HEADER_H + rows * PORT_ROW_H + 6.0;
    Rect::from_min_size(pos2(node.pos.0, node.pos.1) + origin, vec2(NODE_W, height))
}

// Screen position of an input port (left edge)
fn input_port_pos(node: &GraphNode, index: usize, origin: egui::Vec2) -> Pos2 {
    let rect = node_rect(node, origin);
    pos2(
        rect.left(),
        rect.top() + HEADER_H + index as f32 * PORT_ROW_H + PORT_ROW_H * 0.5,
    )
}

// Screen position of an output port (right edge)
fn output_port_pos(node: &GraphNode, index: usize, origin: egui::Vec2) -> Pos2 {
    let rect = node_rect(node, origin);
    pos2(
        rect.right(),
        rect.top() + HEADER_H + index as f32 * PORT_ROW_H + PORT_ROW_H * 0.5,
    )
}

// Paint a node block: body, header, title, and its colored ports
fn paint_node(painter: &egui::Painter, node: &GraphNode, origin: egui::Vec2) {
    let rect = node_rect(node, origin);
    let radius = theme::RADIUS_PANEL;
    painter.rect_filled(rect, radius, theme::PANEL_RAISED);
    painter.rect_stroke(
        rect,
        radius,
        Stroke::new(1.0, theme::STROKE_STRONG),
        StrokeKind::Inside,
    );

    // Header strip
    let header = Rect::from_min_size(rect.min, vec2(rect.width(), HEADER_H));
    painter.rect_filled(header, radius, theme::PANEL_HOVER);
    painter.text(
        header.left_center() + vec2(8.0, 0.0),
        Align2::LEFT_CENTER,
        &node.name,
        FontId::new(13.0, egui::FontFamily::Proportional),
        theme::TEXT,
    );

    let label_font = FontId::new(11.0, egui::FontFamily::Proportional);
    for (i, port) in node.inputs.iter().enumerate() {
        let p = input_port_pos(node, i, origin);
        painter.circle_filled(p, PORT_R, port.kind.color());
        painter.circle_stroke(p, PORT_R, Stroke::new(1.0, theme::BG));
        painter.text(
            p + vec2(9.0, 0.0),
            Align2::LEFT_CENTER,
            &port.name,
            label_font.clone(),
            theme::TEXT_MUTED,
        );
    }
    for (i, port) in node.outputs.iter().enumerate() {
        let p = output_port_pos(node, i, origin);
        painter.circle_filled(p, PORT_R, port.kind.color());
        painter.circle_stroke(p, PORT_R, Stroke::new(1.0, theme::BG));
        painter.text(
            p - vec2(9.0, 0.0),
            Align2::RIGHT_CENTER,
            &port.name,
            label_font.clone(),
            theme::TEXT_MUTED,
        );
    }
}

// Sample a left-to-right cubic bezier between two points into a polyline
fn bezier(p0: Pos2, p3: Pos2) -> Vec<Pos2> {
    let dx = (p3.x - p0.x).abs().max(40.0) * 0.5;
    let p1 = pos2(p0.x + dx, p0.y);
    let p2 = pos2(p3.x - dx, p3.y);
    (0..=BEZIER_STEPS)
        .map(|i| {
            let t = i as f32 / BEZIER_STEPS as f32;
            cubic_point(p0, p1, p2, p3, t)
        })
        .collect()
}

// One point along a cubic bezier at parameter t
fn cubic_point(p0: Pos2, p1: Pos2, p2: Pos2, p3: Pos2, t: f32) -> Pos2 {
    let u = 1.0 - t;
    let (w0, w1, w2, w3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    pos2(
        w0 * p0.x + w1 * p1.x + w2 * p2.x + w3 * p3.x,
        w0 * p0.y + w1 * p1.y + w2 * p2.y + w3 * p3.y,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bezier_starts_and_ends_on_its_anchors() {
        let pts = bezier(pos2(0.0, 0.0), pos2(100.0, 50.0));
        assert_eq!(pts.first().copied(), Some(pos2(0.0, 0.0)));
        assert_eq!(pts.last().copied(), Some(pos2(100.0, 50.0)));
        assert_eq!(pts.len(), BEZIER_STEPS + 1);
    }

    #[test]
    fn port_positions_sit_on_node_edges() {
        let node = GraphNode {
            id: 1,
            name: "N".into(),
            pos: (10.0, 20.0),
            inputs: vec![crate::model::Port {
                name: "in".into(),
                kind: theme::SignalKind::Audio,
            }],
            outputs: vec![crate::model::Port {
                name: "out".into(),
                kind: theme::SignalKind::Audio,
            }],
        };
        let origin = vec2(0.0, 0.0);
        let rect = node_rect(&node, origin);
        assert!((input_port_pos(&node, 0, origin).x - rect.left()).abs() < 1e-6);
        assert!((output_port_pos(&node, 0, origin).x - rect.right()).abs() < 1e-6);
    }
}
