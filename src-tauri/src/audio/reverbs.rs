use crate::audio::delays::DelayLine;
use crate::audio::filters::{OnePoleFilter, OnePoleMode};
use crate::audio::oscillators::SineOscillator;
use crate::audio::{AudioGenerator, AudioProcessor, StereoAudioProcessor};

// Fast Hadamard Transform for 4x4 FDN
fn fast_hadamard_transform_4(signals: &mut [f32; 4]) {
    // Stage 1: 4 -> 2 blocks
    let mut temp = [0.0f32; 4];
    for i in 0..2 {
        temp[i] = signals[i] + signals[i + 2];
        temp[i + 2] = signals[i] - signals[i + 2];
    }
    *signals = temp;

    // Stage 2: 2 -> 1 blocks
    temp[0] = signals[0] + signals[1];
    temp[1] = signals[0] - signals[1];
    temp[2] = signals[2] + signals[3];
    temp[3] = signals[2] - signals[3];
    *signals = temp;

    // Normalize by 1/sqrt(4) for energy conservation
    let scale = 1.0 / (4.0f32).sqrt();
    for i in 0..4 {
        signals[i] *= scale;
    }
}

// Base delay multipliers for diffusion
// Base delay multipliers for feedback
const FEEDBACK_DELAYS: [f32; 4] = [1.0, 1.3, 1.5, 1.7];

pub struct FeedbackStage {
    base_delay: f32,
    delay_lines: [DelayLine; 4],
    lfos: [SineOscillator; 4],
    feedback: f32,
    modulation_depth: f32,
    size: f32,
}

impl FeedbackStage {
    pub fn new(base_delay_ms: f32) -> Self {
        let base_delay = base_delay_ms / 1000.0; // Convert ms to seconds
        let mut delay_lines = [
            DelayLine::new(FEEDBACK_DELAYS[0] * base_delay * 2.0),
            DelayLine::new(FEEDBACK_DELAYS[1] * base_delay * 2.0),
            DelayLine::new(FEEDBACK_DELAYS[2] * base_delay * 2.0),
            DelayLine::new(FEEDBACK_DELAYS[3] * base_delay * 2.0),
        ];

        for i in 0..4 {
            delay_lines[i].set_feedback(0.5);
            delay_lines[i].set_delay_seconds(FEEDBACK_DELAYS[i] * base_delay);
        }

        let lfos = [
            SineOscillator::new(0.19),
            SineOscillator::new(0.37),
            SineOscillator::new(0.29),
            SineOscillator::new(0.41),
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
        for i in 0..4 {
            self.delay_lines[i].set_feedback(self.feedback);
        }
    }

    pub fn set_modulation_depth(&mut self, depth: f32) {
        self.modulation_depth = depth.clamp(0.0, 1.0);
    }

    pub fn set_size(&mut self, size: f32) {
        self.size = size.clamp(0.1, 2.0);
    }

    pub fn process(&mut self, diffusion: [f32; 4]) -> [f32; 4] {
        // Apply LFO modulation to delay times
        for i in 0..4 {
            let lfo_value = self.lfos[i].next_sample();
            let modulated_delay = FEEDBACK_DELAYS[i]
                * self.base_delay
                * self.size
                * (1.0 + lfo_value * self.modulation_depth * 0.1);
            self.delay_lines[i].set_delay_seconds(modulated_delay);
        }

        // Read current echoes from delay lines
        let mut echoes = [0.0f32; 4];
        for i in 0..4 {
            echoes[i] = self.delay_lines[i].read();
        }

        // Apply mixing matrix
        fast_hadamard_transform_4(&mut echoes);

        // Write diffusion input to delay lines with echoes feedback
        for i in 0..4 {
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

    // Feedback stage for late reverberation
    feedback_stage: FeedbackStage,
}

impl FDNReverb {
    pub fn new() -> Self {
        let feedback_stage = FeedbackStage::new(50.0); // 50ms base delay

        Self {
            input_highcut_left: OnePoleFilter::new(10000.0, OnePoleMode::Lowpass),
            input_lowcut_left: OnePoleFilter::new(200.0, OnePoleMode::Highpass),
            input_highcut_right: OnePoleFilter::new(10000.0, OnePoleMode::Lowpass),
            input_lowcut_right: OnePoleFilter::new(200.0, OnePoleMode::Highpass),
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

        let input = [filtered_left, filtered_right, filtered_left, filtered_right];

        // Use the final diffusion stage output as input to feedback stage
        let echoes = self.feedback_stage.process(input);
        let out_left = echoes[0] + echoes[2];
        let out_right = echoes[1] + echoes[3];

        (out_left, out_right)
    }
}

#[cfg(test)]
mod tests {
    use crate::audio::sec_to_samples;

    use super::*;

    #[test]
    fn test_fdn_reverb_basic_operation() {
        let mut reverb = FDNReverb::new();
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
        for _ in 0..sec_to_samples(0.2) {
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
        let mut reverb = FDNReverb::new();
        reverb.set_size(1.0);
        reverb.set_modulation_depth(1.0); // Full modulation

        // Process impulse and capture modulated reverb tail
        let (impulse_l, impulse_r) = reverb.process_stereo(1.0, 0.5);

        let mut outputs_l = Vec::new();
        let mut outputs_r = Vec::new();

        // Process samples to hear modulated reverb tail
        for _ in 0..sec_to_samples(0.5) {
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
    fn test_fast_hadamard_transform_4_energy_conservation() {
        // Test that the energy is conserved when applying the transform
        let test_inputs = [
            [1.0, 2.0, 3.0, 4.0],
            [0.5, 0.5, 0.5, 0.5],
            [0.0, 1.0, 2.0, 3.0],
            [1.0, 0.0, 1.0, 0.0],
            [1.0, 1.0, 1.0, 1.0],
            [0.5, 0.5, 0.5, 0.5],
            [0.0, 0.0, 0.0, 0.0],
            [1.0, 1.0, 1.0, 1.0],
        ];

        for test_input in test_inputs.iter() {
            let mut signals = *test_input;

            // Calculate input energy
            let input_energy: f32 = signals.iter().map(|x| x * x).sum();

            // Apply transform
            fast_hadamard_transform_4(&mut signals);

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
    fn test_fast_hadamard_transform_4_invertability() {
        let original = [1.0, 2.0, 3.0, 4.0];
        let mut signals = original;

        // Apply transform twice
        fast_hadamard_transform_4(&mut signals);
        fast_hadamard_transform_4(&mut signals);

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
