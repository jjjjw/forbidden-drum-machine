use crate::audio::instruments::SupersawSynth;
use crate::audio::{AudioSystem, StereoAudioGenerator};
use crate::sequencing::TonalSequencer;

/// Main TranceRiff system using TonalSequencer
pub struct TranceRiffSystem {
    synth: SupersawSynth,
    sequencer: TonalSequencer,
    bpm: f32,
    is_paused: bool,
    sample_rate: f32,
}

impl TranceRiffSystem {
    pub fn new(sample_rate: f32) -> Self {
        let bpm = 138.0; // Classic trance BPM
        
        Self {
            synth: SupersawSynth::new(sample_rate),
            sequencer: TonalSequencer::new(sample_rate),
            bpm,
            is_paused: false,
            sample_rate,
        }
    }
    
    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm.clamp(60.0, 200.0);
    }
    
    pub fn set_paused(&mut self, paused: bool) {
        self.is_paused = paused;
    }
    
    pub fn set_sequence(&mut self, sequence: Vec<(f32, f32, f32)>) {
        self.sequencer.set_sequence(sequence);
    }
    
    fn handle_synth_event(&mut self, event: &crate::events::ClientEvent) -> Result<(), String> {
        match event.event.as_str() {
            "trigger" => {
                self.synth.trigger();
                Ok(())
            }
            "set_gain" => {
                self.synth.set_gain(event.param());
                Ok(())
            }
            "set_base_frequency" => {
                self.synth.set_base_frequency(event.param());
                Ok(())
            }
            "set_detune" => {
                self.synth.set_detune(event.param());
                Ok(())
            }
            "set_stereo_width" => {
                self.synth.set_stereo_width(event.param());
                Ok(())
            }
            "set_filter_cutoff" => {
                self.synth.set_filter_cutoff(event.param());
                Ok(())
            }
            "set_filter_resonance" => {
                self.synth.set_filter_resonance(event.param());
                Ok(())
            }
            "set_filter_env_amount" => {
                self.synth.set_filter_env_amount(event.param());
                Ok(())
            }
            "set_amp_attack" => {
                self.synth.set_amp_attack(event.param());
                Ok(())
            }
            "set_amp_release" => {
                self.synth.set_amp_release(event.param());
                Ok(())
            }
            "set_filter_attack" => {
                self.synth.set_filter_attack(event.param());
                Ok(())
            }
            "set_filter_release" => {
                self.synth.set_filter_release(event.param());
                Ok(())
            }
            _ => Err(format!("Unknown synth event: {}", event.event)),
        }
    }
    
    fn handle_system_event(&mut self, event: &crate::events::ClientEvent) -> Result<(), String> {
        match event.event.as_str() {
            "set_bpm" => {
                self.set_bpm(event.param());
                Ok(())
            }
            "set_paused" => {
                self.set_paused(event.param() > 0.5);
                Ok(())
            }
            "set_sequence" => {
                // This will be sent from frontend with sequence data
                if let Some(data) = &event.data {
                    if let Some(sequence_data) = data.as_array() {
                        let mut sequence = Vec::new();
                        for item in sequence_data.iter() {
                            if let Some(note) = item.as_array() {
                                if note.len() >= 3 {
                                    let freq = note[0].as_f64().unwrap_or(0.0) as f32;
                                    let duration = note[1].as_f64().unwrap_or(0.0) as f32;
                                    let velocity = note[2].as_f64().unwrap_or(1.0) as f32;
                                    sequence.push((freq, duration, velocity));
                                }
                            }
                        }
                        self.set_sequence(sequence);
                    }
                }
                Ok(())
            }
            "reset_sequence" => {
                self.sequencer.reset();
                Ok(())
            }
            _ => Err(format!("Unknown system event: {}", event.event)),
        }
    }
}

impl AudioSystem for TranceRiffSystem {
    fn next_sample(&mut self) -> (f32, f32) {
        if self.is_paused {
            return (0.0, 0.0);
        }
        
        // Tick the sequencer
        let (should_trigger, frequency, velocity) = self.sequencer.tick();
        
        // Trigger new notes when needed
        if should_trigger && frequency > 0.0 {
            self.synth.set_base_frequency(frequency);
            self.synth.set_gain(velocity);
            self.synth.trigger();
        }
        
        // Generate audio sample
        self.synth.next_sample()
    }
    
    fn handle_client_event(&mut self, event: &crate::events::ClientEvent) -> Result<(), String> {
        match event.node.as_str() {
            "supersaw" => self.handle_synth_event(event),
            "system" => self.handle_system_event(event),
            _ => Err(format!("Unknown node '{}' for trance riff system", event.node)),
        }
    }
    
    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.synth.set_sample_rate(sample_rate);
        self.sequencer.set_sample_rate(sample_rate);
    }
}