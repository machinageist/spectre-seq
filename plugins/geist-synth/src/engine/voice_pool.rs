// =============================================================================
// File: plugins/geist-synth/src/engine/voice_pool.rs
// Layer: synth plugin
// Purpose: polyphony manager + steal modes
// Status: Implemented; fixed voice array, free/oldest/quietest allocation.
// Notes: note_on prefers a free voice, else steals per the steal mode. Render
//        sums every voice additively into the block. Voices allocated once.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use crate::engine::voice::Voice;

// How a new note claims a voice when all are busy
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StealMode {
    // Reclaim the longest-running voice
    #[default]
    Oldest,
    // Reclaim the voice with the lowest current loudness
    Quietest,
}

// Fixed-size polyphonic voice manager
#[derive(Clone, Debug)]
pub struct VoicePool {
    voices: Vec<Voice>,
    ages: Vec<u64>,
    next_age: u64,
    steal_mode: StealMode,
}

impl VoicePool {
    // Build a pool of `polyphony` voices at a sample rate
    pub fn new(sample_rate_hz: f32, polyphony: usize) -> Self {
        let count = polyphony.max(1);
        Self {
            voices: vec![Voice::new(sample_rate_hz); count],
            ages: vec![0; count],
            next_age: 1,
            steal_mode: StealMode::Oldest,
        }
    }

    // Choose how busy voices are reclaimed
    pub fn set_steal_mode(&mut self, mode: StealMode) {
        self.steal_mode = mode;
    }

    // Configure every voice identically (patch application)
    pub fn voices_mut(&mut self) -> &mut [Voice] {
        &mut self.voices
    }

    // Start a note, allocating a free voice or stealing one
    pub fn note_on(&mut self, note: u8, velocity: f32) {
        let index = self.allocate();
        self.voices[index].note_on(note, velocity);
        self.ages[index] = self.next_age;
        self.next_age += 1;
    }

    // Release every voice currently playing `note`
    pub fn note_off(&mut self, note: u8) {
        for voice in &mut self.voices {
            if voice.is_active() && voice.note() == note {
                voice.note_off();
            }
        }
    }

    // Number of voices still producing sound
    pub fn active_voice_count(&self) -> usize {
        self.voices.iter().filter(|v| v.is_active()).count()
    }

    // Silence and idle every voice
    pub fn reset(&mut self) {
        for voice in &mut self.voices {
            voice.reset();
        }
    }

    // Render one block, summing every active voice into `output`
    pub fn render_block(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            *sample = 0.0;
        }
        for voice in &mut self.voices {
            voice.render_additive(output);
        }
    }

    // Pick a free voice, otherwise steal per the steal mode
    fn allocate(&mut self) -> usize {
        if let Some(free) = self.voices.iter().position(|v| !v.is_active()) {
            return free;
        }
        match self.steal_mode {
            StealMode::Oldest => self
                .ages
                .iter()
                .enumerate()
                .min_by_key(|(_, &age)| age)
                .map(|(i, _)| i)
                .unwrap_or(0),
            StealMode::Quietest => self
                .voices
                .iter()
                .enumerate()
                .min_by(|(_, a), (_, b)| {
                    a.amp_level()
                        .partial_cmp(&b.amp_level())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(i, _)| i)
                .unwrap_or(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f32 = 48_000.0;
    const BLOCK: usize = 256;

    fn run_blocks(pool: &mut VoicePool, blocks: usize) {
        let mut buf = vec![0.0f32; BLOCK];
        for _ in 0..blocks {
            pool.render_block(&mut buf);
        }
    }

    #[test]
    fn allocates_free_voices_for_a_chord() {
        let mut pool = VoicePool::new(SAMPLE_RATE, 8);
        pool.note_on(60, 1.0);
        pool.note_on(64, 1.0);
        pool.note_on(67, 1.0);
        assert_eq!(pool.active_voice_count(), 3);
    }

    #[test]
    fn note_off_releases_matching_voice() {
        let mut pool = VoicePool::new(SAMPLE_RATE, 4);
        pool.note_on(60, 1.0);
        run_blocks(&mut pool, 4);
        assert_eq!(pool.active_voice_count(), 1);
        pool.note_off(60);
        // After the release completes the voice frees up
        run_blocks(&mut pool, 400);
        assert_eq!(pool.active_voice_count(), 0);
    }

    #[test]
    fn polyphony_mixes_louder_than_one_voice() {
        let mut one = VoicePool::new(SAMPLE_RATE, 8);
        one.note_on(60, 1.0);
        let mut many = VoicePool::new(SAMPLE_RATE, 8);
        many.note_on(60, 1.0);
        many.note_on(64, 1.0);
        many.note_on(67, 1.0);
        run_blocks(&mut one, 8);
        run_blocks(&mut many, 8);

        let mut a = vec![0.0f32; BLOCK];
        one.render_block(&mut a);
        let mut b = vec![0.0f32; BLOCK];
        many.render_block(&mut b);
        let energy_a: f32 = a.iter().map(|s| s.abs()).sum();
        let energy_b: f32 = b.iter().map(|s| s.abs()).sum();
        assert!(energy_b > energy_a, "chord not louder than note");
    }

    #[test]
    fn oldest_steal_reuses_the_first_voice() {
        let mut pool = VoicePool::new(SAMPLE_RATE, 2);
        pool.set_steal_mode(StealMode::Oldest);
        pool.note_on(60, 1.0); // age 1
        pool.note_on(62, 1.0); // age 2
                               // Both voices busy; a third note steals the oldest (note 60)
        pool.note_on(64, 1.0);
        assert_eq!(pool.active_voice_count(), 2);
        let notes: Vec<u8> = pool.voices.iter().map(|v| v.note()).collect();
        assert!(notes.contains(&64), "stolen voice not retuned to new note");
        assert!(!notes.contains(&60), "oldest voice was not stolen");
        assert!(notes.contains(&62), "wrong voice stolen");
    }

    #[test]
    fn never_exceeds_polyphony() {
        let mut pool = VoicePool::new(SAMPLE_RATE, 3);
        for note in 60..72 {
            pool.note_on(note, 1.0);
        }
        assert!(pool.active_voice_count() <= 3);
    }

    #[test]
    fn quietest_steal_picks_lowest_level() {
        let mut pool = VoicePool::new(SAMPLE_RATE, 2);
        pool.set_steal_mode(StealMode::Quietest);
        pool.note_on(60, 1.0);
        pool.note_on(62, 0.2); // softer voice
        run_blocks(&mut pool, 30); // let levels settle
        pool.note_on(64, 1.0); // steals the quietest (note 62)
        let notes: Vec<u8> = pool.voices.iter().map(|v| v.note()).collect();
        assert!(notes.contains(&60), "loud voice should survive");
        assert!(notes.contains(&64), "new note not allocated");
        assert!(!notes.contains(&62), "quietest voice not stolen");
    }
}
