use crate::audio::delays::DelayLine;
use crate::audio::filters::{OnePoleFilter, OnePoleMode};
use crate::audio::oscillators::SineOscillator;
use crate::audio::{AudioGenerator, AudioProcessor, StereoAudioProcessor};

// Fast Hadamard Transform for 16x16 FDN
fn fast_hadamard_transform_16(signals: &mut [f32; 16]) {
    // Stage 1: 16 -> 8 blocks
    let mut temp = [0.0f32; 16];
    for i in 0..8 {
        temp[i] = signals[i] + signals[i + 8];
        temp[i + 8] = signals[i] - signals[i + 8];
    }
    *signals = temp;

    // Stage 2: 8 -> 4 blocks
    for i in 0..4 {
        temp[i] = signals[i] + signals[i + 4];
        temp[i + 4] = signals[i] - signals[i + 4];
        temp[i + 8] = signals[i + 8] + signals[i + 12];
        temp[i + 12] = signals[i + 8] - signals[i + 12];
    }
    *signals = temp;

    // Stage 3: 4 -> 2 blocks
    for i in 0..2 {
        temp[i] = signals[i] + signals[i + 2];
        temp[i + 2] = signals[i] - signals[i + 2];
        temp[i + 4] = signals[i + 4] + signals[i + 6];
        temp[i + 6] = signals[i + 4] - signals[i + 6];
        temp[i + 8] = signals[i + 8] + signals[i + 10];
        temp[i + 10] = signals[i + 8] - signals[i + 10];
        temp[i + 12] = signals[i + 12] + signals[i + 14];
        temp[i + 14] = signals[i + 12] - signals[i + 14];
    }
    *signals = temp;

    // Stage 4: 2 -> 1 blocks
    for i in 0..8 {
        let base = i * 2;
        temp[base] = signals[base] + signals[base + 1];
        temp[base + 1] = signals[base] - signals[base + 1];
    }
    *signals = temp;

    // Normalize by 1/sqrt(16) for energy conservation
    let scale = 1.0 / (16.0f32).sqrt();
    for i in 0..16 {
        signals[i] *= scale;
    }
}

// Base delay multipliers for 16-delay feedback
const FEEDBACK_DELAYS: [f32; 16] = [
    1.0, 1.3, 1.7, 2.3, 2.9, 3.7, 4.3, 5.3, 6.1, 7.3, 8.9, 10.7, 12.3, 14.9, 17.9, 21.1,
];

pub struct FeedbackStage {
    base_delay: f32,
    delay_lines: [DelayLine; 16],
    lfos: [SineOscillator; 4],
    feedback: f32,
    modulation_depth: f32,
    size: f32,
}

impl FeedbackStage {
    pub fn new(base_delay_ms: f32, sample_rate: f32) -> Self {
        let base_delay = base_delay_ms / 1000.0; // Convert ms to seconds

        // Create 16 delay lines with different delay times
        let delay_lines = [
            DelayLine::new(FEEDBACK_DELAYS[0] * base_delay * 2.0, sample_rate),
            DelayLine::new(FEEDBACK_DELAYS[1] * base_delay * 2.0, sample_rate),
            DelayLine::new(FEEDBACK_DELAYS[2] * base_delay * 2.0, sample_rate),
            DelayLine::new(FEEDBACK_DELAYS[3] * base_delay * 2.0, sample_rate),
            DelayLine::new(FEEDBACK_DELAYS[4] * base_delay * 2.0, sample_rate),
            DelayLine::new(FEEDBACK_DELAYS[5] * base_delay * 2.0, sample_rate),
            DelayLine::new(FEEDBACK_DELAYS[6] * base_delay * 2.0, sample_rate),
            DelayLine::new(FEEDBACK_DELAYS[7] * base_delay * 2.0, sample_rate),
            DelayLine::new(FEEDBACK_DELAYS[8] * base_delay * 2.0, sample_rate),
            DelayLine::new(FEEDBACK_DELAYS[9] * base_delay * 2.0, sample_rate),
            DelayLine::new(FEEDBACK_DELAYS[10] * base_delay * 2.0, sample_rate),
            DelayLine::new(FEEDBACK_DELAYS[11] * base_delay * 2.0, sample_rate),
            DelayLine::new(FEEDBACK_DELAYS[12] * base_delay * 2.0, sample_rate),
            DelayLine::new(FEEDBACK_DELAYS[13] * base_delay * 2.0, sample_rate),
            DelayLine::new(FEEDBACK_DELAYS[14] * base_delay * 2.0, sample_rate),
            DelayLine::new(FEEDBACK_DELAYS[15] * base_delay * 2.0, sample_rate),
        ];

        // Create 4 LFOs with different frequencies
        let lfos = [
            SineOscillator::new(0.19, sample_rate),
            SineOscillator::new(0.37, sample_rate),
            SineOscillator::new(0.29, sample_rate),
            SineOscillator::new(0.41, sample_rate),
        ];

        Self {
            base_delay,
            delay_lines,
            lfos,
            feedback: 0.5,
            modulation_depth: 0.0,
            size: 1.0,
        }
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 1.0);
        for i in 0..16 {
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

    pub fn process(&mut self, diffusion: [f32; 16]) -> [f32; 16] {
        // Generate LFO values (4 LFOs shared across 16 delays)
        let lfo_values = [
            self.lfos[0].next_sample(),
            self.lfos[1].next_sample(),
            self.lfos[2].next_sample(),
            self.lfos[3].next_sample(),
        ];

        // Apply LFO modulation to delay times (cycle through the 4 LFOs)
        for i in 0..16 {
            let lfo_value = lfo_values[i % 4];
            let modulated_delay = FEEDBACK_DELAYS[i]
                * self.base_delay
                * self.size
                * (1.0 + lfo_value * self.modulation_depth * 0.1);
            self.delay_lines[i].set_delay_seconds(modulated_delay);
        }

        // Read current echoes from delay lines
        let mut echoes = [0.0f32; 16];
        for i in 0..16 {
            echoes[i] = self.delay_lines[i].read();
        }

        // Apply mixing matrix
        fast_hadamard_transform_16(&mut echoes);

        // Write diffusion input to delay lines with echoes feedback
        for i in 0..16 {
            self.delay_lines[i].write(diffusion[i], echoes[i]);
        }

        // Phase shift some of the echoes for better diffusion
        for i in 0..8 {
            echoes[i] *= -1.0;
        }

        echoes
    }
}

pub struct FDNReverb {
    input_highcut_left: OnePoleFilter,
    input_lowcut_left: OnePoleFilter,
    input_highcut_right: OnePoleFilter,
    input_lowcut_right: OnePoleFilter,

    // Feedback stage for late reverberation
    feedback_stage: FeedbackStage,
}

impl FDNReverb {
    pub fn new(sample_rate: f32) -> Self {
        let feedback_stage = FeedbackStage::new(10.0, sample_rate); // 10ms base delay

        Self {
            input_highcut_left: OnePoleFilter::new(10000.0, OnePoleMode::Lowpass, sample_rate),
            input_lowcut_left: OnePoleFilter::new(200.0, OnePoleMode::Highpass, sample_rate),
            input_highcut_right: OnePoleFilter::new(10000.0, OnePoleMode::Lowpass, sample_rate),
            input_lowcut_right: OnePoleFilter::new(200.0, OnePoleMode::Highpass, sample_rate),
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

        // Distribute input across 16 channels
        let mut input = [0.0f32; 16];
        for i in 0..8 {
            input[i * 2] = filtered_left;
            input[i * 2 + 1] = filtered_right;
        }

        // Process through feedback stage
        let echoes = self.feedback_stage.process(input);

        // Mix down to stereo - combine odd/even channels
        let mut out_left = 0.0;
        let mut out_right = 0.0;
        for i in 0..8 {
            out_left += echoes[i * 2];
            out_right += echoes[i * 2 + 1];
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
    fn test_fast_hadamard_transform_16_energy_conservation() {
        // Test that the energy is conserved when applying the 16x16 transform
        let test_inputs = [
            [
                1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0,
            ],
            [0.5; 16],
            [
                1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0,
            ],
        ];

        for test_input in test_inputs.iter() {
            let mut signals = *test_input;

            // Calculate input energy
            let input_energy: f32 = signals.iter().map(|x| x * x).sum();

            // Apply transform
            fast_hadamard_transform_16(&mut signals);

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
    fn test_fast_hadamard_transform_16_invertability() {
        let original = [
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
        ];
        let mut signals = original;

        // Apply transform twice
        fast_hadamard_transform_16(&mut signals);
        fast_hadamard_transform_16(&mut signals);

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
