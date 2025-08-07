use crate::audio::instruments::{ChordSynth, ClapDrum, HiHat, KickDrum, SupersawSynth};
use crate::audio::reverbs::ReverbLite;
use crate::audio::{AudioGenerator, AudioSystem, StereoAudioGenerator, StereoAudioProcessor};

/// Auditioner system for testing and tweaking instruments
/// Allows triggering individual instruments without sequencing
pub struct AuditionerSystem {
    // Audio nodes for different instruments
    kick: KickDrum,
    clap: ClapDrum,
    hihat: HiHat,
    chord: ChordSynth,
    supersaw: SupersawSynth,
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
            supersaw: SupersawSynth::new(sample_rate),
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

    fn handle_kick_event(&mut self, event: &crate::events::ClientEvent) -> Result<(), String> {
        match event.event.as_str() {
            "trigger" => {
                self.kick.trigger();
                Ok(())
            }
            "set_gain" => {
                self.kick.set_gain(event.param());
                Ok(())
            }
            "set_base_frequency" => {
                self.kick.set_base_frequency(event.param());
                Ok(())
            }
            "set_frequency_ratio" => {
                self.kick.set_frequency_ratio(event.param());
                Ok(())
            }
            "set_amp_attack" => {
                self.kick.set_amp_attack(event.param());
                Ok(())
            }
            "set_amp_release" => {
                self.kick.set_amp_release(event.param());
                Ok(())
            }
            "set_freq_attack" => {
                self.kick.set_freq_attack(event.param());
                Ok(())
            }
            "set_freq_release" => {
                self.kick.set_freq_release(event.param());
                Ok(())
            }
            _ => Err(format!("Unknown kick event: {}", event.event)),
        }
    }

    fn handle_clap_event(&mut self, event: &crate::events::ClientEvent) -> Result<(), String> {
        match event.event.as_str() {
            "trigger" => {
                self.clap.trigger();
                Ok(())
            }
            "set_gain" => {
                self.clap.set_gain(event.param());
                Ok(())
            }
            _ => Err(format!("Unknown clap event: {}", event.event)),
        }
    }

    fn handle_hihat_event(&mut self, event: &crate::events::ClientEvent) -> Result<(), String> {
        match event.event.as_str() {
            "trigger" => {
                self.hihat.trigger();
                Ok(())
            }
            "set_gain" => {
                self.hihat.set_gain(event.param());
                Ok(())
            }
            "set_length" => {
                self.hihat.set_length(event.param());
                Ok(())
            }
            _ => Err(format!("Unknown hihat event: {}", event.event)),
        }
    }

    fn handle_chord_event(&mut self, event: &crate::events::ClientEvent) -> Result<(), String> {
        match event.event.as_str() {
            "trigger" => {
                self.chord.trigger();
                Ok(())
            }
            "set_gain" => {
                self.chord.set_gain(event.param());
                Ok(())
            }
            "set_base_frequency" => {
                self.chord.set_base_frequency(event.param());
                Ok(())
            }
            "set_modulation_index" => {
                self.chord.set_modulation_index(event.param());
                Ok(())
            }
            "set_feedback" => {
                self.chord.set_feedback(event.param());
                Ok(())
            }
            "set_attack" => {
                self.chord.set_attack(event.param());
                Ok(())
            }
            "set_release" => {
                self.chord.set_release(event.param());
                Ok(())
            }
            _ => Err(format!("Unknown chord event: {}", event.event)),
        }
    }

    fn handle_supersaw_event(&mut self, event: &crate::events::ClientEvent) -> Result<(), String> {
        match event.event.as_str() {
            "trigger" => {
                self.supersaw.trigger();
                Ok(())
            }
            "set_gain" => {
                self.supersaw.set_gain(event.param());
                Ok(())
            }
            "set_base_frequency" => {
                self.supersaw.set_base_frequency(event.param());
                Ok(())
            }
            "set_detune" => {
                self.supersaw.set_detune(event.param());
                Ok(())
            }
            "set_stereo_width" => {
                self.supersaw.set_stereo_width(event.param());
                Ok(())
            }
            "set_filter_cutoff" => {
                self.supersaw.set_filter_cutoff(event.param());
                Ok(())
            }
            "set_filter_resonance" => {
                self.supersaw.set_filter_resonance(event.param());
                Ok(())
            }
            "set_filter_env_amount" => {
                self.supersaw.set_filter_env_amount(event.param());
                Ok(())
            }
            "set_amp_attack" => {
                self.supersaw.set_amp_attack(event.param());
                Ok(())
            }
            "set_amp_release" => {
                self.supersaw.set_amp_release(event.param());
                Ok(())
            }
            "set_filter_attack" => {
                self.supersaw.set_filter_attack(event.param());
                Ok(())
            }
            "set_filter_release" => {
                self.supersaw.set_filter_release(event.param());
                Ok(())
            }
            _ => Err(format!("Unknown supersaw event: {}", event.event)),
        }
    }

    fn handle_reverb_event(&mut self, event: &crate::events::ClientEvent) -> Result<(), String> {
        match event.event.as_str() {
            "set_size" => {
                self.reverb.set_size(event.param());
                Ok(())
            }
            "set_modulation_depth" => {
                self.reverb.set_modulation_depth(event.param());
                Ok(())
            }
            _ => Err(format!("Unknown reverb event: {}", event.event)),
        }
    }

    fn handle_system_event(&mut self, event: &crate::events::ClientEvent) -> Result<(), String> {
        match event.event.as_str() {
            "set_reverb_send" => {
                self.set_reverb_send(event.param());
                Ok(())
            }
            "set_reverb_return" => {
                self.set_reverb_return(event.param());
                Ok(())
            }
            _ => Err(format!("Unknown system event: {}", event.event)),
        }
    }
}

impl AudioSystem for AuditionerSystem {
    fn handle_client_event(&mut self, event: &crate::events::ClientEvent) -> Result<(), String> {
        match event.node.as_str() {
            "kick" => self.handle_kick_event(event),
            "clap" => self.handle_clap_event(event),
            "hihat" => self.handle_hihat_event(event),
            "chord" => self.handle_chord_event(event),
            "supersaw" => self.handle_supersaw_event(event),
            "reverb" => self.handle_reverb_event(event),
            "system" => self.handle_system_event(event),
            _ => Err(format!("Unknown node '{}' for auditioner system", event.node)),
        }
    }

    fn next_sample(&mut self) -> (f32, f32) {
        // Generate samples from mono instruments
        let kick_sample = self.kick.next_sample();
        let clap_sample = self.clap.next_sample();
        let hihat_sample = self.hihat.next_sample();
        let chord_sample = self.chord.next_sample();
        
        // Generate stereo sample from supersaw
        let (supersaw_left, supersaw_right) = self.supersaw.next_sample();

        // Mix all instruments
        let dry_signal = (
            kick_sample + clap_sample + hihat_sample + chord_sample + supersaw_left,
            kick_sample + clap_sample + hihat_sample + chord_sample + supersaw_right,
        );

        // Send to reverb and mix with dry signal
        let reverb_input = (dry_signal.0 * self.reverb_send, dry_signal.1 * self.reverb_send);
        let reverb_output = self.reverb.process(reverb_input.0, reverb_input.1);

        // Final mix: dry signal + reverb return
        (
            dry_signal.0 + reverb_output.0 * self.reverb_return,
            dry_signal.1 + reverb_output.1 * self.reverb_return,
        )
    }


    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.kick.set_sample_rate(sample_rate);
        self.clap.set_sample_rate(sample_rate);
        self.hihat.set_sample_rate(sample_rate);
        self.chord.set_sample_rate(sample_rate);
        self.supersaw.set_sample_rate(sample_rate);
        self.reverb.set_sample_rate(sample_rate);
    }
}
