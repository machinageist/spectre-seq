// =============================================================================
// File: crates/geist-dsp/src/lfo/stepseq.rs
// Layer: DSP primitives
// Purpose: step sequencer LFO shape
// Status: Implemented; clocked CV step sequencer with optional glide.
// Notes: A clock phasor advances one step per cycle; glide one-pole-slews toward
//        each step. Buffer is sized once; process() never allocates.
// Contract: Keep comments terse, declarative, and synchronized with code.
// =============================================================================

use crate::osc::Phasor;

// Clocked step sequencer producing one held (or glided) value per step
#[derive(Clone, Debug)]
pub struct StepSequencer {
    steps: Vec<f32>,
    length: usize,
    index: usize,
    phasor: Phasor,
    last_phase: f32,
    glide_coef: f32,
    value: f32,
}

impl StepSequencer {
    // Allocate a sequencer with capacity for `max_steps` (all zero, no glide)
    pub fn new(max_steps: usize) -> Self {
        let capacity = max_steps.max(1);
        Self {
            steps: vec![0.0; capacity],
            length: capacity,
            index: 0,
            phasor: Phasor::new(),
            last_phase: 0.0,
            glide_coef: 0.0,
            value: 0.0,
        }
    }

    // Set the value of one step; out-of-range indices are ignored
    pub fn set_step(&mut self, index: usize, value: f32) {
        if index < self.steps.len() {
            self.steps[index] = value;
        }
    }

    // Replace the pattern; active length follows the slice, clamped to capacity
    pub fn set_steps(&mut self, values: &[f32]) {
        let n = values.len().min(self.steps.len());
        self.steps[..n].copy_from_slice(&values[..n]);
        self.length = n.max(1);
        if self.index >= self.length {
            self.index = 0;
        }
    }

    // Set the number of active steps, clamped to [1, capacity]
    pub fn set_length(&mut self, length: usize) {
        self.length = length.clamp(1, self.steps.len());
        if self.index >= self.length {
            self.index = 0;
        }
    }

    // Set the clock rate in steps per second
    pub fn set_rate(&mut self, steps_per_second: f32, sample_rate_hz: f32) {
        self.phasor.set_frequency(steps_per_second, sample_rate_hz);
    }

    // Set glide time in seconds; zero gives instant steps
    pub fn set_glide(&mut self, seconds: f32, sample_rate_hz: f32) {
        self.glide_coef = if seconds <= 0.0 {
            0.0
        } else {
            (-1.0 / (seconds * sample_rate_hz)).exp()
        };
    }

    // Restart the pattern at step 0
    pub fn retrigger(&mut self) {
        self.index = 0;
        self.phasor.reset();
        self.last_phase = 0.0;
    }

    // Reset position and output deterministically
    pub fn reset(&mut self) {
        self.index = 0;
        self.phasor.reset();
        self.last_phase = 0.0;
        self.value = 0.0;
    }

    // Index of the current step
    pub fn current_step(&self) -> usize {
        self.index
    }

    // Advance one sample and return the (possibly glided) step value
    #[inline]
    pub fn next_sample(&mut self) -> f32 {
        let phase = self.phasor.tick();
        // A phase wrap clocks the sequence to the next step
        if phase < self.last_phase {
            self.index = (self.index + 1) % self.length;
        }
        self.last_phase = phase;

        let target = self.steps[self.index];
        self.value = target + self.glide_coef * (self.value - target);
        self.value
    }

    // Fill a buffer with successive values
    pub fn process(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            *sample = self.next_sample();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: f32 = 48_000.0;

    // Compress consecutive equal values into the order they appear
    fn transitions(buf: &[f32]) -> Vec<f32> {
        let mut out = Vec::new();
        for &v in buf {
            if out.last() != Some(&v) {
                out.push(v);
            }
        }
        out
    }

    fn seq() -> StepSequencer {
        let mut s = StepSequencer::new(8);
        s.set_steps(&[0.0, 0.25, 0.5, 0.75]);
        // 10 samples per step
        s.set_rate(SAMPLE_RATE / 10.0, SAMPLE_RATE);
        s
    }

    #[test]
    fn steps_cycle_in_order() {
        let mut s = seq();
        let mut buf = vec![0.0f32; 200];
        s.process(&mut buf);
        let order = transitions(&buf);
        // Values appear in pattern order, cycling
        let pattern = [0.0, 0.25, 0.5, 0.75];
        for (i, &v) in order.iter().enumerate() {
            assert_eq!(v, pattern[i % 4], "transition {i} = {v}");
        }
        assert!(order.len() >= 8, "expected multiple cycles, got {order:?}");
    }

    #[test]
    fn without_glide_output_jumps_to_step_values() {
        let mut s = seq();
        let mut buf = vec![0.0f32; 200];
        s.process(&mut buf);
        // Every output is exactly one of the configured step values
        assert!(buf.iter().all(|&v| [0.0, 0.25, 0.5, 0.75].contains(&v)));
    }

    #[test]
    fn length_limits_the_active_steps() {
        let mut s = StepSequencer::new(8);
        s.set_steps(&[0.1, 0.2, 0.3, 0.4, 0.5]);
        s.set_length(3);
        s.set_rate(SAMPLE_RATE / 10.0, SAMPLE_RATE);
        let mut buf = vec![0.0f32; 200];
        s.process(&mut buf);
        // Only the first three steps ever appear
        assert!(buf.iter().all(|&v| [0.1, 0.2, 0.3].contains(&v)));
    }

    #[test]
    fn glide_interpolates_between_steps() {
        let mut s = StepSequencer::new(4);
        s.set_steps(&[0.0, 1.0]);
        s.set_rate(SAMPLE_RATE / 200.0, SAMPLE_RATE); // 200 samples per step
        s.set_glide(0.002, SAMPLE_RATE);
        let mut buf = vec![0.0f32; 400];
        s.process(&mut buf);
        // Some samples land strictly between the two step values
        let between = buf.iter().filter(|&&v| v > 0.05 && v < 0.95).count();
        assert!(between > 0, "glide produced no intermediate values");
        assert!(buf.iter().all(|&v| (-0.01..=1.01).contains(&v)));
    }

    #[test]
    fn retrigger_restarts_at_first_step() {
        let mut s = seq();
        for _ in 0..55 {
            s.next_sample();
        }
        assert!(s.current_step() > 0);
        s.retrigger();
        assert_eq!(s.current_step(), 0);
        assert_eq!(s.next_sample(), 0.0);
    }

    #[test]
    fn rate_controls_steps_per_second() {
        let mut s = StepSequencer::new(8);
        s.set_steps(&[0.0, 0.2, 0.4, 0.6, 0.8, 1.0, 0.5, 0.3]);
        s.set_rate(10.0, SAMPLE_RATE); // 10 steps per second
        let mut buf = vec![0.0f32; SAMPLE_RATE as usize];
        s.process(&mut buf);
        // ~10 step changes across one second
        let changes = transitions(&buf).len() - 1;
        assert!((8..=12).contains(&changes), "step changes = {changes}");
    }
}
