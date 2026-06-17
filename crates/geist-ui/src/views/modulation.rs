// Author: Jeff
// Date: 2026-06-15
// Description: Modulation lens: surface model plus a routes overview.
// Notes: Modulation overview complements destination-visible modulation, not
//        replaces it. draw() lists the graph's CV/control cables as readable
//        source -> destination routes; per-destination depth lives on the knobs.

use egui::RichText;

use crate::model::GraphModel;
use crate::renderer::ViewPlan;
use crate::theme::{self, SignalKind};
use crate::views::{action_chips, LensSurface};

pub fn surface(plan: &ViewPlan) -> LensSurface {
    LensSurface {
        lens: plan.lens,
        title: plan.title.to_string(),
        purpose: "Show what moves what while keeping destinations readable.",
        empty_actions: action_chips(&plan.empty_actions),
    }
}

// Draw modulation routes: each CV/control cable as source -> destination
pub fn draw(ui: &mut egui::Ui, graph: &GraphModel) {
    let routes: Vec<_> = graph
        .cables
        .iter()
        .filter(|c| matches!(c.kind, SignalKind::Cv | SignalKind::Control))
        .collect();

    if routes.is_empty() {
        ui.label(
            RichText::new("No modulation routes yet — connect a CV output to a parameter.")
                .color(theme::TEXT_MUTED),
        );
        return;
    }

    for cable in routes {
        let from = graph.node(cable.from_node);
        let to = graph.node(cable.to_node);
        let (Some(from), Some(to)) = (from, to) else {
            continue;
        };
        let src = from
            .outputs
            .get(cable.from_port)
            .map(|p| p.name.as_str())
            .unwrap_or("out");
        let dst = to
            .inputs
            .get(cable.to_port)
            .map(|p| p.name.as_str())
            .unwrap_or("in");

        ui.horizontal(|ui| {
            ui.label(RichText::new("●").color(cable.kind.color()));
            ui.label(RichText::new(format!("{}.{src}", from.name)).color(theme::TEXT));
            ui.label(RichText::new("→").color(theme::TEXT_MUTED));
            ui.label(RichText::new(format!("{}.{dst}", to.name)).color(theme::TEXT));
        });
    }
}
