// Author: Jeff
// Date: 2026-06-15
// Description: Renderer-facing session model the views draw and mutate.
// Notes: This is the bridge surface between the engine/project and the UI. The
//        app populates it from lock-free snapshots (meters, transport) and applies
//        view edits back to project/audio truth. It is disposable mirror state: it
//        owns no audio truth and never touches the audio callback. A demo()
//        constructor seeds a populated session for examples and tests.

use crate::theme::SignalKind;
use crate::widgets::Taper;
use geist_config::commands::CommandIntent;

// Transport state mirrored from the audio thread
#[derive(Clone, Debug, PartialEq)]
pub struct Transport {
    pub playing: bool,
    pub bpm: f32,
    pub position_beats: f64,
    pub loop_enabled: bool,
    pub loop_start_beats: f64,
    pub loop_end_beats: f64,
}

impl Default for Transport {
    fn default() -> Self {
        Self {
            playing: false,
            bpm: 120.0,
            position_beats: 0.0,
            loop_enabled: false,
            loop_start_beats: 0.0,
            loop_end_beats: 16.0,
        }
    }
}

// One mixer channel: level/pan/state plus a metering snapshot
#[derive(Clone, Debug, PartialEq)]
pub struct ChannelStrip {
    pub name: String,
    pub level: f32,
    pub pan: f32,
    pub muted: bool,
    pub soloed: bool,
    pub armed: bool,
    pub peak: f32,
    pub rms: f32,
    pub inserts: Vec<String>,
    pub sends: Vec<Send>,
}

impl ChannelStrip {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            level: 0.8,
            pan: 0.0,
            muted: false,
            soloed: false,
            armed: false,
            peak: 0.0,
            rms: 0.0,
            inserts: Vec::new(),
            sends: Vec::new(),
        }
    }
}

// A post-fader send to a bus
#[derive(Clone, Debug, PartialEq)]
pub struct Send {
    pub target: String,
    pub amount: f32,
}

// The mixer: channel strips and which one is inspected
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MixerModel {
    pub channels: Vec<ChannelStrip>,
    pub selected: usize,
}

impl MixerModel {
    // True if any channel is soloed; mutes the non-soloed in the engine
    pub fn any_soloed(&self) -> bool {
        self.channels.iter().any(|c| c.soloed)
    }

    // Whether a channel would be audible given mute/solo state
    pub fn is_audible(&self, index: usize) -> bool {
        match self.channels.get(index) {
            None => false,
            Some(channel) => !channel.muted && (!self.any_soloed() || channel.soloed),
        }
    }
}

// One editable parameter of an effect
#[derive(Clone, Debug, PartialEq)]
pub struct ParamSpec {
    pub name: String,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub default: f32,
    pub unit: String,
    // How the knob maps its sweep onto [min, max]
    pub taper: Taper,
}

impl ParamSpec {
    pub fn new(name: impl Into<String>, value: f32, min: f32, max: f32) -> Self {
        Self {
            name: name.into(),
            value,
            min,
            max,
            default: value,
            unit: String::new(),
            taper: Taper::Linear,
        }
    }

    pub fn unit(mut self, unit: impl Into<String>) -> Self {
        self.unit = unit.into();
        self
    }

    // Map the knob logarithmically; suits frequency and time parameters
    pub fn taper(mut self, taper: Taper) -> Self {
        self.taper = taper;
        self
    }
}

// One effect in the visible, modifiable chain
#[derive(Clone, Debug, PartialEq)]
pub struct EffectSlot {
    pub name: String,
    pub bypassed: bool,
    pub wet: f32,
    pub params: Vec<ParamSpec>,
}

impl EffectSlot {
    pub fn new(name: impl Into<String>, params: Vec<ParamSpec>) -> Self {
        Self {
            name: name.into(),
            bypassed: false,
            wet: 1.0,
            params,
        }
    }
}

// The effects chain for the selected signal path
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RackModel {
    pub slots: Vec<EffectSlot>,
    pub selected: Option<usize>,
}

impl RackModel {
    // Move the slot at `from` to `to`, shifting the rest; out-of-range is a no-op
    pub fn reorder(&mut self, from: usize, to: usize) {
        if from >= self.slots.len() || to >= self.slots.len() || from == to {
            return;
        }
        let slot = self.slots.remove(from);
        self.slots.insert(to, slot);
    }

    // Toggle bypass on a slot
    pub fn toggle_bypass(&mut self, index: usize) {
        if let Some(slot) = self.slots.get_mut(index) {
            slot.bypassed = !slot.bypassed;
        }
    }

    // Remove a slot, keeping `selected` valid
    pub fn remove(&mut self, index: usize) {
        if index >= self.slots.len() {
            return;
        }
        self.slots.remove(index);
        self.selected = match self.selected {
            Some(sel) if sel == index => None,
            Some(sel) if sel > index => Some(sel - 1),
            other => other,
        };
    }

    pub fn push(&mut self, slot: EffectSlot) {
        self.slots.push(slot);
    }
}

// A graph port carrying one signal type
#[derive(Clone, Debug, PartialEq)]
pub struct Port {
    pub name: String,
    pub kind: SignalKind,
}

// A semantic node block in the graph
#[derive(Clone, Debug, PartialEq)]
pub struct GraphNode {
    pub id: u64,
    pub name: String,
    pub pos: (f32, f32),
    pub inputs: Vec<Port>,
    pub outputs: Vec<Port>,
}

// A connection between two node ports
#[derive(Clone, Debug, PartialEq)]
pub struct Cable {
    pub from_node: u64,
    pub from_port: usize,
    pub to_node: u64,
    pub to_port: usize,
    pub kind: SignalKind,
}

// The signal graph: nodes and the cables between them
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GraphModel {
    pub nodes: Vec<GraphNode>,
    pub cables: Vec<Cable>,
}

impl GraphModel {
    pub fn node(&self, id: u64) -> Option<&GraphNode> {
        self.nodes.iter().find(|n| n.id == id)
    }
}

// A single note event in a piano-roll pattern
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Note {
    pub pitch: u8,
    pub start_beats: f32,
    pub len_beats: f32,
    pub velocity: f32,
}

// A piano-roll pattern of notes over a number of beats
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PianoRollModel {
    pub notes: Vec<Note>,
    pub length_beats: f32,
}

impl PianoRollModel {
    pub fn add(&mut self, note: Note) {
        self.notes.push(note);
    }

    // Remove the first note covering (pitch, beat); true if one was removed
    pub fn remove_at(&mut self, pitch: u8, beat: f32) -> bool {
        if let Some(index) = self.notes.iter().position(|n| {
            n.pitch == pitch && beat >= n.start_beats && beat < n.start_beats + n.len_beats
        }) {
            self.notes.remove(index);
            true
        } else {
            false
        }
    }
}

// A clip placed on an arrangement lane
#[derive(Clone, Debug, PartialEq)]
pub struct Clip {
    pub lane: usize,
    pub name: String,
    pub start_beats: f32,
    pub len_beats: f32,
    pub kind: SignalKind,
}

// One arrangement lane (a track in time)
#[derive(Clone, Debug, PartialEq)]
pub struct Lane {
    pub name: String,
}

// The arrangement timeline: lanes and clips over beats
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TimelineModel {
    pub lanes: Vec<Lane>,
    pub clips: Vec<Clip>,
    pub length_beats: f32,
}

// A window of output samples for the oscilloscope
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScopeFrame {
    pub samples: Vec<f32>,
}

// Precomputed magnitude bins (0..1) for the spectrum analyzer
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpectrumFrame {
    pub bins: Vec<f32>,
}

// One browsable insertable item
#[derive(Clone, Debug, PartialEq)]
pub struct BrowserItem {
    pub name: String,
    pub category: String,
    pub kind: SignalKind,
    pub intent: CommandIntent,
}

impl BrowserItem {
    // Build a browser item whose default action inserts its visible name
    pub fn new(name: impl Into<String>, category: impl Into<String>, kind: SignalKind) -> Self {
        let name = name.into();
        Self {
            intent: CommandIntent::new(format!("insert:{name}")),
            name,
            category: category.into(),
            kind,
        }
    }

    // Override the action emitted when this item is double-clicked
    pub fn with_intent(mut self, intent: CommandIntent) -> Self {
        self.intent = intent;
        self
    }
}

// The search-first browser of instruments, effects, samples, and presets
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BrowserModel {
    pub items: Vec<BrowserItem>,
    pub query: String,
    pub selected: Option<usize>,
}

impl BrowserModel {
    // Indices of items matching the current query (case-insensitive substring)
    pub fn matches(&self) -> Vec<usize> {
        let q = self.query.trim().to_lowercase();
        self.items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                q.is_empty()
                    || item.name.to_lowercase().contains(&q)
                    || item.category.to_lowercase().contains(&q)
            })
            .map(|(index, _)| index)
            .collect()
    }
}

// One track's step pattern: a gate grid of `rows` notes by `steps` columns.
// Row-major; row 0 is the lowest note (drawn at the bottom). base_midi is the
// MIDI note of row 0, so row r plays base_midi + r.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepPattern {
    pub rows: usize,
    pub steps: usize,
    pub base_midi: u8,
    pub cells: Vec<bool>,
}

impl StepPattern {
    pub fn new(rows: usize, steps: usize, base_midi: u8) -> Self {
        Self { rows, steps, base_midi, cells: vec![false; rows * steps] }
    }

    // Gate state at (row, step); false for out-of-range indices
    pub fn cell(&self, row: usize, step: usize) -> bool {
        if row < self.rows && step < self.steps {
            self.cells[row * self.steps + step]
        } else {
            false
        }
    }

    // Set a gate; out-of-range indices are ignored
    pub fn set(&mut self, row: usize, step: usize, on: bool) {
        if row < self.rows && step < self.steps {
            self.cells[row * self.steps + step] = on;
        }
    }

    // Clear every gate in the pattern
    pub fn clear(&mut self) {
        self.cells.iter_mut().for_each(|c| *c = false);
    }

    // True when no gate is set
    pub fn is_empty(&self) -> bool {
        self.cells.iter().all(|&c| !c)
    }
}

// The step sequencer: one pattern per track and which one is shown
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StepSequencerModel {
    pub tracks: Vec<StepPattern>,
    pub selected: usize,
}

// Which editor the Arrange lens is focused on
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ArrangeTab {
    #[default]
    PianoRoll,
    StepSequencer,
    Timeline,
}

// The complete renderer-facing session: every view reads from one of these
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SessionModel {
    pub transport: Transport,
    pub mixer: MixerModel,
    pub rack: RackModel,
    pub graph: GraphModel,
    pub piano: PianoRollModel,
    pub step_seq: StepSequencerModel,
    pub arrange_tab: ArrangeTab,
    pub timeline: TimelineModel,
    pub scope: ScopeFrame,
    pub spectrum: SpectrumFrame,
    pub browser: BrowserModel,
}

impl SessionModel {
    // A populated session for examples and tests; not engine-backed
    pub fn demo() -> Self {
        let mut mixer = MixerModel::default();
        for name in ["Drums", "Bass", "Lead", "Pad", "Master"] {
            let mut strip = ChannelStrip::new(name);
            strip.inserts = vec!["EQ".into(), "Comp".into()];
            mixer.channels.push(strip);
        }
        mixer.selected = 2;

        let mut rack = RackModel::default();
        rack.push(EffectSlot::new(
            "Saturator",
            vec![ParamSpec::new("Drive", 0.4, 0.0, 1.0), ParamSpec::new("Tone", 0.6, 0.0, 1.0)],
        ));
        rack.push(EffectSlot::new(
            "Delay",
            vec![
                ParamSpec::new("Time", 0.25, 0.0, 1.0).unit("s"),
                ParamSpec::new("Feedback", 0.35, 0.0, 0.95),
            ],
        ));
        rack.push(EffectSlot::new(
            "Reverb",
            vec![ParamSpec::new("Mix", 0.3, 0.0, 1.0), ParamSpec::new("Size", 0.7, 0.0, 1.0)],
        ));
        rack.selected = Some(0);

        let graph = GraphModel {
            nodes: vec![
                GraphNode {
                    id: 1,
                    name: "Wavetable".into(),
                    pos: (40.0, 60.0),
                    inputs: vec![Port { name: "Pitch".into(), kind: SignalKind::Note }],
                    outputs: vec![Port { name: "Out".into(), kind: SignalKind::Audio }],
                },
                GraphNode {
                    id: 2,
                    name: "Filter".into(),
                    pos: (260.0, 60.0),
                    inputs: vec![
                        Port { name: "In".into(), kind: SignalKind::Audio },
                        Port { name: "Cutoff".into(), kind: SignalKind::Cv },
                    ],
                    outputs: vec![Port { name: "Out".into(), kind: SignalKind::Audio }],
                },
                GraphNode {
                    id: 3,
                    name: "Output".into(),
                    pos: (480.0, 60.0),
                    inputs: vec![Port { name: "In".into(), kind: SignalKind::Audio }],
                    outputs: vec![],
                },
            ],
            cables: vec![
                Cable { from_node: 1, from_port: 0, to_node: 2, to_port: 0, kind: SignalKind::Audio },
                Cable { from_node: 2, from_port: 0, to_node: 3, to_port: 0, kind: SignalKind::Audio },
            ],
        };

        let mut piano = PianoRollModel { notes: Vec::new(), length_beats: 16.0 };
        for (i, pitch) in [60u8, 64, 67, 72].iter().enumerate() {
            piano.add(Note {
                pitch: *pitch,
                start_beats: i as f32 * 2.0,
                len_beats: 1.5,
                velocity: 0.9,
            });
        }

        let timeline = TimelineModel {
            lanes: vec![
                Lane { name: "Drums".into() },
                Lane { name: "Bass".into() },
                Lane { name: "Lead".into() },
            ],
            clips: vec![
                Clip { lane: 0, name: "Beat".into(), start_beats: 0.0, len_beats: 8.0, kind: SignalKind::Audio },
                Clip { lane: 1, name: "Bassline".into(), start_beats: 0.0, len_beats: 16.0, kind: SignalKind::Note },
                Clip { lane: 2, name: "Hook".into(), start_beats: 4.0, len_beats: 8.0, kind: SignalKind::Note },
            ],
            length_beats: 32.0,
        };

        // A two-track step sequencer; track 0 carries a four-on-the-floor kick
        let mut kick = StepPattern::new(13, 16, 36);
        for step in (0..16).step_by(4) {
            kick.set(0, step, true);
        }
        let mut lead = StepPattern::new(13, 16, 60);
        for (i, step) in [0usize, 3, 6, 10, 12].into_iter().enumerate() {
            lead.set(i + 2, step, true);
        }
        let step_seq = StepSequencerModel { tracks: vec![kick, lead], selected: 0 };

        let browser = BrowserModel {
            items: vec![
                BrowserItem::new("Wavetable Synth", "Instrument", SignalKind::Note),
                BrowserItem::new("Sampler", "Instrument", SignalKind::Note),
                BrowserItem::new("Reverb", "Effect", SignalKind::Audio),
                BrowserItem::new("Delay", "Effect", SignalKind::Audio),
                BrowserItem::new("Saturator", "Effect", SignalKind::Audio),
                BrowserItem::new("LFO", "Modulator", SignalKind::Cv),
                BrowserItem::new("Kick.wav", "Sample", SignalKind::Audio),
                BrowserItem::new("Warm Pad", "Preset", SignalKind::Note),
            ],
            query: String::new(),
            selected: None,
        };

        Self {
            transport: Transport::default(),
            mixer,
            rack,
            graph,
            piano,
            step_seq,
            arrange_tab: ArrangeTab::default(),
            timeline,
            scope: ScopeFrame::default(),
            spectrum: SpectrumFrame::default(),
            browser,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solo_gates_audibility() {
        let mut mixer = MixerModel {
            channels: vec![
                ChannelStrip::new("A"),
                ChannelStrip::new("B"),
                ChannelStrip::new("C"),
            ],
            selected: 0,
        };
        assert!(mixer.is_audible(0));
        mixer.channels[1].soloed = true;
        assert!(!mixer.is_audible(0));
        assert!(mixer.is_audible(1));
        mixer.channels[0].muted = true;
        assert!(!mixer.is_audible(0));
    }

    #[test]
    fn rack_reorder_moves_slot() {
        let mut rack = RackModel::default();
        rack.push(EffectSlot::new("A", vec![]));
        rack.push(EffectSlot::new("B", vec![]));
        rack.push(EffectSlot::new("C", vec![]));
        rack.reorder(0, 2);
        let names: Vec<_> = rack.slots.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, vec!["B", "C", "A"]);
        // Out-of-range reorder is a no-op
        rack.reorder(0, 9);
        assert_eq!(rack.slots[0].name, "B");
    }

    #[test]
    fn rack_remove_keeps_selection_valid() {
        let mut rack = RackModel::default();
        rack.push(EffectSlot::new("A", vec![]));
        rack.push(EffectSlot::new("B", vec![]));
        rack.push(EffectSlot::new("C", vec![]));
        rack.selected = Some(2);
        rack.remove(0);
        assert_eq!(rack.selected, Some(1));
        rack.selected = Some(0);
        rack.remove(0);
        assert_eq!(rack.selected, None);
    }

    #[test]
    fn piano_remove_finds_covering_note() {
        let mut roll = PianoRollModel { notes: vec![], length_beats: 8.0 };
        roll.add(Note { pitch: 60, start_beats: 1.0, len_beats: 2.0, velocity: 1.0 });
        assert!(!roll.remove_at(60, 0.5));
        assert!(roll.remove_at(60, 2.0));
        assert!(roll.notes.is_empty());
    }

    #[test]
    fn step_pattern_sets_clears_and_reports_empty() {
        let mut pat = StepPattern::new(13, 16, 36);
        assert!(pat.is_empty());
        pat.set(0, 4, true);
        assert!(pat.cell(0, 4));
        assert!(!pat.is_empty());
        // Out-of-range access is safe
        assert!(!pat.cell(99, 99));
        pat.set(99, 99, true);
        pat.clear();
        assert!(pat.is_empty());
    }

    #[test]
    fn browser_search_is_case_insensitive_substring() {
        let model = BrowserModel {
            items: vec![
                BrowserItem::new("Reverb", "Effect", SignalKind::Audio),
                BrowserItem::new("Sampler", "Instrument", SignalKind::Note),
            ],
            query: "rev".into(),
            selected: None,
        };
        assert_eq!(model.matches(), vec![0]);
        let all = BrowserModel { query: String::new(), ..model.clone() };
        assert_eq!(all.matches().len(), 2);
        let by_category = BrowserModel { query: "instr".into(), ..model };
        assert_eq!(by_category.matches(), vec![1]);
    }
}
