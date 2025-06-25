pub mod buffers;
pub mod delays;
pub mod envelopes;
pub mod filters;
pub mod instruments;
pub mod modulators;
pub mod oscillators;
pub mod reverbs;
pub mod systems;
pub mod velvet;

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

pub trait StereoAudioProcessor {
    fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32);
}

pub trait AudioBlockProcessor {
    fn process_block(&mut self, buffer: &mut [f32]);
}

pub trait AudioBlockStereoProcessor {
    fn process_stereo_block(&mut self, left_buffer: &mut [f32], right_buffer: &mut [f32]);
}

pub fn sec_to_samples(seconds: f32) -> u32 {
    (seconds * SAMPLE_RATE).round() as u32
}
