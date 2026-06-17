// Author: Jeff
// Date: 2026-06-15
// Description: Studio shell: transport, lens tabs, monitoring strip, central view.
// Notes: Installs nothing itself (theme is applied once at startup); composes the
//        themed chrome around the active lens. Reads UIState for the lens and
//        SessionModel for content, returns the intents the views emitted this
//        frame for the app to apply to project/audio truth.

use egui::{pos2, vec2, RichText, Sense, Stroke};
use geist_config::commands::CommandIntent;
use geist_config::schema::LensId;

use crate::model::{ScopeFrame, SessionModel, SpectrumFrame};
use crate::state::UIState;
use crate::theme;
use crate::views;
use crate::widgets::Meter;

// What one studio frame produced for the app layer to act on
#[derive(Clone, Debug, Default)]
pub struct StudioResponse {
    pub intents: Vec<CommandIntent>,
    pub lens_changed: bool,
    // The user clicked Save or Load this frame; the app owns the file I/O
    pub save_requested: bool,
    pub load_requested: bool,
}

// Draw the full studio shell into the given root Ui (eframe hands one per frame)
pub fn draw_studio(
    ui: &mut egui::Ui,
    state: &mut UIState,
    session: &mut SessionModel,
) -> StudioResponse {
    let mut out = StudioResponse::default();

    egui::Panel::top("geist_transport")
        .show_inside(ui, |ui| transport_bar(ui, state, session, &mut out));
    egui::Panel::bottom("geist_monitor")
        .resizable(false)
        .show_inside(ui, |ui| monitor_strip(ui, session));
    egui::CentralPanel::default().show_inside(ui, |ui| central(ui, state, session, &mut out.intents));

    out
}

// Top bar: identity, transport controls, and the lens switcher
fn transport_bar(
    ui: &mut egui::Ui,
    state: &mut UIState,
    session: &mut SessionModel,
    out: &mut StudioResponse,
) {
    ui.horizontal(|ui| {
        ui.heading(RichText::new("GEIST").color(theme::ACCENT));
        ui.separator();

        let play_label = if session.transport.playing { "⏸" } else { "▶" };
        if ui.button(play_label).clicked() {
            session.transport.playing = !session.transport.playing;
            out.intents.push(CommandIntent::new("toggle_play"));
        }
        if ui
            .add(
                egui::DragValue::new(&mut session.transport.bpm)
                    .range(40.0..=300.0)
                    .speed(0.5)
                    .suffix(" BPM"),
            )
            .changed()
        {
            out.intents.push(CommandIntent::new("set_bpm"));
        }
        ui.toggle_value(&mut session.transport.loop_enabled, "Loop");
        ui.label(
            RichText::new(format!("{:.1}", session.transport.position_beats))
                .small()
                .color(theme::TEXT_MUTED),
        );

        ui.separator();

        // Session persistence; the app performs the actual file I/O
        if ui.button("Save").clicked() {
            out.save_requested = true;
        }
        if ui.button("Load").clicked() {
            out.load_requested = true;
        }

        ui.separator();

        // Lens switcher; collect first so switch_lens can borrow state mutably
        let visible = state.visible_lenses().to_vec();
        let active = state.active_lens();
        for lens in visible {
            if ui.selectable_label(lens == active, lens_label(lens)).clicked() && lens != active {
                let _ = state.switch_lens(lens);
                out.lens_changed = true;
            }
        }
    });
}

// Bottom strip: oscilloscope, spectrum, and the master meter
fn monitor_strip(ui: &mut egui::Ui, session: &SessionModel) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new("Scope").small().color(theme::TEXT_MUTED));
            scope(ui, &session.scope);
        });
        ui.separator();
        ui.vertical(|ui| {
            ui.label(RichText::new("Spectrum").small().color(theme::TEXT_MUTED));
            spectrum(ui, &session.spectrum);
        });
        ui.separator();
        ui.vertical(|ui| {
            ui.label(RichText::new("Master").small().color(theme::TEXT_MUTED));
            let (peak, rms) = session
                .mixer
                .channels
                .last()
                .map(|c| (c.peak, c.rms))
                .unwrap_or((0.0, 0.0));
            ui.horizontal(|ui| {
                Meter::new(peak).peak(rms.max(peak)).size(vec2(12.0, 84.0)).show(ui);
                Meter::new(peak * 0.92)
                    .peak(rms.max(peak))
                    .size(vec2(12.0, 84.0))
                    .show(ui);
            });
        });
    });
}

// Oscilloscope: the latest output window as a centered waveform
fn scope(ui: &mut egui::Ui, frame: &ScopeFrame) {
    let (rect, _) = ui.allocate_exact_size(vec2(280.0, 84.0), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, theme::RADIUS_CONTROL, theme::INSET);
    painter.hline(rect.x_range(), rect.center().y, Stroke::new(1.0, theme::STROKE));

    if frame.samples.len() >= 2 {
        let last = (frame.samples.len() - 1) as f32;
        let amp = rect.height() * 0.45;
        let points: Vec<_> = frame
            .samples
            .iter()
            .enumerate()
            .map(|(i, &s)| {
                let x = rect.left() + rect.width() * (i as f32 / last);
                pos2(x, rect.center().y - s.clamp(-1.0, 1.0) * amp)
            })
            .collect();
        painter.add(egui::Shape::line(points, Stroke::new(1.0, theme::AUDIO)));
    }
}

// Spectrum analyzer: precomputed magnitude bins as vertical bars
fn spectrum(ui: &mut egui::Ui, frame: &SpectrumFrame) {
    let (rect, _) = ui.allocate_exact_size(vec2(280.0, 84.0), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, theme::RADIUS_CONTROL, theme::INSET);

    let n = frame.bins.len();
    if n == 0 {
        return;
    }
    let bar_w = rect.width() / n as f32;
    for (i, &mag) in frame.bins.iter().enumerate() {
        let m = mag.clamp(0.0, 1.0);
        let x = rect.left() + i as f32 * bar_w;
        let bar = egui::Rect::from_min_max(
            pos2(x + 0.5, rect.bottom() - m * rect.height()),
            pos2(x + bar_w - 0.5, rect.bottom()),
        );
        painter.rect_filled(bar, 0.0, theme::meter_color(m));
    }
}

// Dispatch the central area to the active lens's view
fn central(
    ui: &mut egui::Ui,
    state: &UIState,
    session: &mut SessionModel,
    intents: &mut Vec<CommandIntent>,
) {
    match state.active_lens() {
        LensId::Mix => views::mixer::draw(ui, &mut session.mixer, intents),
        LensId::Shape => views::plugin_rack::draw(ui, &mut session.rack, intents),
        LensId::Build => views::node_graph::draw(ui, &mut session.graph, intents),
        LensId::Browser => views::browser::draw(ui, &mut session.browser, intents),
        LensId::Modulation => views::modulation::draw(ui, &session.graph),
        LensId::Arrange => {
            // One musical editor at a time: piano roll, step sequencer, or timeline
            use crate::model::ArrangeTab;
            ui.horizontal(|ui| {
                ui.selectable_value(&mut session.arrange_tab, ArrangeTab::PianoRoll, "Piano Roll");
                ui.selectable_value(&mut session.arrange_tab, ArrangeTab::StepSequencer, "Step Sequencer");
                ui.selectable_value(&mut session.arrange_tab, ArrangeTab::Timeline, "Timeline");
            });
            ui.separator();
            let playhead = Some(session.transport.position_beats as f32);
            match session.arrange_tab {
                ArrangeTab::PianoRoll => {
                    views::piano_roll::draw(ui, &mut session.piano, playhead, intents)
                }
                ArrangeTab::StepSequencer => {
                    views::step_sequencer::draw(ui, &mut session.step_seq, playhead, intents)
                }
                ArrangeTab::Timeline => {
                    views::arrangement::draw(ui, &session.timeline, &session.transport, intents)
                }
            }
        }
    }
}

// Display name for a lens tab
fn lens_label(lens: LensId) -> &'static str {
    match lens {
        LensId::Arrange => "Arrange",
        LensId::Build => "Build",
        LensId::Shape => "Shape",
        LensId::Mix => "Mix",
        LensId::Browser => "Browser",
        LensId::Modulation => "Modulation",
    }
}
