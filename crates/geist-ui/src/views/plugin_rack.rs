// Author: Jeff
// Date: 2026-06-15
// Description: Shape lens: surface model plus the visible, modifiable FX chain.
// Notes: Controls must remain labeled and command-backed. draw() renders the
//        effects chain as reorderable cards with bypass, wet, and inline param
//        knobs. Structural edits (reorder/remove) are deferred past the loop so
//        the slot vector is never mutated mid-iteration.

use egui::RichText;
use geist_config::commands::CommandIntent;

use crate::model::RackModel;
use crate::renderer::ViewPlan;
use crate::theme;
use crate::views::{action_chips, LensSurface};
use crate::widgets::Knob;

pub fn surface(plan: &ViewPlan) -> LensSurface {
    LensSurface {
        lens: plan.lens,
        title: plan.title.to_string(),
        purpose: "Shape the selected instrument, rack, and pinned parameters.",
        empty_actions: action_chips(&plan.empty_actions),
    }
}

// A structural edit deferred until after the per-slot draw loop
enum RackEdit {
    Move(usize, usize),
    Remove(usize),
}

// Draw the effects chain top-to-bottom with add/reorder/bypass/remove
pub fn draw(ui: &mut egui::Ui, rack: &mut RackModel, intents: &mut Vec<CommandIntent>) {
    ui.menu_button("+ Add Effect", |ui| {
        for name in [
            "Distortion",
            "Phaser",
            "Flanger",
            "Chorus",
            "EQ",
            "Saturator",
        ] {
            if ui.button(name).clicked() {
                intents.push(CommandIntent::new(format!("add_effect:{name}")));
                ui.close();
            }
        }
    });
    ui.add_space(6.0);

    let len = rack.slots.len();
    let mut edit: Option<RackEdit> = None;

    egui::ScrollArea::vertical().show(ui, |ui| {
        for index in 0..len {
            ui.group(|ui| {
                ui.set_width(ui.available_width().min(420.0));

                ui.horizontal(|ui| {
                    ui.label(RichText::new("⠿").color(theme::TEXT_MUTED));

                    let selected = rack.selected == Some(index);
                    let name_resp = ui.selectable_label(
                        selected,
                        RichText::new(&rack.slots[index].name).strong(),
                    );
                    if name_resp.clicked() {
                        rack.selected = Some(index);
                    }
                    // Right-click the device header: bypass, reorder, or remove
                    name_resp.context_menu(|ui| {
                        let bypassed = rack.slots[index].bypassed;
                        if ui
                            .button(if bypassed { "Un-bypass" } else { "Bypass" })
                            .clicked()
                        {
                            rack.slots[index].bypassed = !bypassed;
                            intents.push(CommandIntent::new("toggle_bypass"));
                        }
                        if ui
                            .add_enabled(index > 0, egui::Button::new("Move up"))
                            .clicked()
                        {
                            edit = Some(RackEdit::Move(index, index - 1));
                        }
                        if ui
                            .add_enabled(index + 1 < len, egui::Button::new("Move down"))
                            .clicked()
                        {
                            edit = Some(RackEdit::Move(index, index + 1));
                        }
                        if ui.button("Remove").clicked() {
                            edit = Some(RackEdit::Remove(index));
                        }
                    });

                    if rack.slots[index].bypassed {
                        ui.label(RichText::new("bypassed").small().color(theme::TEXT_MUTED));
                    }

                    // Right-aligned slot controls
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("✕").clicked() {
                            edit = Some(RackEdit::Remove(index));
                        }
                        if ui
                            .add_enabled(index + 1 < len, egui::Button::new("▼").small())
                            .clicked()
                        {
                            edit = Some(RackEdit::Move(index, index + 1));
                        }
                        if ui
                            .add_enabled(index > 0, egui::Button::new("▲").small())
                            .clicked()
                        {
                            edit = Some(RackEdit::Move(index, index - 1));
                        }
                        if ui
                            .toggle_value(&mut rack.slots[index].bypassed, "Bypass")
                            .changed()
                        {
                            intents.push(CommandIntent::new("toggle_bypass"));
                        }
                    });
                });

                // Inline parameter knobs plus the wet/dry mix
                ui.horizontal_wrapped(|ui| {
                    for param in 0..rack.slots[index].params.len() {
                        let min = rack.slots[index].params[param].min;
                        let max = rack.slots[index].params[param].max;
                        let default = rack.slots[index].params[param].default;
                        let taper = rack.slots[index].params[param].taper;
                        let label = rack.slots[index].params[param].name.clone();
                        let unit = rack.slots[index].params[param].unit.clone();
                        Knob::new(&mut rack.slots[index].params[param].value, min..=max)
                            .label(&label)
                            .unit(&unit)
                            .default(default)
                            .taper(taper)
                            .diameter(40.0)
                            .show(ui);
                    }
                    Knob::new(&mut rack.slots[index].wet, 0.0..=1.0)
                        .label("Wet")
                        .default(1.0)
                        .diameter(40.0)
                        .arc_color(theme::ACCENT)
                        .show(ui);
                });
            });
            ui.add_space(6.0);
        }
    });

    match edit {
        Some(RackEdit::Move(from, to)) => rack.reorder(from, to),
        Some(RackEdit::Remove(index)) => {
            rack.remove(index);
            intents.push(CommandIntent::new("remove_effect"));
        }
        None => {}
    }
}
