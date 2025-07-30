use crate::audio::envelopes::AREnvelope;
use crate::audio::oscillators::SineOscillator;
use crate::audio::{AudioGenerator, AudioNode};
use crate::events::NodeEvent;

pub struct KickDrum {
    oscillator: SineOscillator,
    amp_envelope: AREnvelope,
    freq_envelope: AREnvelope,
    base_frequency: f32,
    frequency_ratio: f32,
    gain: f32,
}

impl KickDrum {
    pub fn new(sample_rate: f32) -> Self {
        let mut kick = Self {
            oscillator: SineOscillator::new(60.0, sample_rate),
            amp_envelope: AREnvelope::new(sample_rate),
            freq_envelope: AREnvelope::new(sample_rate),
            base_frequency: 60.0,
            frequency_ratio: 7.0,
            gain: 1.0,
        };

        kick.amp_envelope.set_attack_time(0.005);
        kick.amp_envelope.set_release_time(0.2);
        kick.amp_envelope.set_attack_bias(0.3); // Logarithmic-like
        kick.amp_envelope.set_release_bias(0.7); // Exponential-like

        kick.freq_envelope.set_attack_time(0.002);
        kick.freq_envelope.set_release_time(0.05);
        kick.freq_envelope.set_attack_bias(0.7); // Exponential-like
        kick.freq_envelope.set_release_bias(0.7); // Exponential-like

        kick
    }

    pub fn trigger(&mut self) {
        self.amp_envelope.trigger();
        self.freq_envelope.trigger();
        self.oscillator.reset();
    }

    pub fn set_base_frequency(&mut self, freq: f32) {
        self.base_frequency = freq;
    }

    pub fn set_frequency_ratio(&mut self, ratio: f32) {
        self.frequency_ratio = ratio;
    }

    pub fn set_amp_attack(&mut self, time: f32) {
        self.amp_envelope.set_attack_time(time);
    }

    pub fn set_amp_release(&mut self, time: f32) {
        self.amp_envelope.set_release_time(time);
    }

    pub fn set_freq_attack(&mut self, time: f32) {
        self.freq_envelope.set_attack_time(time);
    }

    pub fn set_freq_release(&mut self, time: f32) {
        self.freq_envelope.set_release_time(time);
    }

    pub fn is_active(&self) -> bool {
        self.amp_envelope.is_active()
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }
}

impl AudioGenerator for KickDrum {
    fn next_sample(&mut self) -> f32 {
        if !self.is_active() {
            return 0.0;
        }

        let amp_env = self.amp_envelope.next_sample();
        let freq_env = self.freq_envelope.next_sample();

        // Use frequency ratio for sharper sweep: starts at base_frequency * ratio, sweeps down to base_frequency
        let start_freq = self.base_frequency * self.frequency_ratio;
        let current_freq = self.base_frequency + (freq_env * (start_freq - self.base_frequency));
        self.oscillator.set_frequency(current_freq);

        let sample = self.oscillator.next_sample();
        sample * amp_env
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.oscillator.set_sample_rate(sample_rate);
        self.amp_envelope.set_sample_rate(sample_rate);
        self.freq_envelope.set_sample_rate(sample_rate);
    }
}

impl AudioNode for KickDrum {
    fn process(&mut self, left_in: f32, right_in: f32) -> (f32, f32) {
        let drum_sample = self.next_sample() * self.gain;
        (left_in + drum_sample, right_in + drum_sample)
    }

    fn handle_event(&mut self, event: NodeEvent) -> Result<(), String> {
        match event {
            NodeEvent::Trigger => {
                self.trigger();
                Ok(())
            }
            NodeEvent::SetGain(gain) => {
                self.set_gain(gain);
                Ok(())
            }
            NodeEvent::SetBaseFrequency(freq) => {
                self.set_base_frequency(freq);
                Ok(())
            }
            NodeEvent::SetFrequencyRatio(ratio) => {
                self.set_frequency_ratio(ratio);
                Ok(())
            }
            NodeEvent::SetAmpAttack(time) => {
                self.set_amp_attack(time);
                Ok(())
            }
            NodeEvent::SetAmpRelease(time) => {
                self.set_amp_release(time);
                Ok(())
            }
            NodeEvent::SetFreqAttack(time) => {
                self.set_freq_attack(time);
                Ok(())
            }
            NodeEvent::SetFreqRelease(time) => {
                self.set_freq_release(time);
                Ok(())
            }
            _ => Err(format!("Unsupported event for KickDrum: {:?}", event)),
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        AudioGenerator::set_sample_rate(self, sample_rate);
    }
}
