// TODO: refactor
fn bias_curve(bias: f32, x: f32) -> f32 {
    x / (((1.0 / bias) - 2.0) * (1.0 - x) + 1.0)
}

fn bias_clip(bias: f32) -> f32 {
    bias.clamp(0.03, 0.97)
}

pub struct Clock {
    samples_per_step: u32,
    current_step_samples: u32,
    current_sample: u32,
    total_samples: u32, // Total samples in the sequence
    total_steps: u8,    // Total steps in the sequence
    current_step: u8,
}

impl Clock {
    pub fn new(total_samples: u32, total_steps: u8) -> Self {
        let samples_per_step = total_samples / total_steps as u32;
        let mut clock = Self {
            samples_per_step,
            current_sample: 0,
            current_step: 0,
            current_step_samples: 16,
            total_samples,
            total_steps,
        };
        clock
    }

    pub fn set_total_samples(&mut self, total_samples: u32) {
        self.total_samples = total_samples;
        self.samples_per_step = total_samples / self.total_steps as u32;
    }

    pub fn tick(&mut self) -> Option<u8> {
        self.current_sample += 1;
        if self.current_sample >= self.total_samples {
            self.current_sample = 0;
            self.current_step = 0;
            self.current_step_samples = 0;
            return None;
        }

        self.current_step_samples += 1;

        if self.current_step_samples >= self.samples_per_step {
            self.current_step_samples = 0;
            let current_step = self.current_step;
            self.current_step = (self.current_step + 1) % self.total_steps;
            Some(current_step)
        } else {
            None
        }
    }

    pub fn get_current_step(&self) -> u8 {
        self.current_step
    }

    pub fn reset(&mut self) {
        self.current_sample = 0;
        self.current_step = 0;
        self.current_step_samples = 0;
    }
}

pub struct BiasedClock {
    total_samples: u32,
    total_steps: u8,

    current_sample: u32,
    current_step: u8,

    bias: f32,
    next_event_sample: u32,
}

impl BiasedClock {
    pub fn new(total_samples: u32, total_steps: u8, bias: f32) -> Self {
        let mut clock = Self {
            total_samples,
            total_steps,
            current_sample: 0,
            current_step: 0,
            bias: bias_clip(bias),
            next_event_sample: 0,
        };
        clock.schedule_next_event();
        clock
    }

    fn schedule_next_event(&mut self) {
        let progress = self.current_step as f32 / self.total_steps as f32;
        let biased_progress = bias_curve(self.bias, progress);
        self.next_event_sample = (biased_progress * self.total_samples as f32) as u32;
    }

    pub fn tick(&mut self) -> Option<u8> {
        if self.current_sample >= self.total_samples {
            self.current_sample = 0;
            self.current_step = 0;
            self.schedule_next_event();
            return None;
        }

        let event = if self.current_sample >= self.next_event_sample {
            let step = self.current_step;
            self.current_step = (self.current_step + 1) % self.total_steps;
            self.schedule_next_event();
            Some(step)
        } else {
            None
        };

        self.current_sample += 1;
        event
    }

    pub fn reset(&mut self) {
        self.current_sample = 0;
        self.current_step = 0;
        self.schedule_next_event();
    }

    pub fn set_total_samples(&mut self, total_samples: u32) {
        self.total_samples = total_samples;
        self.schedule_next_event();
    }

    pub fn set_bias(&mut self, bias: f32) {
        self.bias = bias_clip(bias);
        self.schedule_next_event();
    }

    pub fn get_current_step(&self) -> u8 {
        self.current_step
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bias_curve_basic_behavior() {
        let cases = [(0.1, 0.5), (0.5, 0.5), (0.9, 0.5)];

        for (bias, x) in cases {
            let result = bias_curve(bias, x);
            assert!(
                result >= 0.0 && result <= 1.0,
                "bias_curve({:.2}, {:.2}) = {:.4}, out of bounds",
                bias,
                x,
                result
            );
        }

        assert!((bias_curve(0.3, 0.0) - 0.0).abs() < f32::EPSILON);
        assert!((bias_curve(0.3, 1.0) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_biased_clock_basic_stepping() {
        let total_samples = 44100;
        let total_steps = 8;
        let mut clock = BiasedClock::new(total_samples, total_steps, 0.5);

        let mut steps_triggered = Vec::new();

        for _ in 0..(total_samples * 2) {
            if let Some(step) = clock.tick() {
                steps_triggered.push(step);
            }
        }

        assert_eq!(steps_triggered.len(), total_steps as usize * 2);
        assert_eq!(steps_triggered[0], 0);
        assert_eq!(steps_triggered[total_steps as usize], 0);
    }

    #[test]
    fn test_biased_clock_bias_effect() {
        let total_samples = 44100;
        let total_steps = 16;

        let mut early = BiasedClock::new(total_samples, total_steps, 0.2);
        let mut late = BiasedClock::new(total_samples, total_steps, 0.8);

        let mut early_times = Vec::new();
        let mut late_times = Vec::new();

        for i in 0..(total_samples * 2) {
            if early.tick().is_some() {
                early_times.push(i);
            }
            if late.tick().is_some() {
                late_times.push(i);
            }
        }

        // Compare average sample offsets for bias
        let avg_early = early_times.iter().sum::<u32>() as f32 / early_times.len() as f32;
        let avg_late = late_times.iter().sum::<u32>() as f32 / late_times.len() as f32;

        assert!(
            avg_early < avg_late,
            "Early biased steps should happen earlier than late biased steps"
        );
    }

    #[test]
    fn test_biased_clock_reset() {
        let mut clock = BiasedClock::new(44100, 16, 0.5);
        let mut steps = Vec::new();

        for _ in 0..44100 {
            if let Some(step) = clock.tick() {
                steps.push(step);
            }
        }

        assert!(!steps.is_empty());
        clock.reset();

        let mut steps_after_reset = Vec::new();
        for _ in 0..44100 {
            if let Some(step) = clock.tick() {
                steps_after_reset.push(step);
            }
        }

        assert_eq!(steps, steps_after_reset);
    }

    #[test]
    fn test_biased_clock_step_uniqueness() {
        let mut clock = BiasedClock::new(44100, 16, 0.5);
        let mut unique_steps = std::collections::HashSet::new();

        for _ in 0..44100 {
            if let Some(step) = clock.tick() {
                unique_steps.insert(step);
            }
        }

        assert_eq!(
            unique_steps.len(),
            16,
            "Should hit all steps once per bar (16 steps)"
        );
    }
}
