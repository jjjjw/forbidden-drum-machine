use crate::audio::filters::{OnePoleFilter, OnePoleMode};
use crate::audio::{AudioBlockStereoProcessor, AudioProcessor, StereoAudioProcessor};

#[derive(Clone)]
struct Echo {
    position: usize, // Position in samples from start of echo span
    sign: bool,      // true = +1, false = -1
    amplitude: f32,  // Calculated amplitude based on decay
}

impl Echo {
    fn new(position: usize, sign: bool) -> Self {
        Self {
            position,
            sign,
            amplitude: 0.0,
        }
    }

    fn update_amplitude(&mut self, position_samples: usize, decay_time_samples: f32, gain: f32) {
        // Exponential decay: amplitude = gain * exp(-position / decay_time)
        let decay_factor = -(position_samples as f32) / decay_time_samples;
        self.amplitude = gain * decay_factor.exp() * if self.sign { 1.0 } else { -1.0 };
    }
}

pub struct VelvetNoiseReverb {
    // Parameters
    sample_rate: f32,
    echo_count: usize,
    echo_spacing: usize, // Average spacing between echoes in samples
    decay_time: f32,     // RT60 in seconds
    feedback: f32,
    crosstalk: f32, // Amount of L->R and R->L crosstalk (0.0 to 1.0)

    // Echo definitions for left and right channels
    left_echoes: Vec<Echo>,
    right_echoes: Vec<Echo>,

    // Single circular buffer for each channel
    left_buffer: Vec<f32>,
    right_buffer: Vec<f32>,

    // Buffer positions
    read_position: usize,
    feedback_position: usize,
    buffer_size: usize,

    // Filters for each channel
    left_input_lowcut: OnePoleFilter,
    left_input_highcut: OnePoleFilter,
    right_input_lowcut: OnePoleFilter,
    right_input_highcut: OnePoleFilter,

    // RNG for initialization
    rng: fastrand::Rng,
}

impl VelvetNoiseReverb {
    pub fn new(sample_rate: f32) -> Self {
        let echo_count = 1000; // Number of echoes per channel
        let echo_spacing = (sample_rate * 0.001) as usize; // 1ms average spacing
        let decay_time = 2.0; // 2 second RT60
        let feedback = 0.85;
        let crosstalk = 0.15; // 15% crosstalk between channels

        // Buffer size: 1 second of echoes + 1 second feedback loop
        let buffer_size = (sample_rate * 2.0) as usize;

        let mut reverb = Self {
            sample_rate,
            echo_count,
            echo_spacing,
            decay_time,
            feedback,
            crosstalk,
            left_echoes: Vec::new(),
            right_echoes: Vec::new(),
            left_buffer: vec![0.0; buffer_size],
            right_buffer: vec![0.0; buffer_size],
            read_position: 0,
            feedback_position: (sample_rate * 1.0) as usize, // 1 second ahead
            buffer_size,
            left_input_lowcut: OnePoleFilter::new(200.0, OnePoleMode::Highpass),
            left_input_highcut: OnePoleFilter::new(8000.0, OnePoleMode::Lowpass),
            right_input_lowcut: OnePoleFilter::new(200.0, OnePoleMode::Highpass),
            right_input_highcut: OnePoleFilter::new(8000.0, OnePoleMode::Lowpass),
            rng: fastrand::Rng::new(),
        };

        reverb.initialize_echoes();
        reverb
    }

    fn initialize_echoes(&mut self) {
        self.left_echoes.clear();
        self.right_echoes.clear();

        // Initialize left channel echoes
        let mut left_echoes = Vec::new();
        self.initialize_channel_echoes(&mut left_echoes);
        self.left_echoes = left_echoes;
        
        // Initialize right channel echoes
        let mut right_echoes = Vec::new();
        self.initialize_channel_echoes(&mut right_echoes);
        self.right_echoes = right_echoes;
        
        self.update_amplitudes();
    }

    fn initialize_channel_echoes(&mut self, echoes: &mut Vec<Echo>) {
        echoes.reserve(self.echo_count);

        let mut offset = 0;
        for _ in 0..self.echo_count {
            let spacing = self.rng.usize(1..self.echo_spacing * 2);
            let sign = self.rng.bool();

            offset += spacing;
            if offset >= self.sample_rate as usize {
                break; // Don't exceed 1 second of echoes
            }

            echoes.push(Echo::new(offset, sign));
        }
    }

    fn update_amplitudes(&mut self) {
        let decay_time_samples = self.decay_time * self.sample_rate;

        for echo in &mut self.left_echoes {
            echo.update_amplitude(echo.position, decay_time_samples, 1.0);
        }

        for echo in &mut self.right_echoes {
            echo.update_amplitude(echo.position, decay_time_samples, 1.0);
        }
    }

    pub fn set_crosstalk(&mut self, crosstalk: f32) {
        self.crosstalk = crosstalk.clamp(0.0, 1.0);
    }

    pub fn set_decay_time(&mut self, decay_time: f32) {
        self.decay_time = decay_time.clamp(0.1, 10.0);
        self.update_amplitudes();
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 0.95);
    }


    fn process_channel(&mut self, input: &[f32], output: &mut [f32], echoes: &[Echo], delay_buffer: &mut [f32]) {
        let block_size = input.len();
        
        // Step 1: Write echoes - loop through each echo and write the entire input block
        for echo in echoes {
            let gain = echo.amplitude;
            let mut pos = (self.read_position + echo.position) % self.buffer_size;

            // Optimized loop when we don't wrap around buffer
            if pos + block_size <= self.buffer_size {
                for (i, &sample) in input.iter().enumerate() {
                    delay_buffer[pos + i] += gain * sample;
                }
            } else {
                // Handle buffer wrap-around
                for &sample in input.iter() {
                    delay_buffer[pos] += gain * sample;
                    pos = (pos + 1) % self.buffer_size;
                }
            }
        }

        // Step 2: Read output and apply feedback
        for (i, out_sample) in output.iter_mut().enumerate() {
            let read_pos = (self.read_position + i) % self.buffer_size;
            let feedback_pos = (self.feedback_position + i) % self.buffer_size;

            // Read output from delay buffer
            *out_sample = delay_buffer[read_pos];
            
            // Clear the position after reading
            delay_buffer[read_pos] = 0.0;

            // Write feedback at feedback position
            delay_buffer[feedback_pos] += *out_sample * self.feedback;
        }
    }
}

impl AudioBlockStereoProcessor for VelvetNoiseReverb {
    fn process_stereo_block(&mut self, left_buffer: &mut [f32], right_buffer: &mut [f32]) {
        assert_eq!(left_buffer.len(), right_buffer.len());

        let block_size = left_buffer.len();

        // Store input for processing (we need separate input/output arrays)
        let mut input_left = vec![0.0; block_size];
        let mut input_right = vec![0.0; block_size];
        
        // Apply input filtering and store in input arrays
        for i in 0..block_size {
            input_left[i] = self
                .left_input_highcut
                .process(self.left_input_lowcut.process(left_buffer[i]));
            input_right[i] = self
                .right_input_highcut
                .process(self.right_input_lowcut.process(right_buffer[i]));
        }

        // Apply input crosstalk
        for i in 0..block_size {
            let left_mixed = input_left[i] + self.crosstalk * input_right[i];
            let right_mixed = input_right[i] + self.crosstalk * input_left[i];
            input_left[i] = left_mixed;
            input_right[i] = right_mixed;
        }

        // Process each channel - following the reference algorithm exactly
        let left_echoes = self.left_echoes.clone();
        let right_echoes = self.right_echoes.clone();
        
        // Process left channel
        let mut left_delay_buffer = std::mem::take(&mut self.left_buffer);
        self.process_channel(&input_left, left_buffer, &left_echoes, &mut left_delay_buffer);
        self.left_buffer = left_delay_buffer;
        
        // Process right channel  
        let mut right_delay_buffer = std::mem::take(&mut self.right_buffer);
        self.process_channel(&input_right, right_buffer, &right_echoes, &mut right_delay_buffer);
        self.right_buffer = right_delay_buffer;

        // Apply output crosstalk (subtle)
        for i in 0..block_size {
            let left_with_crosstalk = left_buffer[i] + self.crosstalk * 0.1 * right_buffer[i];
            let right_with_crosstalk = right_buffer[i] + self.crosstalk * 0.1 * left_buffer[i];
            left_buffer[i] = left_with_crosstalk;
            right_buffer[i] = right_with_crosstalk;
        }

        // Update buffer positions
        self.read_position = (self.read_position + block_size) % self.buffer_size;
        self.feedback_position = (self.feedback_position + block_size) % self.buffer_size;
    }
}

// Adapter to make the block processor work with single samples
impl StereoAudioProcessor for VelvetNoiseReverb {
    fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32) {
        let mut left_buf = [left];
        let mut right_buf = [right];

        self.process_stereo_block(&mut left_buf, &mut right_buf);

        (left_buf[0], right_buf[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_velvet_reverb_basic_operation() {
        let mut reverb = VelvetNoiseReverb::new(44100.0);

        // Test silence
        let mut left_buf = [0.0f32; 64];
        let mut right_buf = [0.0f32; 64];

        reverb.process_stereo_block(&mut left_buf, &mut right_buf);

        // Should be silent initially
        assert!(left_buf.iter().all(|&x| x == 0.0));
        assert!(right_buf.iter().all(|&x| x == 0.0));

        // Test impulse response
        let mut left_buf = [0.0f32; 64];
        let mut right_buf = [0.0f32; 64];
        left_buf[0] = 1.0;
        right_buf[0] = 0.5;

        reverb.process_stereo_block(&mut left_buf, &mut right_buf);

        // Process several blocks to hear reverb tail
        let mut max_amp_left = 0.0f32;
        let mut max_amp_right = 0.0f32;
        let mut has_output_left = false;
        let mut has_output_right = false;

        for _ in 0..100 {
            // Process ~0.15 seconds at 64 samples/block
            let mut silence_left = [0.0f32; 64];
            let mut silence_right = [0.0f32; 64];
            reverb.process_stereo_block(&mut silence_left, &mut silence_right);

            for &sample in &silence_left {
                max_amp_left = max_amp_left.max(sample.abs());
                if sample.abs() > 0.001 {
                    has_output_left = true;
                }
            }

            for &sample in &silence_right {
                max_amp_right = max_amp_right.max(sample.abs());
                if sample.abs() > 0.001 {
                    has_output_right = true;
                }
            }
        }

        println!(
            "VelvetNoiseReverb test: max amplitude L={}, R={}",
            max_amp_left, max_amp_right
        );

        // Should produce reverb tail
        assert!(has_output_left, "Should produce left reverb tail");
        assert!(has_output_right, "Should produce right reverb tail");

        // Should remain stable
        assert!(max_amp_left < 2.0, "Left channel should remain stable");
        assert!(max_amp_right < 2.0, "Right channel should remain stable");
    }

    #[test]
    fn test_velvet_reverb_crosstalk() {
        let mut reverb = VelvetNoiseReverb::new(44100.0);
        reverb.set_crosstalk(0.5); // 50% crosstalk for testing

        // Test that left input creates output in right channel
        let mut left_buf = [0.0f32; 64];
        let mut right_buf = [0.0f32; 64];
        left_buf[0] = 1.0;

        reverb.process_stereo_block(&mut left_buf, &mut right_buf);

        // Process several blocks to let crosstalk develop
        let mut has_right_output = false;

        for _ in 0..50 {
            let mut silence_left = [0.0f32; 64];
            let mut silence_right = [0.0f32; 64];
            reverb.process_stereo_block(&mut silence_left, &mut silence_right);

            if silence_right.iter().any(|&x| x.abs() > 0.001) {
                has_right_output = true;
                break;
            }
        }

        assert!(
            has_right_output,
            "Left input should create crosstalk in right channel"
        );

        // Test that right input creates output in left channel
        reverb = VelvetNoiseReverb::new(44100.0);
        reverb.set_crosstalk(0.5);

        let mut left_buf = [0.0f32; 64];
        let mut right_buf = [0.0f32; 64];
        right_buf[0] = 1.0;

        reverb.process_stereo_block(&mut left_buf, &mut right_buf);

        let mut has_left_output = false;

        for _ in 0..50 {
            let mut silence_left = [0.0f32; 64];
            let mut silence_right = [0.0f32; 64];
            reverb.process_stereo_block(&mut silence_left, &mut silence_right);

            if silence_left.iter().any(|&x| x.abs() > 0.001) {
                has_left_output = true;
                break;
            }
        }

        assert!(
            has_left_output,
            "Right input should create crosstalk in left channel"
        );
    }

    #[test]
    fn test_velvet_reverb_parameter_changes() {
        let mut reverb = VelvetNoiseReverb::new(44100.0);

        // Test parameter setting
        reverb.set_decay_time(5.0);
        reverb.set_feedback(0.9);
        reverb.set_crosstalk(0.7);

        // Should not crash and should still work
        let mut left_buf = [0.0f32; 64];
        let mut right_buf = [0.0f32; 64];
        left_buf[0] = 1.0;

        reverb.process_stereo_block(&mut left_buf, &mut right_buf);

        // Should still be stable with higher settings
        for _ in 0..50 {
            let mut silence_left = [0.0f32; 64];
            let mut silence_right = [0.0f32; 64];
            reverb.process_stereo_block(&mut silence_left, &mut silence_right);

            for &sample in &silence_left {
                assert!(
                    sample.abs() < 5.0,
                    "High-feedback reverb should remain stable"
                );
            }
        }
    }
}
