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

        // Proper Schroeder allpass: y[n] = -g*x[n] + x[n-d] + g*y[n-d]
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

pub struct BloomReverb {
    input_highcut: SVF,
    input_lowcut: SVF,

    allpass: [AllpassComb; 7],

    feedback_highpass: SVF,
    feedback_lowpass: SVF,

    feedback_delay: DelayBuffer,
    feedback_gain: f32,
    feedback_time: f32,

    downsample_counter: u32,
    downsample_hold: f32,

    // Parameters
    decay: f32,
    size: f32,

    bit_reduction: f32,
}

impl BloomReverb {
    pub fn new() -> Self {
        // Vintage-inspired allpass delay times (in samples at 44.1kHz)
        // These are prime numbers to avoid resonances
        let delay_times = [347, 113, 797, 277, 1511, 433, 1049];
        let feedback_gains = [0.7, -0.65, 0.6, -0.55, 0.5, -0.45, 0.4];

        let mut allpass = [
            AllpassComb::new(2048),
            AllpassComb::new(2048),
            AllpassComb::new(2048),
            AllpassComb::new(2048),
            AllpassComb::new(2048),
            AllpassComb::new(2048),
            AllpassComb::new(2048),
        ];

        // Configure each allpass filter
        for (i, ap) in allpass.iter_mut().enumerate() {
            ap.set_delay_samples(delay_times[i]);
            ap.set_feedback(feedback_gains[i]);
        }

        Self {
            input_highcut: SVF::new(10000.0, 0.7, FilterMode::Lowpass),
            input_lowcut: SVF::new(100.0, 0.7, FilterMode::Highpass),
            allpass,
            feedback_highpass: SVF::new(200.0, 0.5, FilterMode::Highpass),
            feedback_lowpass: SVF::new(8000.0, 0.5, FilterMode::Lowpass),
            feedback_delay: DelayBuffer::new(8192),
            feedback_gain: 0.85, // Strong feedback for sustained reverb
            feedback_time: 0.002,
            downsample_counter: 0,
            downsample_hold: 0.0,
            decay: 0.5,
            size: 0.5,
            bit_reduction: 12.0, // 12-bit character
        }
    }

    pub fn set_decay(&mut self, decay: f32) {
        self.decay = decay.clamp(0.0, 0.99);
        // Adjust feedback gains based on decay
        let base_gains = [0.7, -0.65, 0.6, -0.55, 0.5, -0.45, 0.4];
        for (i, ap) in self.allpass.iter_mut().enumerate() {
            ap.set_feedback(base_gains[i] * self.decay);
        }
    }

    pub fn set_size(&mut self, size: f32) {
        self.size = size.clamp(0.1, 2.0);
        // Adjust delay times based on size
        let base_delays = [347, 113, 797, 277, 1511, 433, 1049];
        for (i, ap) in self.allpass.iter_mut().enumerate() {
            ap.set_delay_samples((base_delays[i] as f32 * self.size) as usize);
        }
    }

    pub fn set_bit_reduction(&mut self, bits: f32) {
        self.bit_reduction = bits.clamp(8.0, 16.0);
    }

    pub fn set_feedback_highpass(&mut self, freq: f32) {
        self.feedback_highpass.set_cutoff_frequency(freq);
    }

    pub fn set_feedback_lowpass(&mut self, freq: f32) {
        self.feedback_lowpass.set_cutoff_frequency(freq);
    }

    pub fn set_feedback_gain(&mut self, gain: f32) {
        self.feedback_gain = gain.clamp(0.0, 0.99);
    }

    pub fn set_feedback_time(&mut self, time_seconds: f32) {
        self.feedback_time = time_seconds.max(0.001); // Minimum 1ms
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

    // Downsample for vintage grit (every 3rd sample)
    fn apply_downsampling(&mut self, input: f32) -> f32 {
        self.downsample_counter += 1;

        if self.downsample_counter >= 3 {
            self.downsample_counter = 0;
            self.downsample_hold = input;
        }

        self.downsample_hold
    }
}

impl AudioProcessor for BloomReverb {
    fn process(&mut self, input: f32) -> f32 {
        // Read feedback from delay line (like inspiration.gen ap_loop)
        let feedback_samples = sec_to_samples(self.feedback_time);
        let feedback_tap = self.feedback_delay.read(feedback_samples);

        let filtered = self.input_lowcut.process(self.input_highcut.process(input));

        let mixed_input = filtered + feedback_tap * self.feedback_gain;

        // Apply downsampling for grit
        let downsampled = self.apply_downsampling(mixed_input);

        // Apply bit reduction
        let bit_reduced = self.apply_bit_reduction(downsampled);

        // Process through 7 allpass filters
        let mut signal = bit_reduced;
        for ap in &mut self.allpass {
            signal = ap.process(signal);
        }

        // Apply feedback loop filtering
        let filtered_feedback = self
            .feedback_lowpass
            .process(self.feedback_highpass.process(signal));

        // Write filtered output back to feedback delay
        self.feedback_delay.write(filtered_feedback);

        filtered_feedback
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
    fn test_bloom_reverb_basic_operation() {
        let mut reverb = BloomReverb::new();

        // Test silence
        let out = reverb.process(0.0);
        assert_eq!(out, 0.0);

        // Test impulse response
        let impulse = reverb.process(1.0);

        let mut max_amp = 0.0f32;
        let mut outputs = Vec::new();

        // Process silence to hear reverb tail
        for _ in 0..2000 {
            let out = reverb.process(0.0);
            outputs.push(out);
            max_amp = max_amp.max(out.abs());
        }

        println!("BloomReverb test: impulse output = {}", impulse);
        println!("BloomReverb test: max tail amplitude = {}", max_amp);

        // Reverb should be stable
        assert!(max_amp < 10.0, "BloomReverb should remain stable");

        // Should produce reverb tail
        let has_tail = outputs.iter().any(|&x| x.abs() > 0.001);
        assert!(has_tail, "BloomReverb should produce reverb tail");

        // Should reach audible amplitude levels (at least 10% of input)
        assert!(
            max_amp > 0.4,
            "BloomReverb should reach audible amplitude (> 40%), got {}",
            max_amp
        );
    }

    #[test]
    fn test_bloom_reverb_parameters() {
        let mut reverb1 = BloomReverb::new();
        let mut reverb2 = BloomReverb::new();

        // Test parameter setting
        reverb1.set_decay(0.8);
        reverb1.set_size(1.5);
        reverb1.set_bit_reduction(10.0);

        reverb2.set_decay(0.2);
        reverb2.set_size(1.5);
        reverb2.set_bit_reduction(10.0);

        // Send impulse and process several samples to build up reverb tail
        reverb1.process(1.0);
        reverb2.process(1.0);

        let mut max_diff = 0.0f32;
        for _ in 0..100 {
            let out1 = reverb1.process(0.0);
            let out2 = reverb2.process(0.0);
            max_diff = max_diff.max((out1 - out2).abs());
        }

        // Different decay settings should produce different outputs over time
        assert!(
            max_diff > 0.0001,
            "Different decay settings should produce different outputs, max diff: {}",
            max_diff
        );

        println!(
            "BloomReverb parameter test: max difference over time = {}",
            max_diff
        );
    }
}
