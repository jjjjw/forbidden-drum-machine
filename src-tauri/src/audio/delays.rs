use crate::audio::buffers::DelayBuffer;
use crate::events::NodeEvent;
use crate::audio::filters::{OnePoleFilter, OnePoleMode};
use crate::audio::{AudioProcessor, AudioNode};

// Simple delay line without filtering
pub struct DelayLine {
    buffer: DelayBuffer,
    frozen: bool,
    feedback: f32,
    sample_rate: f32,
    gain: f32,
}

impl DelayLine {
    pub fn new(max_delay_seconds: f32, sample_rate: f32) -> Self {
        Self {
            buffer: DelayBuffer::new((max_delay_seconds * sample_rate) as usize),
            frozen: false,
            feedback: 0.0,
            sample_rate,
            gain: 1.0,
        }
    }

    pub fn set_freeze(&mut self, freeze: bool) {
        self.frozen = freeze;
    }

    pub fn set_delay_seconds(&mut self, delay_seconds: f32) {
        let delay_samples = (delay_seconds * self.sample_rate) as usize;
        self.buffer.set_delay_samples(delay_samples);
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(-1.0, 1.0);
    }

    pub fn read(&mut self) -> f32 {
        self.buffer.read()
    }

    pub fn read_at(&self, delay_seconds: f32) -> f32 {
        let delay_samples = (delay_seconds * self.sample_rate) as usize;
        self.buffer.read_at(delay_samples)
    }

    pub fn write(&mut self, input: f32, feedback: f32) {
        self.buffer.write(input + feedback * self.feedback);
    }

    pub fn advance(&mut self) {
        self.buffer.advance();
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }
}

impl AudioProcessor for DelayLine {
    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.read();

        if !self.frozen {
            self.write(input, delayed);
        }

        delayed
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }
}

impl AudioNode for DelayLine {
    fn process_stereo(&mut self, left_in: f32, right_in: f32) -> (f32, f32) {
        let mono_in = (left_in + right_in) * 0.5;
        let delayed = self.process(mono_in) * self.gain;
        (left_in + delayed, right_in + delayed)
    }

    fn handle_event(&mut self, event: NodeEvent) -> Result<(), String> {
        match event {
            NodeEvent::SetGain(gain) => {
                self.set_gain(gain);
                Ok(())
            }
            NodeEvent::SetFeedback(feedback) => {
                self.set_feedback(feedback);
                Ok(())
            }
            NodeEvent::SetDelaySeconds(seconds) => {
                self.set_delay_seconds(seconds);
                Ok(())
            }
            NodeEvent::SetFreeze(freeze) => {
                self.set_freeze(freeze);
                Ok(())
            }
            _ => Err(format!("Unsupported event for DelayLine: {:?}", event))
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        AudioProcessor::set_sample_rate(self, sample_rate);
    }
}

// Delay line with filtering
pub struct FilteredDelayLine {
    delay_line: DelayLine,
    highpass: OnePoleFilter,
    lowpass: OnePoleFilter,
    gain: f32,
}

impl FilteredDelayLine {
    pub fn new(max_delay_seconds: f32, sample_rate: f32) -> Self {
        Self {
            delay_line: DelayLine::new(max_delay_seconds, sample_rate),
            highpass: OnePoleFilter::new(300.0, OnePoleMode::Highpass, sample_rate),
            lowpass: OnePoleFilter::new(8000.0, OnePoleMode::Lowpass, sample_rate),
            gain: 1.0,
        }
    }

    pub fn set_freeze(&mut self, freeze: bool) {
        self.delay_line.set_freeze(freeze);
    }

    pub fn set_delay_seconds(&mut self, delay_seconds: f32) {
        self.delay_line.set_delay_seconds(delay_seconds);
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.delay_line.set_feedback(feedback);
    }

    pub fn set_highpass_freq(&mut self, freq: f32) {
        self.highpass.set_cutoff_frequency(freq);
    }

    pub fn set_lowpass_freq(&mut self, freq: f32) {
        self.lowpass.set_cutoff_frequency(freq);
    }

    pub fn read(&mut self) -> f32 {
        self.delay_line.read()
    }

    pub fn read_at(&self, delay_seconds: f32) -> f32 {
        self.delay_line.read_at(delay_seconds)
    }

    pub fn write(&mut self, input: f32, output: f32) {
        self.delay_line.write(input, output);
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }
}

impl AudioProcessor for FilteredDelayLine {
    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.delay_line.read();
        let filtered = self.lowpass.process(self.highpass.process(delayed));

        if !self.delay_line.frozen {
            self.delay_line.write(input, filtered);
        } else {
            self.delay_line.advance();
        }

        filtered
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        AudioProcessor::set_sample_rate(&mut self.delay_line, sample_rate);
        self.highpass.set_sample_rate(sample_rate);
        self.lowpass.set_sample_rate(sample_rate);
    }
}

impl AudioNode for FilteredDelayLine {
    fn process_stereo(&mut self, left_in: f32, right_in: f32) -> (f32, f32) {
        let mono_in = (left_in + right_in) * 0.5;
        let delayed = self.process(mono_in) * self.gain;
        (left_in + delayed, right_in + delayed)
    }

    fn handle_event(&mut self, event: NodeEvent) -> Result<(), String> {
        match event {
            NodeEvent::SetGain(gain) => {
                self.set_gain(gain);
                Ok(())
            }
            NodeEvent::SetFeedback(feedback) => {
                self.set_feedback(feedback);
                Ok(())
            }
            NodeEvent::SetDelaySeconds(seconds) => {
                self.set_delay_seconds(seconds);
                Ok(())
            }
            NodeEvent::SetFreeze(freeze) => {
                self.set_freeze(freeze);
                Ok(())
            }
            NodeEvent::SetHighpassFreq(freq) => {
                self.set_highpass_freq(freq);
                Ok(())
            }
            NodeEvent::SetLowpassFreq(freq) => {
                self.set_lowpass_freq(freq);
                Ok(())
            }
            _ => Err(format!("Unsupported event for FilteredDelayLine: {:?}", event))
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        AudioProcessor::set_sample_rate(self, sample_rate);
    }
}
#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_delay_line_basic_operation() {
        let sample_rate = 44100.0;
        let mut delay = DelayLine::new(1.0, sample_rate);
        delay.set_delay_seconds(100.0 / sample_rate);
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
        let sample_rate = 44100.0;
        let mut delay = DelayLine::new(1.0, sample_rate);
        let delay_seconds = 100.0 / sample_rate;
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
}

