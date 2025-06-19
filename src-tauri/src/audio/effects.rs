use crate::audio::{sec_to_samples, AudioProcessor, PI, SAMPLE_RATE};

// Tan approximation function
fn tan_a(x: f32) -> f32 {
    let x2 = x * x;
    x * (0.999999492001 + x2 * -0.096524608111)
        / (1.0 + x2 * (-0.429867256894 + x2 * 0.009981877999))
}

#[derive(Clone, Copy)]
enum FilterMode {
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
            self.r = 1.0 / self.q;
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

    pub fn read(&self, delay_samples: f32) -> f32 {
        let delay = delay_samples.max(0.0).min(self.buffer.len() as f32 - 1.0);
        let read_pos_f = self.write_pos as f32 - delay;
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

// Simple allpass filter
pub struct Allpass {
    delay: DelayBuffer,
    time: f32, // Delay time in samples
    g: f32,    // Feedback gain
}

impl Allpass {
    pub fn new(max_delay_samples: usize) -> Self {
        Self {
            delay: DelayBuffer::new(max_delay_samples),
            time: 0.0, // Default delay time
            g: 0.0,    // Default feedback gain
        }
    }

    pub fn set_delay_time(&mut self, time: f32) {
        self.time = time.max(0.0); // Ensure non-negative delay time
    }

    pub fn set_feedback_gain(&mut self, g: f32) {
        self.g = g.clamp(-0.99, 0.99); // Clamp to avoid instability
    }
}

impl AudioProcessor for Allpass {
    fn process(&mut self, input: f32) -> f32 {
        let z = self.delay.read(self.time);
        let x = input + z * self.g;
        let y = z + x * -self.g;
        self.delay.write(x);
        y
    }
}

// Schroeder Allpass filter
pub struct AllpassComb {
    input_buffer: Vec<f32>,
    output_buffer: Vec<f32>,
    write_pos: usize,
    feedback: f32,
    delay_samples: usize,
}

impl AllpassComb {
    pub fn new(max_delay_samples: usize) -> Self {
        Self {
            input_buffer: vec![0.0; max_delay_samples],
            output_buffer: vec![0.0; max_delay_samples],
            write_pos: 0,
            feedback: 0.0, // Default feedback gain
            delay_samples: 0,
        }
    }

    pub fn set_delay_samples(&mut self, delay_samples: usize) {
        self.delay_samples = delay_samples.min(self.input_buffer.len() - 1);
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(-0.99, 0.99);
    }
}

impl AudioProcessor for AllpassComb {
    fn process(&mut self, input: f32) -> f32 {
        let delay = self.delay_samples.min(self.input_buffer.len() - 1);

        // Calculate read position for delayed samples
        let read_pos = (self.write_pos + self.input_buffer.len() - delay) % self.input_buffer.len();

        // Get delayed input and delayed output
        let delayed_input = self.input_buffer[read_pos];
        let delayed_output = self.output_buffer[read_pos];

        // Schroeder allpass: y[n] = -g*x[n] + x[n-d] + g*y[n-d]
        let output = -self.feedback * input + delayed_input + self.feedback * delayed_output;

        // Write to buffers at current position
        self.input_buffer[self.write_pos] = input;
        self.output_buffer[self.write_pos] = output;

        // Advance write position
        self.write_pos = (self.write_pos + 1) % self.input_buffer.len();

        output
    }
}

// Delay line with freeze functionality
pub struct DelayLine {
    buffer: DelayBuffer,
    write_pos: usize,
    frozen: bool,
    highpass: SVF,
    lowpass: SVF,
    delay_samples: f32,
    feedback: f32,
}

impl DelayLine {
    pub fn new(max_delay_samples: usize) -> Self {
        Self {
            buffer: DelayBuffer::new(max_delay_samples),
            write_pos: 0,
            frozen: false,
            highpass: SVF::new(200.0, 0.5, FilterMode::Highpass),
            lowpass: SVF::new(8000.0, 0.5, FilterMode::Lowpass),
            delay_samples: 0.0,
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

    pub fn set_delay_samples(&mut self, delay_samples: f32) {
        self.delay_samples = delay_samples;
    }

    pub fn set_delay_seconds(&mut self, delay_seconds: f32) {
        self.delay_samples = sec_to_samples(delay_seconds);
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(-0.99, 0.99); // Clamp to avoid instability
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
// Prime-based times create natural sounding reverb while avoiding modal resonances
const BASE_DELAYS: [f32; 8] = [
    0.0079, 0.0026, 0.0181, 0.0063, 0.0343, 0.0098, 0.0238, 0.0143,
];

pub struct FDNReverb {
    input_highcut: SVF,
    input_lowcut: SVF,

    // 8 delay lines for FDN
    delay_lines: [DelayBuffer; 8],
    delay_times: [f32; 8],

    // Feedback filters for each delay line
    feedback_highpass: [SVF; 8],
    feedback_lowpass: [SVF; 8],

    // Gain control
    feedback_gain: f32,

    downsample_counter: u32,
    downsample_hold: (f32, f32),

    // Parameters
    decay: f32,
    size: f32,
    bit_reduction: f32,
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
            SVF::new(200.0, 0.5, FilterMode::Highpass),
            SVF::new(180.0, 0.5, FilterMode::Highpass),
            SVF::new(220.0, 0.5, FilterMode::Highpass),
            SVF::new(160.0, 0.5, FilterMode::Highpass),
            SVF::new(240.0, 0.5, FilterMode::Highpass),
            SVF::new(190.0, 0.5, FilterMode::Highpass),
            SVF::new(210.0, 0.5, FilterMode::Highpass),
            SVF::new(170.0, 0.5, FilterMode::Highpass),
        ];

        let feedback_lowpass = [
            SVF::new(8000.0, 0.5, FilterMode::Lowpass),
            SVF::new(7500.0, 0.5, FilterMode::Lowpass),
            SVF::new(8500.0, 0.5, FilterMode::Lowpass),
            SVF::new(7000.0, 0.5, FilterMode::Lowpass),
            SVF::new(9000.0, 0.5, FilterMode::Lowpass),
            SVF::new(7800.0, 0.5, FilterMode::Lowpass),
            SVF::new(8200.0, 0.5, FilterMode::Lowpass),
            SVF::new(7200.0, 0.5, FilterMode::Lowpass),
        ];

        Self {
            input_highcut: SVF::new(10000.0, 0.7, FilterMode::Lowpass),
            input_lowcut: SVF::new(100.0, 0.7, FilterMode::Highpass),
            delay_lines,
            delay_times: BASE_DELAYS,
            feedback_highpass,
            feedback_lowpass,
            feedback_gain: 0.6,
            downsample_counter: 0,
            downsample_hold: (0.0, 0.0),
            decay: 0.5,
            size: 0.5,
            bit_reduction: 12.0,
        }
    }

    pub fn set_decay(&mut self, decay: f32) {
        self.decay = decay.clamp(0.0, 1.0);
        self.feedback_gain = decay;
    }

    pub fn set_size(&mut self, size: f32) {
        self.size = size.clamp(0.1, 2.0);
        for i in 0..8 {
            self.delay_times[i] = BASE_DELAYS[i] * self.size;
        }
    }

    pub fn set_bit_reduction(&mut self, bits: f32) {
        self.bit_reduction = bits.clamp(8.0, 16.0);
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

    pub fn set_feedback_gain(&mut self, gain: f32) {
        self.feedback_gain = gain.clamp(0.0, 0.99);
    }

    // Apply bit reduction for vintage character
    fn apply_bit_reduction(&self, input: f32) -> f32 {
        if self.bit_reduction >= 16.0 {
            return input;
        }

        let levels = 2.0_f32.powf(self.bit_reduction);
        let step = 2.0 / levels;

        // Quantize the signal
        (input / step).round() * step
    }

    // Downsample for vintage grit (every 3rd sample) - stereo version
    fn apply_downsampling(&mut self, left: f32, right: f32) -> (f32, f32) {
        self.downsample_counter += 1;

        if self.downsample_counter >= 3 {
            self.downsample_counter = 0;
            self.downsample_hold = (left, right);
        }

        self.downsample_hold
    }
}

use crate::audio::StereoAudioProcessor;

impl StereoAudioProcessor for FDNReverb {
    fn process_stereo(&mut self, left: f32, right: f32) -> (f32, f32) {
        // Input filtering
        let filtered_left = self.input_lowcut.process(self.input_highcut.process(left));
        let filtered_right = self.input_lowcut.process(self.input_highcut.process(right));

        // Apply downsampling for vintage grit
        let (downsampled_left, downsampled_right) =
            self.apply_downsampling(filtered_left, filtered_right);

        // Apply bit reduction
        let bit_reduced_left = self.apply_bit_reduction(downsampled_left);
        let bit_reduced_right = self.apply_bit_reduction(downsampled_right);

        // Read current delay line outputs for FDN feedback matrix
        let mut delay_outputs = [0.0f32; 8];
        for i in 0..8 {
            let delay_samples = sec_to_samples(self.delay_times[i]);
            delay_outputs[i] = self.delay_lines[i].read(delay_samples);

            // Apply feedback filtering to each delay line output
            delay_outputs[i] = self.feedback_lowpass[i]
                .process(self.feedback_highpass[i].process(delay_outputs[i]));
        }

        // Apply fast Hadamard transform for FDN mixing
        let mut fdn_inputs = delay_outputs.clone();
        fast_hadamard_transform_8(&mut fdn_inputs);
        
        // Apply feedback gain
        for i in 0..8 {
            fdn_inputs[i] *= self.feedback_gain;
        }

        // Add stereo input to delay lines with full gain for better audibility
        fdn_inputs[0] += bit_reduced_left;
        fdn_inputs[1] += bit_reduced_left;
        fdn_inputs[2] += bit_reduced_left;
        fdn_inputs[3] += bit_reduced_left;

        fdn_inputs[4] += bit_reduced_right;
        fdn_inputs[5] += bit_reduced_right;
        fdn_inputs[6] += bit_reduced_right;
        fdn_inputs[7] += bit_reduced_right;

        // Write new inputs to delay lines
        for i in 0..8 {
            self.delay_lines[i].write(fdn_inputs[i]);
        }

        // Create stereo output by mixing delay line outputs with higher gain
        let out_left = (delay_outputs[0] + delay_outputs[2] + delay_outputs[4] + delay_outputs[6]) * 0.5;
        let out_right = (delay_outputs[1] + delay_outputs[3] + delay_outputs[5] + delay_outputs[7]) * 0.5;

        (out_left, out_right)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delay_line_basic_operation() {
        let mut delay = DelayLine::new(1000);
        delay.set_delay_samples(100.0);
        delay.set_feedback(0.0);

        // Test silence with no input
        assert_eq!(delay.process(0.0), 0.0);

        // Test impulse response
        let impulse_out = delay.process(1.0);
        assert_eq!(impulse_out, 0.0); // First sample should be 0 (no delay yet)

        // Process some samples to fill delay
        for _ in 0..99 {
            delay.process(0.0);
        }

        // At 100 samples, we should get our impulse back
        let delayed_impulse = delay.process(0.0);
        assert!(delayed_impulse > 0.0, "Should receive delayed impulse");

        println!(
            "Delay test: impulse {} delayed by 100 samples = {}",
            1.0, delayed_impulse
        );
    }

    #[test]
    fn test_delay_line_feedback_stability() {
        let mut delay = DelayLine::new(1000);
        let delay_samples = 100.0;
        let feedback = 0.4;

        delay.set_delay_samples(delay_samples);
        delay.set_feedback(feedback);

        // Send an impulse
        delay.process(1.0);

        let mut max_amplitude = 0.0f32;
        let mut outputs = Vec::new();

        // Process for several delay cycles to test stability
        for _ in 0..500 {
            let output = delay.process(0.0);
            outputs.push(output);
            max_amplitude = max_amplitude.max(output.abs());
        }

        println!(
            "Delay feedback test: max amplitude over 500 samples = {}",
            max_amplitude
        );

        // With 0.4 feedback, the system should remain stable
        assert!(
            max_amplitude < 2.0,
            "Delay feedback should remain stable, got max amplitude {}",
            max_amplitude
        );

        // Should have some repeating echoes
        let has_echoes = outputs.iter().any(|&x| x.abs() > 0.01);
        assert!(has_echoes, "Delay should produce audible echoes");
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
        for _ in 0..2000 {
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
        assert!(max_amp_l < 10.0, "FDNReverb left should remain stable");
        assert!(max_amp_r < 10.0, "FDNReverb right should remain stable");

        // Should produce reverb tail
        let has_tail_l = outputs_l.iter().any(|&x| x.abs() > 0.001);
        let has_tail_r = outputs_r.iter().any(|&x| x.abs() > 0.001);
        assert!(has_tail_l, "FDNReverb should produce left reverb tail");
        assert!(has_tail_r, "FDNReverb should produce right reverb tail");
    }

    #[test]
    fn test_fdn_reverb_parameters() {
        let mut reverb1 = FDNReverb::new();
        let mut reverb2 = FDNReverb::new();

        // Test parameter setting - make more extreme differences
        reverb1.set_decay(0.9);
        reverb1.set_size(2.0);
        reverb1.set_bit_reduction(16.0);

        reverb2.set_decay(0.1);
        reverb2.set_size(0.5);
        reverb2.set_bit_reduction(8.0);

        // Send impulse and process several samples to build up reverb tail
        reverb1.process_stereo(1.0, 1.0);
        reverb2.process_stereo(1.0, 1.0);

        let mut max_diff_l = 0.0f32;
        let mut max_diff_r = 0.0f32;
        // Process more samples to build up the difference
        for _ in 0..500 {
            let (out1_l, out1_r) = reverb1.process_stereo(0.0, 0.0);
            let (out2_l, out2_r) = reverb2.process_stereo(0.0, 0.0);
            max_diff_l = max_diff_l.max((out1_l - out2_l).abs());
            max_diff_r = max_diff_r.max((out1_r - out2_r).abs());
        }

        // Different decay settings should produce different outputs over time
        assert!(
            max_diff_l > 0.0001,
            "Different decay settings should produce different left outputs, max diff: {}",
            max_diff_l
        );
        assert!(
            max_diff_r > 0.0001,
            "Different decay settings should produce different right outputs, max diff: {}",
            max_diff_r
        );

        println!(
            "FDNReverb parameter test: max difference over time L={}, R={}",
            max_diff_l, max_diff_r
        );
    }

    #[test]
    fn test_fdn_infinite_reverb() {
        let mut reverb = FDNReverb::new();

        // Set decay to 1.0 for infinite reverberation
        reverb.set_decay(1.0);

        // Send a single impulse
        let (initial_l, initial_r) = reverb.process_stereo(1.0, 1.0);

        // Process many samples without input to check if reverb sustains
        let mut min_amplitude = f32::MAX;
        let mut max_amplitude = 0.0f32;

        for i in 0..5000 {
            let (out_l, out_r) = reverb.process_stereo(0.0, 0.0);
            let amplitude = out_l.abs().max(out_r.abs());

            // After initial delay, check for sustained reverb
            if i > 1000 {
                min_amplitude = min_amplitude.min(amplitude);
                max_amplitude = max_amplitude.max(amplitude);
            }
        }

        println!(
            "Infinite reverb test: min_amp={}, max_amp={}",
            min_amplitude, max_amplitude
        );

        // With feedback = 1.0, the reverb should sustain reasonably well
        // Some energy loss is expected due to filtering, but it should show much better sustain than lower feedback values
        assert!(
            min_amplitude > 0.00001,
            "Infinite reverb should sustain amplitude above 0.00001, got min: {}",
            min_amplitude
        );

        // Test that feedback = 1.0 sustains significantly better than feedback = 0.5
        let mut reverb_half = FDNReverb::new();
        reverb_half.set_decay(0.5);
        reverb_half.process_stereo(1.0, 1.0);

        let mut min_amplitude_half = f32::MAX;
        for i in 0..5000 {
            let (out_l, out_r) = reverb_half.process_stereo(0.0, 0.0);
            let amplitude = out_l.abs().max(out_r.abs());
            if i > 1000 {
                min_amplitude_half = min_amplitude_half.min(amplitude);
            }
        }

        // Full feedback should sustain at least 10x better than half feedback
        assert!(
            min_amplitude > min_amplitude_half * 10.0,
            "Full feedback (min: {}) should sustain much better than half feedback (min: {})",
            min_amplitude,
            min_amplitude_half
        );

        // Should also maintain some dynamics
        assert!(
            max_amplitude > min_amplitude * 1.1,
            "Infinite reverb should show some amplitude variation"
        );
    }
}
