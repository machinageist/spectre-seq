// Author: Jeff
// Date: 2026-06-15
// Description: Studio shell: transport, lens tabs, monitoring strip, central view.
// Notes: Installs nothing itself (theme is applied once at startup); composes the
//        themed chrome around the active lens. Reads UIState for the lens and
//        SessionModel for content, returns the intents the views emitted this
//        frame for the app to apply to project/audio truth.

use egui::{pos2, vec2, RichText, Sense, Stroke};
use geist_config::commands::CommandIntent;

use crate::model::{ArrangeTab, ScopeFrame, SessionModel, SpectrumFrame};
use crate::state::{DetailView, MainView, UIState};
use crate::theme;
use crate::views;
use crate::widgets::Meter;

// A discrete transport action a transport button issued this frame. Stop and
// Pause both leave playback halted, so they can't be inferred from a bool diff.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportAction {
    Play,
    Stop,
    Pause,
    ToggleRecord,
}

// What one studio frame produced for the app layer to act on
#[derive(Clone, Debug, Default)]
pub struct StudioResponse {
    pub intents: Vec<CommandIntent>,
    // A transport button pressed this frame, if any
    pub transport: Option<TransportAction>,
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

    // Top: transport + view toggles. Bottoms (outer->inner): monitor, detail.
    // The app shows the playable keyboard as an even-outer bottom panel.
    egui::Panel::top("geist_transport")
        .show_inside(ui, |ui| transport_bar(ui, state, session, &mut out));
    egui::Panel::bottom("geist_monitor")
        .resizable(false)
        .show_inside(ui, |ui| monitor_strip(ui, session));
    egui::Panel::bottom("geist_detail")
        .resizable(true)
        .default_size(210.0)
        .show_inside(ui, |ui| detail_panel(ui, state, session, &mut out.intents));

    // Left: browser bar (toggleable). Right: mixer (toggleable).
    if state.browser_visible() {
        egui::Panel::left("geist_browser")
            .resizable(true)
            .default_size(190.0)
            .show_inside(ui, |ui| {
                ui.add_space(2.0);
                ui.label(RichText::new("Browser").small().color(theme::TEXT_MUTED));
                views::browser::draw(ui, &mut session.browser, &mut out.intents);
            });
    }
    if state.mixer_visible() {
        egui::Panel::right("geist_mixer")
            .resizable(true)
            .default_size(380.0)
            .show_inside(ui, |ui| views::mixer::draw(ui, &mut session.mixer, &mut out.intents));
    }

    // Center: the primary editor (arrangement / session / graph / modulation)
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
    ui.horizontal_wrapped(|ui| {
        ui.heading(RichText::new("GEIST").color(theme::ACCENT));
        ui.separator();

        // Play / Stop / Pause / Record. Play lights while rolling; Record lights red.
        let playing = session.transport.playing;
        let rec = session.transport.recording;
        if ui.add(transport_button("▶", playing)).clicked() {
            out.transport = Some(TransportAction::Play);
        }
        if ui.add(transport_button("⏹", false)).clicked() {
            out.transport = Some(TransportAction::Stop);
        }
        if ui.add(transport_button("⏸", !playing && session.transport.position_beats > 0.0)).clicked() {
            out.transport = Some(TransportAction::Pause);
        }
        if ui.add(transport_button("⏺", rec)).clicked() {
            out.transport = Some(TransportAction::ToggleRecord);
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

        // Center editor selector (Tab toggles Arrangement/Session via shortcuts)
        let mut main = state.main_view();
        ui.selectable_value(&mut main, MainView::Arrangement, "Arrange");
        ui.selectable_value(&mut main, MainView::Session, "Session");
        ui.selectable_value(&mut main, MainView::Graph, "Graph");
        ui.selectable_value(&mut main, MainView::Modulation, "Mod");
        if main != state.main_view() {
            state.set_main_view(main);
        }

        ui.separator();

        // Panel toggles: browser bar, mixer, and the bottom detail contents
        let mut browser = state.browser_visible();
        if ui.toggle_value(&mut browser, "Browser").changed() {
            state.toggle_browser();
        }
        let mut mixer = state.mixer_visible();
        if ui.toggle_value(&mut mixer, "Mixer").changed() {
            state.toggle_mixer();
        }
        let mut detail = state.detail_view();
        ui.selectable_value(&mut detail, DetailView::Clip, "Clip");
        ui.selectable_value(&mut detail, DetailView::Device, "Device");
        if detail != state.detail_view() {
            state.set_detail_view(detail);
        }

        ui.separator();

        // Session persistence; the app performs the actual file I/O
        if ui.button("Save").clicked() {
            out.save_requested = true;
        }
        if ui.button("Load").clicked() {
            out.load_requested = true;
        }
    });
}

// A transport button, tinted with the accent when `lit`
fn transport_button(glyph: &str, lit: bool) -> egui::Button<'static> {
    let text = RichText::new(glyph).color(if lit { theme::ACCENT } else { theme::TEXT });
    let fill = if lit { theme::ACCENT.linear_multiply(0.18) } else { theme::PANEL_RAISED };
    egui::Button::new(text).fill(fill)
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

// Dispatch the central area to the selected primary editor
fn central(
    ui: &mut egui::Ui,
    state: &UIState,
    session: &mut SessionModel,
    intents: &mut Vec<CommandIntent>,
) {
    match state.main_view() {
        MainView::Arrangement => {
            views::arrangement::draw(ui, &mut session.timeline, &session.transport, intents)
        }
        MainView::Session => views::session::draw(ui, &mut session.session_grid, intents),
        MainView::Graph => views::node_graph::draw(ui, &mut session.graph, intents),
        MainView::Modulation => views::modulation::draw(ui, &session.graph),
    }
}

// Bottom detail: the clip editor (piano roll / step sequencer) or the device chain
fn detail_panel(
    ui: &mut egui::Ui,
    state: &UIState,
    session: &mut SessionModel,
    intents: &mut Vec<CommandIntent>,
) {
    match state.detail_view() {
        DetailView::Device => views::plugin_rack::draw(ui, &mut session.rack, intents),
        DetailView::Clip => {
            // Clip sub-tab: piano roll or step sequencer (timeline lives in center)
            ui.horizontal(|ui| {
                ui.selectable_value(&mut session.arrange_tab, ArrangeTab::PianoRoll, "Piano Roll");
                ui.selectable_value(&mut session.arrange_tab, ArrangeTab::StepSequencer, "Step Seq");
            });
            ui.separator();
            let playhead = Some(session.transport.position_beats as f32);
            match session.arrange_tab {
                ArrangeTab::StepSequencer => {
                    views::step_sequencer::draw(ui, &mut session.step_seq, playhead, intents)
                }
                _ => views::piano_roll::draw(ui, &mut session.piano, playhead, intents),
            }
        }
    }
}
