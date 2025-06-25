use crate::audio::delays::DelayLine;
use crate::audio::filters::{OnePoleFilter, OnePoleMode};
use crate::audio::{AudioProcessor, StereoAudioProcessor};

// Fast Hadamard Transform for 8x8 FDN
// This is more efficient than matrix multiplication
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
    temp[0] = signals[0] + signals[1];
    temp[1] = signals[0] - signals[1];
    temp[2] = signals[2] + signals[3];
    temp[3] = signals[2] - signals[3];
    temp[4] = signals[4] + signals[5];
    temp[5] = signals[4] - signals[5];
    temp[6] = signals[6] + signals[7];
    temp[7] = signals[6] - signals[7];

    *signals = temp;

    // Normalize by 1/sqrt(8) for energy conservation
    let scale = 1.0 / (8.0f32).sqrt();
    for i in 0..8 {
        signals[i] *= scale;
    }
}

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

// Base delay times for FDN to avoid resonances (in seconds)
const DIFFUSER_DELAYS: [f32; 4] = [0.046, 0.074, 0.082, 0.106];
const FEEDBACK_DELAYS: [f32; 4] = [0.134, 0.142, 0.158, 0.166];

// TODO: Implement modulation
// TODO: Implement velvet diffusion

pub struct FDNReverb {
    input_highcut: OnePoleFilter,
    input_lowcut: OnePoleFilter,

    // Delay lines for diffusion chain
    diffuser_delay_lines: [DelayLine; 4],
    // Delay lines for feedback chain
    feedback_delay_lines: [DelayLine; 4],

    size: f32,
}

impl FDNReverb {
    pub fn new() -> Self {
        let max_diffuser_delay = DIFFUSER_DELAYS.last().unwrap() * 2.0;
        let mut diffuser_delay_lines = [
            DelayLine::new(max_diffuser_delay),
            DelayLine::new(max_diffuser_delay),
            DelayLine::new(max_diffuser_delay),
            DelayLine::new(max_diffuser_delay),
        ];
        for i in 0..4 {
            diffuser_delay_lines[i].set_feedback(0.5);
        }

        let max_feedback_delay = FEEDBACK_DELAYS.last().unwrap() * 2.0;
        let mut feedback_delay_lines = [
            DelayLine::new(max_feedback_delay),
            DelayLine::new(max_feedback_delay),
            DelayLine::new(max_feedback_delay),
            DelayLine::new(max_feedback_delay),
        ];
        for i in 0..4 {
            feedback_delay_lines[i].set_feedback(0.5);
        }

        Self {
            input_highcut: OnePoleFilter::new(10000.0, OnePoleMode::Lowpass),
            input_lowcut: OnePoleFilter::new(200.0, OnePoleMode::Highpass),
            diffuser_delay_lines,
            feedback_delay_lines,
            size: 1.0,
        }
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        let feedback = feedback.clamp(0.0, 1.0); // Allow feedback up to 1.0
        for i in 0..4 {
            self.feedback_delay_lines[i].set_feedback(feedback);
        }
    }

    pub fn set_diffusion_feedback(&mut self, feedback: f32) {
        let feedback = feedback.clamp(0.0, 1.0); // Allow feedback up to 1.0
        for i in 0..4 {
            self.diffuser_delay_lines[i].set_feedback(feedback);
        }
    }

    pub fn set_size(&mut self, size: f32) {
        self.size = size.clamp(0.1, 2.0);
        for i in 0..4 {
            self.diffuser_delay_lines[i].set_delay_seconds(DIFFUSER_DELAYS[i] * self.size);
            self.feedback_delay_lines[i].set_delay_seconds(FEEDBACK_DELAYS[i] * self.size);
        }
    }
}

impl StereoAudioProcessor for FDNReverb {
    fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32) {
        // Input filtering and scaling for diffusion matrix
        let filtered_left = self.input_lowcut.process(self.input_highcut.process(left)) * 0.5;
        let filtered_right = self.input_lowcut.process(self.input_highcut.process(right)) * 0.5;

        // Diffusion stage
        let mut diffusion = [0.0f32; 4];
        for i in 0..4 {
            if i % 2 == 0 {
                diffusion[i] = self.diffuser_delay_lines[i].process(filtered_left);
            } else {
                diffusion[i] = self.diffuser_delay_lines[i].process(filtered_right);
            }
        }

        // Mix delay outputs signals using Hadamard transform
        fast_hadamard_transform_4(&mut diffusion);

        let mut echoes = [0.0f32; 4];
        for i in 0..4 {
            echoes[i] = self.feedback_delay_lines[i].read();
        }

        fast_hadamard_transform_4(&mut echoes);

        for i in 0..4 {
            // Write the diffusion to the delay lines with the mixed echoes
            self.feedback_delay_lines[i].write(diffusion[i], echoes[i]);
        }

        // Output the echoes mixed with the diffusion (for early reflections)
        let mut out_left = 0.0f32;
        let mut out_right = 0.0f32;
        for i in 0..4 {
            if i % 2 == 0 {
                out_left += diffusion[i] + echoes[i];
            } else {
                out_right += diffusion[i] + echoes[i];
            }
        }

        (out_left, out_right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fdn_reverb_basic_operation() {
        let mut reverb = FDNReverb::new();

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
        for _ in 0..5000 {
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
    fn test_fast_hadamard_transform_8_energy_conservation() {
        // Test energy conservation property: ||H*x|| = ||x|| for orthogonal matrix H
        let test_inputs = [
            [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
            [1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0],
            [0.5, 0.5, 0.5, 0.5, 0.0, 0.0, 0.0, 0.0],
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
    fn test_fast_hadamard_transform_8_invertibility() {
        // Test that applying the transform twice returns to original (up to scaling)
        // Since H₈ is symmetric and H₈² = 8I, we have H₈⁻¹ = H₈/8
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
