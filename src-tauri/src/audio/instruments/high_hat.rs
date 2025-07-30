use crate::audio::envelopes::AREnvelope;
use crate::audio::filters::{FilterMode, SVF};
use crate::audio::oscillators::NoiseGenerator;
use crate::audio::{AudioGenerator, AudioNode, AudioProcessor};
use crate::events::NodeEvent;

pub struct HiHat {
    noise_generator: NoiseGenerator,

    // Three bandpass filters at different frequencies
    filter_7500: SVF,
    filter_7000: SVF,
    filter_8000: SVF,

    // Amplitude envelope
    amp_envelope: AREnvelope,

    // Parameters
    length: f32,
    gain: f32,
}

impl HiHat {
    pub fn new(sample_rate: f32) -> Self {
        let mut hihat = Self {
            noise_generator: NoiseGenerator::new(),

            // Bandpass filters with Q corresponding to bandwidth of 0.3
            // Q ≈ center_freq / bandwidth, so for BW=0.3*center_freq, Q≈3.33
            filter_7500: SVF::new(7500.0, 3.33, FilterMode::Bandpass, sample_rate),
            filter_7000: SVF::new(7000.0, 3.33, FilterMode::Bandpass, sample_rate),
            filter_8000: SVF::new(8000.0, 3.33, FilterMode::Bandpass, sample_rate),

            amp_envelope: AREnvelope::new(sample_rate),

            length: 0.05, // 50ms default
            gain: 1.0,
        };

        // Set up percussive envelope
        hihat.amp_envelope.set_attack_time(0.001); // 1ms attack
        hihat.amp_envelope.set_attack_bias(0.9); // Very fast attack
        hihat.update_release_time();

        hihat
    }

    pub fn trigger(&mut self) {
        self.amp_envelope.trigger();
    }

    pub fn set_length(&mut self, length: f32) {
        self.length = length.max(0.002); // Minimum 2ms
        self.update_release_time();
    }

    fn update_release_time(&mut self) {
        // Release time is length - attack time (1ms)
        self.amp_envelope
            .set_release_time((self.length - 0.001).max(0.001));
        self.amp_envelope.set_release_bias(0.7); // Exponential decay
    }

    pub fn is_active(&self) -> bool {
        self.amp_envelope.is_active()
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }
}

impl AudioGenerator for HiHat {
    fn next_sample(&mut self) -> f32 {
        if !self.is_active() {
            return 0.0;
        }

        // Generate hash noise
        let noise = self.noise_generator.next_sample();

        // Process through three bandpass filters
        let filtered_7500 = self.filter_7500.process(noise);
        let filtered_7000 = self.filter_7000.process(noise);
        let filtered_8000 = self.filter_8000.process(noise);

        // Sum the filtered signals
        let filtered_sum = filtered_7500 + filtered_7000 + filtered_8000;

        // Apply tanh saturation and scale by 0.33
        let saturated = filtered_sum.tanh() * 0.33;

        // Apply envelope
        let amp_env = self.amp_envelope.next_sample();
        saturated * amp_env
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.noise_generator.set_sample_rate(sample_rate);
        self.filter_7500.set_sample_rate(sample_rate);
        self.filter_7000.set_sample_rate(sample_rate);
        self.filter_8000.set_sample_rate(sample_rate);
        self.amp_envelope.set_sample_rate(sample_rate);
    }
}

impl AudioNode for HiHat {
    fn process(&mut self, left_in: f32, right_in: f32) -> (f32, f32) {
        let hihat_sample = self.next_sample() * self.gain;
        (left_in + hihat_sample, right_in + hihat_sample)
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
            NodeEvent::SetAmpRelease(time) => {
                // Interpret amp release as length for hi-hat
                self.set_length(time);
                Ok(())
            }
            _ => Err(format!("Unsupported event for HiHat: {:?}", event)),
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        AudioGenerator::set_sample_rate(self, sample_rate);
    }
}
