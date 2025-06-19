pub mod effects;
pub mod envelopes;
pub mod instruments;
pub mod modulators;
pub mod oscillators;
pub mod systems;

pub const SAMPLE_RATE: f32 = 44100.0;
pub const PI: f32 = std::f32::consts::PI;
pub const TWO_PI: f32 = 2.0 * PI;

// Basic trait for audio generators that produce a single sample output
pub trait AudioGenerator {
    fn next_sample(&mut self) -> f32;
}

pub trait AudioProcessor {
    fn process(&mut self, input: f32) -> f32;
}

pub fn sec_to_samples(seconds: f32) -> f32 {
    seconds * SAMPLE_RATE
}
