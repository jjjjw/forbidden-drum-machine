pub mod buffers;
pub mod delays;
pub mod envelopes;
pub mod filters;
pub mod instruments;
pub mod modulators;
pub mod oscillators;
pub mod reverbs;
pub mod server;
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

pub trait StereoAudioGenerator {
    fn next_sample(&mut self) -> (f32, f32);
    fn set_sample_rate(&mut self, sample_rate: f32);
}

pub trait StereoAudioProcessor {
    fn process(&mut self, left: f32, right: f32) -> (f32, f32);
    fn set_sample_rate(&mut self, sample_rate: f32);
}

/// AudioSystem trait for managing audio processing and events
/// Systems handle all audio processing and event routing internally
pub trait AudioSystem: Send {
    /// Process a single stereo sample and return (left, right)
    fn next_sample(&mut self) -> (f32, f32);

    /// Handle a client event - each system parses and handles its own supported events
    fn handle_client_event(&mut self, event: &crate::events::ClientEvent) -> Result<(), String>;

    /// Set the sample rate for the entire system
    fn set_sample_rate(&mut self, sample_rate: f32);
}
