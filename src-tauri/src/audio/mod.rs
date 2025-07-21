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

pub trait StereoAudioProcessor {
    fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32);
    fn set_sample_rate(&mut self, sample_rate: f32);
}

/// AudioNode trait for the new event-based architecture
/// All instruments and effects should implement this trait
pub trait AudioNode {
    /// Process a single stereo sample (left_in, right_in) -> (left_out, right_out)
    fn process_stereo(&mut self, left_in: f32, right_in: f32) -> (f32, f32);

    /// Handle a typed event
    fn handle_event(&mut self, event: crate::events::NodeEvent) -> Result<(), String>;

    /// Set the sample rate
    fn set_sample_rate(&mut self, sample_rate: f32);
}

/// AudioSystem trait for managing audio nodes and sequences
/// Systems are configurations of audio nodes (instruments + effects) and sequencers
pub trait AudioSystem: Send {
    /// Process a buffer of interleaved stereo samples
    fn generate(&mut self, data: &mut [f32]);

    /// Handle an event for a specific audio node (including system events when node_name is System)
    fn handle_node_event(
        &mut self,
        node_name: crate::events::NodeName,
        event: crate::events::NodeEvent,
    ) -> Result<(), String>;

    /// Set the sequence configuration
    fn set_sequence(&mut self, sequence_config: &serde_json::Value) -> Result<(), String>;

    /// Set the sample rate for the entire system
    fn set_sample_rate(&mut self, sample_rate: f32);
}
