// Author: Jeff
// Date: 2026-07-12
// Description: Native egui shell for iterative Spectre workflow feedback
// Notes: The live engine is wired here and owned by SpectrePrototype, not by AppModel, so the
//   model stays renderer-neutral and the non-Send stream stays on the thread that created it.
//   Persistence wiring remains out of scope until R4-7.

use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke, Vec2};
use spectre_app::bounce_panel::{duration_label, validate_request, BouncePanel, BounceState};
use spectre_app::engine::{
    apply_parameter_edit, engine_status_field, EngineHealth, EngineState, EngineUnavailable,
    LiveEngine,
};
use spectre_app::{open_device_in_shape_from_ui, AppModel, Lens};

const BG: Color32 = Color32::from_rgb(15, 18, 24);
const PANEL: Color32 = Color32::from_rgb(24, 29, 38);
const RAISED: Color32 = Color32::from_rgb(34, 41, 53);
const ACCENT: Color32 = Color32::from_rgb(96, 230, 184);
const WARM: Color32 = Color32::from_rgb(240, 166, 90);
const TEXT: Color32 = Color32::from_rgb(224, 230, 238);
const MUTED: Color32 = Color32::from_rgb(128, 140, 156);

// The fader's range IS the accepted gain descriptor's range; no fader law is invented here
const GAIN_MAX: f32 = 2.0;
const GAIN_RANGE: std::ops::RangeInclusive<f32> = 0.0..=GAIN_MAX;

// One mixer edit, deferred out of the panel closure that borrows the model
#[derive(Debug, Clone, Copy)]
enum TrackMixEdit {
    Level(spectre_core::ObjectId, f32),
    Muted(spectre_core::ObjectId, bool),
    Soloed(spectre_core::ObjectId, bool),
}

struct SpectrePrototype {
    model: AppModel,
    new_track_name: String,
    feedback_status: String,
    // The live stream, when one is open. Owned here rather than by AppModel so the model gains
    // no audio dependency and the thread-affine stream never leaves the UI thread
    engine: Option<LiveEngine>,
    // Why no engine is running; displayed verbatim rather than summarized as "offline"
    engine_unavailable: EngineUnavailable,
    // The start attempt happens on the first frame, not in Default, so a failure is reportable
    engine_attempted: bool,
    // The track-list revision the running engine was built from. While this differs from the
    // model's, the app states that the edit is not audible rather than pretending it is
    engine_revision: u64,
    // Offline bounce state. Owned by the shell, not by AppModel, for the same reason the engine
    // is: the model gains no render dependency and no thread-affine field
    bounce: BouncePanel,
    bounce_open: bool,
}

impl Default for SpectrePrototype {
    fn default() -> Self {
        Self {
            model: AppModel::prototype(),
            new_track_name: String::new(),
            feedback_status: "Type notes while you explore; copy a state-rich report when ready."
                .into(),
            engine: None,
            engine_unavailable: EngineUnavailable::NotAttempted,
            engine_attempted: false,
            engine_revision: 0,
            bounce: BouncePanel::default(),
            bounce_open: false,
        }
    }
}

// Open the default device against the compiled-in backend
// Node identity seed for the app's track graph. Rationale: identities must be stable across a
// rebuild so the same list produces the same graph; the value itself carries no meaning
const APP_GRAPH_SEED: u64 = 0x0053_5045_4354_5245;

#[cfg(feature = "live-audio")]
fn open_engine(model: &AppModel) -> Result<LiveEngine, EngineUnavailable> {
    let backend = spectre_audio::cpal_backend::CpalBackend::new();
    // The note node is the selected track's instrument; every other track renders silence until
    // R4-5 gives each track its own clip
    spectre_app::engine::open_track_engine(
        &backend,
        model.track_list(),
        APP_GRAPH_SEED,
        model.selected_track_id(),
    )
}

// Without a real backend the app declines to fake one; NullBackend would produce a running
// counter and no sound, which is exactly the fake surface the vision prohibits
#[cfg(not(feature = "live-audio"))]
fn open_engine(_model: &AppModel) -> Result<LiveEngine, EngineUnavailable> {
    Err(EngineUnavailable::NoBackendCompiled)
}

impl SpectrePrototype {
    // Attempt to open the device once, recording the reason on failure
    fn start_engine(&mut self) {
        self.engine_attempted = true;
        match open_engine(&self.model) {
            Ok(engine) => {
                self.engine = Some(engine);
                self.engine_unavailable = EngineUnavailable::NotAttempted;
                self.engine_revision = self.model.track_list().structure_revision();
            }
            Err(error) => {
                self.feedback_status = format!("Audio engine did not start: {error}");
                self.engine = None;
                self.engine_unavailable = error;
            }
        }
    }

    // Send first, mutate second. The UI transport changes only when the render thread will see
    // the same change, which is what keeps the app's transport and the bridge's from diverging
    // Delegate to the tested rule in engine.rs; the shell only supplies the borrows
    fn toggle_transport(&mut self) {
        let _ = spectre_app::engine::toggle_transport(
            &mut self.model,
            self.engine.as_mut(),
            &mut self.feedback_status,
        );
    }

    // Apply one mixer edit and publish every effective gain it invalidated.
    //
    // The publication set is part of the contract, not an implementation detail: effective_gain
    // depends on any_soloed(), so ONE solo edit changes the value for EVERY track. Publishing
    // only the edited id would produce a solo that silences nothing
    fn apply_mix_edit(&mut self, edit: TrackMixEdit) {
        let result = match edit {
            TrackMixEdit::Level(id, level) => self.model.set_track_level(id, level),
            TrackMixEdit::Muted(id, muted) => self.model.set_track_muted(id, muted),
            TrackMixEdit::Soloed(id, soloed) => self.model.set_track_soloed(id, soloed),
        };
        match result {
            Ok(invalidated) => {
                if let Some(engine) = self.engine.as_ref() {
                    if let Err(error) = spectre_app::engine::publish_track_gains(
                        engine,
                        self.model.track_list(),
                        &invalidated,
                    ) {
                        self.feedback_status =
                            format!("The mix changed but did not reach live audio: {error}");
                    }
                }
            }
            Err(error) => self.feedback_status = error.to_string(),
        }
    }

    // True while the model's track structure differs from the plan the engine is executing
    fn engine_is_stale(&self) -> bool {
        self.engine.is_some()
            && self.model.track_list().structure_revision() != self.engine_revision
    }

    // Stop the stream, rebuild the plan from the current list, and start again. R4-4 does this
    // as an explicit user action with an audible gap rather than a hot swap; a glitch-free swap
    // needs a fourth control lane and a decision about notes in flight, and is not claimed here
    fn rebuild_engine(&mut self) {
        if let Some(engine) = self.engine.as_mut() {
            let _ = engine.close();
        }
        self.engine = None;
        self.start_engine();
    }

    // What the transport bar renders, derived from the render thread rather than asserted
    fn engine_state(&self) -> Option<EngineState> {
        self.engine.as_ref().map(LiveEngine::state)
    }

    fn engine_health(&self) -> Option<EngineHealth> {
        self.engine.as_ref().map(LiveEngine::health)
    }

    fn configure_style(ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();
        style.visuals.dark_mode = true;
        style.visuals.panel_fill = BG;
        style.visuals.window_fill = PANEL;
        style.visuals.faint_bg_color = RAISED;
        style.visuals.widgets.inactive.bg_fill = RAISED;
        style.visuals.widgets.inactive.fg_stroke.color = TEXT;
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(45, 55, 69);
        style.visuals.widgets.active.bg_fill = Color32::from_rgb(53, 68, 78);
        style.spacing.item_spacing = Vec2::new(10.0, 8.0);
        style.spacing.button_padding = Vec2::new(12.0, 7.0);
        ctx.set_style(style);
    }

    fn transport(&mut self, ctx: &egui::Context) {
        // Both actions mutate self, so they are deferred out of the panel closure that borrows it
        let mut toggle = false;
        let mut retry = false;
        let mut rebuild = false;
        let mut open_bounce = false;
        let stale = self.engine_is_stale();
        egui::TopBottomPanel::top("transport")
            .exact_height(62.0)
            .frame(
                egui::Frame::new()
                    .fill(PANEL)
                    .inner_margin(egui::Margin::symmetric(16, 10)),
            )
            .show(ctx, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(RichText::new("SPECTRE").size(20.0).strong().color(ACCENT));
                    ui.add_space(14.0);
                    if ui
                        .button(if self.model.is_playing() {
                            "■  Stop"
                        } else {
                            "▶  Play"
                        })
                        .clicked()
                    {
                        toggle = true;
                    }
                    ui.add_enabled(false, egui::Button::new("●  Record"))
                        .on_disabled_hover_text(
                            "Recording arrives at R7; no capture path exists yet.",
                        );
                    if ui
                        .selectable_label(self.bounce_open, "Bounce…")
                        .on_hover_text("Render this signal path offline to a 32-bit float WAV")
                        .clicked()
                    {
                        open_bounce = true;
                    }
                    ui.separator();
                    // Tempo and meter are the model's defaults; position is not derived from the
                    // render thread yet, so it reads as unknown rather than as a frozen 001
                    ui.label(RichText::new("120.00 BPM").monospace().color(TEXT))
                        .on_hover_text("Fixed project default; tempo editing arrives with the arrangement.");
                    ui.label(RichText::new("4 / 4").monospace().color(MUTED))
                        .on_hover_text("Fixed project default; meter editing arrives with the arrangement.");
                    ui.label(RichText::new("—").monospace().color(MUTED))
                        .on_hover_text("Playhead position is not reported by the engine yet.");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        match self.engine_state() {
                            Some(EngineState::Running {
                                backend,
                                ref device,
                                sample_rate,
                                frames,
                            }) => {
                                let health = self.engine_health().unwrap_or_else(|| unreachable!());
                                ui.label(
                                    RichText::new(format!(
                                        "{:.0}% headroom",
                                        health.worst_headroom.max(0.0) * 100.0
                                    ))
                                    .monospace()
                                    .color(MUTED),
                                )
                                .on_hover_text(format!(
                                    "blocks {} · xruns {} · plan errors {} · frame rejections {} · notes deferred {} · contaminated {} · stream errors {} · params applied {} · params pending {}",
                                    health.blocks_rendered,
                                    health.xruns,
                                    health.plan_errors,
                                    health.frame_capacity_rejections,
                                    health.notes_deferred,
                                    health.contaminated_nodes,
                                    health.stream_errors,
                                    health.parameters_applied,
                                    health.parameters_pending,
                                ));
                                ui.label(
                                    RichText::new(format!(
                                        "ENGINE RUNNING · {backend} · {device} · {sample_rate} Hz · {frames}"
                                    ))
                                    .small()
                                    .color(ACCENT),
                                );
                            }
                            Some(EngineState::Opened {
                                backend,
                                ref device,
                                sample_rate,
                                frames,
                            }) => {
                                ui.label(RichText::new("blocks 0").monospace().color(MUTED));
                                ui.label(
                                    RichText::new(format!(
                                        "ENGINE OPENED · {backend} · {device} · {sample_rate} Hz · {frames}"
                                    ))
                                    .small()
                                    .color(WARM),
                                )
                                .on_hover_text("The device is open; the driver has not called back yet.");
                            }
                            None => {
                                if ui
                                    .button("Retry engine")
                                    .on_hover_text("Re-attempt opening the default output device.")
                                    .clicked()
                                {
                                    retry = true;
                                }
                                ui.label(
                                    RichText::new("ENGINE UNAVAILABLE").small().color(WARM),
                                )
                                .on_hover_text(self.engine_unavailable.to_string());
                            }
                        }
                        if stale {
                            // The plan the render thread is executing no longer matches the
                            // model. The app says so rather than pretending the edit is audible
                            if ui
                                .button("Rebuild engine")
                                .on_hover_text(
                                    "Track structure changed. The engine is still playing the \
                                     previous plan; rebuilding restarts the stream.",
                                )
                                .clicked()
                            {
                                rebuild = true;
                            }
                            ui.label(RichText::new("PLAN STALE").small().color(WARM));
                        }
                    });
                });
            });
        if toggle {
            self.toggle_transport();
        }
        if retry {
            self.start_engine();
        }
        if rebuild {
            self.rebuild_engine();
        }
        if open_bounce {
            self.bounce_open = !self.bounce_open;
        }
    }

    fn lenses(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("lenses")
            .exact_height(48.0)
            .frame(
                egui::Frame::new()
                    .fill(BG)
                    .inner_margin(egui::Margin::symmetric(16, 8)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    for lens in Lens::ALL {
                        let selected = self.model.lens() == lens;
                        if ui
                            .selectable_label(selected, RichText::new(lens.to_string()).size(14.0))
                            .clicked()
                        {
                            self.model.select_lens(lens);
                        }
                    }
                    ui.separator();
                    ui.label(
                        RichText::new("Space: play/stop · 1–4: switch lens")
                            .small()
                            .color(MUTED),
                    );
                });
            });
    }

    fn track_list(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("tracks")
            .resizable(true)
            .default_width(220.0)
            .min_width(180.0)
            .frame(
                egui::Frame::new()
                    .fill(PANEL)
                    .inner_margin(egui::Margin::same(12)),
            )
            .show(ctx, |ui| {
                ui.label(RichText::new("TRACKS").small().strong().color(MUTED));
                ui.add_space(4.0);
                let mut select = None;
                for track in self.model.tracks() {
                    let selected = self.model.selected_track_id() == Some(track.id());
                    // Mute and solo are the two states that actually change what is rendered, so
                    // they are what the row shows. There is no arm indicator, because there is
                    // no recording path to arm for
                    let marker = if track.is_muted() {
                        "M"
                    } else if track.is_soloed() {
                        "S"
                    } else {
                        "·"
                    };
                    let label = format!("{marker}  {}", track.name());
                    if ui.selectable_label(selected, label).clicked() {
                        select = Some(track.id());
                    }
                }
                if let Some(id) = select {
                    self.model.select_track(id);
                }
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [125.0, 28.0],
                        egui::TextEdit::singleline(&mut self.new_track_name)
                            .hint_text("Track name"),
                    );
                    if ui.button("+").on_hover_text("Add track").clicked() {
                        match self.model.add_track(self.new_track_name.clone()) {
                            Ok(_) => self.new_track_name.clear(),
                            Err(error) => self.feedback_status = error.to_string(),
                        }
                    }
                });
                ui.separator();
                ui.label(RichText::new("BROWSER").small().strong().color(MUTED));
                for item in ["Instruments", "Effects", "Modulators", "Samples", "Plugins"] {
                    ui.add_enabled(false, egui::Button::new(item))
                        .on_disabled_hover_text("Catalog wiring arrives in later milestones.");
                }
            });
    }

    fn inspector(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("inspector")
            .resizable(true)
            .default_width(300.0)
            .min_width(250.0)
            .frame(
                egui::Frame::new()
                    .fill(PANEL)
                    .inner_margin(egui::Margin::same(14)),
            )
            .show(ctx, |ui| {
                ui.label(RichText::new("CONTEXT").small().strong().color(MUTED));
                // Read the track, collect the edits, then apply them after the borrow ends, so
                // one edit can publish to every id whose effective gain it changed
                let mut mix_edit: Option<TrackMixEdit> = None;
                if let Some(track) = self.model.selected_track() {
                    let id = track.id();
                    let (mut muted, mut soloed) = (track.is_muted(), track.is_soloed());
                    let mut level = track.level();
                    ui.heading(track.name());
                    ui.horizontal(|ui| {
                        if ui.toggle_value(&mut muted, "Mute").changed() {
                            mix_edit = Some(TrackMixEdit::Muted(id, muted));
                        }
                        if ui.toggle_value(&mut soloed, "Solo").changed() {
                            mix_edit = Some(TrackMixEdit::Soloed(id, soloed));
                        }
                        ui.add_enabled(false, egui::Button::new("Arm"))
                            .on_disabled_hover_text(
                                "Recording arrives at R7; nothing records yet.",
                            );
                    });
                    // The slider's range is the accepted gain descriptor's, so the UI stops
                    // contradicting the DSP: unity is the descriptor's default, not 100%
                    let range = GAIN_RANGE;
                    if ui
                        .add(egui::Slider::new(&mut level, range).text("Level"))
                        .changed()
                    {
                        mix_edit = Some(TrackMixEdit::Level(id, level));
                    }
                    ui.separator();
                    ui.label(RichText::new("Signal path").strong());
                    ui.label(
                        RichText::new("Instrument  →  Track gain  →  Sum  →  Master").color(MUTED),
                    );
                    ui.add_enabled(false, egui::Button::new("+ Add device"));
                }
                if let Some(edit) = mix_edit {
                    self.apply_mix_edit(edit);
                }
                ui.separator();
                ui.label(
                    RichText::new("PROTOTYPE FEEDBACK")
                        .small()
                        .strong()
                        .color(ACCENT),
                );
                let mut notes = self.model.feedback().to_owned();
                if ui
                    .add_sized(
                        [ui.available_width(), 120.0],
                        egui::TextEdit::multiline(&mut notes)
                            .hint_text("What felt clear, slow, hidden, or musically wrong?"),
                    )
                    .changed()
                {
                    self.model.set_feedback(notes);
                }
                if ui.button("Copy feedback report").clicked() {
                    ctx.copy_text(self.model.feedback_report());
                    self.feedback_status =
                        "Copied. Paste the report into chat for the next iteration.".into();
                }
                ui.label(RichText::new(&self.feedback_status).small().color(MUTED));
            });
    }

    fn arrange(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            for bar in 1..=9 {
                ui.add_sized(
                    [88.0, 22.0],
                    egui::Label::new(RichText::new(bar.to_string()).small().color(MUTED)),
                );
            }
        });
        ui.separator();
        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 170.0), egui::Sense::hover());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, CornerRadius::same(8), Color32::from_rgb(19, 24, 31));
        for index in 0..9 {
            let x = rect.left() + index as f32 * rect.width() / 9.0;
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                Stroke::new(1.0_f32, Color32::from_rgb(40, 47, 59)),
            );
        }
        let clip = egui::Rect::from_min_size(
            rect.min + Vec2::new(6.0, 34.0),
            Vec2::new(rect.width() * 0.43, 86.0),
        );
        painter.rect_filled(clip, CornerRadius::same(6), Color32::from_rgb(47, 113, 99));
        painter.text(
            clip.min + Vec2::new(12.0, 10.0),
            egui::Align2::LEFT_TOP,
            "Pulse Pattern · 8 bars",
            egui::FontId::proportional(14.0),
            TEXT,
        );
        for step in 0..16 {
            let x = clip.left() + 12.0 + step as f32 * (clip.width() - 24.0) / 16.0;
            let h = if step % 4 == 0 {
                30.0
            } else if step % 3 == 0 {
                20.0
            } else {
                11.0
            };
            painter.line_segment(
                [
                    egui::pos2(x, clip.bottom() - 12.0),
                    egui::pos2(x, clip.bottom() - 12.0 - h),
                ],
                Stroke::new(2.0_f32, ACCENT),
            );
        }
    }

    fn build_devices(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("Compiled stereo path · note events → instrument → effects → master")
                .color(MUTED),
        );
        ui.add_space(18.0);
        let mut open_in_shape = None;
        let presentation = self.model.build_presentation();
        let card_count = presentation.cards().len();
        ui.horizontal_top(|ui| {
            for (index, card) in presentation.cards().enumerate() {
                let device = card.device();
                let selected = card.is_selected();
                egui::Frame::new()
                    .fill(if selected {
                        Color32::from_rgb(38, 58, 58)
                    } else {
                        RAISED
                    })
                    .stroke(if selected {
                        Stroke::new(1.5_f32, ACCENT)
                    } else {
                        Stroke::NONE
                    })
                    .corner_radius(8)
                    .inner_margin(14)
                    .show(ui, |ui| {
                        ui.set_width(190.0);
                        ui.label(RichText::new(device.name).size(18.0).strong().color(ACCENT));
                        ui.label(RichText::new(device.role).small().color(MUTED));
                        ui.separator();
                        for parameter in &device.parameters {
                            ui.horizontal(|ui| {
                                ui.label(parameter.descriptor.name);
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            RichText::new(format!("{:.2}", parameter.value))
                                                .monospace(),
                                        );
                                    },
                                );
                            });
                        }
                        ui.add_space(8.0);
                        if selected {
                            ui.label(RichText::new("Selected device").small().color(ACCENT));
                        }
                        let action = card.action();
                        if ui.button(action.label()).clicked() {
                            open_in_shape = Some(action.device_id());
                        }
                    });
                if index + 1 < card_count {
                    ui.label(RichText::new("→").size(24.0).color(WARM));
                }
            }
        });
        if let Some(id) = open_in_shape {
            open_device_in_shape_from_ui(&mut self.model, id, &mut self.feedback_status);
        }
        ui.add_space(18.0);
        ui.label(
            RichText::new(
                "This chain is the live plan: the same compiled plan the offline harness renders. \
                 Shape edits reach it at the next block boundary.",
            )
            .small()
            .color(WARM),
        );
    }

    fn shape_devices(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new(
                "Controls derive directly from backend descriptors and preserve DSP ranges.",
            )
            .color(MUTED),
        );
        // Three states, each stating what an edit actually reaches right now
        let (message, color) = match self.engine_health() {
            None => (
                "No engine is running. Edits change the model and the offline render only."
                    .to_string(),
                WARM,
            ),
            Some(health) if health.parameters_pending > 0 => (
                format!(
                    "{} edit(s) reached the engine but no processor. This is a defect — see the \
                     transport bar's counters.",
                    health.parameters_pending
                ),
                WARM,
            ),
            Some(health) => (
                format!(
                    "Edits reach live audio at the next block boundary. {} applied so far.",
                    health.parameters_applied
                ),
                ACCENT,
            ),
        };
        ui.label(RichText::new(message).small().color(color));
        ui.add_space(12.0);
        let mut edits = Vec::new();
        let presentation = self.model.shape_presentation();
        egui::ScrollArea::vertical().show(ui, |ui| {
            if let Some(device) = presentation.selected_device() {
                egui::Frame::new()
                    .fill(RAISED)
                    .corner_radius(8)
                    .inner_margin(14)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(device.name).size(19.0).strong().color(ACCENT));
                            ui.label(RichText::new(device.role).small().color(MUTED));
                        });
                        ui.add_space(5.0);
                        for parameter in &device.parameters {
                            let descriptor = parameter.descriptor;
                            let mut value = parameter.value;
                            ui.horizontal(|ui| {
                                ui.set_min_width(520.0);
                                ui.label(RichText::new(descriptor.name).strong());
                                let slider = egui::Slider::new(
                                    &mut value,
                                    descriptor.minimum()..=descriptor.maximum(),
                                )
                                .show_value(true)
                                .text(unit_label(descriptor.unit()));
                                ui.add_sized([360.0, 26.0], slider).on_hover_text(format!(
                                    "Range {:.2}…{:.2}; default {:.2}",
                                    descriptor.minimum(),
                                    descriptor.maximum(),
                                    descriptor.default()
                                ));
                                if ui.small_button("Reset").clicked() {
                                    value = descriptor.default();
                                }
                            });
                            if value != parameter.value {
                                edits.push((device.key, descriptor.key.as_str(), value));
                            }
                        }
                    });
            } else if let Some(message) = presentation.empty_state_message() {
                ui.label(RichText::new(message).color(WARM));
            }
        });
        // One call site for both halves: the model stores the clamped value and the same value
        // is published to the live lane, so the two can never drift apart
        for (device_key, parameter_key, value) in edits {
            apply_parameter_edit(
                &mut self.model,
                self.engine.as_ref(),
                device_key,
                parameter_key,
                value,
                &mut self.feedback_status,
            );
        }
    }

    // The bounce panel. Its geometry, its rate, and its block size all come from the running
    // engine when there is one, so the offline render is configured the way the live path is
    fn bounce_panel(&mut self, ctx: &egui::Context) {
        if !self.bounce_open {
            return;
        }
        // The engine's own rate and block size when a stream is open; the corpus fallback with
        // its reason shown otherwise. A bounce never invents a rate
        let (sample_rate, block_frames, from_engine) = match self.engine.as_ref() {
            Some(engine) => {
                let config = engine.config();
                (f64::from(config.sample_rate), config.buffer_frames, true)
            }
            None => {
                let fallback = spectre_offline::bounce::fallback_config(self.bounce.frames);
                (fallback.sample_rate, fallback.block_frames, false)
            }
        };
        let ceiling = spectre_offline::bounce::max_frames(sample_rate);
        let mut start = false;
        let mut cancel = false;

        egui::SidePanel::right("bounce")
            .resizable(true)
            .default_width(320.0)
            .min_width(260.0)
            .frame(
                egui::Frame::new()
                    .fill(PANEL)
                    .inner_margin(egui::Margin::same(14)),
            )
            .show(ctx, |ui| {
                ui.label(RichText::new("BOUNCE").small().strong().color(ACCENT));
                ui.label("Destination");
                ui.text_edit_singleline(&mut self.bounce.destination);

                ui.label("Length (samples)");
                ui.horizontal(|ui| {
                    let mut frames = self.bounce.frames.to_string();
                    if ui.text_edit_singleline(&mut frames).changed() {
                        self.bounce.frames = frames.trim().parse().unwrap_or(0);
                    }
                    ui.label(
                        RichText::new(duration_label(self.bounce.frames, sample_rate)).color(MUTED),
                    );
                });

                ui.label(RichText::new(format!("Sample rate  {sample_rate:.0} Hz")).color(MUTED))
                    .on_hover_text(if from_engine {
                        "Taken from the running engine, so the bounce matches what you hear"
                    } else {
                        "No engine is running; the workspace's own render rate is used"
                    });
                ui.label(RichText::new(format!("Block size  {block_frames}")).color(MUTED))
                    .on_hover_text(if from_engine {
                        "The device's own buffer size. Containment is scoped to the block, so \
                         this number is part of the render, not a preference"
                    } else {
                        "No engine is running; the workspace's own block size is used"
                    });

                let running_engine = self.engine.is_some();
                ui.add_enabled(
                    running_engine,
                    egui::Checkbox::new(
                        &mut self.bounce.compare_against_live,
                        "Compare against the live engine",
                    ),
                )
                .on_disabled_hover_text("No engine is running");

                ui.separator();
                let request =
                    validate_request(&self.bounce.destination, self.bounce.frames, ceiling);
                match self.bounce.state() {
                    BounceState::Running => {
                        cancel = ui.button("Cancel").clicked();
                        let (done, total) = self.bounce.progress();
                        ui.label(format!("Rendering block {done} of {total}"));
                    }
                    _ => {
                        let response =
                            ui.add_enabled(request.is_ok(), egui::Button::new("Start bounce"));
                        start = response.clicked();
                        if let Err(error) = &request {
                            ui.label(RichText::new(error.to_string()).color(ACCENT));
                        }
                    }
                }

                ui.separator();
                match self.bounce.report() {
                    None if self.bounce.message.is_empty() => {
                        ui.label(RichText::new(
                            "No render yet. A bounce renders this project's signal path offline \
                             and writes a 32-bit float WAV.",
                        )
                        .color(MUTED));
                        // Saying less than this would be a fake surface: ProjectDoc carries no
                        // devices, so a bounce today renders the built-in fixture
                        ui.label(
                            RichText::new(
                                "This project holds no tracks or clips yet; a bounce renders the \
                             built-in device fixture.",
                            )
                            .color(MUTED),
                        );
                    }
                    None => {
                        ui.label(RichText::new(&self.bounce.message).color(ACCENT));
                    }
                    Some(report) => {
                        ui.label(&self.bounce.message);
                        for line in [
                            format!("frames  {}", report.frames),
                            format!("blocks  {}", report.blocks),
                            format!("peak  {:.6}", report.peak),
                            format!("hash  0x{:016x}", report.hash),
                            format!("contained  {}", report.contaminated_nodes),
                            format!("denormals flushed  {}", report.denormals_flushed),
                        ] {
                            ui.label(RichText::new(line).monospace().color(MUTED));
                        }
                    }
                }
            });

        if cancel {
            self.bounce.cancel();
        }
        if start {
            let config = spectre_offline::bounce::BounceConfig {
                sample_rate,
                frames: self.bounce.frames,
                block_frames,
                // One checkbox governs both: the per-block log exists only to localize a
                // comparison mismatch, so it is kept exactly when a comparison is being made
                log_block_hashes: self.bounce.compare_against_live,
            };
            // The shell's own device values, not the fixture's literals: what the user hears
            // in Shape is what the bounce renders
            match self.model.device_parameter_snapshot() {
                Err(error) => self.bounce.message = error.to_string(),
                Ok(values) => {
                    let events = spectre_offline::fixture_events(self.bounce.frames).to_vec();
                    if let Err(error) = self.bounce.start(config, values, events) {
                        self.bounce.message = error.to_string();
                    }
                }
            }
        }
    }

    fn workspace(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(BG)
                    .inner_margin(egui::Margin::same(16)),
            )
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading(self.model.lens().to_string());
                    ui.label(RichText::new("one selection, four musical lenses").color(MUTED));
                });
                ui.add_space(8.0);
                match self.model.lens() {
                    Lens::Arrange => self.arrange(ui),
                    Lens::Build => self.build_devices(ui),
                    Lens::Shape => self.shape_devices(ui),
                    Lens::Mix => mix_surface(ui, self.model.track_list()),
                }
            });
    }
}

impl eframe::App for SpectrePrototype {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // First frame opens the device, so a failure is reported in the UI rather than at startup
        if !self.engine_attempted {
            self.start_engine();
        }
        // Retired render state is released here, on the app thread, never on the audio thread
        if let Some(engine) = self.engine.as_mut() {
            engine.reclaim();
        }
        if ctx.input(|input| input.key_pressed(egui::Key::Space)) {
            self.toggle_transport();
        }
        for (key, lens) in [
            (egui::Key::Num1, Lens::Arrange),
            (egui::Key::Num2, Lens::Build),
            (egui::Key::Num3, Lens::Shape),
            (egui::Key::Num4, Lens::Mix),
        ] {
            if ctx.input(|input| input.key_pressed(key)) {
                self.model.select_lens(lens);
            }
        }
        // Collect a finished render on the app thread; never blocks, so a long bounce does not
        // freeze the window
        self.bounce.poll();
        self.transport(ctx);
        self.lenses(ctx);
        self.track_list(ctx);
        self.inspector(ctx);
        self.bounce_panel(ctx);
        self.workspace(ctx);
        ctx.request_repaint_after(std::time::Duration::from_millis(250));
    }
}

fn unit_label(unit: spectre_core::ParamUnit) -> &'static str {
    match unit {
        spectre_core::ParamUnit::Linear => "",
        spectre_core::ParamUnit::Decibels => "dB",
        spectre_core::ParamUnit::Hertz => "Hz",
        spectre_core::ParamUnit::Milliseconds => "ms",
        spectre_core::ParamUnit::Percent => "%",
        spectre_core::ParamUnit::Semitones => "st",
    }
}

fn mix_surface(ui: &mut egui::Ui, tracks: &spectre_project::TrackList) {
    ui.horizontal_top(|ui| {
        for track in tracks.tracks() {
            egui::Frame::new()
                .fill(RAISED)
                .corner_radius(8)
                .inner_margin(12)
                .show(ui, |ui| {
                    ui.set_width(130.0);
                    ui.label(RichText::new(track.name()).strong());
                    // The bar shows the EFFECTIVE gain, not the fader position, so a muted or
                    // solo-silenced track reads as silent instead of reading as its own fader
                    let effective = tracks.effective_gain(track.id()).unwrap_or(0.0);
                    let fraction = (effective / GAIN_MAX).clamp(0.0, 1.0);
                    ui.add_space(100.0 * (1.0 - fraction));
                    ui.add(
                        egui::ProgressBar::new(fraction)
                            .desired_width(105.0)
                            .text(format!("{effective:.2}")),
                    );
                    let route = if track.is_muted() {
                        "Muted"
                    } else if tracks.any_soloed() && !track.is_soloed() {
                        "Solo-silenced"
                    } else {
                        "Master route"
                    };
                    ui.label(RichText::new(route).small().color(MUTED));
                });
        }
        egui::Frame::new()
            .fill(RAISED)
            .corner_radius(8)
            .inner_margin(12)
            .show(ui, |ui| {
                ui.set_width(130.0);
                ui.label(RichText::new("Master").strong().color(ACCENT));
                let fraction = (tracks.master_level() / GAIN_MAX).clamp(0.0, 1.0);
                ui.add_space(100.0 * (1.0 - fraction));
                ui.add(
                    egui::ProgressBar::new(fraction)
                        .desired_width(105.0)
                        .text(format!("{:.2}", tracks.master_level())),
                );
                ui.label(
                    RichText::new(format!("{} track(s) summed", tracks.len()))
                        .small()
                        .color(MUTED),
                );
            });
    });
}

fn smoke_test() {
    // Build the real shell rather than a bare model, so the engine field below reports what the
    // shell actually holds. If engine startup is ever moved into construction, this changes
    let shell = SpectrePrototype::default();
    let model = &shell.model;
    let selected_device = model
        .selected_device()
        .map(|device| format!("{}({})", device.name, device.key))
        .unwrap_or_else(|| "none".into());
    // The smoke path deliberately opens no device. The engine field is DERIVED from the shell's
    // own engine, not written as a literal, so the assertion in tests/smoke_cli.rs actually fails
    // if startup is wired into the headless path — which would break CI on a device-less host
    println!(
        "Spectre prototype ready lens={} tracks={} clips={} transport={} selected_device={} engine={} bounce={}",
        model.lens(),
        model.tracks().len(),
        model.track_list().placement_count(),
        if model.is_playing() {
            "playing"
        } else {
            "stopped"
        },
        selected_device,
        engine_status_field(shell.engine.as_ref()),
        // Derived from the shell's own panel, not written as a literal, so this fails if a
        // render is ever started from the headless path
        shell.bounce.state().as_str()
    );
}

fn main() -> eframe::Result {
    if std::env::args().any(|argument| argument == "--smoke-test") {
        smoke_test();
        return Ok(());
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Spectre · Interaction Prototype")
            .with_inner_size([1420.0, 860.0])
            .with_min_inner_size([1060.0, 680.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Spectre",
        options,
        Box::new(|context| {
            SpectrePrototype::configure_style(&context.egui_ctx);
            Ok(Box::<SpectrePrototype>::default())
        }),
    )
}
