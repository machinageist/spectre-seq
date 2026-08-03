// =============================================================================
// File: app/spectre-seq/src/graph_view.rs
// Layer: application binary
// Purpose: Node-graph view of the live signal chain (synth -> fx -> out)
// Status: Implemented; draggable nodes, bezier cables, click-to-bypass effects.
// Notes: Read-reflects the real device chain. Effect nodes toggle the engine's
//        bypass over the control plane; the cable always routes through because
//        bypassed effects pass audio. Bezier cables are sampled to a polyline so
//        the renderer does not depend on egui's bezier shape API.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use eframe::egui;

use crate::control::{EngineCommand, EngineControl};
use crate::engine::NUM_TRACKS;

// Node box dimensions
const NODE_W: f32 = 116.0;
const NODE_H: f32 = 58.0;
// Canvas height reserved for the rack
const CANVAS_H: f32 = 240.0;
// Bezier sampling resolution for one cable
const CABLE_STEPS: usize = 24;

// Index of each node in the fixed chain
const SYNTH: usize = 0;
const DELAY: usize = 1;
const REVERB: usize = 2;
const OUT: usize = 3;
const NODE_COUNT: usize = 4;

// Draggable node-graph of the current signal chain
pub struct GraphView {
    positions: [egui::Pos2; NODE_COUNT],
    placed: bool,
}

impl GraphView {
    // Build an unplaced view; nodes are laid out on the first frame
    pub fn new() -> Self {
        Self {
            positions: [egui::Pos2::ZERO; NODE_COUNT],
            placed: false,
        }
    }

    // Draw and interact with the rack, toggling effect bypass over `control`
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        control: &mut EngineControl,
        delay_on: &mut bool,
        reverb_on: &mut bool,
    ) {
        let (canvas, _response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), CANVAS_H),
            egui::Sense::hover(),
        );
        let painter = ui.painter_at(canvas);
        painter.rect_filled(canvas, 4.0, egui::Color32::from_gray(16));

        if !self.placed {
            self.place(canvas);
            self.placed = true;
        }

        let labels = ["Synth", "Delay", "Reverb", "Out"];
        let active = [true, *delay_on, *reverb_on, true];

        // Cables route through the chain in order, behind the nodes
        for i in 0..NODE_COUNT - 1 {
            let from = right_port(self.positions[i]);
            let to = left_port(self.positions[i + 1]);
            draw_cable(&painter, from, to, egui::Color32::from_rgb(90, 170, 130));
        }

        // Nodes: draw, drag, and toggle bypass on click
        for i in 0..NODE_COUNT {
            let node_rect =
                egui::Rect::from_min_size(self.positions[i], egui::vec2(NODE_W, NODE_H));
            let id = ui.id().with(("graph_node", i));
            let response = ui.interact(node_rect, id, egui::Sense::click_and_drag());
            if response.dragged() {
                self.positions[i] += response.drag_delta();
            }
            if response.clicked() {
                match i {
                    DELAY => {
                        *delay_on = !*delay_on;
                        // The classic chain is global; toggle every track's delay
                        for track in 0..NUM_TRACKS as u8 {
                            control.send(EngineCommand::SetDelay {
                                track,
                                on: *delay_on,
                            });
                        }
                    }
                    REVERB => {
                        *reverb_on = !*reverb_on;
                        for track in 0..NUM_TRACKS as u8 {
                            control.send(EngineCommand::SetReverb {
                                track,
                                on: *reverb_on,
                            });
                        }
                    }
                    _ => {}
                }
            }

            let node_rect =
                egui::Rect::from_min_size(self.positions[i], egui::vec2(NODE_W, NODE_H));
            draw_node(&painter, node_rect, labels[i], active[i], i);
        }

        // Hint for the bypassable nodes
        painter.text(
            egui::pos2(canvas.left() + 8.0, canvas.bottom() - 18.0),
            egui::Align2::LEFT_CENTER,
            "drag to move · click Delay/Reverb to bypass",
            egui::FontId::proportional(12.0),
            egui::Color32::from_gray(120),
        );
    }

    // Lay the four nodes out left to right, centered vertically
    fn place(&mut self, canvas: egui::Rect) {
        let y = canvas.center().y - NODE_H * 0.5;
        let slot = canvas.width() / NODE_COUNT as f32;
        for i in 0..NODE_COUNT {
            let x = canvas.left() + slot * i as f32 + (slot - NODE_W) * 0.5;
            self.positions[i] = egui::pos2(x, y);
        }
    }
}

// Right-edge output port of a node
fn right_port(pos: egui::Pos2) -> egui::Pos2 {
    egui::pos2(pos.x + NODE_W, pos.y + NODE_H * 0.5)
}

// Left-edge input port of a node
fn left_port(pos: egui::Pos2) -> egui::Pos2 {
    egui::pos2(pos.x, pos.y + NODE_H * 0.5)
}

// Draw one node box, its ports, and its label
fn draw_node(painter: &egui::Painter, rect: egui::Rect, label: &str, active: bool, index: usize) {
    let fill = if active {
        egui::Color32::from_rgb(48, 64, 84)
    } else {
        egui::Color32::from_gray(34)
    };
    let edge = if active {
        egui::Color32::from_rgb(120, 200, 160)
    } else {
        egui::Color32::from_gray(70)
    };
    painter.rect_filled(rect, 6.0, fill);
    painter.rect_stroke(
        rect,
        6.0,
        egui::Stroke::new(1.5, edge),
        egui::StrokeKind::Inside,
    );
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(15.0),
        egui::Color32::from_gray(230),
    );

    // Ports: input on the left for all but the source, output on the right but the sink
    let port_color = egui::Color32::from_rgb(120, 200, 160);
    if index != SYNTH {
        painter.circle_filled(left_port(rect.min), 4.0, port_color);
    }
    if index != OUT {
        painter.circle_filled(right_port(rect.min), 4.0, port_color);
    }
}

// Draw a cable as a sampled cubic bezier polyline
fn draw_cable(painter: &egui::Painter, from: egui::Pos2, to: egui::Pos2, color: egui::Color32) {
    let dx = ((to.x - from.x).abs() * 0.5).max(30.0);
    let c1 = egui::pos2(from.x + dx, from.y);
    let c2 = egui::pos2(to.x - dx, to.y);
    let points: Vec<egui::Pos2> = (0..=CABLE_STEPS)
        .map(|i| cubic(from, c1, c2, to, i as f32 / CABLE_STEPS as f32))
        .collect();
    painter.add(egui::Shape::line(points, egui::Stroke::new(2.0, color)));
}

// Cubic bezier point at parameter t
fn cubic(p0: egui::Pos2, p1: egui::Pos2, p2: egui::Pos2, p3: egui::Pos2, t: f32) -> egui::Pos2 {
    let u = 1.0 - t;
    let w0 = u * u * u;
    let w1 = 3.0 * u * u * t;
    let w2 = 3.0 * u * t * t;
    let w3 = t * t * t;
    egui::pos2(
        w0 * p0.x + w1 * p1.x + w2 * p2.x + w3 * p3.x,
        w0 * p0.y + w1 * p1.y + w2 * p2.y + w3 * p3.y,
    )
}
