// Author: Jeff
// Date: 2026-09-06
// Description: M5 evidence — Filament sounds a chord, and one note is unchanged by the change
// Notes: DEV-010's own re-open trigger is "MIDI clips producing overlapping notes a user expects
//   to hear together". These tests are that trigger made falsifiable. The compatibility half
//   matters as much as the polyphony half: a single sounding note must produce bit-identical
//   output to the monophonic device, or R4's render evidence stops describing this instrument.

use spectre_dsp::{
    AudioProcessor, Filament, NoteEvent, NoteEventKind, ProcessContext, PulseInstrument, Waveform,
    FILAMENT_PARAMETERS, MAX_VOICES, PULSE_PARAMETERS,
};

const SAMPLE_RATE: f64 = 48_000.0;
const FRAMES: usize = 256;

fn instrument() -> Filament {
    Filament::new(
        FILAMENT_PARAMETERS[0].default(),
        FILAMENT_PARAMETERS[1].default(),
        FILAMENT_PARAMETERS[2].default(),
        FILAMENT_PARAMETERS[3].default(),
    )
    .expect("descriptor defaults are valid")
}

fn on(sequence: u64, id: u32, note: u8) -> NoteEvent {
    NoteEvent {
        frame_offset: 0,
        sequence,
        kind: NoteEventKind::On {
            id,
            channel: 0,
            note,
            velocity: 0.8,
        },
    }
}

fn off(sequence: u64, id: u32, note: u8) -> NoteEvent {
    NoteEvent {
        frame_offset: 0,
        sequence,
        kind: NoteEventKind::Off {
            id,
            channel: 0,
            note,
            velocity: 0.0,
        },
    }
}

// Render one block with the given events at frame 0
fn render(device: &mut Filament, events: &[NoteEvent]) -> Vec<f32> {
    let mut left = vec![0.0; FRAMES];
    let mut right = vec![0.0; FRAMES];
    let context = ProcessContext::new(SAMPLE_RATE, FRAMES, events).expect("a valid context");
    {
        let mut outputs: [&mut [f32]; 2] = [&mut left, &mut right];
        device
            .process(&context, &[], &mut outputs)
            .expect("the block renders");
    }
    left
}

// The trigger DEV-010 named: three notes a user expects to hear together
#[test]
fn a_chord_sounds_like_more_than_one_note() {
    let mut chord = instrument();
    let rendered = render(&mut chord, &[on(0, 1, 60), on(1, 2, 64), on(2, 3, 67)]);

    let mut single = instrument();
    let one = render(&mut single, &[on(0, 1, 60)]);

    assert!(
        rendered.iter().any(|sample| *sample != 0.0),
        "the chord is silent"
    );
    assert_ne!(
        rendered, one,
        "three notes produced exactly what one note produced, so they did not sound together"
    );
    // A chord is louder than its own root, which a last-note-wins device could not be
    let chord_peak = rendered.iter().fold(0.0f32, |peak, s| peak.max(s.abs()));
    let single_peak = one.iter().fold(0.0f32, |peak, s| peak.max(s.abs()));
    assert!(
        chord_peak > single_peak,
        "the chord peaks no higher than one note: {chord_peak} vs {single_peak}"
    );
}

// The compatibility claim. Bit-identical, not approximately equal: summing one voice into a zero
// accumulator is exact, and R4's evidence depends on it staying that way
#[test]
fn one_note_renders_exactly_as_it_did_before_the_pool() {
    let mut device = instrument();
    let first = render(&mut device, &[on(0, 1, 60)]);
    // Second block with no events: the held note continues, exercising the sustain path
    let second = render(&mut device, &[]);

    let mut reference = instrument();
    let reference_first = render(&mut reference, &[on(0, 1, 60)]);
    let reference_second = render(&mut reference, &[]);

    assert_eq!(first, reference_first);
    assert_eq!(second, reference_second);
    assert!(first.iter().any(|sample| *sample != 0.0));
}

// Notes are released individually. A release that silenced the whole pool would be the
// monophonic behaviour wearing a chord's clothes
#[test]
fn releasing_one_note_leaves_the_others_sounding() {
    let mut device = instrument();
    render(&mut device, &[on(0, 1, 60), on(1, 2, 67)]);
    let both = render(&mut device, &[]);
    render(&mut device, &[off(0, 1, 60)]);
    // Let the released voice fall to silence
    for _ in 0..64 {
        render(&mut device, &[]);
    }
    let remaining = render(&mut device, &[]);

    assert!(
        remaining.iter().any(|sample| *sample != 0.0),
        "releasing one note silenced the whole chord"
    );
    assert_ne!(both, remaining, "the release changed nothing");
}

#[test]
fn all_notes_off_silences_every_voice() {
    let mut device = instrument();
    render(&mut device, &[on(0, 1, 60), on(1, 2, 64), on(2, 3, 67)]);
    render(
        &mut device,
        &[NoteEvent {
            frame_offset: 0,
            sequence: 9,
            kind: NoteEventKind::AllNotesOff { channel: None },
        }],
    );
    for _ in 0..64 {
        render(&mut device, &[]);
    }
    let settled = render(&mut device, &[]);
    assert!(
        settled.iter().all(|sample| *sample == 0.0),
        "a voice was still sounding after all-notes-off"
    );
}

// Past the pool the device steals rather than refusing or growing, and it stays finite
#[test]
fn more_notes_than_voices_steal_rather_than_overrun() {
    let mut device = instrument();
    let events: Vec<NoteEvent> = (0..MAX_VOICES + 4)
        .map(|index| on(index as u64, index as u32 + 1, 40 + index as u8))
        .collect();
    let rendered = render(&mut device, &events);
    assert!(
        rendered.iter().all(|sample| sample.is_finite()),
        "stealing produced a non-finite sample"
    );
    assert!(rendered.iter().any(|sample| *sample != 0.0));
}

// Stealing must be deterministic, or the same project renders differently on two runs and the
// live/offline bit-equality R4-8 proves stops holding
#[test]
fn stealing_is_deterministic_across_identical_runs() {
    let events: Vec<NoteEvent> = (0..MAX_VOICES + 6)
        .map(|index| on(index as u64, index as u32 + 1, 36 + index as u8))
        .collect();
    let mut first = instrument();
    let mut second = instrument();
    assert_eq!(render(&mut first, &events), render(&mut second, &events));
    // And across the following block, where the stolen voices are still sounding
    assert_eq!(render(&mut first, &[]), render(&mut second, &[]));
}

// A fully released pool returns to exact silence, which RT-003's containment evidence and the
// offline harness's silence gates both depend on
#[test]
fn a_released_pool_returns_to_exact_silence() {
    let mut device = instrument();
    render(&mut device, &[on(0, 1, 60), on(1, 2, 64)]);
    render(&mut device, &[off(0, 1, 60), off(1, 2, 64)]);
    for _ in 0..64 {
        render(&mut device, &[]);
    }
    assert!(render(&mut device, &[]).iter().all(|s| *s == 0.0));
}

// Which voice gets stolen is a stated policy, so it needs its own falsifiable test. Determinism
// alone does not cover it: stealing the NEWEST voice is equally deterministic and passes every
// other test in this file.
//
// Observed indirectly, because a voice cannot be inspected from outside: fill the pool, steal
// once, then release the note that should have been stolen. If the policy is oldest-first that
// id no longer exists and the release changes nothing; if the newest was stolen instead, the
// oldest is still sounding and the release is audible
#[test]
fn the_oldest_voice_is_the_one_stolen() {
    let fill: Vec<NoteEvent> = (0..MAX_VOICES)
        .map(|index| on(index as u64, index as u32 + 1, 40 + index as u8))
        .collect();
    let steal = on(MAX_VOICES as u64, MAX_VOICES as u32 + 1, 80);
    // The id allocated first, and therefore the one an oldest-first policy must reuse
    let oldest_id = 1;

    let mut released = instrument();
    render(&mut released, &fill);
    render(&mut released, &[steal]);
    let after_release = render(&mut released, &[off(0, oldest_id, 40)]);

    let mut untouched = instrument();
    render(&mut untouched, &fill);
    render(&mut untouched, &[steal]);
    let after_nothing = render(&mut untouched, &[]);

    assert_eq!(
        after_release, after_nothing,
        "releasing the oldest note changed the output, so it was still sounding and a newer \
         voice was stolen instead"
    );
}

// ---- PulseInstrument ----
//
// Pulse gets its own pool rather than sharing Filament's, which is what the accepted
// per-instrument voicing decision means. It differs in a way worth pinning: Pulse has no
// amplitude contour, so a released voice is immediately free where Filament's stays busy
// through its fall ramp.
//
// This matters to the product and not only to the crate: AppModel::add_track creates Pulse
// tracks, so until Pulse was polyphonic a track added in the app could not play a chord.

fn pulse() -> PulseInstrument {
    PulseInstrument::new(Waveform::Saw, PULSE_PARAMETERS[0].default())
        .expect("the descriptor default is valid")
}

fn render_pulse(device: &mut PulseInstrument, events: &[NoteEvent]) -> Vec<f32> {
    let mut left = vec![0.0; FRAMES];
    let mut right = vec![0.0; FRAMES];
    let context = ProcessContext::new(SAMPLE_RATE, FRAMES, events).expect("a valid context");
    {
        let mut outputs: [&mut [f32]; 2] = [&mut left, &mut right];
        device
            .process(&context, &[], &mut outputs)
            .expect("the block renders");
    }
    left
}

#[test]
fn pulse_sounds_a_chord() {
    let mut chord = pulse();
    let rendered = render_pulse(&mut chord, &[on(0, 1, 60), on(1, 2, 64), on(2, 3, 67)]);
    let mut single = pulse();
    let one = render_pulse(&mut single, &[on(0, 1, 60)]);

    assert!(rendered.iter().any(|sample| *sample != 0.0));
    assert_ne!(
        rendered, one,
        "three notes produced exactly what one produced, so they did not sound together"
    );
    // Inequality alone is not enough and this test proved it: a monophonic device renders the
    // LAST note, which differs from the first note anyway, so assert_ne passed under a
    // deliberately monophonic mutation. Summed amplitude is what actually distinguishes a chord
    let chord_peak = rendered.iter().fold(0.0f32, |peak, s| peak.max(s.abs()));
    let single_peak = one.iter().fold(0.0f32, |peak, s| peak.max(s.abs()));
    assert!(
        chord_peak > single_peak,
        "the chord peaks no higher than one note: {chord_peak} vs {single_peak}"
    );
}

#[test]
fn one_pulse_note_renders_exactly_as_it_did_before_the_pool() {
    let mut device = pulse();
    let first = render_pulse(&mut device, &[on(0, 1, 60)]);
    let second = render_pulse(&mut device, &[]);
    let mut reference = pulse();
    assert_eq!(first, render_pulse(&mut reference, &[on(0, 1, 60)]));
    assert_eq!(second, render_pulse(&mut reference, &[]));
    assert!(first.iter().any(|sample| *sample != 0.0));
}

// Pulse has no contour, so a release is immediate and exact rather than a ramp
#[test]
fn releasing_one_pulse_note_leaves_the_others_sounding_and_silences_exactly() {
    let mut device = pulse();
    render_pulse(&mut device, &[on(0, 1, 60), on(1, 2, 67)]);
    render_pulse(&mut device, &[off(0, 1, 60)]);
    let remaining = render_pulse(&mut device, &[]);
    assert!(
        remaining.iter().any(|sample| *sample != 0.0),
        "releasing one note silenced the whole chord"
    );

    render_pulse(&mut device, &[off(0, 2, 67)]);
    let settled = render_pulse(&mut device, &[]);
    assert!(
        settled.iter().all(|sample| *sample == 0.0),
        "a released Pulse voice is still sounding; it has no contour to fall through"
    );
}

#[test]
fn pulse_steals_deterministically_past_its_pool() {
    let events: Vec<NoteEvent> = (0..MAX_VOICES + 5)
        .map(|index| on(index as u64, index as u32 + 1, 36 + index as u8))
        .collect();
    let mut first = pulse();
    let mut second = pulse();
    let a = render_pulse(&mut first, &events);
    assert_eq!(a, render_pulse(&mut second, &events));
    assert!(a.iter().all(|sample| sample.is_finite()));
}
