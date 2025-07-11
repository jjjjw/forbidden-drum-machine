use crate::audio::delays::FilteredDelayLine;
use crate::audio::instruments::{ClapDrum, KickDrum};
use crate::audio::reverbs::DownsampledReverb;
use crate::audio::{AudioNode, AudioSystem};

/// Auditioner system for testing and tweaking instruments
/// Allows triggering individual instruments without sequencing
pub struct AuditionerSystem {
    // Audio nodes for different instruments
    kick: KickDrum,
    clap: ClapDrum,

    // Effects (optional, can be bypassed)
    delay: FilteredDelayLine,
    reverb: DownsampledReverb,

    // Effect sends (usually lower for auditioning)
    delay_send: f32,
    reverb_send: f32,

    sample_rate: f32,
}

impl AuditionerSystem {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            kick: KickDrum::new(sample_rate),
            clap: ClapDrum::new(sample_rate),
            delay: FilteredDelayLine::new(0.5, sample_rate),
            reverb: DownsampledReverb::new(sample_rate),

            // Lower sends for cleaner auditioning
            delay_send: 0.1,
            reverb_send: 0.15,

            sample_rate,
        }
    }

    /// Set delay send level
    pub fn set_delay_send(&mut self, send: f32) {
        self.delay_send = send.clamp(0.0, 1.0);
    }

    /// Set reverb send level
    pub fn set_reverb_send(&mut self, send: f32) {
        self.reverb_send = send.clamp(0.0, 1.0);
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
            NodeName::Delay => self.delay.handle_event(event),
            NodeName::Reverb => self.reverb.handle_event(event),
        }
    }

    fn handle_system_event(
        &mut self,
        event: crate::events::SystemEvent,
    ) -> Result<(), String> {
        match event {
            crate::events::SystemEvent::SetDelaySend(send) => {
                self.set_delay_send(send);
                Ok(())
            }
            crate::events::SystemEvent::SetReverbSend(send) => {
                self.set_reverb_send(send);
                Ok(())
            }
            _ => Err(format!(
                "Unsupported system event for Auditioner: {:?}",
                event
            )),
        }
    }

    fn process_stereo(&mut self, left_in: f32, right_in: f32) -> (f32, f32) {
        // Start with input signal - no sequencer, only triggered sounds
        let mut signal = (left_in, right_in);

        // Add instruments (they produce sound only when triggered)
        signal = self.kick.process_stereo(signal.0, signal.1);
        signal = self.clap.process_stereo(signal.0, signal.1);

        // Apply sends and process through effects
        let delay_input = (signal.0 * self.delay_send, signal.1 * self.delay_send);
        let reverb_input = (signal.0 * self.reverb_send, signal.1 * self.reverb_send);

        // Process effects
        let delay_output = self.delay.process_stereo(delay_input.0, delay_input.1);
        let reverb_output = self.reverb.process_stereo(reverb_input.0, reverb_input.1);

        // Mix dry and wet signals - more dry signal for auditioning
        let dry_level = 0.8; // Clearer dry signal for parameter tweaking
        let wet_level = 0.2; // Subtle effects

        let output_left = signal.0 * dry_level + (delay_output.0 + reverb_output.0) * wet_level;
        let output_right = signal.1 * dry_level + (delay_output.1 + reverb_output.1) * wet_level;

        (output_left, output_right)
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
        self.delay.set_sample_rate(sample_rate);
        self.reverb.set_sample_rate(sample_rate);
    }
}
