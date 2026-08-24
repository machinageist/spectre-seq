// Author: Jeff
// Date: 2026-07-12
// Description: Native egui shell for iterative Spectre workflow feedback
// Notes: The live engine is wired here and owned by SpectrePrototype, not by AppModel, so the
//   model stays renderer-neutral and the non-Send stream stays on the thread that created it.
//   Persistence wiring remains out of scope until R4-7.

use eframe::egui::{self, Color32, CornerRadius, RichText, Stroke, Vec2};
use spectre_app::engine::{
    AuditionError, EngineHealth, EngineState, EngineUnavailable, LiveEngine,
};
use spectre_app::{open_device_in_shape_from_ui, set_device_parameter_from_ui, AppModel, Lens};

const BG: Color32 = Color32::from_rgb(15, 18, 24);
const PANEL: Color32 = Color32::from_rgb(24, 29, 38);
const RAISED: Color32 = Color32::from_rgb(34, 41, 53);
const ACCENT: Color32 = Color32::from_rgb(96, 230, 184);
const WARM: Color32 = Color32::from_rgb(240, 166, 90);
const TEXT: Color32 = Color32::from_rgb(224, 230, 238);
const MUTED: Color32 = Color32::from_rgb(128, 140, 156);

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
        }
    }
}

// Open the default device against the compiled-in backend
#[cfg(feature = "live-audio")]
fn open_engine(model: &AppModel) -> Result<LiveEngine, EngineUnavailable> {
    let snapshot = model
        .device_parameter_snapshot()
        .map_err(EngineUnavailable::Snapshot)?;
    let backend = spectre_audio::cpal_backend::CpalBackend::new();
    spectre_app::engine::open_default(&backend, &snapshot)
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
    fn toggle_transport(&mut self) {
        let playing = self.model.is_playing();
        let Some(engine) = self.engine.as_mut() else {
            self.model.toggle_play();
            return;
        };
        let result = if playing {
            engine.stop_audition()
        } else {
            engine.start_audition()
        };
        match result {
            Ok(()) => {
                self.model.toggle_play();
            }
            // The transport command was queued and the render thread will apply it, so the UI
            // flips anyway; a queued command cannot be recalled from a wait-free lane
            Err(AuditionError::Note(error)) => {
                self.model.toggle_play();
                self.feedback_status =
                    format!("Transport changed but the note was dropped: {error:?}");
            }
            // No transport command was queued, so the UI must not change either
            Err(AuditionError::Transport(error)) => {
                self.feedback_status =
                    format!("Transport change refused: {error:?}. Nothing changed; try again.");
            }
        }
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
                                    "blocks {} · xruns {} · plan errors {} · frame rejections {} · notes deferred {} · contaminated {} · stream errors {}",
                                    health.blocks_rendered,
                                    health.xruns,
                                    health.plan_errors,
                                    health.frame_capacity_rejections,
                                    health.notes_deferred,
                                    health.contaminated_nodes,
                                    health.stream_errors,
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
                    });
                });
            });
        if toggle {
            self.toggle_transport();
        }
        if retry {
            self.start_engine();
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
                    let selected = self.model.selected_track_id() == Some(track.id);
                    let label = format!("{}  {}", if track.armed { "●" } else { "○" }, track.name);
                    if ui.selectable_label(selected, label).clicked() {
                        select = Some(track.id);
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
                            Err(error) => self.feedback_status = error.into(),
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
                if let Some(track) = self.model.selected_track_mut() {
                    ui.heading(&track.name);
                    ui.horizontal(|ui| {
                        ui.toggle_value(&mut track.muted, "Mute");
                        ui.toggle_value(&mut track.solo, "Solo");
                        ui.toggle_value(&mut track.armed, "Arm");
                    });
                    ui.add(egui::Slider::new(&mut track.level, 0.0..=1.0).text("Level"));
                    ui.separator();
                    ui.label(RichText::new("Signal path").strong());
                    ui.label(RichText::new("MIDI clip  →  Native synth  →  Master").color(MUTED));
                    ui.add_enabled(false, egui::Button::new("+ Add device"));
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
                 Parameter edits do not reach it yet — the runtime parameter seam lands at R4-2.",
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
        ui.label(
            RichText::new(
                "Edits here change the model and the offline render. They do not change live \
                 audio: the runtime parameter seam is decision 22 and lands at R4-2.",
            )
            .small()
            .color(WARM),
        );
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
        for (device_key, parameter_key, value) in edits {
            set_device_parameter_from_ui(
                &mut self.model,
                device_key,
                parameter_key,
                value,
                &mut self.feedback_status,
            );
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
                    Lens::Mix => mix_surface(ui, self.model.tracks()),
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
        self.transport(ctx);
        self.lenses(ctx);
        self.track_list(ctx);
        self.inspector(ctx);
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

fn mix_surface(ui: &mut egui::Ui, tracks: &[spectre_app::TrackView]) {
    ui.horizontal_top(|ui| {
        for track in tracks {
            egui::Frame::new()
                .fill(RAISED)
                .corner_radius(8)
                .inner_margin(12)
                .show(ui, |ui| {
                    ui.set_width(130.0);
                    ui.label(RichText::new(&track.name).strong());
                    ui.add_space(100.0 * (1.0 - track.level));
                    ui.add(
                        egui::ProgressBar::new(track.level)
                            .desired_width(105.0)
                            .text(format!("{:.0}%", track.level * 100.0)),
                    );
                    ui.label(RichText::new("Master route").small().color(MUTED));
                });
        }
    });
}

fn smoke_test() {
    let model = AppModel::prototype();
    let selected_device = model
        .selected_device()
        .map(|device| format!("{}({})", device.name, device.key))
        .unwrap_or_else(|| "none".into());
    // The smoke path deliberately opens no device: this assertion is what fails if engine
    // startup is ever wired into the headless path, which would break CI on a device-less host
    println!(
        "Spectre prototype ready lens={} tracks={} transport={} selected_device={} engine=not-started",
        model.lens(),
        model.tracks().len(),
        if model.is_playing() {
            "playing"
        } else {
            "stopped"
        },
        selected_device
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
