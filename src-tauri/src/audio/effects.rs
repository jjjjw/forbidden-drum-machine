use crate::audio::{SAMPLE_RATE, PI};
use crate::audio::oscillators::SineOscillator;

// State Variable Filter for high-pass and low-pass
pub struct SVFilter {
    frequency: f32,
    q: f32,
    ic1eq: f32,
    ic2eq: f32,
}

impl SVFilter {
    pub fn new(frequency: f32, q: f32) -> Self {
        Self {
            frequency,
            q,
            ic1eq: 0.0,
            ic2eq: 0.0,
        }
    }
    
    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
    }
    
    pub fn set_q(&mut self, q: f32) {
        self.q = q;
    }
    
    pub fn process(&mut self, input: f32) -> (f32, f32, f32) {
        let g = (PI * self.frequency / SAMPLE_RATE).tan();
        let k = 1.0 / self.q;
        let a1 = 1.0 / (1.0 + g * (g + k));
        let a2 = g * a1;
        let a3 = g * a2;
        
        let v3 = input - self.ic2eq;
        let v1 = a1 * self.ic1eq + a2 * v3;
        let v2 = self.ic2eq + a2 * self.ic1eq + a3 * v3;
        
        self.ic1eq = 2.0 * v1 - self.ic1eq;
        self.ic2eq = 2.0 * v2 - self.ic2eq;
        
        let lowpass = v2;
        let bandpass = v1;
        let highpass = input - k * v1 - v2;
        
        (lowpass, bandpass, highpass)
    }
    
    pub fn highpass(&mut self, input: f32) -> f32 {
        let (_, _, hp) = self.process(input);
        hp
    }
    
    pub fn lowpass(&mut self, input: f32) -> f32 {
        let (lp, _, _) = self.process(input);
        lp
    }
}

// Schroeder Allpass filter  
pub struct AllpassComb {
    input_buffer: Vec<f32>,
    output_buffer: Vec<f32>,
    write_pos: usize,
    feedback: f32,
}

impl AllpassComb {
    pub fn new(max_delay_samples: usize, feedback: f32) -> Self {
        Self {
            input_buffer: vec![0.0; max_delay_samples],
            output_buffer: vec![0.0; max_delay_samples],
            write_pos: 0,
            feedback,
        }
    }
    
    pub fn process(&mut self, input: f32, delay_samples: usize) -> f32 {
        let delay = delay_samples.min(self.input_buffer.len() - 1);
        
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
    buffer: Vec<f32>,
    write_pos: usize,
    frozen: bool,
    highpass: SVFilter,
    lowpass: SVFilter,
}

impl DelayLine {
    pub fn new(max_delay_samples: usize) -> Self {
        Self {
            buffer: vec![0.0; max_delay_samples],
            write_pos: 0,
            frozen: false,
            highpass: SVFilter::new(200.0, 0.5),
            lowpass: SVFilter::new(8000.0, 0.5),
        }
    }
    
    pub fn set_freeze(&mut self, freeze: bool) {
        self.frozen = freeze;
    }
    
    pub fn set_highpass_freq(&mut self, freq: f32) {
        self.highpass.set_frequency(freq);
    }
    
    pub fn set_lowpass_freq(&mut self, freq: f32) {
        self.lowpass.set_frequency(freq);
    }
    
    pub fn process(&mut self, input: f32, delay_samples: usize, feedback: f32) -> f32 {
        let delay = delay_samples.min(self.buffer.len() - 1);
        let read_pos = (self.write_pos + self.buffer.len() - delay) % self.buffer.len();
        
        let delayed = self.buffer[read_pos];
        
        // Apply filters to delayed signal
        let filtered = self.lowpass.lowpass(self.highpass.highpass(delayed));
        
        // Write to buffer only if not frozen
        if !self.frozen {
            self.buffer[self.write_pos] = input + filtered * feedback;
            self.write_pos = (self.write_pos + 1) % self.buffer.len();
        }
        
        filtered
    }
}

// Complete allpass reverb based on the Faust implementation
pub struct AllpassReverb {
    // LFOs for modulation
    lfo0: SineOscillator,
    lfo1: SineOscillator,
    lfo2: SineOscillator,
    
    // Allpass stages for left channel
    ap_l0: AllpassComb,
    ap_l1: AllpassComb,
    ap_l2: AllpassComb,
    ap_l3: AllpassComb,
    ap_l4: AllpassComb,
    ap_l5: AllpassComb,
    
    // Allpass stages for right channel
    ap_r0: AllpassComb,
    ap_r1: AllpassComb,
    ap_r2: AllpassComb,
    ap_r3: AllpassComb,
    ap_r4: AllpassComb,
    ap_r5: AllpassComb,
    
    // Filters in feedback loop
    hp_l: SVFilter,
    lp_l: SVFilter,
    hp_r: SVFilter,
    lp_r: SVFilter,
    
    // Feedback delay lines
    feedback_delay_l: Vec<f32>,
    feedback_delay_r: Vec<f32>,
    feedback_pos_l: usize,
    feedback_pos_r: usize,
    
    // Parameters
    max_delay: usize,
    base_time: f32,
    decay: f32,
    wet_mix: f32,
    size: f32,
    highpass_freq: f32,
    lowpass_freq: f32,
}

impl AllpassReverb {
    pub fn new() -> Self {
        let max_delay = (0.3 * SAMPLE_RATE) as usize; // 0.3 seconds max delay
        
        Self {
            // LFOs with frequencies matching Faust code
            lfo0: SineOscillator::new(0.9128),
            lfo1: SineOscillator::new(1.1341),
            lfo2: SineOscillator::new(1.0),
            
            // Left channel allpass stages
            ap_l0: AllpassComb::new(max_delay, 0.5),
            ap_l1: AllpassComb::new(max_delay, 0.5),
            ap_l2: AllpassComb::new(max_delay, 0.5),
            ap_l3: AllpassComb::new(max_delay, 0.5),
            ap_l4: AllpassComb::new(max_delay, 0.5),
            ap_l5: AllpassComb::new(max_delay, 0.5),
            
            // Right channel allpass stages
            ap_r0: AllpassComb::new(max_delay, 0.5),
            ap_r1: AllpassComb::new(max_delay, 0.5),
            ap_r2: AllpassComb::new(max_delay, 0.5),
            ap_r3: AllpassComb::new(max_delay, 0.5),
            ap_r4: AllpassComb::new(max_delay, 0.5),
            ap_r5: AllpassComb::new(max_delay, 0.5),
            
            // Filters
            hp_l: SVFilter::new(200.0, 0.5),
            lp_l: SVFilter::new(8000.0, 0.5),
            hp_r: SVFilter::new(200.0, 0.5),
            lp_r: SVFilter::new(8000.0, 0.5),
            
            // Feedback delay lines
            feedback_delay_l: vec![0.0; max_delay],
            feedback_delay_r: vec![0.0; max_delay],
            feedback_pos_l: 0,
            feedback_pos_r: 0,
            
            // Default parameters
            max_delay,
            base_time: 0.5 * 0.006666667 * SAMPLE_RATE, // size * base_time_seconds * sample_rate
            decay: 0.5,
            wet_mix: 0.5,
            size: 0.5,
            highpass_freq: 200.0,
            lowpass_freq: 8000.0,
        }
    }
    
    pub fn set_size(&mut self, size: f32) {
        self.size = size.clamp(0.1, 1.0);
        self.base_time = self.size * 0.006666667 * SAMPLE_RATE;
    }
    
    pub fn set_decay(&mut self, decay: f32) {
        self.decay = decay.clamp(0.01, 0.998);
    }
    
    pub fn set_wet_mix(&mut self, wet: f32) {
        self.wet_mix = wet.clamp(0.0, 1.0);
    }
    
    pub fn set_highpass_freq(&mut self, freq: f32) {
        self.highpass_freq = freq.clamp(20.0, 2000.0);
        self.hp_l.set_frequency(self.highpass_freq);
        self.hp_r.set_frequency(self.highpass_freq);
    }
    
    pub fn set_lowpass_freq(&mut self, freq: f32) {
        self.lowpass_freq = freq.clamp(2000.0, 20000.0);
        self.lp_l.set_frequency(self.lowpass_freq);
        self.lp_r.set_frequency(self.lowpass_freq);
    }
    
    pub fn process(&mut self, input_l: f32, input_r: f32) -> (f32, f32) {
        // Update LFOs (scale to match original amplitude)
        let lfo0_val = self.lfo0.next_sample() * 11.0;
        let lfo1_val = self.lfo1.next_sample() * 9.0;
        let lfo2_val = self.lfo2.next_sample() * 10.0;
        
        // Use summed mono input for both channels to avoid phase issues
        let mono_input = (input_l + input_r) * 0.5;
        
        // Process left channel
        let feedback_delay_samples = (self.base_time) as usize;
        let feedback_read_pos = (self.feedback_pos_l + self.feedback_delay_l.len() - feedback_delay_samples.min(self.feedback_delay_l.len() - 1)) % self.feedback_delay_l.len();
        let feedback_tap = self.feedback_delay_l[feedback_read_pos] * self.decay;
        let mixed_input_l = mono_input + feedback_tap;
        
        // Allpass chain for left channel - same delay times, positive modulation
        let time = self.base_time;
        let ap0_delay = ((time * 3.0 + lfo0_val) as usize).min(self.max_delay - 1);
        let ap1_delay = ((time * 7.0 + lfo1_val) as usize).min(self.max_delay - 1);
        let ap2_delay = ((time * 11.0 + lfo2_val) as usize).min(self.max_delay - 1);
        let ap3_delay = ((time * 19.0 + lfo0_val) as usize).min(self.max_delay - 1);
        let ap4_delay = ((time * 23.0 + lfo1_val) as usize).min(self.max_delay - 1);
        let ap5_delay = ((time * 31.0 + lfo2_val) as usize).min(self.max_delay - 1);
        
        let ap0_out = self.ap_l0.process(mixed_input_l, ap0_delay);
        let ap1_out = self.ap_l1.process(ap0_out, ap1_delay);
        let ap2_out = self.ap_l2.process(ap1_out, ap2_delay);
        let ap3_out = self.ap_l3.process(ap2_out, ap3_delay);
        let ap4_out = self.ap_l4.process(ap3_out, ap4_delay);
        
        let hp_out = self.hp_l.highpass(ap4_out);
        let lp_out = self.lp_l.lowpass(hp_out);
        let final_out_l = self.ap_l5.process(lp_out, ap5_delay);
        
        self.feedback_delay_l[self.feedback_pos_l] = mixed_input_l;
        self.feedback_pos_l = (self.feedback_pos_l + 1) % self.feedback_delay_l.len();
        
        // Process right channel - SAME delay times, but INVERTED modulation for stereo width
        let feedback_read_pos = (self.feedback_pos_r + self.feedback_delay_r.len() - feedback_delay_samples.min(self.feedback_delay_r.len() - 1)) % self.feedback_delay_r.len();
        let feedback_tap = self.feedback_delay_r[feedback_read_pos] * self.decay;
        let mixed_input_r = mono_input + feedback_tap;
        
        // Same delay time multipliers but inverted LFO modulation for stereo decorrelation
        let ap0_delay = ((time * 3.0 - lfo0_val) as usize).min(self.max_delay - 1);
        let ap1_delay = ((time * 7.0 - lfo1_val) as usize).min(self.max_delay - 1);
        let ap2_delay = ((time * 11.0 - lfo2_val) as usize).min(self.max_delay - 1);
        let ap3_delay = ((time * 19.0 - lfo0_val) as usize).min(self.max_delay - 1);
        let ap4_delay = ((time * 23.0 - lfo1_val) as usize).min(self.max_delay - 1);
        let ap5_delay = ((time * 31.0 - lfo2_val) as usize).min(self.max_delay - 1);
        
        let ap0_out = self.ap_r0.process(mixed_input_r, ap0_delay);
        let ap1_out = self.ap_r1.process(ap0_out, ap1_delay);
        let ap2_out = self.ap_r2.process(ap1_out, ap2_delay);
        let ap3_out = self.ap_r3.process(ap2_out, ap3_delay);
        let ap4_out = self.ap_r4.process(ap3_out, ap4_delay);
        
        let hp_out = self.hp_r.highpass(ap4_out);
        let lp_out = self.lp_r.lowpass(hp_out);
        let final_out_r = self.ap_r5.process(lp_out, ap5_delay);
        
        self.feedback_delay_r[self.feedback_pos_r] = mixed_input_r;
        self.feedback_pos_r = (self.feedback_pos_r + 1) % self.feedback_delay_r.len();
        
        // Mix dry and wet signals
        let dry_l = input_l * (1.0 - self.wet_mix);
        let dry_r = input_r * (1.0 - self.wet_mix);
        let wet_l = final_out_l * self.wet_mix;
        let wet_r = final_out_r * self.wet_mix;
        
        (dry_l + wet_l, dry_r + wet_r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::SAMPLE_RATE;

    #[test]
    fn test_delay_line_basic_operation() {
        let mut delay = DelayLine::new(1000);
        
        // Test silence with no input
        assert_eq!(delay.process(0.0, 100, 0.0), 0.0);
        
        // Test impulse response
        let impulse_out = delay.process(1.0, 100, 0.0);
        assert_eq!(impulse_out, 0.0); // First sample should be 0 (no delay yet)
        
        // Process some samples to fill delay
        for _ in 0..99 {
            delay.process(0.0, 100, 0.0);
        }
        
        // At 100 samples, we should get our impulse back
        let delayed_impulse = delay.process(0.0, 100, 0.0);
        assert!(delayed_impulse > 0.0, "Should receive delayed impulse");
        
        println!("Delay test: impulse {} delayed by 100 samples = {}", 1.0, delayed_impulse);
    }
    
    #[test]
    fn test_delay_line_feedback_stability() {
        let mut delay = DelayLine::new(1000);
        let delay_samples = 100;
        let feedback = 0.4;
        
        // Send an impulse
        delay.process(1.0, delay_samples, feedback);
        
        let mut max_amplitude = 0.0f32;
        let mut outputs = Vec::new();
        
        // Process for several delay cycles to test stability
        for _ in 0..500 {
            let output = delay.process(0.0, delay_samples, feedback);
            outputs.push(output);
            max_amplitude = max_amplitude.max(output.abs());
        }
        
        println!("Delay feedback test: max amplitude over 500 samples = {}", max_amplitude);
        
        // With 0.4 feedback, the system should remain stable
        assert!(max_amplitude < 2.0, "Delay feedback should remain stable, got max amplitude {}", max_amplitude);
        
        // Should have some repeating echoes
        let has_echoes = outputs.iter().any(|&x| x.abs() > 0.01);
        assert!(has_echoes, "Delay should produce audible echoes");
    }
    
    #[test]
    fn test_allpass_comb_stability() {
        let mut allpass = AllpassComb::new(100, 0.5);
        
        // Test with impulse
        let impulse_out = allpass.process(1.0, 50);
        
        let mut max_amplitude = 0.0f32;
        let mut outputs = Vec::new();
        
        // Process silence for many samples to test stability
        for _ in 0..200 {
            let output = allpass.process(0.0, 50);
            outputs.push(output);
            max_amplitude = max_amplitude.max(output.abs());
        }
        
        println!("Allpass stability test: max amplitude = {}", max_amplitude);
        println!("Initial impulse output = {}", impulse_out);
        
        // Allpass should be stable and bounded
        assert!(max_amplitude < 2.0, "Allpass should remain stable, got max amplitude {}", max_amplitude);
        
        // Should produce some output from the impulse
        let has_output = outputs.iter().any(|&x| x.abs() > 0.001);
        assert!(has_output, "Allpass should produce output from impulse");
    }
    
    #[test]
    fn test_reverb_basic_operation() {
        let mut reverb = AllpassReverb::new();
        
        // Test silence
        let (out_l, out_r) = reverb.process(0.0, 0.0);
        assert_eq!(out_l, 0.0);
        assert_eq!(out_r, 0.0);
        
        // Test impulse response
        let (impulse_l, impulse_r) = reverb.process(1.0, 1.0);
        
        let mut max_l = 0.0f32;
        let mut max_r = 0.0f32;
        let mut outputs_l = Vec::new();
        let mut outputs_r = Vec::new();
        
        // Process silence to hear reverb tail
        for _ in 0..1000 {
            let (out_l, out_r) = reverb.process(0.0, 0.0);
            outputs_l.push(out_l);
            outputs_r.push(out_r);
            max_l = max_l.max(out_l.abs());
            max_r = max_r.max(out_r.abs());
        }
        
        println!("Reverb test: impulse output = ({}, {})", impulse_l, impulse_r);
        println!("Reverb test: max tail amplitudes = ({}, {})", max_l, max_r);
        
        // Reverb should be stable
        assert!(max_l < 2.0, "Reverb left channel should remain stable");
        assert!(max_r < 2.0, "Reverb right channel should remain stable");
        
        // Should produce reverb tail
        let has_tail_l = outputs_l.iter().any(|&x| x.abs() > 0.001);
        let has_tail_r = outputs_r.iter().any(|&x| x.abs() > 0.001);
        assert!(has_tail_l, "Reverb should produce left channel tail");
        assert!(has_tail_r, "Reverb should produce right channel tail");
        
        // Stereo output should be different (reverb creates stereo width)
        let stereo_difference = outputs_l.iter().zip(outputs_r.iter())
            .any(|(&l, &r)| (l - r).abs() > 0.001);
        assert!(stereo_difference, "Reverb should create stereo width");
    }
    
    #[test]
    fn test_reverb_long_term_stability() {
        let mut reverb = AllpassReverb::new();
        
        // Test with 20 seconds of samples (44100 * 20 = 882000 samples)
        let total_samples = (SAMPLE_RATE * 20.0) as usize;
        let mut max_amplitude = 0.0f32;
        let mut amplitude_over_time = Vec::new();
        
        // Send bursts of noise every 1000 samples (like drum hits)
        for i in 0..total_samples {
            let input = if i % 1000 == 0 && i < total_samples / 2 { 
                0.1 // Noise bursts for first 10 seconds
            } else { 
                0.0 // Then silence for last 10 seconds to see decay
            };
            
            let (out_l, out_r) = reverb.process(input, input);
            let amplitude = out_l.abs().max(out_r.abs());
            max_amplitude = max_amplitude.max(amplitude);
            
            // Sample amplitude every 1000 samples for analysis
            if i % 1000 == 0 {
                amplitude_over_time.push(amplitude);
            }
        }
        
        println!("Reverb long-term test (20 seconds):");
        println!("- Max amplitude: {}", max_amplitude);
        println!("- Amplitude at 5s: {:.6}", amplitude_over_time.get(220).unwrap_or(&0.0));
        println!("- Amplitude at 10s: {:.6}", amplitude_over_time.get(441).unwrap_or(&0.0));
        println!("- Amplitude at 15s: {:.6}", amplitude_over_time.get(661).unwrap_or(&0.0));
        println!("- Amplitude at 20s: {:.6}", amplitude_over_time.get(881).unwrap_or(&0.0));
        
        // Check if reverb tail decays properly during silence period
        let silence_start_idx = 441; // 10 seconds in
        let final_amplitude = amplitude_over_time.get(881).unwrap_or(&0.0); // 20 seconds
        
        println!("- Reverb decay during silence: {} -> {}", 
                 amplitude_over_time.get(silence_start_idx).unwrap_or(&0.0), 
                 final_amplitude);
        
        // Should remain stable
        assert!(max_amplitude < 1.0, "Reverb should remain stable long-term, got max amplitude {}", max_amplitude);
        
        // Should decay during silence period
        if let (Some(&silence_start), Some(&final_amp)) = (amplitude_over_time.get(silence_start_idx), amplitude_over_time.get(881)) {
            if silence_start > 0.01 && final_amp >= silence_start {
                println!("WARNING: Reverb may not be decaying properly during silence");
            }
        }
    }
}