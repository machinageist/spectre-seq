// =============================================================================
// File: crates/geist-stacksynth/src/schema.rs
// Layer: internal synth device (generator-stack synth)
// Purpose: Patch schema: groups, modules, routes, lanes, modulators, macros
// Status: S0 implemented; spec section refs in comments point at
//         docs/specs/geist-modular-synth-spec.md.
// Notes: Plain serde DTOs with stable ids and explicit ordering arrays.
//        Numeric ranges are Geist decisions unless the spec marks them PP.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use serde::{Deserialize, Serialize};

// Compatibility limits, enforced by the validator (spec §15.2)
pub const MAX_GENERATOR_MODULES: usize = 32;
pub const MAX_MODULATORS: usize = 32;
pub const LANE_COUNT: usize = 3;
pub const MACRO_COUNT: usize = 8;
pub const MAX_AUTOMATION_SLOTS: usize = 64;

// Current schema version for migration
pub const SCHEMA_VERSION: u32 = 1;

// Stable id for a generator-area module
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModuleId(pub u32);

// Stable id for a generator group
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GroupId(pub u32);

// Stable id for a modulator slot
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ModulatorId(pub u32);

// Reference to an external asset (sample, wavetable, shape) by library id/path
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetRef(pub String);

// Common parameter block shared by every sound source (spec §2.3)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommonGenParams {
    // Amplitude 0.0..=2.0, displayed 0-200%
    pub level: f32,
    // Offset from played note in semitones, cents folded into the fraction
    pub pitch_semis: f32,
    // Frequency multiplier; 0.0 disables keytracking
    pub harmonic: f32,
    // Fixed linear offset in Hz, signed, never clamped at zero
    pub shift_hz: f32,
    // Start phase offset in degrees
    pub phase_offset_deg: f32,
    // Per-note random phase range in degrees
    pub phase_random_deg: f32,
}

impl Default for CommonGenParams {
    // Neutral source: unity level, keytracked, zero offsets
    fn default() -> Self {
        Self {
            level: 1.0,
            pitch_semis: 0.0,
            harmonic: 1.0,
            shift_hz: 0.0,
            phase_offset_deg: 0.0,
            phase_random_deg: 0.0,
        }
    }
}

// Analog oscillator waveform (spec §3.1)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalogWaveform {
    Sawtooth,
    Pulse,
    Triangle,
    Sine,
}

// Noise flavor (spec §3.2)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NoiseType {
    Colored,
    SteppedKeytracked,
    SmoothKeytracked,
}

// Per-note noise sequence behavior (spec §3.2)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeedMode {
    Stable,
    Random,
}

// Sample loop behavior (spec §3.3)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopMode {
    Off,
    Infinite,
    Sustain,
    PingPong,
    Reverse,
}

// Grain spawn scheduling (spec §3.4)
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum GrainSpawn {
    // Fixed spawn rate in Hz
    FreeHz(f32),
    // Tempo-synced note length in beats
    SyncedBeats(f32),
    // Auto rate targeting a concurrent grain count
    Density(f32),
}

// In-stack distortion transfer family (spec §4.1)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistortionType {
    Overdrive,
    Saturate,
    Foldback,
    Sine,
    HardClip,
    Quantize,
}

// In-stack filter response (spec §4.2)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterType {
    LowPass,
    BandPass,
    HighPass,
    Notch,
    LowShelf,
    Peak,
    HighShelf,
}

// Nonlinear filter response; allpass is generator-area only (spec §4.3)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NlFilterType {
    LowPass,
    BandPass,
    HighPass,
    Notch,
    AllPass,
}

// Filter steepness (spec §4.2)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterSlope {
    TwoPole,
    FourPole,
}

// Output bus destination (spec §5.1)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SendDest {
    // Effect lane index 0..LANE_COUNT
    Lane(u8),
    Master,
    Sideband,
}

// Shared output-module tail: gain/pan/enable/destination (spec §5.1)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OutputCommon {
    pub gain: f32,
    pub pan: f32,
    // Off mutes the bus send but keeps the module available as a mod source
    pub enabled: bool,
    pub send_to: SendDest,
}

impl Default for OutputCommon {
    // Default routes to lane 1 enabled, matching public behavior (spec §5.1)
    fn default() -> Self {
        Self {
            gain: 1.0,
            pan: 0.0,
            enabled: true,
            send_to: SendDest::Lane(0),
        }
    }
}

// ADSR data for the envelope output; DSP lands in S4
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdsrParams {
    pub attack_s: f32,
    pub decay_s: f32,
    pub sustain: f32,
    pub release_s: f32,
}

impl Default for AdsrParams {
    // Musical defaults mirroring geist-synth's voice envelope
    fn default() -> Self {
        Self {
            attack_s: 0.005,
            decay_s: 0.1,
            sustain: 0.8,
            release_s: 0.3,
        }
    }
}

// Oscillator-unison settings (spec §9.2-9.3); per-mode extras in UnisonMode
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UnisonSettings {
    pub mode: UnisonMode,
    pub voices: u8,
    pub detune: f32,
    pub spread: f32,
    pub blend: f32,
}

impl Default for UnisonSettings {
    // Single voice = unison off
    fn default() -> Self {
        Self {
            mode: UnisonMode::Hard { bias: 0.0 },
            voices: 1,
            detune: 0.0,
            spread: 0.0,
            blend: 1.0,
        }
    }
}

// Unison mode; the fifth knob differs per mode (spec §9.3)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum UnisonMode {
    Hard { bias: f32 },
    Smooth { bias: f32 },
    Synthetic { bias: f32 },
    FrequencyStack { range: f32 },
    PitchStack { range: f32 },
    Shepard { center: f32 },
    Chord { chord: u8, balance: f32 },
}

// One generator-area module: id + kind-specific parameters
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub id: ModuleId,
    pub kind: ModuleKind,
}

// Every generator-area module family (spec §2.4)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ModuleKind {
    AnalogOsc {
        common: CommonGenParams,
        waveform: AnalogWaveform,
        // Sync ratio >= 1.0; 1.0 = off (spec §3.1)
        sync: f32,
        // Pulse width 0..1, pulse waveform only
        pulse_width: f32,
        unison: UnisonSettings,
    },
    NoiseGen {
        common: CommonGenParams,
        noise_type: NoiseType,
        // Spectral falloff in dB/octave: 0 white, -3 pink, -6 brown (spec §3.2)
        slope_db_oct: f32,
        // 0 mono .. 1 full stereo
        stereo: f32,
        seed: SeedMode,
    },
    SamplePlayer {
        common: CommonGenParams,
        sample: AssetRef,
        // Root note as MIDI note number with fraction
        root_note: f32,
        // Playback start position, normalized 0..1
        offset: f32,
        loop_mode: LoopMode,
        loop_start: f32,
        loop_length: f32,
        // Crossfade region length, normalized against loop length
        xfade: f32,
        unison: UnisonSettings,
    },
    Granular {
        common: CommonGenParams,
        sample: AssetRef,
        root_note: f32,
        // Grain spawn position, normalized 0..1
        cursor: f32,
        grain_length_ms: f32,
        // Scales grain length by played pitch (spec §3.4)
        keytrack_length: bool,
        spawn: GrainSpawn,
        env_attack: f32,
        env_decay: f32,
        env_curve: f32,
        random_position: f32,
        random_timing: f32,
        random_pitch: f32,
        random_level: f32,
        random_pan: f32,
        random_reverse: f32,
        align_phases: bool,
        warm_start: bool,
        // Chord table index + octave range + picking pattern; Geist-original list
        chord: u8,
        chord_range: u8,
        chord_pattern: u8,
    },
    WavetableOsc {
        common: CommonGenParams,
        table: AssetRef,
        // Frame position 0..255 continuous
        frame: f32,
        // Pre-phase-mod lowpass amount 0..1 (spec §3.5)
        bandlimit: f32,
        unison: UnisonSettings,
    },
    Distortion {
        dist_type: DistortionType,
        drive: f32,
        bias: f32,
        spread: f32,
        mix: f32,
    },
    Filter {
        filter_type: FilterType,
        cutoff_hz: f32,
        q: f32,
        // Shelf/peak only; ignored by other types
        gain_db: f32,
        slope: FilterSlope,
    },
    NonlinearFilter {
        filter_type: NlFilterType,
        cutoff_hz: f32,
        q: f32,
        drive: f32,
        // 0 = Clean; higher values select Geist-original color modes (spec §17.2)
        color_mode: u8,
    },
    Mix {
        level: f32,
        invert: bool,
    },
    Aux {
        level: f32,
        invert: bool,
    },
    EnvelopeOutput {
        env: AdsrParams,
        out: OutputCommon,
    },
    CurveOutput {
        curve: AssetRef,
        loop_start: f32,
        loop_end: f32,
        loop_mode: LoopMode,
        out: OutputCommon,
    },
}

impl ModuleKind {
    // Report whether this module generates sound on its own
    pub fn is_source(&self) -> bool {
        matches!(
            self,
            ModuleKind::AnalogOsc { .. }
                | ModuleKind::NoiseGen { .. }
                | ModuleKind::SamplePlayer { .. }
                | ModuleKind::Granular { .. }
                | ModuleKind::WavetableOsc { .. }
        )
    }

    // Report whether this module needs signal from modules above (spec §2.2)
    pub fn requires_input(&self) -> bool {
        matches!(
            self,
            ModuleKind::Distortion { .. }
                | ModuleKind::Filter { .. }
                | ModuleKind::NonlinearFilter { .. }
                | ModuleKind::Mix { .. }
                | ModuleKind::EnvelopeOutput { .. }
                | ModuleKind::CurveOutput { .. }
        )
    }

    // Report whether this module bridges the voice to a bus (spec §5)
    pub fn is_output(&self) -> bool {
        matches!(
            self,
            ModuleKind::EnvelopeOutput { .. } | ModuleKind::CurveOutput { .. }
        )
    }
}

// One generator group: ordered modules, implicit routing boundary (spec §6)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Group {
    pub id: GroupId,
    pub name: String,
    // UI-only collapse state, no DSP effect
    pub minimized: bool,
    // Top-to-bottom stack order; implicit signal flow follows this order
    pub modules: Vec<Module>,
}

// Audio-rate modulation target parameter (spec §7)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioRateTarget {
    // Classic FM; tuning stable
    Phase,
    // Linear FM, pitch-invariant waveform
    Harmonic,
    // Linear FM in Hz
    Shift,
    // Ring/amplitude modulation
    Level,
    // Exponential FM
    Pitch,
    // Processor drive amount
    Drive,
    // Filter operating frequency
    Cutoff,
    // Aux explicit input; carries a mandatory one-sample delay (spec §4.5)
    AuxIn,
}

// One audio-rate modulation edge (spec §7.3)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AudioRateRoute {
    pub source: ModuleId,
    pub target: ModuleId,
    pub target_param: AudioRateTarget,
    pub depth: f32,
    pub enabled: bool,
}

// Modulator output polarity (spec §10.1)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OutputRange {
    Unipolar,
    Bipolar,
    Inverted,
}

// Modulator trigger policy (spec §10.3)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TriggerMode {
    Auto,
    Never,
    Always,
    Legato,
}

// LFO/random rate: free-running Hz or tempo-synced beats
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum RateMode {
    FreeHz(f32),
    SyncedBeats(f32),
}

// Random modulator voice behavior (spec §10.2)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RandomVoiceMode {
    Unison,
    Independent,
}

// Modulator families for the S6 core set (spec §10.2); bus-fed and MPE
// modulators are deferred per plan S6.5
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ModulatorKind {
    Envelope { env: AdsrParams, seamless: bool },
    Lfo { rate: RateMode, phase_deg: f32, shape: AssetRef },
    Curve { rate: RateMode, curve: AssetRef, loop_start: f32, loop_end: f32, loop_mode: LoopMode, lock: bool },
    Random { rate: RateMode, jitter: f32, smooth: f32, chaos: f32, voice_mode: RandomVoiceMode },
    Note,
    Velocity,
    NoteGate,
    PitchWheel,
    MidiCc { cc: u8 },
    Scale { factor: f32 },
    LowerLimit { limit: f32 },
    UpperLimit { limit: f32 },
    Remap { shape: AssetRef },
    SampleHold { threshold: f32 },
}

// One modulator lane slot (spec §10.1)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModulatorSlot {
    pub id: ModulatorId,
    pub kind: ModulatorKind,
    pub output_range: OutputRange,
    pub trigger: TriggerMode,
    // Modulatable master depth of this modulator's output
    pub depth: f32,
}

// Control-rate route source: a modulator slot or a macro knob
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlSource {
    Modulator(ModulatorId),
    Macro(u8),
}

// Control-rate route target: a module parameter or another modulator
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ControlTarget {
    // Module parameter addressed by a stable key; keys defined per kind in S6
    ModuleParam { module: ModuleId, param: String },
    // Another modulator's depth/trigger/parameter (spec §10.1)
    ModulatorParam { modulator: ModulatorId, param: String },
}

// One control-rate modulation route (spec §10.1)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ControlRoute {
    pub source: ControlSource,
    pub target: ControlTarget,
    pub amount: f32,
    // Non-linear response applied to the route, 0 = linear
    pub curvature: f32,
    pub enabled: bool,
}

// One macro knob: renameable control source with host automation (spec §1.2)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MacroKnob {
    pub name: String,
    pub value: f32,
}

impl Default for MacroKnob {
    // Unnamed macro at rest
    fn default() -> Self {
        Self {
            name: String::new(),
            value: 0.0,
        }
    }
}

// Monophonic note handling (spec §9.1)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonoMode {
    Retrig,
    Legato,
}

// Glide application policy (spec §9.1)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GlideMode {
    Always,
    Legato,
}

// Voice section settings (spec §9.1)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VoiceSettings {
    // 1 = monophonic with mono_mode semantics
    pub polyphony: u8,
    pub mono_mode: MonoMode,
    pub glide_enabled: bool,
    pub glide_time_s: f32,
    pub glide_mode: GlideMode,
}

impl Default for VoiceSettings {
    // Eight voices, retrig mono, glide off
    fn default() -> Self {
        Self {
            polyphony: 8,
            mono_mode: MonoMode::Retrig,
            glide_enabled: false,
            glide_time_s: 0.05,
            glide_mode: GlideMode::Always,
        }
    }
}

// Master section: global pitch transform + final gain (spec §1.2)
// Bend range lives in project state, not here (spec §14.2)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MasterSettings {
    pub pitch_semis: f32,
    pub gain: f32,
}

impl Default for MasterSettings {
    // Neutral master
    fn default() -> Self {
        Self {
            pitch_semis: 0.0,
            gain: 1.0,
        }
    }
}

// One effect lane; effect content is hosted FxKind slots per plan D3 (spec §8)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Lane {
    // Per-voice processing; poly lanes must form a prefix (spec §8.1)
    pub poly: bool,
    pub mute: bool,
    pub solo: bool,
    pub gain: f32,
    // Dry/wet 0..1
    pub mix: f32,
    // None = master; Some(i) = lane index strictly to the right
    pub send_to: Option<u8>,
    // Hosted effect references: engine FxKind code + enabled flag
    pub effects: Vec<LaneEffect>,
}

impl Default for Lane {
    // Pass-through lane sending to master
    fn default() -> Self {
        Self {
            poly: false,
            mute: false,
            solo: false,
            gain: 1.0,
            mix: 1.0,
            send_to: None,
            effects: Vec::new(),
        }
    }
}

// One hosted lane effect slot (kind code mirrors the app's FxKind table)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LaneEffect {
    pub kind: u32,
    pub enabled: bool,
}

// A complete patch (spec §14.2 payload checklist)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Patch {
    pub schema_version: u32,
    pub name: String,
    pub author: String,
    pub description: String,
    pub tags: Vec<String>,
    pub groups: Vec<Group>,
    pub audio_routes: Vec<AudioRateRoute>,
    pub lanes: [Lane; LANE_COUNT],
    pub modulators: Vec<ModulatorSlot>,
    pub control_routes: Vec<ControlRoute>,
    pub macros: [MacroKnob; MACRO_COUNT],
    pub voice: VoiceSettings,
    pub global_unison: UnisonSettings,
    pub master: MasterSettings,
}

impl Default for Patch {
    // Empty init patch at the current schema version
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            name: String::new(),
            author: String::new(),
            description: String::new(),
            tags: Vec::new(),
            groups: Vec::new(),
            audio_routes: Vec::new(),
            lanes: Default::default(),
            modulators: Vec::new(),
            control_routes: Vec::new(),
            macros: Default::default(),
            voice: VoiceSettings::default(),
            global_unison: UnisonSettings::default(),
            master: MasterSettings::default(),
        }
    }
}

impl Patch {
    // Count generator-area modules across all groups
    pub fn module_count(&self) -> usize {
        self.groups.iter().map(|g| g.modules.len()).sum()
    }

    // Find a module by id anywhere in the stack
    pub fn find_module(&self, id: ModuleId) -> Option<&Module> {
        self.groups
            .iter()
            .flat_map(|g| g.modules.iter())
            .find(|m| m.id == id)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    // Build a small two-group fixture patch exercising every schema area
    pub fn fixture_patch() -> Patch {
        let osc = Module {
            id: ModuleId(1),
            kind: ModuleKind::AnalogOsc {
                common: CommonGenParams::default(),
                waveform: AnalogWaveform::Sawtooth,
                sync: 1.0,
                pulse_width: 0.5,
                unison: UnisonSettings::default(),
            },
        };
        let filter = Module {
            id: ModuleId(2),
            kind: ModuleKind::Filter {
                filter_type: FilterType::LowPass,
                cutoff_hz: 1200.0,
                q: 0.7,
                gain_db: 0.0,
                slope: FilterSlope::TwoPole,
            },
        };
        let out = Module {
            id: ModuleId(3),
            kind: ModuleKind::EnvelopeOutput {
                env: AdsrParams::default(),
                out: OutputCommon::default(),
            },
        };
        let mod_osc = Module {
            id: ModuleId(4),
            kind: ModuleKind::WavetableOsc {
                common: CommonGenParams::default(),
                table: AssetRef("factory/basic".into()),
                frame: 0.0,
                bandlimit: 0.0,
                unison: UnisonSettings::default(),
            },
        };
        let mut patch = Patch {
            groups: vec![
                Group {
                    id: GroupId(1),
                    name: "Main".into(),
                    minimized: false,
                    modules: vec![osc, filter, out],
                },
                Group {
                    id: GroupId(2),
                    name: "FM source".into(),
                    minimized: false,
                    modules: vec![mod_osc],
                },
            ],
            audio_routes: vec![AudioRateRoute {
                source: ModuleId(4),
                target: ModuleId(1),
                target_param: AudioRateTarget::Phase,
                depth: 0.5,
                enabled: true,
            }],
            modulators: vec![ModulatorSlot {
                id: ModulatorId(1),
                kind: ModulatorKind::Lfo {
                    rate: RateMode::FreeHz(2.0),
                    phase_deg: 0.0,
                    shape: AssetRef("shapes/sine".into()),
                },
                output_range: OutputRange::Bipolar,
                trigger: TriggerMode::Auto,
                depth: 1.0,
            }],
            control_routes: vec![ControlRoute {
                source: ControlSource::Modulator(ModulatorId(1)),
                target: ControlTarget::ModuleParam {
                    module: ModuleId(2),
                    param: "cutoff".into(),
                },
                amount: 0.3,
                curvature: 0.0,
                enabled: true,
            }],
            ..Patch::default()
        };
        patch.name = "Fixture".into();
        patch
    }

    #[test]
    fn patch_round_trips_through_serde() {
        let patch = fixture_patch();
        let json = serde_json::to_string(&patch).expect("serialize");
        let back: Patch = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(patch, back);
    }

    #[test]
    fn module_kind_classification_is_consistent() {
        let patch = fixture_patch();
        let osc = &patch.find_module(ModuleId(1)).unwrap().kind;
        let filter = &patch.find_module(ModuleId(2)).unwrap().kind;
        let out = &patch.find_module(ModuleId(3)).unwrap().kind;
        assert!(osc.is_source() && !osc.requires_input() && !osc.is_output());
        assert!(!filter.is_source() && filter.requires_input() && !filter.is_output());
        assert!(!out.is_source() && out.requires_input() && out.is_output());
    }

    #[test]
    fn module_count_spans_groups() {
        assert_eq!(fixture_patch().module_count(), 4);
    }
}
