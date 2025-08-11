use crate::audio::envelopes::Segment;
use crate::audio::filters::{FilterMode, SVF};
use crate::audio::oscillators::NoiseGenerator;
use crate::audio::{AudioGenerator, AudioProcessor};

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
}

impl AudioGenerator for ClapDrum {
    fn next_sample(&mut self) -> f32 {
        if !self.is_active() {
            return 0.0;
        }

        // Update the multi-segment envelope
        self.update_envelope();

        // Generate noise and process through three bandpass filters
        let noise = self.noise_generator.next_sample();

        // Process through all three bandpass filters and sum
        let filtered_1320 = self.filter_1320.process(noise);
        let filtered_1100 = self.filter_1100.process(noise);
        let filtered_1420 = self.filter_1420.process(noise);

        // Sum the filtered signals and apply 10dB gain (10.dbamp ≈ 3.16)
        let filtered_sum = (filtered_1320 + filtered_1100 + filtered_1420) * 3.16;

        // Apply envelope and tanh saturation
        (filtered_sum * self.envelope_value).tanh() * self.gain
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

