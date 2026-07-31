// Author: Jeff
// Date: 2026-06-15
// Description: Studio shell: transport, lens tabs, monitoring strip, central view.
// Notes: Installs nothing itself (theme is applied once at startup); composes the
//        themed chrome around the active lens. Reads UIState for the lens and
//        SessionModel for content, returns the intents the views emitted this
//        frame for the app to apply to project/audio truth.

use egui::{pos2, vec2, RichText, Sense, Stroke};
use geist_config::commands::CommandIntent;
use geist_config::schema::{LensId, PanelId};

use crate::model::{ScopeFrame, SessionModel, SpectrumFrame};
use crate::state::{UIState, WorkspacePane};
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
    handle_pane_focus_shortcuts(ui, state);

    let transport_pressed = egui::Panel::top("geist_transport")
        .frame(pane_frame(state.focused_pane() == WorkspacePane::Transport))
        .show_inside(ui, |ui| {
            transport_bar(ui, state, session, &mut out);
            pointer_pressed_in(ui)
        })
        .inner;
    if transport_pressed {
        state.focus_pane(WorkspacePane::Transport);
    }

    let monitor_pressed = egui::Panel::bottom("geist_monitor")
        .resizable(false)
        .frame(pane_frame(state.focused_pane() == WorkspacePane::Monitor))
        .show_inside(ui, |ui| {
            monitor_strip(ui, session);
            pointer_pressed_in(ui)
        })
        .inner;
    if monitor_pressed {
        state.focus_pane(WorkspacePane::Monitor);
    }

    let left_panel = state.layout().left_panel;
    if state.left_panel_open() {
        if let Some(panel) = left_panel {
            let pressed = egui::Panel::left("geist_left_tile")
                .resizable(true)
                .default_size(260.0)
                .size_range(180.0..=420.0)
                .frame(pane_frame(state.focused_pane() == WorkspacePane::Left))
                .show_inside(ui, |ui| {
                    side_panel(ui, panel, state, session, &mut out.intents);
                    pointer_pressed_in(ui)
                })
                .inner;
            if pressed {
                state.focus_pane(WorkspacePane::Left);
            }
        }
    }

    let right_panel = state.layout().right_panel;
    if state.right_panel_open() {
        if let Some(panel) = right_panel {
            let pressed = egui::Panel::right("geist_right_tile")
                .resizable(true)
                .default_size(260.0)
                .size_range(180.0..=420.0)
                .frame(pane_frame(state.focused_pane() == WorkspacePane::Right))
                .show_inside(ui, |ui| {
                    side_panel(ui, panel, state, session, &mut out.intents);
                    pointer_pressed_in(ui)
                })
                .inner;
            if pressed {
                state.focus_pane(WorkspacePane::Right);
            }
        }
    }

    let main_pressed = egui::CentralPanel::default()
        .frame(pane_frame(state.focused_pane() == WorkspacePane::Main))
        .show_inside(ui, |ui| {
            central(ui, state, session, &mut out.intents);
            pointer_pressed_in(ui)
        })
        .inner;
    if main_pressed {
        state.focus_pane(WorkspacePane::Main);
    }

    out
}

// Hyprland-like vertical focus navigation; Alt avoids compositor Super bindings
fn handle_pane_focus_shortcuts(ui: &egui::Ui, state: &mut UIState) {
    if ui.ctx().egui_wants_keyboard_input() {
        return;
    }
    let down = ui.input_mut(|input| input.consume_key(egui::Modifiers::ALT, egui::Key::J));
    let up = ui.input_mut(|input| input.consume_key(egui::Modifiers::ALT, egui::Key::K));
    let left = ui.input_mut(|input| input.consume_key(egui::Modifiers::ALT, egui::Key::H));
    let right = ui.input_mut(|input| input.consume_key(egui::Modifiers::ALT, egui::Key::L));
    if down {
        state.cycle_pane_focus(true);
    } else if up {
        state.cycle_pane_focus(false);
    } else if left {
        state.focus_horizontally(false);
    } else if right {
        state.focus_horizontally(true);
    }
}

// Focused tiles use the same strong border cue as the active Hyprland window
fn pane_frame(focused: bool) -> egui::Frame {
    egui::Frame::new()
        .fill(theme::PANEL)
        .stroke(Stroke::new(
            if focused { 2.0_f32 } else { 1.0_f32 },
            if focused {
                theme::ACCENT
            } else {
                theme::STROKE
            },
        ))
        .corner_radius(theme::RADIUS_PANEL)
        .inner_margin(egui::Margin::same(8))
}

fn pointer_pressed_in(ui: &egui::Ui) -> bool {
    ui.rect_contains_pointer(ui.max_rect()) && ui.input(|input| input.pointer.any_pressed())
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

        let play_label = if session.transport.playing {
            "⏸"
        } else {
            "▶"
        };
        if ui.button(play_label).clicked() {
            session.transport.playing = !session.transport.playing;
            out.intents.push(CommandIntent::new("toggle_play"));
        }
        // Record arm: lit red while recording; the app captures armed-track notes
        let rec = session.transport.recording;
        let rec_label = RichText::new("⏺").color(if rec {
            theme::ACCENT
        } else {
            theme::TEXT_MUTED
        });
        if ui
            .add(egui::Button::new(rec_label).fill(if rec {
                theme::ACCENT.linear_multiply(0.18)
            } else {
                theme::PANEL_RAISED
            }))
            .clicked()
        {
            session.transport.recording = !session.transport.recording;
            out.intents.push(CommandIntent::new("toggle_record"));
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
        if state.layout().left_panel.is_some()
            && ui
                .small_button(if state.left_panel_open() {
                    "◀"
                } else {
                    "▶"
                })
                .on_hover_text("Toggle left tile")
                .clicked()
        {
            state.toggle_left_panel();
        }
        if state.layout().right_panel.is_some()
            && ui
                .small_button(if state.right_panel_open() {
                    "▶"
                } else {
                    "◀"
                })
                .on_hover_text("Toggle right tile")
                .clicked()
        {
            state.toggle_right_panel();
        }

        ui.separator();

        // Lens switcher; collect first so switch_lens can borrow state mutably
        let visible = state.visible_lenses().to_vec();
        let active = state.active_lens();
        for lens in visible {
            if ui
                .selectable_label(lens == active, lens_label(lens))
                .clicked()
                && lens != active
            {
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
                Meter::new(peak)
                    .peak(rms.max(peak))
                    .size(vec2(12.0, 84.0))
                    .show(ui);
                Meter::new(peak * 0.92)
                    .peak(rms.max(peak))
                    .size(vec2(12.0, 84.0))
                    .show(ui);
            });
        });
    });
}

fn side_panel(
    ui: &mut egui::Ui,
    panel: PanelId,
    state: &UIState,
    session: &mut SessionModel,
    intents: &mut Vec<CommandIntent>,
) {
    ui.heading(panel_label(panel));
    ui.separator();
    match panel {
        PanelId::Browser => views::browser::draw(ui, &mut session.browser, intents),
        PanelId::ContextShelf => context_shelf(ui, state, intents),
        PanelId::ModulationOverview => views::modulation::draw(ui, &session.graph),
        PanelId::Meters => monitor_strip(ui, session),
        PanelId::MacroStrip => views::plugin_rack::draw(ui, &mut session.rack, intents),
    }
}

fn context_shelf(ui: &mut egui::Ui, state: &UIState, intents: &mut Vec<CommandIntent>) {
    let Some(selection) = state.selected_object() else {
        ui.label(RichText::new("Select an object to inspect it").color(theme::TEXT_MUTED));
        return;
    };
    let (label, key) = match selection {
        crate::state::SelectedObject::Track(id) => (format!("Track · {id}"), "track"),
        crate::state::SelectedObject::Clip(id) => (format!("Clip · {id}"), "clip"),
        crate::state::SelectedObject::Node(id) => (format!("Node · {id}"), "node"),
        crate::state::SelectedObject::Cable(id) => (format!("Cable · {id}"), "cable"),
        crate::state::SelectedObject::Parameter(id) => (format!("Parameter · {id}"), "parameter"),
        crate::state::SelectedObject::ModulationRoute(id) => {
            (format!("Modulation · {id}"), "modulation_route")
        }
    };
    ui.label(label);
    ui.add_space(6.0);
    let actions = state
        .workflow()
        .context_shelf
        .get(key)
        .map(|shelf| shelf.actions.as_slice())
        .unwrap_or_default();
    if actions.is_empty() {
        ui.label(RichText::new("No contextual actions").color(theme::TEXT_MUTED));
    }
    for action in actions {
        if ui.button(action.replace('_', " ")).clicked() {
            intents.push(CommandIntent::new(action.clone()));
        }
    }
}

fn panel_label(panel: PanelId) -> &'static str {
    match panel {
        PanelId::Browser => "Browser",
        PanelId::ContextShelf => "Inspector",
        PanelId::ModulationOverview => "Modulation",
        PanelId::Meters => "Meters",
        PanelId::MacroStrip => "Macros",
    }
}

// Oscilloscope: the latest output window as a centered waveform
fn scope(ui: &mut egui::Ui, frame: &ScopeFrame) {
    let (rect, _) = ui.allocate_exact_size(vec2(280.0, 84.0), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, theme::RADIUS_CONTROL, theme::INSET);
    painter.hline(
        rect.x_range(),
        rect.center().y,
        Stroke::new(1.0_f32, theme::STROKE),
    );

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
        painter.add(egui::Shape::line(points, Stroke::new(1.0_f32, theme::AUDIO)));
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
                ui.selectable_value(
                    &mut session.arrange_tab,
                    ArrangeTab::PianoRoll,
                    "Piano Roll",
                );
                ui.selectable_value(
                    &mut session.arrange_tab,
                    ArrangeTab::StepSequencer,
                    "Step Sequencer",
                );
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
                    views::arrangement::draw(ui, &mut session.timeline, &session.transport, intents)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focused_pane_frame_uses_the_active_window_border() {
        let focused = pane_frame(true);
        let inactive = pane_frame(false);
        assert_eq!(focused.stroke.color, theme::ACCENT);
        assert_eq!(focused.stroke.width, 2.0);
        assert_eq!(inactive.stroke.color, theme::STROKE);
        assert_eq!(inactive.stroke.width, 1.0);
    }
}
