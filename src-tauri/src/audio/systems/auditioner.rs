use crate::audio::instruments::{ChordSynth, ClapDrum, KickDrum};
use crate::audio::{AudioNode, AudioSystem};

/// Auditioner system for testing and tweaking instruments
/// Allows triggering individual instruments without sequencing
pub struct AuditionerSystem {
    // Audio nodes for different instruments
    kick: KickDrum,
    clap: ClapDrum,
    chord: ChordSynth,
    sample_rate: f32,
}

impl AuditionerSystem {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            kick: KickDrum::new(sample_rate),
            clap: ClapDrum::new(sample_rate),
            chord: ChordSynth::new(sample_rate),
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
            NodeName::Chord => self.chord.handle_event(event),
            _ => Err(format!("Unsupported node for Auditioner: {:?}", node_name)),
        }
    }

    fn next_sample(&mut self) -> (f32, f32) {
        // Start with silence (no input signal)
        let mut signal = (0.0, 0.0);

        // Add instruments
        signal = self.kick.process(signal.0, signal.1);
        signal = self.clap.process(signal.0, signal.1);
        signal = self.chord.process(signal.0, signal.1);

        signal
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
        self.chord.set_sample_rate(sample_rate);
    }
}
