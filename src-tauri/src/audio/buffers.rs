use crate::audio::sec_to_samples;

// Integer delay buffer
pub struct DelayBuffer {
    buffer: Vec<f32>,
    delay_samples: usize,
    write_pos: usize,
}

impl DelayBuffer {
    pub fn new(max_samples: usize) -> Self {
        Self {
            buffer: vec![0.0; max_samples],
            write_pos: 0,
            delay_samples: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn set_delay_seconds(&mut self, seconds: f32) {
        self.delay_samples = (sec_to_samples(seconds) as usize)
            .max(0)
            .min(self.buffer.len() - 1);
    }

    pub fn set_delay_samples(&mut self, samples: usize) {
        self.delay_samples = samples.max(0).min(self.buffer.len() - 1);
    }

    pub fn read(&self) -> f32 {
        let read_pos = if self.delay_samples <= self.write_pos {
            self.write_pos - self.delay_samples
        } else {
            self.buffer.len() - (self.delay_samples - self.write_pos)
        };
        self.buffer[read_pos]
    }

    pub fn write(&mut self, value: f32) {
        self.buffer[self.write_pos] = value;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_delay_buffer_basic_operation() {
        let mut buffer = DelayBuffer::new(100);

        // Test initial silence
        buffer.set_delay_samples(10);
        assert_eq!(buffer.read(), 0.0);

        // Write an impulse
        buffer.write(1.0);

        // Should read the value just written when delay=1
        buffer.set_delay_samples(1);
        assert_eq!(buffer.read(), 1.0);

        // Fill with zeros to advance the buffer
        for _ in 0..10 {
            buffer.write(0.0);
        }

        // At 11 samples delay, should read back the impulse
        buffer.set_delay_samples(11);
        let delayed = buffer.read();
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
        buffer.set_delay_samples(delay_samples);

        // Write a sequence of values
        for i in 0..100 {
            let input = (i as f32) * 0.1;

            if i >= 20 {
                // After delay_samples, we should read back the earlier value
                let delayed = buffer.read();
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
        buffer.set_delay_samples(delay_samples);
        let feedback = 0.9;

        // Send impulse
        buffer.write(1.0);

        let mut max_output = 0.0f32;
        let mut outputs = Vec::new();

        // Run feedback loop for many cycles
        for i in 0..500 {
            let delayed = buffer.read();
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
}
