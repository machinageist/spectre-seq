// Author: Jeff
// Date: 2026-06-15
// Description: Browser lens: surface model plus the search-first insert palette.
// Notes: Browser remains search-first and context-aware. draw() filters items by
//        the live query and emits an insert intent on double-click; selection is
//        disposable UI state.

use egui::{vec2, RichText, Sense};
use spectre_config::commands::CommandIntent;

use crate::model::BrowserModel;
use crate::renderer::ViewPlan;
use crate::theme;
use crate::views::{action_chips, LensSurface};

pub fn surface(plan: &ViewPlan) -> LensSurface {
    LensSurface {
        lens: plan.lens,
        title: plan.title.to_string(),
        purpose: "Find and insert sounds, plugins, nodes, samples, and commands.",
        empty_actions: action_chips(&plan.empty_actions),
    }
}

// Draw the search box and the filtered, double-click-to-insert result list
pub fn draw(ui: &mut egui::Ui, browser: &mut BrowserModel, intents: &mut Vec<CommandIntent>) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("Search").color(theme::TEXT_MUTED));
        ui.add(
            egui::TextEdit::singleline(&mut browser.query)
                .hint_text("instruments · effects · samples · presets")
                .desired_width(240.0),
        );
        if !browser.query.is_empty() && ui.small_button("✕").clicked() {
            browser.query.clear();
        }
    });
    ui.separator();

    let matches = browser.matches();
    ui.label(
        RichText::new(format!(
            "{} result(s) — double-click to insert",
            matches.len()
        ))
        .small()
        .color(theme::TEXT_MUTED),
    );
    ui.add_space(4.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        for idx in matches {
            let selected = browser.selected == Some(idx);
            let name = browser.items[idx].name.clone();
            let category = browser.items[idx].category.clone();
            let color = browser.items[idx].kind.color();

            let resp = ui.horizontal(|ui| {
                // Signal-type color dot
                let (dot, _) = ui.allocate_exact_size(vec2(12.0, 12.0), Sense::hover());
                ui.painter().circle_filled(dot.center(), 4.0, color);
                ui.selectable_label(selected, RichText::new(format!("{name}    {category}")))
            });

            let label = resp.inner;
            if label.clicked() {
                browser.selected = Some(idx);
            }
            if label.double_clicked() {
                intents.push(browser.items[idx].intent.clone());
            }
        }
    });
}
