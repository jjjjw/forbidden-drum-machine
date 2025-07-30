use crate::audio::instruments::{ChordSynth, ClapDrum, HiHat, KickDrum};
use crate::audio::reverbs::ReverbLite;
use crate::audio::{AudioNode, AudioSystem};

/// Auditioner system for testing and tweaking instruments
/// Allows triggering individual instruments without sequencing
pub struct AuditionerSystem {
    // Audio nodes for different instruments
    kick: KickDrum,
    clap: ClapDrum,
    hihat: HiHat,
    chord: ChordSynth,
    reverb: ReverbLite,

    // Send/return levels for reverb
    reverb_send: f32,
    reverb_return: f32,

    sample_rate: f32,
}

impl AuditionerSystem {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            kick: KickDrum::new(sample_rate),
            clap: ClapDrum::new(sample_rate),
            hihat: HiHat::new(sample_rate),
            chord: ChordSynth::new(sample_rate),
            reverb: ReverbLite::new(sample_rate),
            reverb_send: 0.3,   // Default 30% send to reverb
            reverb_return: 0.5, // Default 50% reverb return
            sample_rate,
        }
    }

    pub fn set_reverb_send(&mut self, send: f32) {
        self.reverb_send = send.clamp(0.0, 1.0);
    }

    pub fn set_reverb_return(&mut self, return_level: f32) {
        self.reverb_return = return_level.clamp(0.0, 1.0);
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
            NodeName::HiHat => self.hihat.handle_event(event),
            NodeName::Chord => self.chord.handle_event(event),
            NodeName::Reverb => self.reverb.handle_event(event),
            NodeName::System => {
                // Handle system events for reverb send/return
                use crate::events::NodeEvent;
                match event {
                    NodeEvent::SetReverbSend(send) => {
                        self.set_reverb_send(send);
                        Ok(())
                    }
                    NodeEvent::SetReverbReturn(return_level) => {
                        self.set_reverb_return(return_level);
                        Ok(())
                    }
                    _ => Err(format!(
                        "Unsupported system event for Auditioner: {:?}",
                        event
                    )),
                }
            }
            _ => Err(format!("Unsupported node for Auditioner: {:?}", node_name)),
        }
    }

    fn next_sample(&mut self) -> (f32, f32) {
        // Start with silence (no input signal)
        let mut signal = (0.0, 0.0);

        // Add instruments
        signal = self.kick.process(signal.0, signal.1);
        signal = self.clap.process(signal.0, signal.1);
        signal = self.hihat.process(signal.0, signal.1);
        signal = self.chord.process(signal.0, signal.1);

        // Send to reverb and mix with dry signal
        let reverb_input = (signal.0 * self.reverb_send, signal.1 * self.reverb_send);
        let reverb_output = self.reverb.process(reverb_input.0, reverb_input.1);

        // Final mix: dry signal + reverb return
        (
            signal.0 + reverb_output.0 * self.reverb_return,
            signal.1 + reverb_output.1 * self.reverb_return,
        )
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
        self.hihat.set_sample_rate(sample_rate);
        self.chord.set_sample_rate(sample_rate);
        self.reverb.set_sample_rate(sample_rate);
    }
}
