// Author: Jeff
// Date: 2026-06-15
// Description: Mix lens: surface model plus the vertical-strip mixer drawing.
// Notes: Mixer surface is routing and monitoring, not hardware-console cosplay.
//        draw() renders vertical fader strips with adjacent meters, pan, and
//        mute/solo/arm; continuous values mutate the model, discrete toggles emit
//        intents for the app to apply to audio truth.

use egui::{vec2, RichText};
use geist_config::commands::CommandIntent;

use crate::model::MixerModel;
use crate::renderer::ViewPlan;
use crate::theme;
use crate::views::{action_chips, LensSurface};
use crate::widgets::{Fader, Knob, Meter};

pub fn surface(plan: &ViewPlan) -> LensSurface {
    LensSurface {
        lens: plan.lens,
        title: plan.title.to_string(),
        purpose: "Balance levels, monitor health, and inspect routing.",
        empty_actions: action_chips(&plan.empty_actions),
    }
}

// Draw the mixer: one vertical strip per channel, scrolling horizontally
pub fn draw(ui: &mut egui::Ui, mixer: &mut MixerModel, intents: &mut Vec<CommandIntent>) {
    egui::ScrollArea::horizontal().show(ui, |ui| {
        ui.horizontal_top(|ui| {
            for index in 0..mixer.channels.len() {
                strip(ui, mixer, index, intents);
            }
        });
    });
}

// One channel strip: name, pan, fader+meter, transport buttons, insert chips
fn strip(ui: &mut egui::Ui, mixer: &mut MixerModel, index: usize, intents: &mut Vec<CommandIntent>) {
    let selected = mixer.selected == index;
    let name = mixer.channels[index].name.clone();
    let audible = mixer.is_audible(index);

    ui.group(|ui| {
        ui.set_width(74.0);
        ui.vertical_centered(|ui| {
            if ui
                .selectable_label(selected, RichText::new(name).strong())
                .clicked()
            {
                mixer.selected = index;
            }

            Knob::new(&mut mixer.channels[index].pan, -1.0..=1.0)
                .label("Pan")
                .default(0.0)
                .diameter(32.0)
                .show(ui);

            ui.horizontal(|ui| {
                Fader::new(&mut mixer.channels[index].level, 0.0..=1.5)
                    .default(0.8)
                    .size(vec2(30.0, 150.0))
                    .show(ui);
                // Dim the meter when the channel is muted or solo-gated out
                let peak = if audible { mixer.channels[index].peak } else { 0.0 };
                Meter::new(peak)
                    .peak(mixer.channels[index].rms.max(peak))
                    .size(vec2(10.0, 150.0))
                    .show(ui);
            });

            ui.horizontal(|ui| {
                if ui.toggle_value(&mut mixer.channels[index].muted, "M").changed() {
                    intents.push(CommandIntent::new("set_track_mute"));
                }
                if ui.toggle_value(&mut mixer.channels[index].soloed, "S").changed() {
                    intents.push(CommandIntent::new("set_track_solo"));
                }
                if ui.toggle_value(&mut mixer.channels[index].armed, "R").changed() {
                    intents.push(CommandIntent::new("set_track_arm"));
                }
            });

            // Inserts as compact chips
            for insert in mixer.channels[index].inserts.clone() {
                ui.label(
                    RichText::new(insert)
                        .small()
                        .color(theme::TEXT_MUTED)
                        .background_color(theme::PANEL_RAISED),
                );
            }
        });
    });
}
