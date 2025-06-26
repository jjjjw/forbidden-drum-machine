use crate::audio::delays::DelayLine;
use crate::audio::filters::{OnePoleFilter, OnePoleMode};
use crate::audio::oscillators::SineOscillator;
use crate::audio::{AudioGenerator, AudioProcessor, StereoAudioProcessor};

// Fast Hadamard Transform for 8x8
fn fast_hadamard_transform_8(signals: &mut [f32; 8]) {
    // Stage 1: 8 -> 4 blocks
    let mut temp = [0.0f32; 8];
    for i in 0..4 {
        temp[i] = signals[i] + signals[i + 4];
        temp[i + 4] = signals[i] - signals[i + 4];
    }
    *signals = temp;

    // Stage 2: 4 -> 2 blocks
    for i in 0..2 {
        temp[i] = signals[i] + signals[i + 2];
        temp[i + 2] = signals[i] - signals[i + 2];
        temp[i + 4] = signals[i + 4] + signals[i + 6];
        temp[i + 6] = signals[i + 4] - signals[i + 6];
    }
    *signals = temp;

    // Stage 3: 2 -> 1 blocks
    for i in 0..4 {
        let base = i * 2;
        temp[base] = signals[base] + signals[base + 1];
        temp[base + 1] = signals[base] - signals[base + 1];
    }
    *signals = temp;

    // Normalize by 1/sqrt(8) for energy conservation
    let scale = 1.0 / (8.0f32).sqrt();
    for i in 0..8 {
        signals[i] *= scale;
    }
}

// Householder transform for feedback stage mixing
fn householder_transform_8(signals: &mut [f32; 8]) {
    // Use Householder reflection with vector v = [1, 1, 1, 1, 1, 1, 1, 1]
    // H = I - 2vv^T / |v|^2
    let sum: f32 = signals.iter().sum();
    let reflection_coeff = 2.0 / 8.0; // 2 * |v|^2 / |v|^2 where |v|^2 = 8

    for i in 0..8 {
        signals[i] = signals[i] - reflection_coeff * sum;
    }

    // Energy conservation normalization
    let scale = 1.0 / (8.0f32).sqrt();
    for i in 0..8 {
        signals[i] *= scale;
    }
}

pub struct DiffusionStage {
    delay_lines: [DelayLine; 8],
    flip_polarity: [bool; 8],
}

impl DiffusionStage {
    pub fn new(min_delay_seconds: f32, max_delay_seconds: f32, sample_rate: f32) -> Self {
        let mut flip_polarity = [false; 8];
        let mut delay_lines = Vec::new();

        // Calculate segment size
        let total_range = max_delay_seconds - min_delay_seconds;
        let segment_size = total_range / 8.0;

        // Divide range into 8 equal segments, one channel per segment
        for c in 0..8 {
            let segment_start = min_delay_seconds + (c as f32 * segment_size);
            let segment_end = segment_start + segment_size;

            // Convert to microseconds for integer random generation
            let segment_start_us = (segment_start * 1_000_000.0) as i32;
            let segment_end_us = (segment_end * 1_000_000.0) as i32;

            let random_delay_us = fastrand::i32(segment_start_us..segment_end_us) as f32;
            let delay_seconds = random_delay_us / 1_000_000.0; // Convert back to seconds

            let mut delay_line = DelayLine::new(delay_seconds, sample_rate);
            delay_line.set_delay_seconds(delay_seconds);
            delay_lines.push(delay_line);
            flip_polarity[c] = fastrand::bool();
        }

        Self {
            delay_lines: [
                delay_lines.pop().unwrap(),
                delay_lines.pop().unwrap(),
                delay_lines.pop().unwrap(),
                delay_lines.pop().unwrap(),
                delay_lines.pop().unwrap(),
                delay_lines.pop().unwrap(),
                delay_lines.pop().unwrap(),
                delay_lines.pop().unwrap(),
            ],
            flip_polarity,
        }
    }

    pub fn process(&mut self, input: [f32; 8]) -> [f32; 8] {
        // Delay all channels
        let mut delayed = [0.0f32; 8];
        for i in 0..8 {
            delayed[i] = self.delay_lines[i].process(input[i]);
        }

        // Apply Hadamard transform
        fast_hadamard_transform_8(&mut delayed);

        // Flip polarities based on random values
        for i in 0..8 {
            if self.flip_polarity[i] {
                delayed[i] = -delayed[i];
            }
        }

        delayed
    }
}

pub struct FeedbackStage {
    base_delays: [f32; 8],
    delay_lines: [DelayLine; 8],
    lfos: [SineOscillator; 4],
    feedback: f32,
    modulation_depth: f32,
    size: f32,
}

impl FeedbackStage {
    pub fn new(min_delay_seconds: f32, max_delay_seconds: f32, sample_rate: f32) -> Self {
        let mut delay_lines = Vec::new();
        let mut base_delays = [0f32; 8];

        // Create 8 delay lines with exponential distribution between min and max
        for c in 0..8 {
            let r = (c as f32) / 7.0; // 0 to 1 over 8 channels (0/7 to 7/7)
            let delay_seconds = min_delay_seconds * (max_delay_seconds / min_delay_seconds).powf(r);
            delay_lines.push(DelayLine::new(delay_seconds * 2.5, sample_rate));
            base_delays[c] = delay_seconds; // Store in seconds
        }

        // Create 4 LFOs with different frequencies
        let lfos = [
            SineOscillator::new(0.19, sample_rate),
            SineOscillator::new(0.37, sample_rate),
            SineOscillator::new(0.29, sample_rate),
            SineOscillator::new(0.41, sample_rate),
        ];

        Self {
            base_delays,
            delay_lines: [
                delay_lines.pop().unwrap(),
                delay_lines.pop().unwrap(),
                delay_lines.pop().unwrap(),
                delay_lines.pop().unwrap(),
                delay_lines.pop().unwrap(),
                delay_lines.pop().unwrap(),
                delay_lines.pop().unwrap(),
                delay_lines.pop().unwrap(),
            ],
            lfos,
            feedback: 0.5,
            modulation_depth: 0.0,
            size: 1.0,
        }
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 1.0);
        for i in 0..8 {
            self.delay_lines[i].set_feedback(self.feedback);
        }
    }

    pub fn set_modulation_depth(&mut self, depth: f32) {
        self.modulation_depth = depth.clamp(0.0, 1.0);
    }

    pub fn set_size(&mut self, size: f32) {
        self.size = size.clamp(0.1, 2.0);
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        for lfo in &mut self.lfos {
            lfo.set_sample_rate(sample_rate);
        }
    }

    pub fn process(&mut self, diffusion: [f32; 8]) -> [f32; 8] {
        // Generate LFO values (4 LFOs shared across 8 delays)
        let lfo_values = [
            self.lfos[0].next_sample(),
            self.lfos[1].next_sample(),
            self.lfos[2].next_sample(),
            self.lfos[3].next_sample(),
        ];

        // Apply LFO modulation to delay times (cycle through the 4 LFOs)
        for i in 0..8 {
            let lfo_value = lfo_values[i % 4];
            let modulated_delay =
                self.base_delays[i] * self.size * (1.0 + lfo_value * self.modulation_depth * 0.1);
            self.delay_lines[i].set_delay_seconds(modulated_delay);
        }

        // Read current echoes from delay lines
        let mut echoes = [0.0f32; 8];
        for i in 0..8 {
            echoes[i] = self.delay_lines[i].read();
        }

        // Apply Hadamard transform (TODO: Householder didn't work)
        fast_hadamard_transform_8(&mut echoes);

        // Write diffusion input to delay lines with echoes feedback
        for i in 0..8 {
            self.delay_lines[i].write(diffusion[i], echoes[i]);
        }

        echoes
    }
}

pub struct FDNReverb {
    input_highcut_left: OnePoleFilter,
    input_lowcut_left: OnePoleFilter,
    input_highcut_right: OnePoleFilter,
    input_lowcut_right: OnePoleFilter,

    // 4 diffusion stages with specified delay times
    diffusion_stages: [DiffusionStage; 4],

    // Feedback stage for late reverberation
    feedback_stage: FeedbackStage,
}

impl FDNReverb {
    pub fn new(sample_rate: f32) -> Self {
        let feedback_stage = FeedbackStage::new(0.1, 0.2, sample_rate); // 100-200ms range

        // Create 4 diffusion stages with delay times: 10-100ms
        let diffusion_stages = [
            DiffusionStage::new(0.01, 0.1, sample_rate),
            DiffusionStage::new(0.01, 0.1, sample_rate),
            DiffusionStage::new(0.01, 0.1, sample_rate),
            DiffusionStage::new(0.01, 0.1, sample_rate),
        ];

        Self {
            input_highcut_left: OnePoleFilter::new(10000.0, OnePoleMode::Lowpass, sample_rate),
            input_lowcut_left: OnePoleFilter::new(200.0, OnePoleMode::Highpass, sample_rate),
            input_highcut_right: OnePoleFilter::new(10000.0, OnePoleMode::Lowpass, sample_rate),
            input_lowcut_right: OnePoleFilter::new(200.0, OnePoleMode::Highpass, sample_rate),
            diffusion_stages,
            feedback_stage,
        }
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback_stage.set_feedback(feedback);
    }

    pub fn set_size(&mut self, size: f32) {
        self.feedback_stage.set_size(size);
    }

    pub fn set_modulation_depth(&mut self, depth: f32) {
        self.feedback_stage.set_modulation_depth(depth);
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.feedback_stage.set_sample_rate(sample_rate);
    }
}

impl StereoAudioProcessor for FDNReverb {
    fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32) {
        // Input filtering and scaling
        let filtered_left = self
            .input_highcut_left
            .process(self.input_lowcut_left.process(left * 0.5));
        let filtered_right = self
            .input_highcut_right
            .process(self.input_lowcut_right.process(right * 0.5));

        // Distribute input across 8 channels
        let mut reflections = [0.0f32; 8];
        for i in 0..4 {
            reflections[i * 2] = filtered_left;
            reflections[i * 2 + 1] = filtered_right;
        }

        // Process through 4 diffusion stages
        for stage in &mut self.diffusion_stages {
            reflections = stage.process(reflections);
        }

        // Process through feedback stage
        let echoes = self.feedback_stage.process(reflections);

        // Mix down to stereo - combine odd/even channels and add reflections
        let mut out_left = 0.0;
        let mut out_right = 0.0;
        for i in 0..4 {
            out_left += echoes[i * 2] + reflections[i * 2];
            out_right += echoes[i * 2 + 1] + reflections[i * 2 + 1];
        }

        (out_left, out_right)
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.set_sample_rate(sample_rate);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fdn_reverb_basic_operation() {
        let sample_rate = 44100.0;
        let mut reverb = FDNReverb::new(sample_rate);
        reverb.set_size(1.0); // Initialize delay times

        // Test silence
        let (out_l, out_r) = reverb.process_stereo(0.0, 0.0);
        assert_eq!(out_l, 0.0);
        assert_eq!(out_r, 0.0);

        // Test impulse response
        let (impulse_l, impulse_r) = reverb.process_stereo(1.0, 0.5);

        let mut max_amp_l = 0.0f32;
        let mut max_amp_r = 0.0f32;
        let mut outputs_l = Vec::new();
        let mut outputs_r = Vec::new();

        // Process silence to hear reverb tail
        for _ in 0..(0.2 * sample_rate) as usize {
            let (out_l, out_r) = reverb.process_stereo(0.0, 0.0);
            outputs_l.push(out_l);
            outputs_r.push(out_r);
            max_amp_l = max_amp_l.max(out_l.abs());
            max_amp_r = max_amp_r.max(out_r.abs());
        }

        println!(
            "FDNReverb test: impulse output L={}, R={}",
            impulse_l, impulse_r
        );
        println!(
            "FDNReverb test: max tail amplitude L={}, R={}",
            max_amp_l, max_amp_r
        );

        // Reverb should be stable
        assert!(max_amp_l < 1.0, "FDNReverb left should remain stable");
        assert!(max_amp_r < 1.0, "FDNReverb right should remain stable");

        // Should produce reverb tail
        let has_tail_l = outputs_l.iter().any(|&x| x.abs() > 0.1);
        let has_tail_r = outputs_r.iter().any(|&x| x.abs() > 0.1);
        assert!(has_tail_l, "FDNReverb should produce left reverb tail");
        assert!(has_tail_r, "FDNReverb should produce right reverb tail");
    }

    #[test]
    fn test_fdn_reverb_modulation() {
        let sample_rate = 44100.0;
        let mut reverb = FDNReverb::new(sample_rate);
        reverb.set_size(1.0);
        reverb.set_modulation_depth(1.0); // Full modulation

        // Process impulse and capture modulated reverb tail
        let _impulse = reverb.process_stereo(1.0, 0.5);

        let mut outputs_l = Vec::new();
        let mut outputs_r = Vec::new();

        // Process samples to hear modulated reverb tail
        for _ in 0..(0.5 * sample_rate) as usize {
            let (out_l, out_r) = reverb.process_stereo(0.0, 0.0);
            outputs_l.push(out_l);
            outputs_r.push(out_r);
        }

        // Should produce stable modulated reverb
        let max_amp_l = outputs_l.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
        let max_amp_r = outputs_r.iter().map(|x| x.abs()).fold(0.0f32, f32::max);

        assert!(
            max_amp_l < 2.0,
            "Modulated FDNReverb left should remain stable"
        );
        assert!(
            max_amp_r < 2.0,
            "Modulated FDNReverb right should remain stable"
        );

        // Should still produce reverb tail
        let has_tail_l = outputs_l.iter().any(|&x| x.abs() > 0.01);
        let has_tail_r = outputs_r.iter().any(|&x| x.abs() > 0.01);
        assert!(
            has_tail_l,
            "Modulated FDNReverb should produce left reverb tail"
        );
        assert!(
            has_tail_r,
            "Modulated FDNReverb should produce right reverb tail"
        );

        println!(
            "Modulated FDNReverb test: max tail amplitude L={}, R={}",
            max_amp_l, max_amp_r
        );
    }

    #[test]
    fn test_fast_hadamard_transform_8_energy_conservation() {
        // Test that the energy is conserved when applying the 8x8 transform
        let test_inputs = [
            [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
            [0.5; 8],
            [1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0],
        ];

        for test_input in test_inputs.iter() {
            let mut signals = *test_input;

            // Calculate input energy
            let input_energy: f32 = signals.iter().map(|x| x * x).sum();

            // Apply transform
            fast_hadamard_transform_8(&mut signals);

            // Calculate output energy
            let output_energy: f32 = signals.iter().map(|x| x * x).sum();

            assert!(
                (input_energy - output_energy).abs() < 1e-4,
                "Energy not conserved: input={}, output={}, diff={}",
                input_energy,
                output_energy,
                (input_energy - output_energy).abs()
            );
        }
    }

    #[test]
    fn test_fast_hadamard_transform_8_invertability() {
        let original = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let mut signals = original;

        // Apply transform twice
        fast_hadamard_transform_8(&mut signals);
        fast_hadamard_transform_8(&mut signals);

        for (i, (&result, &orig)) in signals.iter().zip(original.iter()).enumerate() {
            assert!(
                (result - orig).abs() < 1e-6,
                "Invertibility test failed at index {}: expected {}, got {}",
                i,
                orig,
                result
            );
        }
    }
}
