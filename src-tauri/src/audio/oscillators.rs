use crate::audio::{AudioGenerator, TWO_PI};
use once_cell::sync::Lazy;

const SINE_TABLE_SIZE: usize = 4096;

static SINE_TABLE: Lazy<Vec<f32>> = Lazy::new(|| {
    (0..SINE_TABLE_SIZE)
        .map(|i| (i as f32 * TWO_PI / SINE_TABLE_SIZE as f32).sin())
        .collect()
});

pub struct PhaseGenerator {
    phase: f32,
    phase_increment: f32,
    frequency: f32,
    sample_rate: f32,
}

impl PhaseGenerator {
    pub fn new(frequency: f32, sample_rate: f32) -> Self {
        Self {
            phase: 0.0,
            frequency: frequency,
            sample_rate,
            phase_increment: frequency / sample_rate,
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
        self.phase_increment = frequency / self.sample_rate;
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.phase_increment = self.frequency / sample_rate;
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
    }

    pub fn next_sample(&mut self) -> f32 {
        let sample = self.phase;
        self.phase += self.phase_increment;

        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        sample
    }
}

pub struct SineOscillator {
    phase_gen: PhaseGenerator,
}

impl SineOscillator {
    pub fn new(frequency: f32, sample_rate: f32) -> Self {
        Self {
            phase_gen: PhaseGenerator::new(frequency, sample_rate),
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.phase_gen.set_frequency(frequency);
    }

    pub fn reset(&mut self) {
        self.phase_gen.reset();
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.phase_gen.set_sample_rate(sample_rate);
    }
}

impl AudioGenerator for SineOscillator {
    fn next_sample(&mut self) -> f32 {
        let phase = self.phase_gen.next_sample();
        let table_index = ((phase * SINE_TABLE_SIZE as f32) as usize) % SINE_TABLE_SIZE;
        let sample = SINE_TABLE[table_index];
        sample
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.set_sample_rate(sample_rate);
    }
}

pub struct NoiseGenerator {
    rng: fastrand::Rng,
}

impl NoiseGenerator {
    pub fn new() -> Self {
        Self {
            rng: fastrand::Rng::new(),
        }
    }
}

impl AudioGenerator for NoiseGenerator {
    fn next_sample(&mut self) -> f32 {
        self.rng.f32() * 2.0 - 1.0
    }

    fn set_sample_rate(&mut self, _sample_rate: f32) {
        // NoiseGenerator doesn't depend on sample rate
    }
}
