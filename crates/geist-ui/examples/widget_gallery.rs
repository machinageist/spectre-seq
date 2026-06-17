// Author: Jeff
// Date: 2026-06-15
// Description: Live gallery of the tactile-dark theme and core widgets.
// Notes: Not a test target; run with `cargo run -p geist-ui --example widget_gallery`
//        to eyeball the theme, knobs, faders, and meters. Holds throwaway demo
//        state only; no engine, no project truth.

use eframe::egui;
use geist_ui::theme::{self, GeistTheme, SignalKind};
use geist_ui::widgets::{Fader, Knob, Meter};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([720.0, 460.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Geist UI — Widget Gallery",
        options,
        Box::new(|cc| {
            GeistTheme::apply(&cc.egui_ctx);
            Ok(Box::new(Gallery::default()))
        }),
    )
}

// Throwaway demo values for the gallery controls
struct Gallery {
    cutoff: f32,
    resonance: f32,
    drive: f32,
    level_a: f32,
    level_b: f32,
    phase: f32,
}

impl Default for Gallery {
    fn default() -> Self {
        Self {
            cutoff: 1_500.0,
            resonance: 0.9,
            drive: 0.3,
            level_a: 0.8,
            level_b: 0.6,
            phase: 0.0,
        }
    }
}

impl eframe::App for Gallery {
    // eframe 0.34 hands a root Ui; wrap content via show_inside
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // Animate the meters so the gradient and peak behavior are visible
        self.phase += 0.03;
        let meter_a = (0.5 + 0.5 * self.phase.sin()) * self.level_a + 0.05;
        let meter_b = (0.5 + 0.5 * (self.phase * 0.7).cos()) * self.level_b + 0.05;

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.heading("Geist UI");
            ui.label(
                egui::RichText::new("Tactile dark — VCV × Phase Plant")
                    .color(theme::TEXT_MUTED),
            );
            ui.separator();

            ui.horizontal(|ui| {
                // Knobs with signal-colored value arcs
                Knob::new(&mut self.cutoff, 20.0..=18_000.0)
                    .label("Cutoff")
                    .unit("Hz")
                    .default(1_500.0)
                    .arc_color(SignalKind::Audio.color())
                    .show(ui);
                Knob::new(&mut self.resonance, 0.5..=6.0)
                    .label("Reso")
                    .default(0.9)
                    .show(ui);
                Knob::new(&mut self.drive, 0.0..=1.0)
                    .label("Drive")
                    .default(0.3)
                    .modulation(0.25)
                    .arc_color(SignalKind::Cv.color())
                    .show(ui);

                ui.add_space(16.0);

                // Faders with adjacent meters, mixer-style
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        Fader::new(&mut self.level_a, 0.0..=1.5).label("A").show(ui);
                        Meter::new(meter_a).peak(meter_a.max(0.95)).show(ui);
                    });
                });
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        Fader::new(&mut self.level_b, 0.0..=1.5).label("B").show(ui);
                        Meter::new(meter_b).peak(meter_b).show(ui);
                    });
                });
            });
        });

        ctx.request_repaint();
    }
}
