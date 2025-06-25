use crate::audio::envelopes::{AREnvelope, CurveType};
use crate::audio::oscillators::{NoiseGenerator, SineOscillator};
use crate::audio::AudioGenerator;

pub struct KickDrum {
    oscillator: SineOscillator,
    amp_envelope: AREnvelope,
    freq_envelope: AREnvelope,
    base_frequency: f32,
    frequency_mod_amount: f32,
}

impl KickDrum {
    pub fn new(sample_rate: f32) -> Self {
        let mut kick = Self {
            oscillator: SineOscillator::new(60.0, sample_rate),
            amp_envelope: AREnvelope::new(sample_rate),
            freq_envelope: AREnvelope::new(sample_rate),
            base_frequency: 60.0,
            frequency_mod_amount: 40.0,
        };

        kick.amp_envelope.set_attack_time(0.005);
        kick.amp_envelope.set_release_time(0.2);
        kick.amp_envelope.set_attack_curve(CurveType::Logarithmic);
        kick.amp_envelope.set_release_curve(CurveType::Exponential);

        kick.freq_envelope.set_attack_time(0.002);
        kick.freq_envelope.set_release_time(0.05);
        kick.freq_envelope.set_attack_curve(CurveType::Exponential);
        kick.freq_envelope.set_release_curve(CurveType::Exponential);

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

    pub fn set_frequency_mod_amount(&mut self, amount: f32) {
        self.frequency_mod_amount = amount;
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
}

impl AudioGenerator for KickDrum {
    fn next_sample(&mut self) -> f32 {
        if !self.is_active() {
            return 0.0;
        }

        let amp_env = self.amp_envelope.next_sample();
        let freq_env = self.freq_envelope.next_sample();

        let current_freq = self.base_frequency + (freq_env * self.frequency_mod_amount);
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
        snare.amp_envelope.set_attack_curve(CurveType::Linear);
        snare.amp_envelope.set_release_curve(CurveType::Exponential);

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
