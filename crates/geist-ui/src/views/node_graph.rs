// Author: Jeff
// Date: 2026-06-15
// Description: Build lens: surface model plus the spatial node-graph drawing.
// Notes: Graph empty actions are workflow-configured so modular profiles can lead.
//        draw() renders semantic node blocks with signal-colored typed ports and
//        bezier cables; nodes drag to move, ports drag to patch. Any output
//        connects to any input; kind mismatch tints the live cable amber as
//        feedback but never refuses (Jeff, 2026-07-03). Dragging a patched
//        input picks its cable up for re-route or removal. Connect/disconnect
//        emit graph_connect:/graph_disconnect: intents for the app layer.

use egui::{pos2, vec2, Align2, FontId, Pos2, Rect, Sense, Shape, Stroke, StrokeKind};
use geist_config::commands::CommandIntent;

use crate::model::{GraphModel, GraphNode};
use crate::renderer::ViewPlan;
use crate::theme;
use crate::theme::SignalKind;
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
const PORT_HIT_R: f32 = 9.0;
const GRID_STEP: f32 = 36.0;
const BEZIER_STEPS: usize = 24;
const CABLE_W_MONO: f32 = 2.0;
const CABLE_W_POLY: f32 = 3.5;

// A patch drag in flight, anchored at one port; lives in egui temp memory
#[derive(Clone, Copy, PartialEq)]
struct PendingCable {
    node: u64,
    port: usize,
    is_output: bool,
    kind: SignalKind,
}

// One port's screen geometry resolved for hit-testing
struct PortHit {
    node: u64,
    port: usize,
    is_output: bool,
    kind: SignalKind,
    pos: Pos2,
}

// Stroke width for a cable by its polyphonic channel count
fn cable_stroke_width(channels: u16) -> f32 {
    if channels > 1 {
        CABLE_W_POLY
    } else {
        CABLE_W_MONO
    }
}

// Draw the graph canvas: grid, cables, draggable node blocks, patch drags
pub fn draw(ui: &mut egui::Ui, graph: &mut GraphModel, intents: &mut Vec<CommandIntent>) {
    let (canvas, canvas_resp) = ui.allocate_exact_size(ui.available_size(), Sense::click());
    let origin = canvas.min.to_vec2();
    let painter = ui.painter_at(canvas);
    let pending_id = ui.id().with("pending_cable");
    let sel_id = ui.id().with("graph_selected");
    let mut pending: Option<PendingCable> = ui.memory(|m| m.data.get_temp(pending_id));
    let mut selected: Option<u64> = ui.memory(|m| m.data.get_temp(sel_id));

    // Backdrop and faint grid for depth
    painter.rect_filled(canvas, 0.0, theme::BG);
    paint_grid(&painter, canvas);

    // Move nodes first so cables and bodies render at the new positions;
    // port hit-zones register later, so a drag on a port never moves the node.
    // A click (no drag) selects; a click on empty canvas below clears it
    for index in 0..graph.nodes.len() {
        let rect = node_rect(&graph.nodes[index], origin);
        let resp = ui.interact(
            rect,
            ui.id().with(("gnode", graph.nodes[index].id)),
            Sense::click_and_drag(),
        );
        if resp.clicked() {
            selected = Some(graph.nodes[index].id);
        }
        if resp.dragged() && pending.is_none() {
            selected = Some(graph.nodes[index].id);
            graph.nodes[index].pos.0 += resp.drag_delta().x;
            graph.nodes[index].pos.1 += resp.drag_delta().y;
        }
    }

    // A bare click on empty canvas clears the selection
    if canvas_resp.clicked() {
        selected = None;
    }

    // Port hit-zones: registered after node bodies so they win the pointer.
    // A drag off an output (or a free input) starts a patch; a drag off a
    // patched input picks that cable up so it can be re-routed or dropped
    let hits = port_hits(graph, origin);
    for hit in &hits {
        let rect = Rect::from_center_size(hit.pos, vec2(PORT_HIT_R * 2.0, PORT_HIT_R * 2.0));
        let resp = ui.interact(
            rect,
            ui.id().with(("gport", hit.node, hit.port, hit.is_output)),
            Sense::drag(),
        );
        if resp.drag_started() && pending.is_none() {
            pending = Some(start_patch(graph, hit, intents));
        }
    }

    // Cables under the nodes; width tracks the polyphonic channel count
    for cable in &graph.cables {
        let (Some(from), Some(to)) = (graph.node(cable.from_node), graph.node(cable.to_node))
        else {
            continue;
        };
        let p0 = output_port_pos(from, cable.from_port, origin);
        let p3 = input_port_pos(to, cable.to_port, origin);
        painter.add(Shape::line(
            bezier(p0, p3),
            Stroke::new(cable_stroke_width(cable.channels), cable.kind.color()),
        ));
    }

    // Node bodies, headers, ports, labels; the selected node gets an accent ring
    for node in &graph.nodes {
        paint_node(&painter, node, origin);
        if selected == Some(node.id) {
            painter.rect_stroke(
                node_rect(node, origin).expand(2.0),
                theme::RADIUS_PANEL,
                Stroke::new(2.0, theme::ACCENT),
                StrokeKind::Outside,
            );
        }
    }

    // Drop a browser item onto the canvas to add a rack node at the pointer.
    // Hovering a payload previews an accent marker; releasing adds and selects
    if let Some(pointer) = ui.input(|i| i.pointer.latest_pos()) {
        if canvas_resp.dnd_hover_payload::<CommandIntent>().is_some() {
            painter.circle_stroke(pointer, PORT_HIT_R, Stroke::new(2.0, theme::ACCENT));
        }
        if let Some(intent) = canvas_resp.dnd_release_payload::<CommandIntent>() {
            if let Some(name) = node_name_from_intent(&intent) {
                let pos = (pointer.x - origin.x, pointer.y - origin.y);
                let id = graph.add_node(name.clone(), SignalKind::Audio, pos);
                selected = Some(id);
                intents.push(CommandIntent::new(format!("graph_add:{name}")));
            }
        }
    }

    // Delete/Backspace removes the selected node and its cables while the
    // canvas holds the pointer and no text field is focused
    if let Some(id) = selected {
        let pressed =
            ui.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace));
        if pressed && ui.rect_contains_pointer(canvas) && ui.memory(|m| m.focused().is_none()) {
            if graph.remove_node(id) {
                intents.push(CommandIntent::new(format!("graph_remove_node:{id}")));
            }
            selected = None;
        }
    }

    // Live patch cable: anchor to pointer, amber when kinds disagree
    if let Some(p) = pending {
        let target = hover_target(&hits, ui, &p);
        if let Some(anchor) = anchor_pos(graph, &p, origin) {
            let tip = target
                .map(|t| t.pos)
                .or_else(|| ui.input(|i| i.pointer.latest_pos()))
                .unwrap_or(anchor);
            let color = match target {
                Some(t) if t.kind != p.kind => theme::METER_MID,
                _ => p.kind.color(),
            };
            let pts = if p.is_output {
                bezier(anchor, tip)
            } else {
                bezier(tip, anchor)
            };
            painter.add(Shape::line(pts, Stroke::new(CABLE_W_MONO, color)));
            if let Some(t) = target {
                painter.circle_stroke(t.pos, PORT_HIT_R, Stroke::new(2.0, color));
            }
        } else {
            pending = None;
        }

        // Drop: over an opposite port connects (any-to-any); elsewhere cancels
        if ui.input(|i| i.pointer.any_released()) {
            if let Some(t) = target {
                let (from_node, from_port, to_node, to_port) = if p.is_output {
                    (p.node, p.port, t.node, t.port)
                } else {
                    (t.node, t.port, p.node, p.port)
                };
                if graph.connect(from_node, from_port, to_node, to_port) {
                    intents.push(CommandIntent::new(format!(
                        "graph_connect:{from_node}:{from_port}:{to_node}:{to_port}"
                    )));
                }
            }
            pending = None;
        }
    }

    ui.memory_mut(|m| {
        match pending {
            Some(p) => {
                m.data.insert_temp(pending_id, p);
            }
            None => {
                m.data.remove::<PendingCable>(pending_id);
            }
        }
        match selected {
            Some(id) => {
                m.data.insert_temp(sel_id, id);
            }
            None => {
                m.data.remove::<u64>(sel_id);
            }
        }
    });
}

// The rack-node label a dropped browser payload should create, if any.
// Browser payloads name a device to add; non-device commands yield None
fn node_name_from_intent(intent: &CommandIntent) -> Option<String> {
    let cmd = intent.command.as_str();
    if cmd == "show_device" {
        return Some("Geist Synth".into());
    }
    for prefix in ["add_effect:", "select_device:", "insert:"] {
        if let Some(name) = cmd.strip_prefix(prefix) {
            return Some(name.into());
        }
    }
    None
}

// Begin a patch drag from a port; picking up a patched input re-routes its
// cable from the far output and reports the break as a disconnect intent
fn start_patch(
    graph: &mut GraphModel,
    hit: &PortHit,
    intents: &mut Vec<CommandIntent>,
) -> PendingCable {
    if !hit.is_output {
        if let Some(old) = graph.disconnect_input(hit.node, hit.port) {
            intents.push(CommandIntent::new(format!(
                "graph_disconnect:{}:{}:{}:{}",
                old.from_node, old.from_port, old.to_node, old.to_port
            )));
            return PendingCable {
                node: old.from_node,
                port: old.from_port,
                is_output: true,
                kind: old.kind,
            };
        }
    }
    PendingCable {
        node: hit.node,
        port: hit.port,
        is_output: hit.is_output,
        kind: hit.kind,
    }
}

// Resolve every port to its screen position for hit-testing
fn port_hits(graph: &GraphModel, origin: egui::Vec2) -> Vec<PortHit> {
    let mut hits = Vec::new();
    for node in &graph.nodes {
        for (i, port) in node.inputs.iter().enumerate() {
            hits.push(PortHit {
                node: node.id,
                port: i,
                is_output: false,
                kind: port.kind,
                pos: input_port_pos(node, i, origin),
            });
        }
        for (i, port) in node.outputs.iter().enumerate() {
            hits.push(PortHit {
                node: node.id,
                port: i,
                is_output: true,
                kind: port.kind,
                pos: output_port_pos(node, i, origin),
            });
        }
    }
    hits
}

// The opposite-direction port under the pointer, if any
fn hover_target<'a>(
    hits: &'a [PortHit],
    ui: &egui::Ui,
    pending: &PendingCable,
) -> Option<&'a PortHit> {
    let pointer = ui.input(|i| i.pointer.latest_pos())?;
    hits.iter()
        .filter(|h| h.is_output != pending.is_output)
        .find(|h| h.pos.distance(pointer) <= PORT_HIT_R)
}

// Screen anchor of the pending patch's fixed end; None if the node vanished
fn anchor_pos(graph: &GraphModel, pending: &PendingCable, origin: egui::Vec2) -> Option<Pos2> {
    let node = graph.node(pending.node)?;
    Some(if pending.is_output {
        output_port_pos(node, pending.port, origin)
    } else {
        input_port_pos(node, pending.port, origin)
    })
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
    use crate::model::Port;

    fn two_nodes() -> GraphModel {
        let port = |name: &str, kind: SignalKind| Port {
            name: name.into(),
            kind,
        };
        GraphModel {
            nodes: vec![
                GraphNode {
                    id: 1,
                    name: "Osc".into(),
                    pos: (0.0, 0.0),
                    inputs: vec![port("V/Oct", SignalKind::Cv)],
                    outputs: vec![port("Out", SignalKind::Audio)],
                },
                GraphNode {
                    id: 2,
                    name: "VCA".into(),
                    pos: (200.0, 0.0),
                    inputs: vec![port("In", SignalKind::Audio), port("Gain", SignalKind::Cv)],
                    outputs: vec![port("Out", SignalKind::Audio)],
                },
            ],
            cables: Vec::new(),
        }
    }

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

    #[test]
    fn connect_takes_the_output_kind_and_replaces_the_input_cable() {
        let mut graph = two_nodes();
        assert!(graph.connect(1, 0, 2, 0));
        assert_eq!(graph.cables.len(), 1);
        assert_eq!(graph.cables[0].kind, SignalKind::Audio);
        // A second connection to the same input replaces, never stacks
        assert!(graph.connect(2, 0, 2, 0));
        assert_eq!(graph.cables.len(), 1);
        assert_eq!(graph.cables[0].from_node, 2);
    }

    #[test]
    fn outputs_fan_out_and_mismatched_kinds_still_connect() {
        let mut graph = two_nodes();
        // Audio out into both the audio In and the CV Gain: any-to-any holds
        assert!(graph.connect(1, 0, 2, 0));
        assert!(graph.connect(1, 0, 2, 1));
        assert_eq!(graph.cables.len(), 2);
        // The cable carries the output's kind even into a CV input
        assert_eq!(graph.cables[1].kind, SignalKind::Audio);
    }

    #[test]
    fn connect_rejects_missing_ends_without_side_effects() {
        let mut graph = two_nodes();
        assert!(!graph.connect(9, 0, 2, 0), "unknown source node");
        assert!(!graph.connect(1, 5, 2, 0), "output index out of range");
        assert!(!graph.connect(1, 0, 2, 9), "input index out of range");
        assert!(graph.cables.is_empty());
    }

    #[test]
    fn disconnect_input_returns_the_removed_cable() {
        let mut graph = two_nodes();
        graph.connect(1, 0, 2, 0);
        let old = graph.disconnect_input(2, 0).expect("cable was patched");
        assert_eq!((old.from_node, old.to_node), (1, 2));
        assert!(graph.cables.is_empty());
        assert!(graph.disconnect_input(2, 0).is_none());
    }

    #[test]
    fn start_patch_from_patched_input_picks_up_the_cable() {
        let mut graph = two_nodes();
        graph.connect(1, 0, 2, 0);
        let mut intents = Vec::new();
        let hit = PortHit {
            node: 2,
            port: 0,
            is_output: false,
            kind: SignalKind::Audio,
            pos: pos2(0.0, 0.0),
        };
        let pending = start_patch(&mut graph, &hit, &mut intents);
        // The drag continues from the far output with the cable removed
        assert!(pending.is_output);
        assert_eq!((pending.node, pending.port), (1, 0));
        assert!(graph.cables.is_empty());
        assert_eq!(intents.len(), 1);
        assert_eq!(intents[0].command, "graph_disconnect:1:0:2:0");
    }

    #[test]
    fn start_patch_from_free_port_anchors_there() {
        let mut graph = two_nodes();
        let mut intents = Vec::new();
        let hit = PortHit {
            node: 1,
            port: 0,
            is_output: true,
            kind: SignalKind::Audio,
            pos: pos2(0.0, 0.0),
        };
        let pending = start_patch(&mut graph, &hit, &mut intents);
        assert!(pending.is_output);
        assert_eq!((pending.node, pending.port), (1, 0));
        assert!(intents.is_empty());
    }

    #[test]
    fn poly_cables_render_thicker_than_mono() {
        assert!(cable_stroke_width(16) > cable_stroke_width(1));
        assert_eq!(cable_stroke_width(1), CABLE_W_MONO);
        assert_eq!(cable_stroke_width(2), CABLE_W_POLY);
    }

    #[test]
    fn add_node_assigns_a_fresh_id_and_generic_ports() {
        let mut graph = two_nodes();
        let id = graph.add_node("Reverb", SignalKind::Audio, (50.0, 60.0));
        assert_eq!(id, 3, "next id is max + 1");
        let node = graph.node(id).unwrap();
        assert_eq!(node.name, "Reverb");
        assert_eq!(node.pos, (50.0, 60.0));
        assert_eq!(node.inputs.len(), 1);
        assert_eq!(node.outputs.len(), 1);
        // First id in an empty graph is 1
        let mut empty = GraphModel::default();
        assert_eq!(empty.add_node("VCO", SignalKind::Audio, (0.0, 0.0)), 1);
    }

    #[test]
    fn remove_node_drops_the_node_and_all_its_cables() {
        let mut graph = two_nodes();
        graph.connect(1, 0, 2, 0);
        assert!(graph.remove_node(2));
        assert!(graph.node(2).is_none());
        assert!(graph.cables.is_empty(), "cables touching the node are gone");
        // Removing an unknown id is a no-op
        assert!(!graph.remove_node(99));
        assert_eq!(graph.nodes.len(), 1);
    }

    #[test]
    fn remove_node_keeps_unrelated_cables() {
        let mut graph = two_nodes();
        let third = graph.add_node("Reverb", SignalKind::Audio, (400.0, 0.0));
        graph.connect(1, 0, 2, 0);
        graph.connect(2, 0, third, 0);
        graph.remove_node(third);
        // The 1->2 cable survives; only the 2->3 cable is removed
        assert_eq!(graph.cables.len(), 1);
        assert_eq!((graph.cables[0].from_node, graph.cables[0].to_node), (1, 2));
    }

    #[test]
    fn node_name_from_intent_maps_browser_payloads() {
        let name = |c: &str| node_name_from_intent(&CommandIntent::new(c));
        assert_eq!(name("show_device").as_deref(), Some("Geist Synth"));
        assert_eq!(name("add_effect:Distortion").as_deref(), Some("Distortion"));
        assert_eq!(name("select_device:Filter").as_deref(), Some("Filter"));
        assert_eq!(name("insert:Delay").as_deref(), Some("Delay"));
        // Non-device commands do not create nodes
        assert_eq!(name("session_stop_all"), None);
    }
}
