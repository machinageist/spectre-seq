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
use spectre_app::project::{
    adopt, adopt_recovered, autosave_action, commit_save, is_dirty, open_gate, project_envelope,
    run_autosave, AutosaveAction, OpenGate, SaveOutcome,
};
use spectre_app::{open_device_in_shape_from_ui, AppModel, Lens};
use spectre_core::{BeatTicks, ObjectId};
use spectre_project::{TrackEffect, TrackInsert, TrackInstrument};

const BG: Color32 = Color32::from_rgb(15, 18, 24);
const PANEL: Color32 = Color32::from_rgb(24, 29, 38);
const RAISED: Color32 = Color32::from_rgb(34, 41, 53);
const ACCENT: Color32 = Color32::from_rgb(96, 230, 184);
const WARM: Color32 = Color32::from_rgb(240, 166, 90);
const TEXT: Color32 = Color32::from_rgb(224, 230, 238);
const MUTED: Color32 = Color32::from_rgb(128, 140, 156);

// The fader's range IS the accepted gain descriptor's range; no fader law is invented here
const GAIN_MAX: f32 = 2.0;

// The shell's window geometry. Named rather than inlined because three separate places depend on
// these numbers -- the window builder, two layout comments, and r4-qa-protocol.md row 10, which
// asks an operator to check both sizes by name. They were literals in all of them until 2026-08-29
const WINDOW_DEFAULT_SIZE: [f32; 2] = [1420.0, 860.0];
const WINDOW_MIN_SIZE: [f32; 2] = [1060.0, 680.0];

// Arrange lane geometry. The label column is wide enough for the longest track name the model
// admits without truncating at WINDOW_MIN_SIZE's width
const LANE_LABEL_WIDTH: f32 = 120.0;
const LANE_HEIGHT: f32 = 46.0;
const GAIN_RANGE: std::ops::RangeInclusive<f32> = 0.0..=GAIN_MAX;
// The accepted tempo bounds, not a second pair invented for the widget. TempoMap refuses outside
// them anyway; matching here means the control cannot offer a value the model would reject
const TEMPO_RANGE: std::ops::RangeInclusive<f64> =
    spectre_core::tempo::MIN_BPM..=spectre_core::tempo::MAX_BPM;
// Highest bar the loop spinners offer. Not a model bound -- BeatTicks reaches far past this --
// but a drag control needs an end, and 999 bars is well past any loop a musician sets by hand
const LOOP_MAX_BAR: u32 = 999;
// Velocity for a note written from the list. The descriptor range is 0..=1 and this is a plain
// mezzo-forte; a velocity field is a control the piano roll will own rather than this list
const DEFAULT_NOTE_VELOCITY: f32 = 0.8;

// A structure edit requested from the track list, applied after the panel closes because each
// one mutates the model the panel is borrowing
// A device-chain edit requested from the inspector, applied after the panel closes because each
// one mutates the model the panel is borrowing
// A clip edit requested from the inspector, applied after the panel closes
#[derive(Debug, Clone, Copy)]
enum ClipAction {
    Create(ObjectId),
    Delete(ObjectId),
}

// A note edit requested from the clip inspector, applied after the panel closes
#[derive(Debug, Clone, Copy)]
enum NoteAction {
    Add,
    Remove(usize),
}

#[derive(Debug, Clone, Copy)]
enum ChainAction {
    Add(ObjectId),
    Remove(ObjectId, usize),
    Move(ObjectId, usize, usize),
    Depth(ObjectId, usize, f32),
}

#[derive(Debug, Clone, Copy)]
enum TrackAction {
    Delete(ObjectId),
    Move(ObjectId, usize),
}

// One-based bars to the tick domain the model stores. 4/4 until the meter is editable, which is
// the same assumption the transport's own "4 / 4" label still makes
// The name a musician sees for each instrument. Not derived from the enum's Debug, so renaming a
// variant cannot silently change what the product calls it
fn instrument_label(instrument: TrackInstrument) -> &'static str {
    match instrument {
        TrackInstrument::Pulse => "Pulse",
        TrackInstrument::Filament => "Filament",
    }
}

// Ticks in one bar. 4/4 until the meter is editable, the same assumption the loop control makes
fn bar_ticks() -> i64 {
    spectre_core::TICKS_PER_BEAT * 4
}

fn bars_to_ticks(bars: (u32, u32)) -> (BeatTicks, BeatTicks) {
    let per_bar = spectre_core::TICKS_PER_BEAT * 4;
    (
        BeatTicks(i64::from(bars.0 - 1) * per_bar),
        BeatTicks(i64::from(bars.1 - 1) * per_bar),
    )
}

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
    // The bar range the loop control shows, one-based the way a musician counts. Shell state:
    // the model holds the loop in ticks, and this is only what the two spinners display
    loop_bars: (u32, u32),
    // The note the "+ Note" button writes: start tick, length, pitch. Shell state, because it is
    // what the fields show rather than anything the project holds
    new_note: (i64, i64, u8),
    // The bytes the sidecar holds, tracked the same derived way saved_snapshot is, so the
    // autosave trigger compares content rather than consulting a clock
    autosaved_snapshot: Option<Vec<u8>>,
    // Last autosave outcome, shown verbatim beside the save status. An autosave that failed
    // silently would be worse than none, because the musician would believe work was protected
    autosave_status: String,
    // Unsaved work found beside the opened project, held until the musician answers. Never
    // applied on its own: a musician looking at a project must know whether it is what they
    // saved or what a crash recovered, and only an explicit act can guarantee that
    recovery_offer: Option<Box<spectre_project::recovery::RecoveryOffer>>,
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
            loop_bars: (1, 5),
            new_note: (0, 480, 60),
            autosaved_snapshot: None,
            autosave_status: String::new(),
            recovery_offer: None,
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
                // The graph builds every device from its DESCRIPTOR DEFAULTS, so without this a
                // value the musician saved is shown by Shape and not played by the engine -- the
                // UI and the audio disagreeing about the same control
                if let Err(error) =
                    spectre_app::engine::publish_stored_parameters(&self.model, &engine)
                {
                    self.feedback_status =
                        format!("Stored device values did not reach live audio: {error}");
                }
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

    // The rate the stream was actually opened at, which is the device's own rate rather than a
    // constant. The position readout converts samples to ticks through it, so a literal here
    // misreports the playhead on every device that does not happen to run at 48 kHz -- the
    // macOS qualification drill opened at 88 200 Hz, where a 48 000 literal reads ~1.8x fast
    fn engine_sample_rate(&self) -> Option<spectre_core::SampleRate> {
        let rate = self.engine.as_ref()?.config().sample_rate;
        spectre_core::SampleRate::new(rate)
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
        // Collected during the draw and applied after it, for the same reason save and open are:
        // mutating the model mid-draw would leave the rest of this frame rendering stale state
        let mut tempo_edit: Option<f64> = None;
        let mut loop_edit: Option<Option<(BeatTicks, BeatTicks)>> = None;
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
                    // The project's own tempo, editable. It was a string literal until
                    // 2026-09-06, so nothing could be written at any other tempo
                    let mut bpm = self.model.tempo_map().segments()[0].bpm;
                    if ui
                        .add(
                            egui::DragValue::new(&mut bpm)
                                .speed(0.5)
                                .range(TEMPO_RANGE)
                                .suffix(" BPM"),
                        )
                        .on_hover_text("Project tempo. Undoable, and republished to the engine.")
                        .changed()
                    {
                        tempo_edit = Some(bpm);
                    }
                    // Loop-first composition is the vision's own first core-loop item, and the
                    // transport carried a loop region the product could not set. Bars rather
                    // than ticks, because that is what a musician means by "loop four bars"
                    let looping = self.model.loop_ticks().is_some();
                    if ui
                        .selectable_label(looping, "⟲ Loop")
                        .on_hover_text("Loop the bar range beside this control.")
                        .clicked()
                    {
                        loop_edit = Some(if looping {
                            None
                        } else {
                            Some(bars_to_ticks(self.loop_bars))
                        });
                    }
                    let mut bars = self.loop_bars;
                    let start = ui.add(
                        egui::DragValue::new(&mut bars.0)
                            .speed(0.25)
                            .range(1..=LOOP_MAX_BAR)
                            .prefix("bar "),
                    );
                    let end = ui.add(
                        egui::DragValue::new(&mut bars.1)
                            .speed(0.25)
                            .range(2..=LOOP_MAX_BAR + 1)
                            .prefix("to "),
                    );
                    if start.changed() || end.changed() {
                        // Keep the range non-empty as it is dragged, so the model never sees a
                        // region it would have to refuse
                        bars.1 = bars.1.max(bars.0 + 1);
                        self.loop_bars = bars;
                        if looping {
                            loop_edit = Some(Some(bars_to_ticks(bars)));
                        }
                    }
                    ui.label(RichText::new("4 / 4").monospace().color(MUTED))
                        .on_hover_text("Fixed project default; meter editing arrives with the arrangement.");
                    let position = self
                        .engine_health()
                        .zip(self.engine_sample_rate())
                        .and_then(|(health, rate)| {
                            spectre_app::engine::bars_beats(
                                health.position_samples,
                                self.model.tempo_map(),
                                self.model.meter_map(),
                                rate,
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
        // Tempo converts ticks to samples, so a change must reach the render thread's baked
        // material as well as the model, or the readout and the audio disagree
        if let Some(bpm) = tempo_edit {
            match self.model.set_tempo(bpm) {
                Ok(()) => {
                    self.republish_schedules();
                    // The loop is stored in ticks and sent in samples, so a tempo change moves
                    // where it lands. Republishing it here is what stops a loop from drifting
                    // off the bar line the musician set it on
                    self.republish_loop();
                }
                Err(error) => self.project_status = format!("tempo refused: {error:?}"),
            }
        }
        if let Some(region) = loop_edit {
            match self.model.set_loop(region) {
                Ok(()) => self.republish_loop(),
                Err(error) => self.project_status = format!("loop refused: {error}"),
            }
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
        let mut undo = false;
        let mut redo = false;
        let mut track_action: Option<TrackAction> = None;
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
                let count = self.model.tracks().len();
                for (index, track) in self.model.tracks().iter().enumerate() {
                    let id = track.id();
                    let selected = self.model.selected_track_id() == Some(id);
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
                    ui.horizontal(|ui| {
                        let label = format!("{marker}  {}", track.name());
                        if ui.selectable_label(selected, label).clicked() {
                            select = Some(id);
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Delete, then reorder. Every one of these was built and
                            // reversible with no way to reach it, so a track added by
                            // mistake could never be removed
                            if ui
                                .small_button("✕")
                                .on_hover_text("Delete this track")
                                .clicked()
                            {
                                track_action = Some(TrackAction::Delete(id));
                            }
                            if ui
                                .add_enabled(index + 1 < count, egui::Button::new("▾").small())
                                .on_hover_text("Move down")
                                .clicked()
                            {
                                track_action = Some(TrackAction::Move(id, index + 1));
                            }
                            if ui
                                .add_enabled(index > 0, egui::Button::new("▴").small())
                                .on_hover_text("Move up")
                                .clicked()
                            {
                                track_action = Some(TrackAction::Move(id, index - 1));
                            }
                        });
                    });
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
                    // Disabled rather than hidden, so the control's existence is not a
                    // function of whether it would currently do anything
                    if ui
                        .add_enabled(self.model.can_undo(), egui::Button::new("Undo"))
                        .on_disabled_hover_text("Nothing to undo.")
                        .clicked()
                    {
                        undo = true;
                    }
                    if ui
                        .add_enabled(self.model.can_redo(), egui::Button::new("Redo"))
                        .on_disabled_hover_text("Nothing to redo.")
                        .clicked()
                    {
                        redo = true;
                    }
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
                // Stated only when a sidecar exists. Reporting "no autosave" on a clean project
                // would read as a warning about a condition that is correct
                if !self.autosave_status.is_empty() {
                    ui.label(RichText::new(&self.autosave_status).color(MUTED));
                }
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
        // Track structure edits change the graph's shape, so the transport reports PLAN STALE
        // afterwards rather than the change being silently inaudible
        if let Some(action) = track_action {
            let outcome = match action {
                TrackAction::Delete(id) => self.model.remove_track(id).map(|_| ()),
                TrackAction::Move(id, to) => self.model.reorder_track(id, to).map(|_| ()),
            };
            if let Err(error) = outcome {
                self.feedback_status = error.to_string();
            }
        }
        if undo || redo {
            let outcome = if undo {
                self.model.undo()
            } else {
                self.model.redo()
            };
            match outcome {
                Ok(true) => {
                    // A reversed edit can change the graph shape, so the running plan is stale
                    self.engine_revision = self.engine_revision.wrapping_sub(1);
                    self.project_status = if undo {
                        "Undid one edit."
                    } else {
                        "Redid one edit."
                    }
                    .into();
                }
                Ok(false) => {}
                Err(error) => self.project_status = format!("{error}"),
            }
        }
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
        // The same name project_dirty() builds its comparison bytes from. Two copies of this
        // fallback could drift, and then the dirty marker would compare a document built with
        // one name against a file written with another and never read clean
        let snapshot = project_envelope(&self.model, &self.project_name());
        // commit_save owns the save-then-retire-the-sidecar order; this match only renders it
        match commit_save(&path, &snapshot) {
            SaveOutcome::Saved => {
                self.saved_snapshot = spectre_project::to_bytes(&snapshot).ok();
                self.autosaved_snapshot = None;
                self.autosave_status.clear();
                self.discard_armed = false;
                self.project_status = format!("Saved to {}", path.display());
            }
            SaveOutcome::SavedSidecarRetained => {
                self.saved_snapshot = spectre_project::to_bytes(&snapshot).ok();
                self.discard_armed = false;
                self.project_status = format!("Saved to {}", path.display());
                self.autosave_status =
                    "Saved, but the autosave sidecar could not be removed. It will be offered \
                     on the next open and can be declined."
                        .into();
            }
            SaveOutcome::DurabilityUncertain => {
                // The replacement happened but its directory entry may not survive a power
                // loss. The project stays dirty, no second replacement is attempted, and the
                // sidecar is deliberately kept
                self.project_status = format!(
                    "{} is in place, but the directory entry may not survive a power loss. \
                     Save again.",
                    path.display()
                );
            }
            SaveOutcome::Failed(error) => {
                self.project_status = format!(
                    "{error}. Nothing was written. {} is unchanged.",
                    path.display()
                );
            }
        }
    }

    // Present unsaved work found beside the project. Throwaway affordance for R5 slice 6: the
    // adoption rule lives in spectre_app::project, and only the drawing is here.
    //
    // Drawn as a panel rather than a modal. A modal would block a workspace that has nothing
    // wrong with it, and the musician may want to look at what they saved before deciding
    fn recovery_panel(&mut self, ctx: &egui::Context) {
        let Some(offer) = self.recovery_offer.as_ref() else {
            return;
        };
        let mut recover = false;
        let mut keep_saved = false;
        egui::TopBottomPanel::top("recovery").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.label(
                RichText::new("Unsaved work was found beside this project")
                    .strong()
                    .color(TEXT),
            );
            for line in offer.describe() {
                ui.label(RichText::new(format!("  • {line}")).color(MUTED));
            }
            // Stated, not implied. A difference list a caller could read as exhaustive would
            // let a musician decline work this comparison never looked at
            ui.label(
                RichText::new(format!(
                    "Not compared: {}. Recovering loads the unsaved work without writing \
                     either file; both are kept until you Save.",
                    offer.uncompared().join(", ")
                ))
                .color(MUTED),
            );
            ui.horizontal(|ui| {
                recover = ui
                    .button("Recover unsaved work")
                    .on_hover_text("Loads it as unsaved. Neither file on disk changes.")
                    .clicked();
                keep_saved = ui
                    .button("Keep the saved version")
                    .on_hover_text("Discards the unsaved work and removes its sidecar.")
                    .clicked();
            });
            ui.add_space(6.0);
        });

        let path = std::path::PathBuf::from(self.project_path.trim());
        if recover {
            let offer = self
                .recovery_offer
                .take()
                .expect("the offer was just drawn");
            match adopt_recovered(&mut self.model, &offer) {
                Ok(saved_bytes) => {
                    // The bytes the PROJECT FILE holds, never the recovered ones, so the marker
                    // reads unsaved. Setting this from the recovery would report "Saved" over
                    // work no file holds
                    self.saved_snapshot = saved_bytes;
                    // The sidecar still holds this work and stays until a manual Save retires it
                    self.autosaved_snapshot = self.current_project_bytes();
                    self.engine_revision = self.engine_revision.wrapping_sub(1);
                    self.project_status =
                        "Recovered unsaved work. It is not saved yet — press Save to keep it."
                            .into();
                }
                Err(error) => {
                    self.project_status = format!("Unsaved work could not be adopted: {error}");
                    // Put it back: a refused adoption must not silently drop the offer
                    self.recovery_offer = Some(offer);
                }
            }
        } else if keep_saved {
            self.recovery_offer = None;
            match spectre_project::recovery::decline(&path) {
                Ok(()) => self.autosave_status = "Unsaved work discarded.".into(),
                Err(error) => {
                    self.autosave_status = format!("Unsaved work could not be discarded: {error}");
                }
            }
        }
    }

    // Rebake and republish every track's material to the running stream. Called after any edit
    // that changes what the render thread should be playing -- a note, a clip, or the tempo that
    // converts their ticks to samples. A refused publication is reported rather than swallowed,
    // because a musician whose edit did not reach the engine must be told
    fn republish_schedules(&mut self) {
        let Some(engine) = self.engine.as_mut() else {
            return;
        };
        if let Err(error) =
            engine.publish_schedules(self.model.track_list(), self.model.tempo_map())
        {
            self.project_status = format!("Edit did not reach the engine: {error}");
        }
    }

    // Send the model's loop region to the running stream, converted to samples there
    fn republish_loop(&mut self) {
        let region = self.model.loop_ticks();
        let Some(engine) = self.engine.as_mut() else {
            return;
        };
        if let Err(error) = engine.publish_loop(region, self.model.tempo_map()) {
            self.project_status = format!("Loop did not reach the engine: {error}");
        }
    }

    // Journal unsaved work to the sidecar when the document's own content says it should be.
    // Throwaway affordance for R5 slice 6: the decision and the write both live in
    // spectre_app::project, and only this call site is drawn
    fn maybe_autosave(&mut self) {
        let path = self.project_path.trim().to_string();
        let current = self.current_project_bytes();
        let action = autosave_action(
            !path.is_empty(),
            current.as_deref(),
            self.saved_snapshot.as_deref(),
            self.autosaved_snapshot.as_deref(),
        );
        let path = std::path::PathBuf::from(&path);
        match action {
            AutosaveAction::Write => {
                let snapshot = project_envelope(&self.model, &self.project_name());
                match run_autosave(&path, &snapshot) {
                    Ok(()) => {
                        self.autosaved_snapshot = current;
                        self.autosave_status =
                            format!("Unsaved work journaled beside {}", path.display());
                    }
                    Err(error) => {
                        self.autosave_status = format!("Autosave failed: {error}");
                    }
                }
            }
            AutosaveAction::Discard => {
                if self.autosaved_snapshot.is_some() {
                    let _ = spectre_project::journal::discard_autosave(&path);
                    self.autosaved_snapshot = None;
                    self.autosave_status.clear();
                }
            }
            AutosaveAction::Skip => {}
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
                    // Inspect AFTER the saved project is live, so the musician is looking at
                    // what they saved while deciding whether to take the unsaved work instead
                    self.autosaved_snapshot = None;
                    self.autosave_status.clear();
                    self.recovery_offer = None;
                    match spectre_project::recovery::inspect(&path) {
                        Ok(spectre_project::recovery::Recovery::Available(offer)) => {
                            self.recovery_offer = Some(offer);
                        }
                        Ok(spectre_project::recovery::Recovery::Redundant) => {
                            // The sidecar matched the project exactly. Retiring it here is not
                            // a discard of work: there is none to lose
                            let _ = spectre_project::journal::discard_autosave(&path);
                        }
                        Ok(spectre_project::recovery::Recovery::Nothing) => {}
                        Err(error) => {
                            self.autosave_status =
                                format!("Unsaved work could not be inspected: {error}");
                        }
                    }
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
                let mut instrument_edit: Option<(ObjectId, TrackInstrument)> = None;
                let mut rename_edit: Option<(ObjectId, String)> = None;
                let mut master_edit: Option<f32> = None;
                let mut clip_action: Option<ClipAction> = None;
                let mut chain_action: Option<ChainAction> = None;
                if let Some(track) = self.model.selected_track() {
                    let id = track.id();
                    let (mut muted, mut soloed) = (track.is_muted(), track.is_soloed());
                    let mut level = track.level();
                    // Editable rather than a heading: rename_track was built and reversible with
                    // no way to reach it, so a track kept whatever name it was created with
                    let mut name = track.name().to_string();
                    if ui
                        .add(
                            egui::TextEdit::singleline(&mut name)
                                .desired_width(f32::INFINITY)
                                .hint_text("Track name"),
                        )
                        .changed()
                    {
                        rename_edit = Some((id, name));
                    }
                    // Which synth the track plays. Every track was a Pulse saw for the life of
                    // the project until 2026-09-07, because nothing called set_instrument
                    let current = track.instrument();
                    egui::ComboBox::from_label("Instrument")
                        .selected_text(instrument_label(current))
                        .show_ui(ui, |ui| {
                            for candidate in [TrackInstrument::Pulse, TrackInstrument::Filament] {
                                if ui
                                    .selectable_label(
                                        current == candidate,
                                        instrument_label(candidate),
                                    )
                                    .clicked()
                                    && candidate != current
                                {
                                    instrument_edit = Some((id, candidate));
                                }
                            }
                        });
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
                    // The device chain. Gloam was unreachable on any track a musician made:
                    // Track::new starts with an empty chain and nothing in the shell ever built
                    // a TrackInsert, so the ordered chain model had no surface at all
                    ui.separator();
                    ui.label(RichText::new("EFFECTS").small().strong().color(MUTED));
                    let chain_len = track.inserts().len();
                    for (position, insert) in track.inserts().iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Gloam").color(TEXT));
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.small_button("✕").on_hover_text("Remove").clicked() {
                                        chain_action = Some(ChainAction::Remove(id, position));
                                    }
                                    if ui
                                        .add_enabled(
                                            position + 1 < chain_len,
                                            egui::Button::new("▾").small(),
                                        )
                                        .clicked()
                                    {
                                        chain_action =
                                            Some(ChainAction::Move(id, position, position + 1));
                                    }
                                    if ui
                                        .add_enabled(position > 0, egui::Button::new("▴").small())
                                        .clicked()
                                    {
                                        chain_action =
                                            Some(ChainAction::Move(id, position, position - 1));
                                    }
                                },
                            );
                        });
                        // Addressed by POSITION, so the second Gloam in a chain is reachable --
                        // Shape resolves by key and would only ever find the first
                        let mut depth = insert.depth();
                        if ui
                            .add(egui::Slider::new(&mut depth, 0.0..=1.0).text("Depth"))
                            .changed()
                        {
                            chain_action = Some(ChainAction::Depth(id, position, depth));
                        }
                    }
                    if ui
                        .button("+ Add Gloam")
                        .on_hover_text("Append an effect to this track's chain.")
                        .clicked()
                    {
                        chain_action = Some(ChainAction::Add(id));
                    }

                    // Clip creation. Without it a musician has nowhere to write a note: the clip
                    // API was complete and reversible with no surface, and a project created in
                    // the app started at zero clips and stayed there
                    ui.separator();
                    ui.label(RichText::new("CLIPS").small().strong().color(MUTED));
                    ui.horizontal(|ui| {
                        if ui
                            .button("+ Clip")
                            .on_hover_text("Add a one-bar clip after this track's last one.")
                            .clicked()
                        {
                            clip_action = Some(ClipAction::Create(id));
                        }
                        let selected = self.model.selected_clip();
                        if ui
                            .add_enabled(selected.is_some(), egui::Button::new("Delete clip"))
                            .on_disabled_hover_text("Select a clip in Arrange first.")
                            .clicked()
                        {
                            if let Some(placement) = selected {
                                clip_action = Some(ClipAction::Delete(placement));
                            }
                        }
                    });
                }
                if let Some(edit) = mix_edit {
                    self.apply_mix_edit(edit);
                }
                ui.separator();
                // The master fader. set_master_level and publish_master_gain were both built
                // with no caller, so the output level could not be changed at all
                let mut master = self.model.track_list().master_level();
                ui.label(RichText::new("MASTER").small().strong().color(MUTED));
                if ui
                    .add(egui::Slider::new(&mut master, GAIN_RANGE).text("Level"))
                    .changed()
                {
                    master_edit = Some(master);
                }
                // A shape change: the instrument node itself differs, so the running plan is
                // stale until rebuilt. The transport says PLAN STALE rather than pretending the
                // change is audible, which is the rule R4-4 established for structure edits
                if let Some((id, instrument)) = instrument_edit {
                    if let Err(error) = self.model.set_track_instrument(id, instrument) {
                        self.feedback_status = format!("{error}");
                    }
                }
                // A blank name is refused by the model, so the field simply does not take
                if let Some((id, name)) = rename_edit {
                    if let Err(error) = self.model.rename_track(id, &name) {
                        self.feedback_status = error.to_string();
                    }
                }
                // Add, remove and move change the graph's shape, so the transport reports PLAN
                // STALE. A depth edit travels the parameter lane and is audible at once
                if let Some(action) = chain_action {
                    let outcome = match action {
                        ChainAction::Add(id) => self
                            .model
                            .append_effect(id, TrackInsert::new(TrackEffect::Gloam, 0.5)),
                        ChainAction::Remove(id, position) => self.model.remove_effect(id, position),
                        ChainAction::Move(id, from, to) => self.model.move_effect(id, from, to),
                        ChainAction::Depth(id, position, depth) => {
                            let stored = self.model.set_effect_depth(id, position, depth);
                            if stored.is_ok() {
                                if let Some(engine) = self.engine.as_ref() {
                                    let _ = spectre_app::engine::publish_effect_parameter(
                                        engine,
                                        self.model.track_list(),
                                        id,
                                        position,
                                        spectre_dsp::GLOAM_DEPTH,
                                        depth,
                                    );
                                }
                            }
                            stored
                        }
                    };
                    if let Err(error) = outcome {
                        self.feedback_status = error.to_string();
                    }
                }
                // Both change what should be playing, so both republish
                if let Some(action) = clip_action {
                    let outcome = match action {
                        ClipAction::Create(track) => {
                            // Appended after the track's last clip, because placements may not
                            // overlap and starting every new clip at zero would be refused as
                            // soon as a track had one
                            let start = self
                                .model
                                .track_list()
                                .get(track)
                                .map(|entry| {
                                    entry
                                        .clips()
                                        .placements()
                                        .iter()
                                        .enumerate()
                                        .map(|(index, placement)| {
                                            placement.start().0
                                                + entry
                                                    .clips()
                                                    .length_at(index)
                                                    .map_or(0, |length| length.0)
                                        })
                                        .max()
                                        .unwrap_or(0)
                                })
                                .unwrap_or(0);
                            self.model
                                .create_clip(
                                    track,
                                    "Clip",
                                    BeatTicks(spectre_core::TICKS_PER_BEAT * 4),
                                    BeatTicks(start),
                                )
                                .map(|_| ())
                        }
                        ClipAction::Delete(placement) => self.model.delete_clip(placement),
                    };
                    match outcome {
                        Ok(()) => self.republish_schedules(),
                        Err(error) => self.feedback_status = error.to_string(),
                    }
                }
                if let Some(level) = master_edit {
                    self.model.set_master_level(level);
                    // The master gain has its own parameter target, so it publishes rather than
                    // rebuilding: a fader move must not restart the stream
                    if let Some(engine) = self.engine.as_mut() {
                        let _ = spectre_app::engine::publish_master_gain(
                            engine,
                            self.model.track_list(),
                        );
                    }
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
    // one would clip the note list at WINDOW_MIN_SIZE
    fn clip_inspector(&mut self, ui: &mut egui::Ui) {
        let mut republish = false;
        let mut note_action: Option<NoteAction> = None;
        let mut clip_move: Option<i64> = None;
        let mut clip_resize: Option<i64> = None;
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
        // Where the placement sits, read before the panel draws so the field shows it
        let start_ticks = self
            .model
            .clip_track(placement)
            .and_then(|track| self.model.track_list().get(track))
            .and_then(|entry| entry.clips().get(placement))
            .map_or(0, |entry| entry.start().0);
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
                            // Deactivating a clip changes what should be playing, so the render
                            // thread needs the rebaked material or it keeps playing the old
                            republish = true;
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
                // A list, not a piano roll: R4-5 §3.1 makes this a deliberate accessibility
                // choice under decision 17, not an aesthetic one, and it states the piano roll is
                // additive later over this same model. Editable here because a read-only list
                // meant nothing in the product could write a note at all
                egui::ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        egui::Grid::new("clip-note-list")
                            .num_columns(6)
                            .striped(true)
                            .show(ui, |ui| {
                                for header in ["start", "length", "pitch", "vel", "ch", ""] {
                                    ui.label(RichText::new(header).small().color(MUTED));
                                }
                                ui.end_row();
                                for (index, note) in notes.iter().enumerate() {
                                    ui.label(note.0.to_string());
                                    ui.label(note.1.to_string());
                                    ui.label(note.2.to_string());
                                    ui.label(format!("{:.2}", note.3));
                                    ui.label(note.4.to_string());
                                    if ui.small_button("✕").on_hover_text("Delete note").clicked()
                                    {
                                        note_action = Some(NoteAction::Remove(index));
                                    }
                                    ui.end_row();
                                }
                            });
                    });
                if notes.is_empty() {
                    ui.label(RichText::new("no notes in this clip").small().color(MUTED));
                }
                ui.add_space(6.0);
                // Where the clip sits and how long it is. Fields rather than a drag, matching the
                // list idiom: every clip was appended after the previous one and could never be
                // moved, so a musician could not leave a gap or place one at bar 5
                ui.horizontal(|ui| {
                    let mut start_bar = start_ticks / bar_ticks() + 1;
                    if ui
                        .add(
                            egui::DragValue::new(&mut start_bar)
                                .speed(0.25)
                                .range(1..=LOOP_MAX_BAR)
                                .prefix("start bar "),
                        )
                        .changed()
                    {
                        clip_move = Some((start_bar - 1) * bar_ticks());
                    }
                    let mut bars = (length.0 / bar_ticks()).max(1);
                    if ui
                        .add(
                            egui::DragValue::new(&mut bars)
                                .speed(0.25)
                                .range(1..=64)
                                .suffix(" bar(s)"),
                        )
                        .changed()
                    {
                        clip_resize = Some(bars * bar_ticks());
                    }
                });
                ui.add_space(6.0);
                // Writing a note. Fields rather than a canvas, matching the list above; the
                // model refuses anything outside the clip or outside MIDI's range
                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut self.new_note.0)
                            .speed(60.0)
                            .prefix("at "),
                    );
                    ui.add(
                        egui::DragValue::new(&mut self.new_note.1)
                            .speed(60.0)
                            .range(1..=i64::MAX)
                            .prefix("len "),
                    );
                    ui.add(
                        egui::DragValue::new(&mut self.new_note.2)
                            .range(0..=127)
                            .prefix("note "),
                    );
                    if ui.button("+ Note").clicked() {
                        note_action = Some(NoteAction::Add);
                    }
                });
            });
        // Both change what should be playing, so both republish
        if let Some(start) = clip_move {
            if let Some(placement) = self.model.selected_clip() {
                match self.model.move_clip(placement, BeatTicks(start)) {
                    Ok(()) => republish = true,
                    Err(error) => self.feedback_status = error.to_string(),
                }
            }
        }
        if let Some(length) = clip_resize {
            if let Some(clip) = self.selected_clip_id() {
                match self.model.set_clip_length(clip, BeatTicks(length)) {
                    Ok(()) => republish = true,
                    Err(error) => self.feedback_status = error.to_string(),
                }
            }
        }
        // Applied after the panel closes, and every one republishes: a note the musician wrote
        // that the render thread has not been told about is a note they do not hear
        if let Some(action) = note_action {
            if let Some(clip) = self.selected_clip_id() {
                let outcome = match action {
                    NoteAction::Remove(index) => self.model.remove_note(clip, index),
                    NoteAction::Add => {
                        let (start, length, pitch) = self.new_note;
                        spectre_project::ClipNote::new(
                            BeatTicks(start),
                            BeatTicks(length),
                            0,
                            pitch,
                            DEFAULT_NOTE_VELOCITY,
                        )
                        .and_then(|note| self.model.add_note(clip, note))
                    }
                };
                match outcome {
                    Ok(()) => republish = true,
                    Err(error) => self.feedback_status = error.to_string(),
                }
            }
        }
        if republish {
            self.republish_schedules();
        }
    }

    // The clip the selected placement refers to, which is what note edits address
    fn selected_clip_id(&self) -> Option<ObjectId> {
        let placement = self.model.selected_clip()?;
        let track = self.model.clip_track(placement)?;
        Some(
            self.model
                .track_list()
                .get(track)?
                .clips()
                .get(placement)?
                .clip(),
        )
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
        // Shortcuts are refused while a text field holds focus. Without this, typing a space
        // into the track-name or project-path field toggles the transport and typing a digit
        // switches lens: the keystroke reaches both the field and the shortcut. Focus is read
        // from memory, so it reflects the field that was focused when the key was pressed
        if !ctx.wants_keyboard_input() {
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
        }
        // Collect a finished render on the app thread; never blocks, so a long bounce does not
        // freeze the window
        self.bounce.poll();
        // While an offer is pending the model still holds the SAVED project, so autosaving now
        // would overwrite the sidecar with the very work the offer exists to protect
        if self.recovery_offer.is_none() {
            self.maybe_autosave();
        }
        self.transport(ctx);
        self.recovery_panel(ctx);
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
            .with_inner_size(WINDOW_DEFAULT_SIZE)
            .with_min_inner_size(WINDOW_MIN_SIZE),
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
