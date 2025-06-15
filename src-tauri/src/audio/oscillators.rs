use crate::audio::{SAMPLE_RATE, TWO_PI};
use once_cell::sync::Lazy;

const SINE_TABLE_SIZE: usize = 4096;

static SINE_TABLE: Lazy<Vec<f32>> = Lazy::new(|| {
    (0..SINE_TABLE_SIZE)
        .map(|i| (i as f32 * TWO_PI / SINE_TABLE_SIZE as f32).sin())
        .collect()
});

pub struct SineOscillator {
    phase: f32,
    frequency: f32,
}

impl SineOscillator {
    pub fn new(frequency: f32) -> Self {
        Self {
            phase: 0.0,
            frequency,
        }
    }
    
    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }
    
    pub fn next_sample(&mut self) -> f32 {
        let table_index = (self.phase * SINE_TABLE_SIZE as f32 / TWO_PI) as usize % SINE_TABLE_SIZE;
        let sample = SINE_TABLE[table_index];
        
        self.phase += TWO_PI * self.frequency / SAMPLE_RATE;
        
        if self.phase >= TWO_PI {
            self.phase -= TWO_PI;
        }
        
        sample
    }
    
    pub fn reset(&mut self) {
        self.phase = 0.0;
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
    
    pub fn next_sample(&mut self) -> f32 {
        self.rng.f32() * 2.0 - 1.0
    }
}