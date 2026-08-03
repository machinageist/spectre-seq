// =============================================================================
// File: app/spectre-seq/src/gui.rs
// Layer: application binary
// Purpose: egui front-end: a playable keyboard, transport, and output meter
// Status: Implemented; minimal playable synth window over the control plane.
// Notes: The UI owns no audio truth. It reads the meter and sends EngineCommands;
//        the audio thread is the single source of sound. Key press/release is
//        edge-detected each frame for both the on-screen keys and the computer
//        keyboard, so notes start and stop cleanly.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use eframe::egui;
use spectre_ui::theme::{self, SpectreTheme};
use spectre_ui::widgets::{Fader, Knob, Meter};

use crate::control::{EngineCommand, EngineControl};
use crate::engine::{default_grid_for, empty_grid, Engine, Grid, NUM_TRACKS, SEQ_ROWS, SEQ_STEPS};
use crate::graph_view::GraphView;
use crate::project::{self, PatchState};

// Which central view is showing
#[derive(Clone, Copy, PartialEq, Eq)]
enum View {
    Instrument,
    Sequencer,
    Mixer,
    Graph,
}

// On-screen keyboard spans two octaves starting at C3
const KEYBOARD_BASE_MIDI: u8 = 48;
const KEYBOARD_KEYS: usize = 25;
// Velocity for notes played from the UI
const UI_VELOCITY: f32 = 0.9;

// Filter/master macro defaults, matching the synth's startup patch
const DEFAULT_CUTOFF_HZ: f32 = 1_500.0;
const DEFAULT_RESONANCE: f32 = 0.9;
const DEFAULT_GAIN: f32 = 1.0;
// Reverb mix default, matching the reverb node's startup value
const DEFAULT_REVERB_MIX: f32 = 0.3;
// Oscillator unison defaults: single voice, no detune
const DEFAULT_UNISON_VOICES: usize = 1;
const DEFAULT_DETUNE_CENTS: f32 = 0.0;
const MAX_UNISON_VOICES: usize = 7;
// Transport tempo default
const DEFAULT_BPM: f32 = 120.0;
// Per-track mixer level default, matching the engine
const DEFAULT_TRACK_LEVEL: f32 = 0.8;

// Computer-keyboard to MIDI map: one octave from C4, tracker-style
const COMPUTER_KEYS: [(egui::Key, u8); 13] = [
    (egui::Key::Z, 60),
    (egui::Key::S, 61),
    (egui::Key::X, 62),
    (egui::Key::D, 63),
    (egui::Key::C, 64),
    (egui::Key::V, 65),
    (egui::Key::G, 66),
    (egui::Key::B, 67),
    (egui::Key::H, 68),
    (egui::Key::N, 69),
    (egui::Key::J, 70),
    (egui::Key::M, 71),
    (egui::Key::Comma, 72),
];

// Playable synth window driving the audio engine over the control plane
pub struct SpectreApp {
    // Held so the audio stream stays open for the window's lifetime
    _engine: Engine,
    control: EngineControl,
    sample_rate_hz: u32,
    channels: u16,
    // Transport play state and tempo shown by the transport bar
    playing: bool,
    bpm: f32,
    // Filter macro base cutoff/resonance and master gain shown by the knobs
    cutoff_hz: f32,
    resonance: f32,
    gain: f32,
    // Oscillator unison voices/detune shown by the osc controls
    unison_voices: usize,
    detune_cents: f32,
    // Effects chain state shown by the FX controls
    delay_on: bool,
    reverb_on: bool,
    reverb_mix: f32,
    // Held state of each on-screen key, for edge detection
    key_held: [bool; KEYBOARD_KEYS],
    // Held state of each computer-keyboard note, for edge detection
    computer_held: [bool; COMPUTER_KEYS.len()],
    // Active central view and the node-graph state
    view: View,
    graph: GraphView,
    // Track currently played by the keyboard and edited in the sequencer
    selected_track: usize,
    // Per-track step patterns mirrored from the audio sequencers
    grids: [Grid; NUM_TRACKS],
    // Per-track mixer state
    track_level: [f32; NUM_TRACKS],
    track_muted: [bool; NUM_TRACKS],
    track_soloed: [bool; NUM_TRACKS],
    // Last save/load result shown under the transport bar
    status: String,
    // Whether the tactile-dark theme has been installed on the egui context
    themed: bool,
}

impl SpectreApp {
    // Wrap a running engine and its control handle in a window
    pub fn new(engine: Engine, control: EngineControl) -> Self {
        let sample_rate_hz = engine.sample_rate_hz();
        let channels = engine.channels();
        Self {
            _engine: engine,
            control,
            sample_rate_hz,
            channels,
            playing: false,
            bpm: DEFAULT_BPM,
            cutoff_hz: DEFAULT_CUTOFF_HZ,
            resonance: DEFAULT_RESONANCE,
            gain: DEFAULT_GAIN,
            unison_voices: DEFAULT_UNISON_VOICES,
            detune_cents: DEFAULT_DETUNE_CENTS,
            delay_on: false,
            reverb_on: false,
            reverb_mix: DEFAULT_REVERB_MIX,
            key_held: [false; KEYBOARD_KEYS],
            computer_held: [false; COMPUTER_KEYS.len()],
            view: View::Instrument,
            graph: GraphView::new(),
            selected_track: 1,
            grids: std::array::from_fn(default_grid_for),
            track_level: [DEFAULT_TRACK_LEVEL; NUM_TRACKS],
            track_muted: [false; NUM_TRACKS],
            track_soloed: [false; NUM_TRACKS],
            status: String::new(),
            themed: false,
        }
    }

    // Snapshot the current parameters for saving
    fn current_patch(&self) -> PatchState {
        PatchState {
            bpm: self.bpm,
            cutoff_hz: self.cutoff_hz,
            resonance: self.resonance,
            gain: self.gain,
            unison_voices: self.unison_voices,
            detune_cents: self.detune_cents,
            delay_on: self.delay_on,
            reverb_on: self.reverb_on,
            reverb_mix: self.reverb_mix,
        }
    }

    // Apply a loaded patch to the UI and push every value to the engine
    fn apply_patch(&mut self, patch: PatchState) {
        self.bpm = patch.bpm;
        self.cutoff_hz = patch.cutoff_hz;
        self.resonance = patch.resonance;
        self.gain = patch.gain;
        self.unison_voices = patch.unison_voices;
        self.detune_cents = patch.detune_cents;
        self.delay_on = patch.delay_on;
        self.reverb_on = patch.reverb_on;
        self.reverb_mix = patch.reverb_mix;

        self.control.send(EngineCommand::SetBpm(patch.bpm));
        self.control.send(EngineCommand::SetGain(patch.gain));
        // The classic GUI runs one global patch; broadcast it to every track
        self.broadcast(|track| EngineCommand::SetCutoff {
            track,
            hz: patch.cutoff_hz,
        });
        self.broadcast(|track| EngineCommand::SetResonance {
            track,
            resonance: patch.resonance,
        });
        self.broadcast(|track| EngineCommand::SetUnisonVoices {
            track,
            voices: patch.unison_voices,
        });
        self.broadcast(|track| EngineCommand::SetDetune {
            track,
            cents: patch.detune_cents,
        });
        self.broadcast(|track| EngineCommand::SetDelay {
            track,
            on: patch.delay_on,
        });
        self.broadcast(|track| EngineCommand::SetReverb {
            track,
            on: patch.reverb_on,
        });
        self.broadcast(|track| EngineCommand::SetReverbMix {
            track,
            mix: patch.reverb_mix,
        });
    }

    // Send a per-track command to every track; the classic GUI runs one patch
    fn broadcast(&mut self, make: impl Fn(u8) -> EngineCommand) {
        for track in 0..NUM_TRACKS as u8 {
            self.control.send(make(track));
        }
    }

    // Emit note on/off on the selected track as a key's held state changes
    fn edge(&mut self, midi: u8, now_down: bool, held: &mut bool) {
        let track = self.selected_track as u8;
        if now_down && !*held {
            self.control.send(EngineCommand::NoteOn {
                track,
                key: midi,
                velocity: UI_VELOCITY,
            });
        } else if !now_down && *held {
            self.control
                .send(EngineCommand::NoteOff { track, key: midi });
        }
        *held = now_down;
    }

    // Poll the computer keyboard and play mapped notes
    fn handle_computer_keys(&mut self, ctx: &egui::Context) {
        let down = ctx.input(|i| COMPUTER_KEYS.map(|(key, _)| i.keys_down.contains(&key)));
        for (index, (_, midi)) in COMPUTER_KEYS.iter().enumerate() {
            let mut held = self.computer_held[index];
            self.edge(*midi, down[index], &mut held);
            self.computer_held[index] = held;
        }
    }

    // Draw the transport row: demo toggle, panic, and stream info
    fn transport_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Geist DAW");
            ui.separator();
            let transport_label = if self.playing { "⏸ Stop" } else { "▶ Play" };
            if ui.button(transport_label).clicked() {
                self.playing = !self.playing;
                self.control.send(EngineCommand::SetPlaying(self.playing));
            }
            let bpm = egui::DragValue::new(&mut self.bpm)
                .range(40.0..=300.0)
                .speed(0.5)
                .suffix(" BPM");
            if ui.add(bpm).changed() {
                self.control.send(EngineCommand::SetBpm(self.bpm));
            }
            if ui.button("All Notes Off").clicked() {
                self.control.send(EngineCommand::AllNotesOff);
                self.key_held = [false; KEYBOARD_KEYS];
                self.computer_held = [false; COMPUTER_KEYS.len()];
            }
            ui.separator();
            if ui.button("Save").clicked() {
                self.status = match project::save(&self.current_patch()) {
                    Ok(path) => format!("Saved {}", path.display()),
                    Err(err) => format!("Save failed: {err}"),
                };
            }
            if ui.button("Load").clicked() {
                match project::load(&self.current_patch()) {
                    Ok(patch) => {
                        self.apply_patch(patch);
                        self.status = "Loaded session".to_string();
                    }
                    Err(err) => self.status = format!("Load failed: {err}"),
                }
            }
            ui.separator();
            ui.selectable_value(&mut self.view, View::Instrument, "Instrument");
            ui.selectable_value(&mut self.view, View::Sequencer, "Sequencer");
            ui.selectable_value(&mut self.view, View::Mixer, "Mixer");
            ui.selectable_value(&mut self.view, View::Graph, "Graph");
            ui.separator();
            ui.label(format!("{} Hz · {} ch", self.sample_rate_hz, self.channels));
        });
        if !self.status.is_empty() {
            ui.label(
                egui::RichText::new(&self.status)
                    .small()
                    .color(egui::Color32::from_gray(150)),
            );
        }
    }

    // Draw the master output meter from the latest published peak
    fn meter(&self, ui: &mut egui::Ui) {
        let level = self.control.level().clamp(0.0, 1.0);
        ui.horizontal(|ui| {
            ui.label("Output");
            Meter::new(level)
                .peak(level)
                .size(egui::vec2(14.0, 54.0))
                .show(ui);
            ui.label(format!("{level:.2}"));
        });
    }

    // Draw the per-track mixer as vertical fader strips
    fn mixer(&mut self, ui: &mut egui::Ui) {
        ui.label("Mixer");
        ui.add_space(4.0);
        ui.horizontal_top(|ui| {
            for track in 0..NUM_TRACKS {
                ui.group(|ui| {
                    ui.set_width(72.0);
                    ui.vertical_centered(|ui| {
                        if ui
                            .selectable_label(
                                self.selected_track == track,
                                format!("Track {}", track + 1),
                            )
                            .clicked()
                        {
                            self.selected_track = track;
                        }
                        if Fader::new(&mut self.track_level[track], 0.0..=1.5)
                            .default(DEFAULT_TRACK_LEVEL)
                            .size(egui::vec2(34.0, 150.0))
                            .show(ui)
                            .changed()
                        {
                            self.control.send(EngineCommand::SetTrackLevel {
                                track: track as u8,
                                level: self.track_level[track],
                            });
                        }
                        ui.horizontal(|ui| {
                            if ui.toggle_value(&mut self.track_muted[track], "M").changed() {
                                self.control.send(EngineCommand::SetTrackMute {
                                    track: track as u8,
                                    on: self.track_muted[track],
                                });
                            }
                            if ui
                                .toggle_value(&mut self.track_soloed[track], "S")
                                .changed()
                            {
                                self.control.send(EngineCommand::SetTrackSolo {
                                    track: track as u8,
                                    on: self.track_soloed[track],
                                });
                            }
                        });
                    });
                });
            }
        });
    }

    // Draw the clickable step-sequencer grid for the selected track
    fn sequencer_grid(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Track:");
            for track in 0..NUM_TRACKS {
                ui.selectable_value(&mut self.selected_track, track, format!("{}", track + 1));
            }
            if ui.button("Clear").clicked() {
                self.grids[self.selected_track] = empty_grid();
                self.control.send(EngineCommand::ClearPattern {
                    track: self.selected_track as u8,
                });
            }
        });
        ui.add_space(4.0);

        let track = self.selected_track;
        let cell = 22.0;
        let gap = 2.0;
        let stride = cell + gap;
        let (area, _response) = ui.allocate_exact_size(
            egui::vec2(SEQ_STEPS as f32 * stride, SEQ_ROWS as f32 * stride),
            egui::Sense::hover(),
        );
        let painter = ui.painter_at(area);

        for row in 0..SEQ_ROWS {
            // Row 0 is the lowest note, drawn at the bottom
            let display_row = SEQ_ROWS - 1 - row;
            for step in 0..SEQ_STEPS {
                let min = egui::pos2(
                    area.left() + step as f32 * stride,
                    area.top() + display_row as f32 * stride,
                );
                let cell_rect = egui::Rect::from_min_size(min, egui::vec2(cell, cell));
                let id = ui.id().with(("seq_cell", row, step));
                let response = ui.interact(cell_rect, id, egui::Sense::click());
                if response.clicked() {
                    let on = !self.grids[track][row][step];
                    self.grids[track][row][step] = on;
                    self.control.send(EngineCommand::SetCell {
                        track: track as u8,
                        step: step as u8,
                        row: row as u8,
                        on,
                    });
                }
                let color = if self.grids[track][row][step] {
                    theme::ACCENT
                } else if step % 4 == 0 {
                    theme::PANEL_RAISED
                } else {
                    theme::INSET
                };
                painter.rect_filled(cell_rect, 2.0, color);
            }
        }
    }

    // Draw the live output oscilloscope from the latest scope window
    fn scope(&mut self, ui: &mut egui::Ui) {
        self.control.update_scope();
        let (rect, _response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 120.0),
            egui::Sense::hover(),
        );
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 3.0, theme::INSET);
        // Zero line
        painter.hline(
            rect.x_range(),
            rect.center().y,
            egui::Stroke::new(1.0, theme::STROKE),
        );

        let samples = self.control.scope_view();
        if samples.len() >= 2 {
            let mid = rect.center().y;
            let amp = rect.height() * 0.45;
            let last = (samples.len() - 1) as f32;
            let points: Vec<egui::Pos2> = samples
                .iter()
                .enumerate()
                .map(|(i, &s)| {
                    let x = rect.left() + rect.width() * (i as f32 / last);
                    let y = mid - s.clamp(-1.0, 1.0) * amp;
                    egui::pos2(x, y)
                })
                .collect();
            painter.add(egui::Shape::line(
                points,
                egui::Stroke::new(1.0, theme::AUDIO),
            ));
        }
    }

    // Draw the oscillator unison macros and emit commands as they change
    fn osc_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let voices = egui::Slider::new(&mut self.unison_voices, 1..=MAX_UNISON_VOICES)
                .integer()
                .text("Unison");
            if ui.add(voices).changed() {
                let voices = self.unison_voices;
                self.broadcast(|track| EngineCommand::SetUnisonVoices { track, voices });
            }
            if Knob::new(&mut self.detune_cents, 0.0..=50.0)
                .label("Detune")
                .unit("¢")
                .default(DEFAULT_DETUNE_CENTS)
                .show(ui)
                .changed()
            {
                let cents = self.detune_cents;
                self.broadcast(|track| EngineCommand::SetDetune { track, cents });
            }
        });
    }

    // Draw the filter + master macros and emit commands as they change
    fn synth_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if Knob::new(&mut self.cutoff_hz, 20.0..=18_000.0)
                .label("Cutoff")
                .unit("Hz")
                .default(DEFAULT_CUTOFF_HZ)
                .arc_color(theme::AUDIO)
                .show(ui)
                .changed()
            {
                let hz = self.cutoff_hz;
                self.broadcast(|track| EngineCommand::SetCutoff { track, hz });
            }
            if Knob::new(&mut self.resonance, 0.5..=6.0)
                .label("Reso")
                .default(DEFAULT_RESONANCE)
                .show(ui)
                .changed()
            {
                let resonance = self.resonance;
                self.broadcast(|track| EngineCommand::SetResonance { track, resonance });
            }
            if Knob::new(&mut self.gain, 0.0..=1.5)
                .label("Master")
                .default(DEFAULT_GAIN)
                .show(ui)
                .changed()
            {
                self.control.send(EngineCommand::SetGain(self.gain));
            }
        });
    }

    // Draw the effects-chain controls and emit commands as they change
    fn fx_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("FX:");
            if ui.toggle_value(&mut self.delay_on, "Delay").changed() {
                let on = self.delay_on;
                self.broadcast(|track| EngineCommand::SetDelay { track, on });
            }
            if ui.toggle_value(&mut self.reverb_on, "Reverb").changed() {
                let on = self.reverb_on;
                self.broadcast(|track| EngineCommand::SetReverb { track, on });
            }
            if Knob::new(&mut self.reverb_mix, 0.0..=1.0)
                .label("Verb Mix")
                .default(DEFAULT_REVERB_MIX)
                .arc_color(theme::ACCENT)
                .show(ui)
                .changed()
            {
                let mix = self.reverb_mix;
                self.broadcast(|track| EngineCommand::SetReverbMix { track, mix });
            }
        });
    }

    // Draw the clickable keyboard and emit notes for pressed keys
    fn keyboard(&mut self, ui: &mut egui::Ui) {
        ui.label("Click keys or play Z S X D C V G B H N J M ,");
        ui.horizontal_wrapped(|ui| {
            for index in 0..KEYBOARD_KEYS {
                let midi = KEYBOARD_BASE_MIDI + index as u8;
                let button = egui::Button::new(note_name(midi)).min_size(egui::vec2(34.0, 56.0));
                let response = ui.add(button);
                let now_down = response.is_pointer_button_down_on();
                let mut held = self.key_held[index];
                self.edge(midi, now_down, &mut held);
                self.key_held[index] = held;
            }
        });
    }
}

impl eframe::App for SpectreApp {
    // eframe 0.34 hands a root Ui; wrap content in panels via show_inside
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Clone the context (Arc-cheap) so the Ui stays free to borrow mutably
        let ctx = ui.ctx().clone();
        // Install the tactile-dark theme once, on the first frame
        if !self.themed {
            SpectreTheme::apply(&ctx);
            self.themed = true;
        }
        self.handle_computer_keys(&ctx);

        egui::Panel::top("transport").show_inside(ui, |ui| {
            self.transport_bar(ui);
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.add_space(8.0);
            self.scope(ui);
            ui.add_space(8.0);
            self.meter(ui);
            ui.add_space(12.0);
            match self.view {
                View::Instrument => {
                    self.osc_controls(ui);
                    ui.add_space(6.0);
                    self.synth_controls(ui);
                    ui.add_space(6.0);
                    self.fx_controls(ui);
                }
                View::Sequencer => self.sequencer_grid(ui),
                View::Mixer => self.mixer(ui),
                View::Graph => {
                    self.graph.show(
                        ui,
                        &mut self.control,
                        &mut self.delay_on,
                        &mut self.reverb_on,
                    );
                }
            }
            ui.add_space(16.0);
            self.keyboard(ui);
        });

        // Animate the meter and keep key polling live
        ctx.request_repaint();
    }
}

// MIDI note number to a short name like "C4" or "F#3"
fn note_name(midi: u8) -> String {
    const NAMES: [&str; 12] = [
        "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
    ];
    let octave = midi as i32 / 12 - 1;
    format!("{}{}", NAMES[(midi % 12) as usize], octave)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_names_are_conventional() {
        assert_eq!(note_name(60), "C4");
        assert_eq!(note_name(69), "A4");
        assert_eq!(note_name(61), "C#4");
    }

    #[test]
    fn keyboard_spans_two_octaves_from_c3() {
        assert_eq!(note_name(KEYBOARD_BASE_MIDI), "C3");
        assert_eq!(
            note_name(KEYBOARD_BASE_MIDI + KEYBOARD_KEYS as u8 - 1),
            "C5"
        );
    }
}
