use crate::audio::instruments::{ClapDrum, KickDrum};
use crate::audio::{AudioNode, AudioSystem};

/// Auditioner system for testing and tweaking instruments
/// Allows triggering individual instruments without sequencing
pub struct AuditionerSystem {
    // Audio nodes for different instruments
    kick: KickDrum,
    clap: ClapDrum,
    sample_rate: f32,
}

impl AuditionerSystem {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            kick: KickDrum::new(sample_rate),
            clap: ClapDrum::new(sample_rate),
            sample_rate,
        }
    }
}

impl AudioSystem for AuditionerSystem {
    fn handle_node_event(
        &mut self,
        node_name: crate::events::NodeName,
        event: crate::events::NodeEvent,
    ) -> Result<(), String> {
        use crate::events::NodeName;
        match node_name {
            NodeName::Kick => self.kick.handle_event(event),
            NodeName::Clap => self.clap.handle_event(event),
            _ => Err(format!("Unsupported node for Auditioner: {:?}", node_name)),
        }
    }

    fn generate(&mut self, data: &mut [f32]) {
        // Process each stereo frame
        for frame in data.chunks_mut(2) {
            // Start with silence - no input signal for auditioner
            let (mut left, mut right) = (0.0, 0.0);
            
            // Add kick drum output
            let (kick_left, kick_right) = self.kick.process_stereo(left, right);
            left = kick_left;
            right = kick_right;
            
            // Add clap drum output
            let (clap_left, clap_right) = self.clap.process_stereo(left, right);
            
            // Write final output to buffer
            frame[0] = clap_left;
            frame[1] = clap_right;
        }
    }

    fn set_sequence(&mut self, sequence_config: &serde_json::Value) -> Result<(), String> {
        // Auditioner doesn't use sequences, but we can use this for configuration
        println!(
            "AuditionerSystem: set_sequence called with: {}",
            sequence_config
        );
        Ok(())
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.kick.set_sample_rate(sample_rate);
        self.clap.set_sample_rate(sample_rate);
    }
}
