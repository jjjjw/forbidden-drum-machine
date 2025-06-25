pub mod buffers;
pub mod delays;
pub mod envelopes;
pub mod filters;
pub mod instruments;
pub mod modulators;
pub mod oscillators;
pub mod reverbs;
pub mod systems;

pub const PI: f32 = std::f32::consts::PI;
pub const TWO_PI: f32 = 2.0 * PI;

// Basic trait for audio generators that produce a single sample output
pub trait AudioGenerator {
    fn next_sample(&mut self) -> f32;
    fn set_sample_rate(&mut self, sample_rate: f32);
}

pub trait AudioProcessor {
    fn process(&mut self, input: f32) -> f32;
    fn set_sample_rate(&mut self, sample_rate: f32);
}

pub trait StereoAudioProcessor {
    fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32);
    fn set_sample_rate(&mut self, sample_rate: f32);
}
