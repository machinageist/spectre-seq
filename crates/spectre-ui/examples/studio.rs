// Author: Jeff
// Date: 2026-06-15
// Description: Runnable demo of the full studio shell over a demo session.
// Notes: Not a test target; run with `cargo run -p spectre-ui --example studio`.
//        Animates meters/scope/spectrum and advances transport so every lens —
//        mixer, build graph, effects chain, arrange + piano roll, browser,
//        modulation — can be exercised. No engine; demo state only.

use eframe::egui;
use spectre_ui::model::SessionModel;
use spectre_ui::shell::draw_studio;
use spectre_ui::state::UIState;
use spectre_ui::theme::SpectreTheme;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1180.0, 760.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Spectre Seq — Studio",
        options,
        Box::new(|cc| {
            SpectreTheme::apply(&cc.egui_ctx);
            Ok(Box::new(Studio::new()))
        }),
    )
}

struct Studio {
    state: UIState,
    session: SessionModel,
    phase: f32,
}

impl Studio {
    fn new() -> Self {
        // Every lens visible via the studio default; demo session for content
        Self {
            state: UIState::studio(),
            session: SessionModel::demo(),
            phase: 0.0,
        }
    }

    // Synthesize lively monitor data and advance transport
    fn animate(&mut self) {
        self.phase += 0.03;
        if self.session.transport.playing {
            let beats_per_frame = self.session.transport.bpm / 60.0 / 60.0;
            self.session.transport.position_beats += beats_per_frame as f64;
            let len = self.session.timeline.length_beats as f64;
            if len > 0.0 && self.session.transport.position_beats > len {
                self.session.transport.position_beats = 0.0;
            }
        }

        // Oscilloscope: a drifting blend of two sines
        self.session.scope.samples = (0..256)
            .map(|i| {
                let t = i as f32 / 256.0;
                0.6 * (t * std::f32::consts::TAU * 3.0 + self.phase).sin()
                    + 0.3 * (t * std::f32::consts::TAU * 7.0).sin()
            })
            .collect();

        // Spectrum: decaying bars modulated by phase
        self.session.spectrum.bins = (0..48)
            .map(|k| {
                let falloff = 1.0 - k as f32 / 48.0;
                (0.2 + 0.8 * (self.phase * 0.6 + k as f32 * 0.35).sin().abs()) * falloff
            })
            .collect();

        // Mixer meters
        for (i, ch) in self.session.mixer.channels.iter_mut().enumerate() {
            let v = (0.5 + 0.45 * (self.phase + i as f32 * 0.8).sin()) * ch.level;
            ch.peak = v.clamp(0.0, 1.2);
            ch.rms = (v * 0.8).clamp(0.0, 1.0);
        }
    }
}

impl eframe::App for Studio {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.animate();
        let _response = draw_studio(ui, &mut self.state, &mut self.session);
        ctx.request_repaint();
    }
}
