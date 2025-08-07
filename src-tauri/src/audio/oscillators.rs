use crate::audio::{AudioGenerator, TWO_PI};
use once_cell::sync::Lazy;

const SINE_TABLE_SIZE: usize = 8192;
const SINE_TABLE_MASK: usize = SINE_TABLE_SIZE - 1;

static SINE_TABLE: Lazy<Vec<f32>> = Lazy::new(|| {
    (0..SINE_TABLE_SIZE)
        .map(|i| (i as f32 * TWO_PI / SINE_TABLE_SIZE as f32).sin())
        .collect()
});

static SAW_TABLE: Lazy<Vec<f32>> = Lazy::new(|| {
    (0..SINE_TABLE_SIZE)
        .map(|i| {
            let phase = i as f32 / SINE_TABLE_SIZE as f32;
            2.0 * phase - 1.0
        })
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
        let table_index = ((phase * SINE_TABLE_SIZE as f32) as usize) & SINE_TABLE_MASK;
        let sample = SINE_TABLE[table_index];
        sample
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.set_sample_rate(sample_rate);
    }
}

pub struct SawOscillator {
    phase_gen: PhaseGenerator,
}

impl SawOscillator {
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

impl AudioGenerator for SawOscillator {
    fn next_sample(&mut self) -> f32 {
        let phase = self.phase_gen.next_sample();
        let table_index = ((phase * SINE_TABLE_SIZE as f32) as usize) & SINE_TABLE_MASK;
        let sample = SAW_TABLE[table_index];
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

pub struct PMOscillator {
    phase_gen: PhaseGenerator,
    feedback: f32,
    last_output: f32,
}

impl PMOscillator {
    pub fn new(frequency: f32, sample_rate: f32) -> Self {
        Self {
            phase_gen: PhaseGenerator::new(frequency, sample_rate),
            feedback: 0.0,
            last_output: 0.0,
        }
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.phase_gen.set_frequency(frequency);
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 0.99);
    }

    pub fn reset(&mut self) {
        self.phase_gen.reset();
        self.last_output = 0.0;
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.phase_gen.set_sample_rate(sample_rate);
    }

    pub fn next_sample_with_pm(&mut self, phase_mod: f32) -> f32 {
        let phase = self.phase_gen.next_sample();
        let modulated_phase = (phase + phase_mod + self.last_output * self.feedback).fract();
        let table_index = ((modulated_phase * SINE_TABLE_SIZE as f32) as usize) & SINE_TABLE_MASK;
        let sample = SINE_TABLE[table_index];
        self.last_output = sample;
        sample
    }
}

impl AudioGenerator for PMOscillator {
    fn next_sample(&mut self) -> f32 {
        self.next_sample_with_pm(0.0)
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.set_sample_rate(sample_rate);
    }
}

/// Hash-based noise generator that simulates Hasher.ar(Sweep.ar) from SuperCollider
/// Creates chaotic noise by applying a hash function to a linear ramp (sweep)
pub struct HasherNoise {
    phase_gen: PhaseGenerator,
}

impl HasherNoise {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            // Use very slow frequency for sweep (1 Hz means full ramp every second)
            phase_gen: PhaseGenerator::new(1.0, sample_rate),
        }
    }

    pub fn reset(&mut self) {
        self.phase_gen.reset();
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.phase_gen.set_sample_rate(sample_rate);
    }
}

impl AudioGenerator for HasherNoise {
    fn next_sample(&mut self) -> f32 {
        // Get phase (0.0 to 1.0) from the sweep
        let phase = self.phase_gen.next_sample();
        
        // Hash function on the phase to create chaotic noise
        let hash_input = (phase * 1000000.0) as u32;
        let hash = hash_input
            .wrapping_mul(0x45d9f3b)
            .wrapping_add(0x119de1f3)
            .wrapping_mul(0x45d9f3b);
        
        // Convert to float in range -1 to 1
        (hash as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.set_sample_rate(sample_rate);
    }
}

