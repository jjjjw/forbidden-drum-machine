use crate::audio::envelopes::AREnvelope;
use crate::audio::oscillators::NoiseGenerator;
use crate::audio::AudioGenerator;

pub struct SnareDrum {
    noise_generator: NoiseGenerator,
    amp_envelope: AREnvelope,
}

impl SnareDrum {
    pub fn new(sample_rate: f32) -> Self {
        let mut snare = Self {
            noise_generator: NoiseGenerator::new(),
            amp_envelope: AREnvelope::new(sample_rate),
        };

        snare.amp_envelope.set_attack_time(0.001);
        snare.amp_envelope.set_release_time(0.08);
        snare.amp_envelope.set_attack_bias(0.5); // Linear
        snare.amp_envelope.set_release_bias(0.7); // Exponential-like

        snare
    }

    pub fn trigger(&mut self) {
        self.amp_envelope.trigger();
    }

    pub fn set_amp_attack(&mut self, time: f32) {
        self.amp_envelope.set_attack_time(time);
    }

    pub fn set_amp_release(&mut self, time: f32) {
        self.amp_envelope.set_release_time(time);
    }

    pub fn is_active(&self) -> bool {
        self.amp_envelope.is_active()
    }
}

impl AudioGenerator for SnareDrum {
    fn next_sample(&mut self) -> f32 {
        if !self.is_active() {
            return 0.0;
        }

        let amp_env = self.amp_envelope.next_sample();
        let sample = self.noise_generator.next_sample();
        sample * amp_env
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.noise_generator.set_sample_rate(sample_rate);
        self.amp_envelope.set_sample_rate(sample_rate);
    }
}
