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
use spectre_app::project::{adopt, is_dirty, open_gate, project_envelope, OpenGate};
use spectre_app::{open_device_in_shape_from_ui, AppModel, Lens};
use spectre_core::{BeatTicks, ObjectId};

const BG: Color32 = Color32::from_rgb(15, 18, 24);
const PANEL: Color32 = Color32::from_rgb(24, 29, 38);
const RAISED: Color32 = Color32::from_rgb(34, 41, 53);
const ACCENT: Color32 = Color32::from_rgb(96, 230, 184);
const WARM: Color32 = Color32::from_rgb(240, 166, 90);
const TEXT: Color32 = Color32::from_rgb(224, 230, 238);
const MUTED: Color32 = Color32::from_rgb(128, 140, 156);

// The fader's range IS the accepted gain descriptor's range; no fader law is invented here
const GAIN_MAX: f32 = 2.0;

// Arrange lane geometry. The label column is wide enough for the longest track name the model
// admits without truncating at the 1060px minimum width this shell supports
const LANE_LABEL_WIDTH: f32 = 120.0;
const LANE_HEIGHT: f32 = 46.0;
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
    // A file path is shell state, not project state, so it lives here rather than on AppModel
    project_path: String,
    // Last save or load outcome, shown verbatim. Never summarized as "failed"
    project_status: String,
    // The bytes the destination holds, as of the last successful save or open. Dirty is DERIVED
    // by comparing the current document against this rather than set by hand at each mutation
    // site: a flag maintained at call sites is a flag someone forgets at the next one, and a
    // marker that reads "Saved" over unsaved work is exactly the product-killing defect the
    // project-safety pillar is about
    saved_snapshot: Option<Vec<u8>>,
    // One interaction of arming for the discard-and-open confirm. No modal: a modal blocks a
    // workspace that has nothing wrong with it
    discard_armed: bool,
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
            project_path: String::new(),
            project_status: String::new(),
            saved_snapshot: None,
            discard_armed: false,
        }
    }
}

// Open the default device against the compiled-in backend
use spectre_app::engine::APP_GRAPH_SEED;

#[cfg(feature = "live-audio")]
fn open_engine(model: &AppModel) -> Result<LiveEngine, EngineUnavailable> {
    let backend = spectre_audio::cpal_backend::CpalBackend::new();
    // The selected track's instrument is the primary note node — the one the lane's live
    // ingress reaches. Every other track with clip material gets its own clip voice
    spectre_app::engine::open_track_engine(
        &backend,
        model.track_list(),
        model.tempo_map(),
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
        let project_dirty = self.project_dirty();
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
                    // Leading side, beside the wordmark, where nothing else sits — so it cannot
                    // collide with R4-1's engine cluster or R4-4's rebuild control on the right
                    if project_dirty {
                        ui.label(RichText::new("•  unsaved").color(WARM))
                            .on_hover_text("This project has changes that are not in a file yet.");
                    }
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
                    // Tempo and meter are the model's defaults; the position comes from the
                    // render thread's own published playhead, or reads as unknown when no block
                    // has rendered -- never as a frozen 1.1.1
                    ui.label(RichText::new("120.00 BPM").monospace().color(TEXT))
                        .on_hover_text("Fixed project default; tempo editing arrives with the arrangement.");
                    ui.label(RichText::new("4 / 4").monospace().color(MUTED))
                        .on_hover_text("Fixed project default; meter editing arrives with the arrangement.");
                    let position = self.engine_health().and_then(|health| {
                        spectre_app::engine::bars_beats(
                            health.position_samples,
                            self.model.tempo_map(),
                            self.model.meter_map(),
                            spectre_core::SampleRate::new(48_000).expect("48 kHz is valid"),
                        )
                    });
                    match position {
                        Some(text) => {
                            ui.label(RichText::new(text).monospace().color(TEXT))
                                .on_hover_text("Bars.beats.sixteenths, from the render thread's published playhead.");
                        }
                        None => {
                            ui.label(RichText::new("—").monospace().color(MUTED))
                                .on_hover_text("No block has rendered yet, so there is no position to report.");
                        }
                    }
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
        // Both actions mutate self, so they are deferred out of the panel closure that borrows it
        let mut save = false;
        let mut open = false;
        let dirty = self.project_dirty();
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
                ui.label(RichText::new("PROJECT").small().strong().color(MUTED));
                ui.add_sized(
                    [ui.available_width(), 28.0],
                    egui::TextEdit::singleline(&mut self.project_path)
                        .hint_text("Project file path"),
                );
                let has_path = !self.project_path.trim().is_empty();
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(has_path, egui::Button::new("Save project"))
                        .on_disabled_hover_text("Type a file path to save into.")
                        .clicked()
                    {
                        save = true;
                    }
                    // The discard confirm replaces the button in place for one interaction, so
                    // one press can never throw away unsaved work
                    let open_label = if dirty && self.discard_armed {
                        "Discard and open"
                    } else {
                        "Open project"
                    };
                    if ui
                        .add_enabled(has_path, egui::Button::new(open_label))
                        .on_disabled_hover_text("Type a file path to open.")
                        .clicked()
                    {
                        open = true;
                    }
                });
                let status = if self.project_status.is_empty() {
                    // States a property the code has, in the same slice as the behavior
                    "No project file yet. Type a path and save — Spectre replaces the file \
                     atomically, so an interrupted save leaves the old file intact."
                } else {
                    self.project_status.as_str()
                };
                ui.label(RichText::new(status).color(MUTED));
                // The word changes, not only the colour
                let (marker, tint) = if dirty {
                    ("Unsaved changes", WARM)
                } else {
                    ("Saved", MUTED)
                };
                ui.label(RichText::new(marker).color(tint));
                ui.separator();
                ui.label(RichText::new("BROWSER").small().strong().color(MUTED));
                for item in ["Instruments", "Effects", "Modulators", "Samples", "Plugins"] {
                    ui.add_enabled(false, egui::Button::new(item))
                        .on_disabled_hover_text("Catalog wiring arrives in later milestones.");
                }
            });
        if save {
            self.save_project();
        }
        if open {
            self.open_project();
        }
    }

    // The project name a save would write, taken from the path's own stem
    fn project_name(&self) -> String {
        std::path::Path::new(self.project_path.trim())
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".into())
    }

    // The bytes a save would write right now
    fn current_project_bytes(&self) -> Option<Vec<u8>> {
        spectre_project::to_bytes(&project_envelope(&self.model, &self.project_name())).ok()
    }

    // True when the document differs from what was last written. Before any save there is
    // nothing on disk, so the answer is yes
    fn project_dirty(&self) -> bool {
        is_dirty(
            self.saved_snapshot.as_deref(),
            self.current_project_bytes().as_deref(),
        )
    }

    // Build the snapshot, write it atomically, and report exactly what happened.
    // Blocks the UI thread; it does not touch the audio thread, because the bridge is not on
    // this path at all
    fn save_project(&mut self) {
        let path = std::path::PathBuf::from(self.project_path.trim());
        let name = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".into());
        let snapshot = project_envelope(&self.model, &name);
        match spectre_project::save_project_atomic(&path, &snapshot) {
            Ok(_) => {
                self.saved_snapshot = spectre_project::to_bytes(&snapshot).ok();
                self.discard_armed = false;
                self.project_status = format!("Saved to {}", path.display());
            }
            Err(spectre_project::SaveError::Io {
                stage: spectre_project::SaveStage::SyncParentDirectory,
                ..
            }) => {
                // The replacement happened but its directory entry may not survive a power
                // loss. The project stays dirty and no second replacement is attempted
                self.project_status = format!(
                    "{} is in place, but the directory entry may not survive a power loss. \
                     Save again.",
                    path.display()
                );
            }
            Err(error) => {
                self.project_status = format!(
                    "{error}. Nothing was written. {} is unchanged.",
                    path.display()
                );
            }
        }
    }

    // Load, then adopt — never the other way round. The live project stays whole until a load
    // has fully succeeded and been accepted
    fn open_project(&mut self) {
        if open_gate(self.project_dirty(), self.discard_armed) == OpenGate::ArmDiscard {
            self.discard_armed = true;
            self.project_status = "Unsaved changes. Press again to discard them and open.".into();
            return;
        }
        let path = std::path::PathBuf::from(self.project_path.trim());
        match spectre_project::load_project(&path) {
            Err(error) => self.project_status = error.to_string(),
            Ok(envelope) => match adopt(&mut self.model, envelope) {
                Err(error) => self.project_status = error.to_string(),
                Ok(()) => {
                    // What the file holds is now what the model holds, so the marker reads
                    // saved without anyone asserting that it should
                    self.saved_snapshot = self.current_project_bytes();
                    self.discard_armed = false;
                    self.project_status = format!("Opened {}", path.display());
                    // The loaded list is a different graph shape, so the running engine is stale
                    self.engine_revision = self.engine_revision.wrapping_sub(1);
                }
            },
        }
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

    // Arrange: one clip lane per track, drawn from the project's own placements.
    //
    // This drew a hardcoded "Pulse Pattern - 8 bars" rectangle with invented step lines until
    // 2026-08-28 -- a surface that corresponded to no project data, which is the fake surface the
    // vision prohibits and the reason NullBackend was refused elsewhere in this file. R4-5 §3.1
    // specifies a per-track clip lane, a clip inspector, and a note list; the model API existed
    // and nothing drew it
    fn arrange(&mut self, ui: &mut egui::Ui) {
        // Eight bars of 4/4 at the accepted 960 PPQ, matching the ruler drawn below. A fixed
        // span rather than a fitted one: a lane that rescales as clips are added moves every
        // other clip under the pointer
        const VISIBLE_BARS: i64 = 8;
        let span_ticks = (VISIBLE_BARS * 4 * spectre_core::TICKS_PER_BEAT) as f32;

        ui.horizontal(|ui| {
            ui.add_sized(
                [LANE_LABEL_WIDTH, 22.0],
                egui::Label::new(RichText::new("track").small().color(MUTED)),
            );
            for bar in 1..=VISIBLE_BARS {
                ui.add_sized(
                    [78.0, 22.0],
                    egui::Label::new(RichText::new(bar.to_string()).small().color(MUTED)),
                );
            }
        });
        ui.separator();

        let selected_clip = self.model.selected_clip();
        let mut clicked: Option<ObjectId> = None;
        let rows: Vec<_> = self
            .model
            .track_list()
            .tracks()
            .iter()
            .map(|track| {
                let placements: Vec<_> = track
                    .clips()
                    .placements()
                    .iter()
                    .map(|placement| {
                        let clip = self.model.track_list().clip(placement.clip());
                        (
                            placement.id(),
                            placement.start(),
                            clip.map(|c| c.length()).unwrap_or(BeatTicks(0)),
                            clip.map(|c| c.name().to_string()).unwrap_or_default(),
                            placement.is_active(),
                        )
                    })
                    .collect();
                (track.name().to_string(), placements)
            })
            .collect();

        if rows.is_empty() {
            ui.add_space(24.0);
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("No tracks. Add one to place a clip.").color(MUTED));
            });
            return;
        }

        for (name, placements) in &rows {
            ui.horizontal(|ui| {
                ui.add_sized(
                    [LANE_LABEL_WIDTH, LANE_HEIGHT],
                    egui::Label::new(RichText::new(name).color(TEXT)),
                );
                let (rect, response) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), LANE_HEIGHT),
                    egui::Sense::click(),
                );
                let painter = ui.painter_at(rect);
                painter.rect_filled(rect, CornerRadius::same(6), Color32::from_rgb(19, 24, 31));
                for bar in 0..VISIBLE_BARS {
                    let x = rect.left() + bar as f32 * rect.width() / VISIBLE_BARS as f32;
                    painter.line_segment(
                        [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                        Stroke::new(1.0_f32, Color32::from_rgb(40, 47, 59)),
                    );
                }

                if placements.is_empty() {
                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "no clips on this track",
                        egui::FontId::proportional(12.0),
                        MUTED,
                    );
                    return;
                }

                for (id, start, length, clip_name, active) in placements {
                    let x0 = rect.left() + (start.0 as f32 / span_ticks) * rect.width();
                    let width = (length.0 as f32 / span_ticks) * rect.width();
                    // A zero-width clip would be invisible and unclickable; the minimum keeps a
                    // very short clip addressable rather than silently absent from the surface
                    let clip_rect = egui::Rect::from_min_size(
                        egui::pos2(x0, rect.top() + 6.0),
                        Vec2::new(width.max(6.0), LANE_HEIGHT - 12.0),
                    )
                    .intersect(rect);
                    let is_selected = selected_clip == Some(*id);
                    // An inactive placement is drawn, not hidden: it is still project data, and
                    // hiding it would make "muted" and "deleted" look identical
                    let fill = if !*active {
                        Color32::from_rgb(38, 46, 44)
                    } else if is_selected {
                        Color32::from_rgb(62, 148, 130)
                    } else {
                        Color32::from_rgb(47, 113, 99)
                    };
                    painter.rect_filled(clip_rect, CornerRadius::same(5), fill);
                    if is_selected {
                        painter.rect_stroke(
                            clip_rect,
                            CornerRadius::same(5),
                            Stroke::new(1.5_f32, ACCENT),
                            egui::StrokeKind::Inside,
                        );
                    }
                    painter.text(
                        clip_rect.min + Vec2::new(8.0, 6.0),
                        egui::Align2::LEFT_TOP,
                        clip_name,
                        egui::FontId::proportional(12.0),
                        if *active { TEXT } else { MUTED },
                    );
                    if response.clicked() {
                        if let Some(position) = response.interact_pointer_pos() {
                            if clip_rect.contains(position) {
                                clicked = Some(*id);
                            }
                        }
                    }
                }
            });
            ui.add_space(6.0);
        }

        if let Some(placement) = clicked {
            // Selection is the only mutation this lens performs; a failed select leaves the
            // previous selection untouched rather than clearing it
            let _ = self.model.select_clip(placement);
        }

        self.clip_inspector(ui);
    }

    // R4-5 §3.1's clip inspector and note list. Drawn inline under the lanes rather than as a
    // trailing side panel, because the lens body is already inside a panel and nesting a second
    // one would clip the note list at the 1060x680 minimum this shell supports
    fn clip_inspector(&mut self, ui: &mut egui::Ui) {
        let Some(placement) = self.model.selected_clip() else {
            ui.add_space(10.0);
            ui.label(
                RichText::new("Select a clip to inspect it.")
                    .small()
                    .color(MUTED),
            );
            return;
        };
        let Some(label) = self.model.clip_label(placement) else {
            return;
        };
        let Some((length, notes, active)) = self.model.selected_clip_detail() else {
            return;
        };

        ui.add_space(10.0);
        ui.separator();
        egui::Frame::new()
            .fill(RAISED)
            .corner_radius(8)
            .inner_margin(14)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&label).size(16.0).strong().color(ACCENT));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut is_active = active;
                        if ui.checkbox(&mut is_active, "active").changed() {
                            let _ = self.model.set_clip_active(placement, is_active);
                        }
                    });
                });
                ui.label(
                    RichText::new(format!(
                        "{} ticks · {} note{}",
                        length.0,
                        notes.len(),
                        if notes.len() == 1 { "" } else { "s" }
                    ))
                    .small()
                    .color(MUTED),
                );
                ui.separator();
                if notes.is_empty() {
                    ui.label(RichText::new("no notes in this clip").small().color(MUTED));
                    return;
                }
                // A list, not a piano roll: R4-5 §3.1 makes this a deliberate accessibility
                // choice under decision 17, not an aesthetic one
                egui::ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        egui::Grid::new("clip-note-list")
                            .num_columns(5)
                            .striped(true)
                            .show(ui, |ui| {
                                for header in ["start", "length", "pitch", "vel", "ch"] {
                                    ui.label(RichText::new(header).small().color(MUTED));
                                }
                                ui.end_row();
                                for note in &notes {
                                    ui.label(note.0.to_string());
                                    ui.label(note.1.to_string());
                                    ui.label(note.2.to_string());
                                    ui.label(format!("{:.2}", note.3));
                                    ui.label(note.4.to_string());
                                    ui.end_row();
                                }
                            });
                    });
            });
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
        "Spectre prototype ready lens={} tracks={} clips={} transport={} selected_device={} engine={} bounce={} project={}",
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
        shell.bounce.state().as_str(),
        // Read from the shell's own project state for the same reason: this must fail if the
        // headless launch ever opens a file
        match shell.saved_snapshot {
            None => "none",
            Some(_) => "loaded",
        }
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
