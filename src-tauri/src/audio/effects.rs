use crate::audio::{sec_to_samples, AudioProcessor, StereoAudioProcessor, PI, SAMPLE_RATE};

// Tan approximation function
fn tan_a(x: f32) -> f32 {
    let x2 = x * x;
    x * (0.999999492001 + x2 * -0.096524608111)
        / (1.0 + x2 * (-0.429867256894 + x2 * 0.009981877999))
}

#[derive(Clone, Copy)]
pub enum FilterMode {
    Lowpass,
    Highpass,
    Bandpass,
}

// SVF implementation matching Emilie Gillet's stmlib version
pub struct SVF {
    // State variables
    y0: f32,
    y1: f32,

    // Filter outputs
    lp: f32,
    hp: f32,
    bp: f32,

    // Filter parameters
    mode: FilterMode,
    cf: f32, // Cutoff frequency
    q: f32,  // Resonance

    // Precomputed coefficients
    g: f32,
    r: f32,
    h: f32,
    rpg: f32,

    coeffs_dirty: bool,
}

impl SVF {
    pub fn new(cf: f32, q: f32, mode: FilterMode) -> Self {
        let mut svf = Self {
            y0: 0.0,
            y1: 0.0,
            lp: 0.0,
            hp: 0.0,
            bp: 0.0,
            mode,
            cf,
            q,
            g: 0.0,
            r: 0.0,
            h: 0.0,
            rpg: 0.0,
            coeffs_dirty: true,
        };
        svf.update_coefficients();
        svf
    }

    fn update_coefficients(&mut self) {
        if self.coeffs_dirty {
            self.g = tan_a(self.cf * PI / SAMPLE_RATE);
            self.r = 1.0 / self.q.max(0.001); // Prevent division by zero
            self.h = 1.0 / (1.0 + self.r * self.g + self.g * self.g);
            self.rpg = self.r + self.g;
            self.coeffs_dirty = false;
        }
    }

    pub fn set_cutoff_frequency(&mut self, cf: f32) {
        if (self.cf - cf).abs() > f32::EPSILON {
            self.cf = cf;
            self.coeffs_dirty = true;
        }
    }

    pub fn set_resonance(&mut self, q: f32) {
        if (self.q - q).abs() > f32::EPSILON {
            self.q = q;
            self.coeffs_dirty = true;
        }
    }

    pub fn set_mode(&mut self, mode: FilterMode) {
        self.mode = mode;
    }

    pub fn reset(&mut self) {
        self.y0 = 0.0;
        self.y1 = 0.0;
        self.lp = 0.0;
        self.hp = 0.0;
        self.bp = 0.0;
    }
}

impl AudioProcessor for SVF {
    fn process(&mut self, input: f32) -> f32 {
        self.update_coefficients();

        self.hp = (input - self.rpg * self.y0 - self.y1) * self.h;
        self.bp = self.g * self.hp + self.y0;
        self.y0 = self.g * self.hp + self.bp;
        self.lp = self.g * self.bp + self.y1;
        self.y1 = self.g * self.bp + self.lp;

        match self.mode {
            FilterMode::Lowpass => self.lp,
            FilterMode::Highpass => self.hp,
            FilterMode::Bandpass => self.bp,
        }
    }
}

// Delay line structure for allpass filter
pub struct FractionalDelayBuffer {
    buffer: Vec<f32>,
    write_pos: usize,
}

impl FractionalDelayBuffer {
    pub fn new(max_samples: usize) -> Self {
        Self {
            buffer: vec![0.0; max_samples],
            write_pos: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn read(&self, delay_samples: f32) -> f32 {
        let delay = delay_samples.max(0.0).min(self.buffer.len() as f32 - 1.0);
        let read_pos_f = self.write_pos as f32 - delay - 1.0;
        let read_pos = if read_pos_f < 0.0 {
            read_pos_f + self.buffer.len() as f32
        } else {
            read_pos_f
        };

        // Linear interpolation
        let idx = read_pos.floor() as usize % self.buffer.len();
        let frac = read_pos - read_pos.floor();
        let next_idx = (idx + 1) % self.buffer.len();

        self.buffer[idx] * (1.0 - frac) + self.buffer[next_idx] * frac
    }

    pub fn write(&mut self, value: f32) {
        self.buffer[self.write_pos] = value;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
    }
}

pub struct DelayBuffer {
    buffer: Vec<f32>,
    write_pos: usize,
}

impl DelayBuffer {
    pub fn new(max_samples: usize) -> Self {
        Self {
            buffer: vec![0.0; max_samples],
            write_pos: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn read(&self, delay_samples: usize) -> f32 {
        let delay = delay_samples.max(0).min(self.buffer.len() - 1);
        let read_pos = if delay <= self.write_pos {
            self.write_pos - delay
        } else {
            self.buffer.len() - (delay - self.write_pos)
        };
        self.buffer[read_pos]
    }

    pub fn write(&mut self, value: f32) {
        self.buffer[self.write_pos] = value;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
    }
}

// Simple allpass filter
pub struct Allpass {
    delay: DelayBuffer,
    delay_samples: usize, // Delay time in samples
    g: f32,               // Feedback gain
}

impl Allpass {
    pub fn new(max_delay_samples: usize) -> Self {
        Self {
            delay: DelayBuffer::new(max_delay_samples),
            delay_samples: 0, // Default delay time
            g: 0.0,           // Default feedback gain
        }
    }

    pub fn set_delay_seconds(&mut self, time: f32) {
        self.delay_samples = (sec_to_samples(time) as usize)
            .max(0)
            .min(self.delay.len() - 1);
    }

    pub fn set_feedback(&mut self, g: f32) {
        self.g = g.clamp(-0.99, 0.99); // Clamp to avoid instability
    }
}

impl AudioProcessor for Allpass {
    fn process(&mut self, input: f32) -> f32 {
        let z = self.delay.read(self.delay_samples);
        let x = input + z * self.g;
        let y = z + x * -self.g;
        self.delay.write(x);
        y
    }
}

// Schroeder Allpass filter
pub struct AllpassComb {
    input_buffer: DelayBuffer,
    output_buffer: DelayBuffer,
    feedback: f32,
    delay_samples: usize,
}

impl AllpassComb {
    pub fn new(max_delay_samples: usize) -> Self {
        Self {
            input_buffer: DelayBuffer::new(max_delay_samples),
            output_buffer: DelayBuffer::new(max_delay_samples),
            feedback: 0.0,    // Default feedback gain
            delay_samples: 0, // Default delay time
        }
    }

    pub fn set_delay_seconds(&mut self, delay_seconds: f32) {
        let delay_samples = sec_to_samples(delay_seconds) as usize;
        self.delay_samples = delay_samples.min(self.input_buffer.len() - 1);
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(-0.99, 0.99);
    }
}

impl AudioProcessor for AllpassComb {
    fn process(&mut self, input: f32) -> f32 {
        // Get delayed input and delayed output
        let delayed_input = self.input_buffer.read(self.delay_samples);
        let delayed_output = self.output_buffer.read(self.delay_samples);

        // Schroeder allpass: y[n] = -g*x[n] + x[n-d] + g*y[n-d]
        let output = -self.feedback * input + delayed_input + self.feedback * delayed_output;

        // Write to buffers at current position
        self.input_buffer.write(input);
        self.output_buffer.write(output);

        output
    }
}

// Delay line with freeze functionality
pub struct DelayLine {
    buffer: DelayBuffer,
    frozen: bool,
    highpass: SVF,
    lowpass: SVF,
    delay_samples: usize,
    feedback: f32,
}

impl DelayLine {
    pub fn new(max_delay_samples: usize) -> Self {
        Self {
            buffer: DelayBuffer::new(max_delay_samples),
            frozen: false,
            highpass: SVF::new(200.0, 0.5, FilterMode::Highpass),
            lowpass: SVF::new(8000.0, 0.5, FilterMode::Lowpass),
            delay_samples: 0,
            feedback: 0.0,
        }
    }

    pub fn set_freeze(&mut self, freeze: bool) {
        self.frozen = freeze;
    }

    pub fn set_highpass_freq(&mut self, freq: f32) {
        self.highpass.set_cutoff_frequency(freq);
    }

    pub fn set_lowpass_freq(&mut self, freq: f32) {
        self.lowpass.set_cutoff_frequency(freq);
    }

    pub fn set_delay_seconds(&mut self, delay_seconds: f32) {
        self.delay_samples = sec_to_samples(delay_seconds) as usize;
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(-1.0, 1.0); // Allow feedback of 1.0
    }
}

impl AudioProcessor for DelayLine {
    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer.read(self.delay_samples);

        // Apply filters to delayed signal
        let filtered = self.lowpass.process(self.highpass.process(delayed));

        // Write to buffer only if not frozen
        if !self.frozen {
            self.buffer.write(input + filtered * self.feedback);
        }

        filtered
    }
}

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

// Base delay times for FDN to avoid resonances (in seconds)
const BASE_DELAYS: [f32; 8] = [0.046, 0.074, 0.082, 0.106, 0.134, 0.142, 0.158, 0.166];

pub struct FDNReverb {
    input_highcut: SVF,
    input_lowcut: SVF,

    // 8 delay lines for FDN
    delay_lines: [DelayBuffer; 8],
    delays_samples: [usize; 8],

    // Feedback filters for each delay line
    feedback_highpass: [SVF; 8],
    feedback_lowpass: [SVF; 8],

    // Gain control
    feedback: f32,

    size: f32,
}

impl FDNReverb {
    pub fn new() -> Self {
        let delay_lines = [
            DelayBuffer::new(4096),
            DelayBuffer::new(4096),
            DelayBuffer::new(4096),
            DelayBuffer::new(4096),
            DelayBuffer::new(4096),
            DelayBuffer::new(4096),
            DelayBuffer::new(4096),
            DelayBuffer::new(4096),
        ];

        let feedback_highpass = [
            SVF::new(200.0, 0.0, FilterMode::Highpass),
            SVF::new(180.0, 0.0, FilterMode::Highpass),
            SVF::new(220.0, 0.0, FilterMode::Highpass),
            SVF::new(160.0, 0.0, FilterMode::Highpass),
            SVF::new(240.0, 0.0, FilterMode::Highpass),
            SVF::new(190.0, 0.0, FilterMode::Highpass),
            SVF::new(210.0, 0.0, FilterMode::Highpass),
            SVF::new(170.0, 0.0, FilterMode::Highpass),
        ];

        let feedback_lowpass = [
            SVF::new(8000.0, 0.0, FilterMode::Lowpass),
            SVF::new(7500.0, 0.0, FilterMode::Lowpass),
            SVF::new(8500.0, 0.0, FilterMode::Lowpass),
            SVF::new(7000.0, 0.0, FilterMode::Lowpass),
            SVF::new(9000.0, 0.0, FilterMode::Lowpass),
            SVF::new(7800.0, 0.0, FilterMode::Lowpass),
            SVF::new(8200.0, 0.0, FilterMode::Lowpass),
            SVF::new(7200.0, 0.0, FilterMode::Lowpass),
        ];

        let mut delays_samples = [0usize; 8];
        for i in 0..8 {
            delays_samples[i] = sec_to_samples(BASE_DELAYS[i]) as usize;
        }

        Self {
            input_highcut: SVF::new(10000.0, 0.0, FilterMode::Lowpass),
            input_lowcut: SVF::new(100.0, 0.0, FilterMode::Highpass),
            delay_lines,
            delays_samples: delays_samples,
            feedback_highpass,
            feedback_lowpass,
            feedback: 0.9,
            size: 1.0,
        }
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 1.0); // Allow feedback up to 1.0
    }

    pub fn set_size(&mut self, size: f32) {
        self.size = size.clamp(0.1, 2.0);
        for i in 0..8 {
            self.delays_samples[i] = (sec_to_samples(BASE_DELAYS[i]) * self.size) as usize;
        }
    }

    pub fn set_feedback_highpass(&mut self, freq: f32) {
        for hp in &mut self.feedback_highpass {
            hp.set_cutoff_frequency(freq);
        }
    }

    pub fn set_feedback_lowpass(&mut self, freq: f32) {
        for lp in &mut self.feedback_lowpass {
            lp.set_cutoff_frequency(freq);
        }
    }
}

// const DIFFUSOR_BASE_DELAYS: [f32; 8] = [0.001, 0.002, 0.003, 0.004, 0.005, 0.006, 0.007, 0.008];

// struct Diffusor {
//     allpass_filters: [Allpass; 8],
// }

// impl Diffusor {
//     pub fn new(sample_rate: f32) -> Self {
//         let mut allpass_filters = [Allpass::new(sample_rate); 8];
//         for i in 0..8 {
//             allpass_filters[i].set_delay_time(DIFFUSOR_BASE_DELAYS[i]);
//         }
//         Self { allpass_filters }
//     }

//     pub fn process(&mut self, input: f32) -> f32 {
//         let mut output = input;
//         for filter in &mut self.allpass_filters {
//             output = filter.process(output);
//         }
//         output
//     }
// }

impl StereoAudioProcessor for FDNReverb {
    fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32) {
        // Input filtering
        // let filtered_left = self.input_lowcut.process(self.input_highcut.process(left));
        // let filtered_right = self.input_lowcut.process(self.input_highcut.process(right));

        // Read current delay line outputs
        let mut delay_outputs = [0.0f32; 8];
        for i in 0..8 {
            delay_outputs[i] = self.delay_lines[i].read(self.delays_samples[i]);
        }

        // Mix delay outputs signals using Hadamard transform
        fast_hadamard_transform_8(&mut delay_outputs);

        // Filter the FDN outputs
        // let mut filtered_fdn = [0.0f32; 8];
        // for i in 0..8 {
        //     filtered_fdn[i] = self.feedback_lowpass[i]
        //         .process(self.feedback_highpass[i].process(delay_outputs[i]));
        // }

        // Write the mixed outputs to the delay lines + apply feedback + add the input
        let scaled_left = left * 0.25;
        let scaled_right = right * 0.25;
        for i in 0..8 {
            // Apply feedback to mixed outputs (this is the cross-coupling)
            let feedback_output = delay_outputs[i] * self.feedback;
            if i % 2 == 0 {
                // Even indices use left input
                self.delay_lines[i].write(scaled_left + feedback_output);
            } else {
                // Odd indices use right input
                self.delay_lines[i].write(scaled_right + feedback_output);
            }
        }

        // Output the delayed signals
        let out_left = delay_outputs[0] + delay_outputs[2] + delay_outputs[4] + delay_outputs[6];
        let out_right = delay_outputs[1] + delay_outputs[3] + delay_outputs[5] + delay_outputs[7];

        (out_left, out_right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delay_line_basic_operation() {
        let mut delay = DelayLine::new(1000);
        delay.set_delay_seconds(100.0 / sec_to_samples(1.0));
        delay.set_feedback(0.0);

        // Test silence with no input
        assert_eq!(delay.process(0.0), 0.0);

        // Test impulse response
        let impulse_out = delay.process(1.0);
        assert_eq!(impulse_out, 0.0); // First sample should be 0 (no delay yet)

        // Process some samples to fill delay
        for _ in 0..98 {
            delay.process(0.0);
        }

        // Check for impulse in the next few samples (filters may cause slight delay)
        let mut max_output = 0.0f32;
        for _ in 0..10 {
            let output = delay.process(0.0);
            max_output = max_output.max(output.abs());
        }

        assert!(
            max_output > 0.0,
            "Should receive delayed impulse, max output: {}",
            max_output
        );

        println!(
            "DelayLine test: impulse {} -> max delayed output = {}",
            1.0, max_output
        );
    }

    #[test]
    fn test_delay_line_feedback_one() {
        let mut delay = DelayLine::new(1000);
        let delay_seconds = 100.0 / sec_to_samples(1.0);
        let feedback = 1.0; // Exactly 1.0 feedback

        delay.set_delay_seconds(delay_seconds);
        delay.set_feedback(feedback);

        // Send an impulse
        let impulse_output = delay.process(1.0);
        assert_eq!(impulse_output, 0.0); // No immediate output

        let mut max_amplitude = 0.0f32;
        let mut outputs = Vec::new();

        // Process for many samples to test unity feedback behavior
        for i in 0..500 {
            let output = delay.process(0.0);
            outputs.push(output);
            max_amplitude = max_amplitude.max(output.abs());

            // Sample some outputs for debugging
            if i < 20 || i % 50 == 0 {
                println!("Sample {}: output = {:.6}", i, output);
            }
        }

        println!(
            "DelayLine feedback=1.0 test: max amplitude = {:.6}",
            max_amplitude
        );

        // With feedback=1.0, the signal should be preserved (not grow indefinitely due to filters)
        assert!(
            max_amplitude > 0.1,
            "DelayLine with feedback=1.0 should preserve signal, got max amplitude {:.6}",
            max_amplitude
        );

        // Should not grow without bound (filters will limit it)
        assert!(
            max_amplitude < 5.0,
            "DelayLine with feedback=1.0 should not grow excessively, got max amplitude {:.6}",
            max_amplitude
        );

        // Signal should persist in later samples
        let late_samples = &outputs[200..300];
        let has_late_signal = late_samples.iter().any(|&x| x.abs() > 0.01);
        assert!(has_late_signal, "Signal should persist with unity feedback");
    }

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
        let has_tail_l = outputs_l.iter().any(|&x| x.abs() > 0.2);
        let has_tail_r = outputs_r.iter().any(|&x| x.abs() > 0.2);
        assert!(has_tail_l, "FDNReverb should produce left reverb tail");
        assert!(has_tail_r, "FDNReverb should produce right reverb tail");
    }

    #[test]
    fn test_delay_buffer_basic_operation() {
        let mut buffer = DelayBuffer::new(100);

        // Test initial silence
        assert_eq!(buffer.read(10), 0.0);

        // Write an impulse
        buffer.write(1.0);

        // Should read the value just written when delay=1
        assert_eq!(buffer.read(1), 1.0);

        // Fill with zeros to advance the buffer
        for _ in 0..10 {
            buffer.write(0.0);
        }

        // At 11 samples delay, should read back the impulse
        let delayed = buffer.read(11);
        assert!(
            (delayed - 1.0).abs() < 1e-6,
            "Expected 1.0, got {}",
            delayed
        );

        println!(
            "DelayBuffer test: impulse delayed by 10 samples = {}",
            delayed
        );
    }

    #[test]
    fn test_delay_buffer_continuous_signal() {
        let mut buffer = DelayBuffer::new(50);
        let delay_samples = 20;

        // Write a sequence of values
        for i in 0..100 {
            let input = (i as f32) * 0.1;

            if i >= 20 {
                // After delay_samples, we should read back the earlier value
                let delayed = buffer.read(delay_samples);
                let expected = ((i - 20) as f32) * 0.1;
                assert!(
                    (delayed - expected).abs() < 1e-6,
                    "At sample {}: expected {}, got {}",
                    i,
                    expected,
                    delayed
                );
            }

            buffer.write(input);
        }
    }

    #[test]
    fn test_delay_buffer_feedback_loop() {
        let mut buffer = DelayBuffer::new(100);
        let delay_samples = 25;
        let feedback = 0.9;

        // Send impulse
        buffer.write(1.0);

        let mut max_output = 0.0f32;
        let mut outputs = Vec::new();

        // Run feedback loop for many cycles
        for i in 0..500 {
            let delayed = buffer.read(delay_samples);
            let output = delayed;
            let feedback_input = delayed * feedback;

            // Add small input decay to simulate real conditions
            let input = if i == 0 { 1.0 } else { 0.0 };
            buffer.write(input + feedback_input);

            outputs.push(output);
            max_output = max_output.max(output.abs());

            // Print some key samples
            if i < 50 || i % 50 == 0 {
                println!(
                    "Sample {}: delayed={:.6}, feedback_input={:.6}",
                    i, delayed, feedback_input
                );
            }
        }

        println!("DelayBuffer feedback test: max output = {:.6}", max_output);

        // Should have sustained oscillation with 0.9 feedback
        assert!(
            max_output > 0.1,
            "Feedback loop should sustain signal, max output: {}",
            max_output
        );

        // Check that signal persists for a reasonable time
        let late_samples = &outputs[200..300];
        let has_late_signal = late_samples.iter().any(|&x| x.abs() > 0.01);
        assert!(has_late_signal, "Signal should persist with high feedback");
    }

    #[test]
    fn test_fast_hadamard_transform_8_impulse() {
        // Test with impulse input [1,0,0,0,0,0,0,0]
        let mut signals = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        fast_hadamard_transform_8(&mut signals);

        // Expected result: Hadamard transform of impulse should be constant across all outputs
        // H₈ * [1,0,0,0,0,0,0,0]ᵀ = [1,1,1,1,1,1,1,1]ᵀ / √8
        let expected_value = 1.0 / (8.0f32).sqrt();

        for (i, &value) in signals.iter().enumerate() {
            assert!(
                (value - expected_value).abs() < 1e-6,
                "Impulse transform failed at index {}: expected {}, got {}",
                i,
                expected_value,
                value
            );
        }
    }

    #[test]
    fn test_fast_hadamard_transform_8_partial_ones() {
        // Test with partial ones input [1,1,1,0,0,0,0,0]
        let mut signals = [1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        fast_hadamard_transform_8(&mut signals);

        // The Hadamard transform is linear, so H₈ * [1,1,1,0,0,0,0,0]ᵀ =
        // H₈ * [1,0,0,0,0,0,0,0]ᵀ + H₈ * [0,1,0,0,0,0,0,0]ᵀ + H₈ * [0,0,1,0,0,0,0,0]ᵀ
        let scale = 1.0 / (8.0f32).sqrt();
        let expected = [
            3.0 * scale,  // Sum of first 3 rows of H₈ column 0
            1.0 * scale,  // Sum of first 3 rows of H₈ column 1
            1.0 * scale,  // Sum of first 3 rows of H₈ column 2
            -1.0 * scale, // Sum of first 3 rows of H₈ column 3
            3.0 * scale,  // Sum of first 3 rows of H₈ column 4
            1.0 * scale,  // Sum of first 3 rows of H₈ column 5
            1.0 * scale,  // Sum of first 3 rows of H₈ column 6
            -1.0 * scale, // Sum of first 3 rows of H₈ column 7
        ];

        for (i, (&result, &expected_val)) in signals.iter().zip(expected.iter()).enumerate() {
            assert!(
                (result - expected_val).abs() < 1e-6,
                "Partial ones transform failed at index {}: expected {}, got {}",
                i,
                expected_val,
                result
            );
        }
    }

    #[test]
    fn test_fast_hadamard_transform_8_all_ones() {
        // Test with all ones input [1,1,1,1,1,1,1,1]
        let mut signals = [1.0; 8];
        fast_hadamard_transform_8(&mut signals);

        // H₈ * [1,1,1,1,1,1,1,1]ᵀ should give [8,0,0,0,0,0,0,0]ᵀ / √8 = [√8,0,0,0,0,0,0,0]ᵀ
        let expected_first = (8.0f32).sqrt();

        assert!(
            (signals[0] - expected_first).abs() < 1e-6,
            "All ones transform failed at index 0: expected {}, got {}",
            expected_first,
            signals[0]
        );

        for i in 1..8 {
            assert!(
                signals[i].abs() < 1e-6,
                "All ones transform failed at index {}: expected 0, got {}",
                i,
                signals[i]
            );
        }
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

        // Should be back to original scaled by the square of the normalization factor
        // Since we normalize by 1/√8 each time, applying twice gives us (1/√8)² = 1/8
        // But the Hadamard transform itself when applied twice gives us 8 times the original
        // So the net effect is: original * 8 * (1/√8) * (1/√8) = original * 8 * (1/8) = original
        let scale_factor = 1.0;

        for (i, (&result, &orig)) in signals.iter().zip(original.iter()).enumerate() {
            let expected = orig * scale_factor;
            assert!(
                (result - expected).abs() < 1e-6,
                "Invertibility test failed at index {}: expected {}, got {}",
                i,
                expected,
                result
            );
        }
    }
}
