use crate::audio::envelopes::AREnvelope;
use crate::audio::filters::{FilterMode, SVF};
use crate::audio::oscillators::SawOscillator;
use crate::audio::{AudioGenerator, AudioProcessor, StereoAudioGenerator};

/// Supersaw oscillator using multiple detuned saw oscillators
/// Generates stereo output with voices panned across the stereo field
pub struct SupersawOscillator {
    oscillators: Vec<SawOscillator>,
    base_frequency: f32,
    detune: f32,
    gain: f32,
    num_voices: usize,
    stereo_width: f32,
}

impl SupersawOscillator {
    pub fn new(frequency: f32, sample_rate: f32, num_voices: usize) -> Self {
        let num_voices = num_voices.clamp(1, 16);

        let mut oscillators = Vec::with_capacity(num_voices);

        for _ in 0..num_voices {
            oscillators.push(SawOscillator::new(frequency, sample_rate));
        }

        let mut supersaw = Self {
            oscillators,
            base_frequency: frequency,
            detune: 1.0,
            gain: 1.0 / num_voices as f32,
            num_voices,
            stereo_width: 0.8,
        };

        supersaw.update_frequencies();
        supersaw
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.base_frequency = frequency;
        self.update_frequencies();
    }

    pub fn set_detune(&mut self, detune: f32) {
        self.detune = detune.clamp(0.0, 2.0);
        self.update_frequencies();
    }

    pub fn set_stereo_width(&mut self, width: f32) {
        self.stereo_width = width.clamp(0.0, 1.0);
    }

    fn update_frequencies(&mut self) {
        for (i, osc) in self.oscillators.iter_mut().enumerate() {
            if i == 0 && self.num_voices > 1 {
                osc.set_frequency(self.base_frequency);
            } else {
                let voice_detune = if self.num_voices == 1 {
                    0.0
                } else {
                    let detune_cents =
                        (i as f32 * 7.0 * self.detune) * if i % 2 == 1 { 1.0 } else { -1.0 };
                    detune_cents
                };
                let detune_ratio = 2.0_f32.powf(voice_detune / 1200.0);
                osc.set_frequency(self.base_frequency * detune_ratio);
            }
        }
    }

    pub fn reset(&mut self) {
        for osc in &mut self.oscillators {
            osc.reset();
        }
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        for osc in &mut self.oscillators {
            osc.set_sample_rate(sample_rate);
        }
    }
}

impl StereoAudioGenerator for SupersawOscillator {
    fn next_sample(&mut self) -> (f32, f32) {
        let mut left = 0.0;
        let mut right = 0.0;

        for (i, osc) in self.oscillators.iter_mut().enumerate() {
            let sample = osc.next_sample();

            // Pan voices across stereo field
            let pan = if self.num_voices == 1 {
                0.5 // Center for single voice
            } else {
                (i as f32) / ((self.num_voices - 1) as f32)
            };

            // Apply stereo width
            let adjusted_pan = 0.5 + (pan - 0.5) * self.stereo_width;

            // Equal power panning
            let pan_radians = adjusted_pan * std::f32::consts::PI * 0.5;
            let left_gain = pan_radians.cos();
            let right_gain = pan_radians.sin();

            left += sample * left_gain * self.gain;
            right += sample * right_gain * self.gain;
        }

        (left, right)
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.set_sample_rate(sample_rate);
    }
}

pub struct SupersawSynth {
    oscillator: SupersawOscillator,
    filter_left: SVF,
    filter_right: SVF,
    amp_envelope: AREnvelope,
    filter_envelope: AREnvelope,

    base_frequency: f32,
    gain: f32,
    filter_cutoff: f32,
    filter_resonance: f32,
    filter_env_amount: f32,
}

impl SupersawSynth {
    pub fn new(sample_rate: f32) -> Self {
        let mut amp_envelope = AREnvelope::new(sample_rate);
        amp_envelope.set_attack_time(0.01);
        amp_envelope.set_release_time(0.5);

        let mut filter_envelope = AREnvelope::new(sample_rate);
        filter_envelope.set_attack_time(0.3);
        filter_envelope.set_release_time(0.3);

        Self {
            oscillator: SupersawOscillator::new(440.0, sample_rate, 7),
            filter_left: SVF::new(1000.0, 0.7, FilterMode::Lowpass, sample_rate),
            filter_right: SVF::new(1000.0, 0.7, FilterMode::Lowpass, sample_rate),
            amp_envelope,
            filter_envelope,

            base_frequency: 440.0,
            gain: 0.5,
            filter_cutoff: 1000.0,
            filter_resonance: 0.7,
            filter_env_amount: 2000.0,
        }
    }

    pub fn trigger(&mut self) {
        if !self.amp_envelope.is_active() {
            self.oscillator.reset();
        }
        self.amp_envelope.trigger();
        self.filter_envelope.trigger();
    }

    pub fn set_base_frequency(&mut self, frequency: f32) {
        self.base_frequency = frequency;
        self.oscillator.set_frequency(frequency);
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain.clamp(0.0, 1.0);
    }

    pub fn set_detune(&mut self, detune: f32) {
        self.oscillator.set_detune(detune);
    }

    pub fn set_stereo_width(&mut self, width: f32) {
        self.oscillator.set_stereo_width(width);
    }

    pub fn set_filter_cutoff(&mut self, cutoff: f32) {
        self.filter_cutoff = cutoff.clamp(20.0, 20000.0);
        self.filter_left.set_cutoff_frequency(self.filter_cutoff);
        self.filter_right.set_cutoff_frequency(self.filter_cutoff);
    }

    pub fn set_filter_resonance(&mut self, resonance: f32) {
        self.filter_resonance = resonance.clamp(0.1, 10.0);
        self.filter_left.set_resonance(self.filter_resonance);
        self.filter_right.set_resonance(self.filter_resonance);
    }

    pub fn set_filter_env_amount(&mut self, amount: f32) {
        self.filter_env_amount = amount;
    }

    pub fn set_amp_attack(&mut self, attack: f32) {
        self.amp_envelope.set_attack_time(attack);
    }

    pub fn set_amp_release(&mut self, release: f32) {
        self.amp_envelope.set_release_time(release);
    }

    pub fn set_filter_attack(&mut self, attack: f32) {
        self.filter_envelope.set_attack_time(attack);
    }

    pub fn set_filter_release(&mut self, release: f32) {
        self.filter_envelope.set_release_time(release);
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.oscillator.set_sample_rate(sample_rate);
        self.filter_left.set_sample_rate(sample_rate);
        self.filter_right.set_sample_rate(sample_rate);
        self.amp_envelope.set_sample_rate(sample_rate);
        self.filter_envelope.set_sample_rate(sample_rate);
    }
}

impl StereoAudioGenerator for SupersawSynth {
    fn next_sample(&mut self) -> (f32, f32) {
        if !self.amp_envelope.is_active() {
            return (0.0, 0.0);
        }

        let (osc_left, osc_right) = self.oscillator.next_sample();
        let amp_env = self.amp_envelope.next_sample();
        let filter_env = self.filter_envelope.next_sample();

        // Modulate filter cutoff with envelope
        let modulated_cutoff = self.filter_cutoff + (filter_env * self.filter_env_amount);
        self.filter_left.set_cutoff_frequency(modulated_cutoff);
        self.filter_right.set_cutoff_frequency(modulated_cutoff);

        // Process through filters
        let filtered_left = self.filter_left.process(osc_left);
        let filtered_right = self.filter_right.process(osc_right);

        // Apply amplitude envelope and gain
        let final_left = filtered_left * amp_env * self.gain;
        let final_right = filtered_right * amp_env * self.gain;

        (final_left.tanh(), final_right.tanh())
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.set_sample_rate(sample_rate);
    }
}
