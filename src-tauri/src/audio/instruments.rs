use crate::audio::envelopes::{AREnvelope, Segment};
use crate::audio::filters::{FilterMode, SVF};
use crate::audio::oscillators::{NoiseGenerator, SineOscillator};
use crate::audio::{AudioGenerator, AudioProcessor, AudioNode};

pub struct KickDrum {
    oscillator: SineOscillator,
    amp_envelope: AREnvelope,
    freq_envelope: AREnvelope,
    base_frequency: f32,
    frequency_mod_amount: f32,
    gain: f32,
}

impl KickDrum {
    pub fn new(sample_rate: f32) -> Self {
        let mut kick = Self {
            oscillator: SineOscillator::new(60.0, sample_rate),
            amp_envelope: AREnvelope::new(sample_rate),
            freq_envelope: AREnvelope::new(sample_rate),
            base_frequency: 60.0,
            frequency_mod_amount: 40.0,
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

impl AudioNode for KickDrum {
    fn process_stereo(&mut self, left_in: f32, right_in: f32) -> (f32, f32) {
        let drum_sample = self.next_sample() * self.gain;
        (left_in + drum_sample, right_in + drum_sample)
    }

    fn handle_event(&mut self, event_type: &str, parameter: f32) -> Result<(), String> {
        match event_type {
            "trigger" => {
                self.trigger();
                Ok(())
            }
            "set_gain" => {
                self.set_gain(parameter);
                Ok(())
            }
            "set_base_frequency" => {
                self.set_base_frequency(parameter);
                Ok(())
            }
            "set_frequency_mod_amount" => {
                self.set_frequency_mod_amount(parameter);
                Ok(())
            }
            "set_amp_attack" => {
                self.set_amp_attack(parameter);
                Ok(())
            }
            "set_amp_release" => {
                self.set_amp_release(parameter);
                Ok(())
            }
            "set_freq_attack" => {
                self.set_freq_attack(parameter);
                Ok(())
            }
            "set_freq_release" => {
                self.set_freq_release(parameter);
                Ok(())
            }
            _ => Err(format!("Unknown event type: {}", event_type))
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        AudioGenerator::set_sample_rate(self, sample_rate);
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

pub struct ClapDrum {
    noise_generator: NoiseGenerator,

    // Three bandpass filters at different frequencies
    filter_1320: SVF,
    filter_1100: SVF,
    filter_1420: SVF,

    // Multi-segment envelope using individual Segments
    // Pattern: [0, 1, 0, 1, 0, 1, 0] with randomized timing
    envelope_segments: [Segment; 6], // 6 segments for the 7-point envelope
    current_segment: usize,
    envelope_value: f32,
    is_envelope_active: bool,

    // For stereo output variation
    left_active: bool,
    right_active: bool,

    sample_rate: f32,
    gain: f32,
}

impl ClapDrum {
    pub fn new(sample_rate: f32) -> Self {
        // Create the multi-segment envelope with randomized timing
        // SuperCollider: [0, 1, 0, 1, 0, 1, 0] with durations [Rand(0.001, 0.01), 0.01, 0.001, 0.01, 0.001, 0.08]
        let envelope_segments = [
            Segment::new(0.0, 1.0, fastrand::f32() * 0.009 + 0.001, 0.9, sample_rate), // 0->1: 0.001-0.01s, fast attack
            Segment::new(1.0, 0.0, 0.01, 0.1, sample_rate), // 1->0: 0.01s, fast decay
            Segment::new(0.0, 1.0, 0.001, 0.9, sample_rate), // 0->1: 0.001s, fast attack
            Segment::new(1.0, 0.0, 0.01, 0.1, sample_rate), // 1->0: 0.01s, fast decay
            Segment::new(0.0, 1.0, 0.001, 0.9, sample_rate), // 0->1: 0.001s, fast attack
            Segment::new(1.0, 0.0, 0.08, 0.3, sample_rate), // 1->0: 0.08s, slow final decay
        ];

        Self {
            noise_generator: NoiseGenerator::new(),

            filter_1320: SVF::new(1320.0, 10.0, FilterMode::Bandpass, sample_rate), // Q=10 for narrow band
            filter_1100: SVF::new(1100.0, 10.0, FilterMode::Bandpass, sample_rate),
            filter_1420: SVF::new(1420.0, 10.0, FilterMode::Bandpass, sample_rate),

            envelope_segments,
            current_segment: 0,
            envelope_value: 0.0,
            is_envelope_active: false,
            left_active: true,  // Left channel gets full envelope
            right_active: true, // Right channel also active but could be varied

            sample_rate,
            gain: 1.0,
        }
    }

    pub fn trigger(&mut self) {
        // Randomize the first segment timing (like SuperCollider Rand)
        self.envelope_segments[0].set_duration_seconds(fastrand::f32() * 0.009 + 0.001);

        // Start the envelope sequence
        self.current_segment = 0;
        self.envelope_value = 0.0;
        self.is_envelope_active = true;
        self.envelope_segments[0].trigger();

        // Randomly vary left/right activity (mimicking SuperCollider's different left/right envelopes)
        self.left_active = true;
        self.right_active = fastrand::bool(); // Sometimes right channel is different
    }

    pub fn is_active(&self) -> bool {
        self.is_envelope_active
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }

    fn update_envelope(&mut self) {
        if !self.is_envelope_active {
            self.envelope_value = 0.0;
            return;
        }

        if self.current_segment >= self.envelope_segments.len() {
            self.is_envelope_active = false;
            self.envelope_value = 0.0;
            return;
        }

        // Get current segment value
        if self.envelope_segments[self.current_segment].is_active() {
            self.envelope_value = self.envelope_segments[self.current_segment].next_sample();
        } else if self.envelope_segments[self.current_segment].is_finished() {
            // Move to next segment
            self.current_segment += 1;
            if self.current_segment < self.envelope_segments.len() {
                self.envelope_segments[self.current_segment].trigger();
                self.envelope_value = self.envelope_segments[self.current_segment].next_sample();
            } else {
                self.is_envelope_active = false;
                self.envelope_value = 0.0;
            }
        }
    }

    pub fn next_sample_stereo(&mut self) -> (f32, f32) {
        if !self.is_active() {
            return (0.0, 0.0);
        }

        // Update the multi-segment envelope
        self.update_envelope();

        // Generate noise and process through three bandpass filters
        let noise = self.noise_generator.next_sample();

        // Process through all three bandpass filters and sum (like SuperCollider)
        let filtered_1320 = self.filter_1320.process(noise);
        let filtered_1100 = self.filter_1100.process(noise);
        let filtered_1420 = self.filter_1420.process(noise);

        // Sum the filtered signals and apply 10dB gain (10.dbamp ≈ 3.16)
        let filtered_sum = (filtered_1320 + filtered_1100 + filtered_1420) * 3.16;

        // Apply envelope and create stereo output
        let left = if self.left_active {
            filtered_sum * self.envelope_value
        } else {
            0.0
        };

        let right = if self.right_active {
            filtered_sum * self.envelope_value
        } else {
            0.0
        };

        // Apply tanh saturation like SuperCollider
        (left.tanh(), right.tanh())
    }
}

impl AudioGenerator for ClapDrum {
    fn next_sample(&mut self) -> f32 {
        let (left, right) = self.next_sample_stereo();
        (left + right) * 0.5 // Mono mix
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.noise_generator.set_sample_rate(sample_rate);
        self.filter_1320.set_sample_rate(sample_rate);
        self.filter_1100.set_sample_rate(sample_rate);
        self.filter_1420.set_sample_rate(sample_rate);

        // Update all envelope segments
        for segment in &mut self.envelope_segments {
            segment.set_sample_rate(sample_rate);
        }
    }
}

impl AudioNode for ClapDrum {
    fn process_stereo(&mut self, left_in: f32, right_in: f32) -> (f32, f32) {
        let (clap_left, clap_right) = self.next_sample_stereo();
        (left_in + clap_left * self.gain, right_in + clap_right * self.gain)
    }

    fn handle_event(&mut self, event_type: &str, parameter: f32) -> Result<(), String> {
        match event_type {
            "trigger" => {
                self.trigger();
                Ok(())
            }
            "set_gain" => {
                self.set_gain(parameter);
                Ok(())
            }
            _ => Err(format!("Unknown event type: {}", event_type))
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        AudioGenerator::set_sample_rate(self, sample_rate);
    }
}
