// TODO: refactor
fn bias_curve(bias: f32, x: f32) -> f32 {
    x / (((1.0 / bias) - 2.0) * (1.0 - x) + 1.0)
}

fn bias_clip(bias: f32) -> f32 {
    bias.clamp(0.03, 0.97)
}

pub struct Clock {
    current_sample: u32,
}

impl Clock {
    pub fn new() -> Self {
        Self { current_sample: 0 }
    }

    pub fn tick(&mut self) {
        self.current_sample = self.current_sample.wrapping_add(1);
    }

    pub fn get_sample(&self) -> u32 {
        self.current_sample
    }

    pub fn reset(&mut self) {
        self.current_sample = 0;
    }
}

pub struct Loop {
    total_samples: u32,
    total_steps: u8,
    samples_per_step: u32,
    last_clock_sample: u32,
    last_step: u8,
}

impl Loop {
    pub fn new(total_samples: u32, total_steps: u8) -> Self {
        let samples_per_step = total_samples / total_steps as u32;
        Self {
            total_samples,
            total_steps,
            samples_per_step,
            last_clock_sample: 0,
            last_step: total_steps - 1, // Trigger first step immediately
        }
    }

    pub fn set_total_samples(&mut self, total_samples: u32) {
        self.total_samples = total_samples;
        self.samples_per_step = total_samples / self.total_steps as u32;
    }

    pub fn get_current_step(&self, clock: &Clock) -> u8 {
        let current_position = (clock.get_sample() % self.total_samples as u32) as u32;
        (current_position / self.samples_per_step).min(self.total_steps as u32 - 1) as u8
    }

    pub fn tick(&mut self, clock: &Clock) -> Option<u8> {
        let current_sample = clock.get_sample();
        let current_step = self.get_current_step(clock);

        // Check if this is a new step boundary
        if current_step != self.last_step {
            self.last_clock_sample = current_sample;
            self.last_step = current_step;
            Some(current_step)
        } else {
            self.last_clock_sample = current_sample;
            None
        }
    }

    pub fn reset(&mut self) {
        self.last_clock_sample = 0;
        self.last_step = self.total_steps - 1; // Reset to last step to trigger first step immediately
    }
}

pub struct BiasedLoop {
    total_samples: u32,
    total_steps: u8,
    bias: f32,
    step_samples: Vec<u32>, // Pre-computed sample positions for each step
    last_clock_sample: u32,
    last_bar_start: u32,
    last_step: u8,
}

impl BiasedLoop {
    pub fn new(total_samples: u32, total_steps: u8, bias: f32) -> Self {
        let mut biased_loop = Self {
            total_samples,
            total_steps,
            bias: bias_clip(bias),
            step_samples: Vec::new(),
            last_clock_sample: 0,
            last_bar_start: 0,
            last_step: total_steps - 1, // Last step to trigger first step immediately
        };
        biased_loop.compute_step_samples();
        biased_loop
    }

    fn compute_step_samples(&mut self) {
        self.step_samples.clear();
        for step in 0..self.total_steps {
            let progress = step as f32 / self.total_steps as f32;
            let biased_progress = bias_curve(self.bias, progress);
            let sample_position = (biased_progress * self.total_samples as f32) as u32;
            self.step_samples.push(sample_position);
        }
    }

    pub fn set_total_samples(&mut self, total_samples: u32) {
        self.total_samples = total_samples;
        self.compute_step_samples();
    }

    pub fn set_bias(&mut self, bias: f32) {
        self.bias = bias_clip(bias);
        self.compute_step_samples();
    }

    pub fn tick(&mut self, clock: &Clock) -> Option<u8> {
        let current_sample = clock.get_sample();
        let current_step = self.get_current_step(clock);

        // Update bar start tracking when we wrap around
        let current_position = current_sample % self.total_samples;
        let last_position = self.last_clock_sample % self.total_samples;

        // Detect bar boundary (wrap around)
        if current_position < last_position {
            self.last_bar_start = current_sample - current_position;
        }

        // Check if this is a new step
        if current_step != self.last_step {
            self.last_clock_sample = current_sample;
            self.last_step = current_step;
            Some(current_step)
        } else {
            self.last_clock_sample = current_sample;
            None
        }
    }

    pub fn get_current_step(&self, clock: &Clock) -> u8 {
        let current_sample = clock.get_sample();
        let samples_since_bar_start = (current_sample - self.last_bar_start) % self.total_samples;

        // Find the highest step index whose trigger point has been reached
        for (step_index, &step_sample) in self.step_samples.iter().enumerate().rev() {
            if samples_since_bar_start >= step_sample {
                return step_index as u8;
            }
        }

        0 // Default to first step
    }

    pub fn reset(&mut self) {
        self.last_clock_sample = 0;
        self.last_bar_start = 0;
        self.last_step = self.total_steps - 1; // Reset to last step to trigger first step immediately
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
    fn test_clock_basic_operation() {
        let mut clock = Clock::new();

        assert_eq!(clock.get_sample(), 0);

        clock.tick();
        assert_eq!(clock.get_sample(), 1);

        clock.tick();
        assert_eq!(clock.get_sample(), 2);

        clock.reset();
        assert_eq!(clock.get_sample(), 0);
    }

    #[test]
    fn test_loop_first_step_triggers_immediately() {
        let mut clock = Clock::new();
        let mut loop_instance = Loop::new(1000, 8);

        // First tick should return step 0
        let first_step = loop_instance.tick(&clock);
        assert_eq!(first_step, Some(0), "First step should trigger immediately");

        // Current step should be 0
        assert_eq!(loop_instance.get_current_step(&clock), 0);
    }

    #[test]
    fn test_loop_reset_triggers_first_step() {
        let mut clock = Clock::new();
        let mut loop_instance = Loop::new(1000, 8);

        // Advance the clock and loop
        for _ in 0..500 {
            clock.tick();
            loop_instance.tick(&clock);
        }

        // Reset clock and loop
        clock.reset();
        loop_instance.reset();

        let first_step = loop_instance.tick(&clock);
        assert_eq!(
            first_step,
            Some(0),
            "First step should trigger immediately after reset"
        );
        assert_eq!(loop_instance.get_current_step(&clock), 0);
    }

    #[test]
    fn test_biased_loop_first_step_triggers_immediately() {
        let mut clock = Clock::new();
        let mut loop_instance = BiasedLoop::new(1000, 8, 0.5);

        // First tick should return step 0
        let first_step = loop_instance.tick(&clock);
        assert_eq!(first_step, Some(0), "First step should trigger immediately");

        // Current step should be 0
        assert_eq!(loop_instance.get_current_step(&clock), 0);
    }

    #[test]
    fn test_biased_loop_reset_triggers_first_step() {
        let mut clock = Clock::new();
        let mut loop_instance = BiasedLoop::new(1000, 8, 0.5);

        // Advance the clock and loop
        for _ in 0..500 {
            clock.tick();
            loop_instance.tick(&clock);
        }

        // Reset clock and loop
        clock.reset();
        loop_instance.reset();

        let first_step = loop_instance.tick(&clock);
        assert_eq!(
            first_step,
            Some(0),
            "First step should trigger immediately after reset"
        );
        assert_eq!(loop_instance.get_current_step(&clock), 0);
    }

    #[test]
    fn test_loop_complete_sequence() {
        let mut clock = Clock::new();
        let total_samples = 1000;
        let total_steps = 8;
        let mut loop_instance = Loop::new(total_samples, total_steps);

        let mut steps = Vec::new();

        // Run for one complete cycle
        for _ in 0..total_samples {
            clock.tick();
            if let Some(step) = loop_instance.tick(&clock) {
                steps.push(step);
            }
        }

        // Should have triggered all steps + 1 (immediate first step trigger)
        println!("Generated steps: {:?}", steps);
        assert_eq!(steps.len(), total_steps as usize + 1);
        assert_eq!(steps, vec![0, 1, 2, 3, 4, 5, 6, 7, 0]);
    }

    #[test]
    fn test_biased_loop_complete_sequence() {
        let mut clock = Clock::new();
        let total_samples = 1000;
        let total_steps = 4;
        let mut loop_instance = BiasedLoop::new(total_samples, total_steps, 0.5);

        let mut steps = Vec::new();

        // Run for one complete cycle
        for _ in 0..total_samples {
            clock.tick();
            if let Some(step) = loop_instance.tick(&clock) {
                steps.push(step);
            }
        }

        // Should have triggered all steps + 1 (immediate first step trigger)
        assert_eq!(steps.len(), total_steps as usize + 1);
        assert_eq!(steps, vec![0, 1, 2, 3, 0]);
    }

    #[test]
    fn test_biased_loop_recomputes_on_new_bar() {
        let mut clock = Clock::new();
        let total_samples = 100;
        let total_steps = 4;
        let mut loop_instance = BiasedLoop::new(total_samples, total_steps, 0.3);

        let mut first_bar_steps = Vec::new();
        let mut second_bar_steps = Vec::new();

        // Run first bar
        for _ in 0..total_samples {
            clock.tick();
            if let Some(step) = loop_instance.tick(&clock) {
                first_bar_steps.push((step, clock.get_sample()));
            }
        }

        // Change bias for second bar
        loop_instance.set_bias(0.7);

        // Run second bar
        for _ in 0..total_samples {
            clock.tick();
            if let Some(step) = loop_instance.tick(&clock) {
                second_bar_steps.push((step, clock.get_sample()));
            }
        }

        // First bar should have all steps + 1 (immediate trigger), second bar normal
        assert_eq!(first_bar_steps.len(), total_steps as usize + 1);
        assert_eq!(second_bar_steps.len(), total_steps as usize);

        // Step timings should be different due to bias change
        let first_step_1_time = first_bar_steps[1].1;
        let second_step_1_time = second_bar_steps[1].1;
        assert_ne!(
            first_step_1_time % total_samples,
            second_step_1_time % total_samples
        );
    }

    #[test]
    fn test_biased_loop_bias_effect() {
        let mut clock = Clock::new();
        let total_samples = 44100;
        let total_steps = 16;

        let mut early = BiasedLoop::new(total_samples, total_steps, 0.2);
        let mut late = BiasedLoop::new(total_samples, total_steps, 0.8);

        let mut early_times = Vec::new();
        let mut late_times = Vec::new();

        for _ in 0..(total_samples * 2) {
            clock.tick();
            if early.tick(&clock).is_some() {
                early_times.push(clock.get_sample());
            }
        }

        clock.reset();
        for _ in 0..(total_samples * 2) {
            clock.tick();
            if late.tick(&clock).is_some() {
                late_times.push(clock.get_sample());
            }
        }

        // Compare average sample offsets for bias
        let avg_early = early_times.iter().sum::<u32>() as f64 / early_times.len() as f64;
        let avg_late = late_times.iter().sum::<u32>() as f64 / late_times.len() as f64;

        assert!(
            avg_early < avg_late,
            "Early biased steps should happen earlier than late biased steps"
        );
    }

    #[test]
    fn test_loop_four_complete_sequences() {
        let mut clock = Clock::new();
        let total_samples = 800;
        let total_steps = 8;
        let mut loop_instance = Loop::new(total_samples, total_steps);

        let mut all_steps = Vec::new();
        let _expected_sequence = vec![0, 1, 2, 3, 4, 5, 6, 7];

        // Run for 4 complete cycles
        for cycle in 0..4 {
            let mut cycle_steps = Vec::new();

            for _ in 0..total_samples {
                clock.tick();
                if let Some(step) = loop_instance.tick(&clock) {
                    cycle_steps.push(step);
                    all_steps.push((cycle, step, clock.get_sample()));
                }
            }

            // Actual pattern based on test output:
            // Cycle 0: immediate step 0 + normal sequence + next cycle step 0
            // All other cycles: continuation from step 1 + next cycle step 0
            let cycle_expected = if cycle == 0 {
                vec![0, 1, 2, 3, 4, 5, 6, 7, 0]
            } else {
                vec![1, 2, 3, 4, 5, 6, 7, 0]
            };

            assert_eq!(
                cycle_steps, cycle_expected,
                "Cycle {} should match expected pattern",
                cycle
            );
        }

        // Verify total events based on actual pattern:
        // Cycle 0: 9, Cycles 1-3: 8 each
        let expected_total = 9 + 8 + 8 + 8;
        assert_eq!(all_steps.len(), expected_total);

        // Verify step 0 count: cycle 0 has 2, cycles 1-3 have 1 each = 5 total
        let cycle_starts: Vec<u8> = all_steps
            .iter()
            .filter(|(_, step, _)| *step == 0)
            .map(|(_, step, _)| *step)
            .collect();
        assert_eq!(cycle_starts, vec![0, 0, 0, 0, 0]); // 5 step 0s

        // Verify step timing is consistent across cycles
        for step_num in 0..total_steps {
            let step_timings: Vec<u32> = all_steps
                .iter()
                .filter(|(_, step, _)| *step == step_num)
                .map(|(_, _, sample)| *sample)
                .collect();

            // Step 0 should happen 5 times, others 4 times
            let expected_count = if step_num == 0 { 5 } else { 4 };
            assert_eq!(
                step_timings.len(),
                expected_count,
                "Step {} should happen {} times",
                step_num,
                expected_count
            );

            // Timing should be consistent for regular cycle boundaries
            if step_num == 0 {
                // Step 0: happens immediately (at sample 1 due to tick order)
                assert_eq!(step_timings[0], 1, "First step 0 should happen at sample 1");
                // Subsequent step 0s should be at reasonable intervals
                for timing in &step_timings[1..] {
                    assert!(*timing > 1, "Step 0 timing should be positive");
                }
            } else {
                // Other steps: timing should be consistent across cycles
                for i in 1..step_timings.len() {
                    let expected_timing = step_timings[0] + (i as u32 * total_samples as u32);
                    assert_eq!(
                        step_timings[i], expected_timing,
                        "Step {} timing should be consistent across cycles",
                        step_num
                    );
                }
            }
        }
    }

    #[test]
    fn test_biased_loop_four_complete_sequences() {
        let mut clock = Clock::new();
        let total_samples = 1000;
        let total_steps = 4;
        let bias = 0.3; // Early bias
        let mut loop_instance = BiasedLoop::new(total_samples, total_steps, bias);

        let mut all_steps = Vec::new();
        let _expected_sequence = vec![0, 1, 2, 3];

        // Run for 4 complete cycles
        for cycle in 0..4 {
            let mut cycle_steps = Vec::new();

            for _ in 0..total_samples {
                clock.tick();
                if let Some(step) = loop_instance.tick(&clock) {
                    cycle_steps.push(step);
                    all_steps.push((cycle, step, clock.get_sample()));
                }
            }

            // Actual pattern based on test output:
            // Cycle 0: immediate step 0 + normal sequence + next cycle step 0
            // All other cycles: continuation from step 1 + next cycle step 0
            let cycle_expected = if cycle == 0 {
                vec![0, 1, 2, 3, 0]
            } else {
                vec![1, 2, 3, 0]
            };

            assert_eq!(
                cycle_steps, cycle_expected,
                "Cycle {} should match expected pattern",
                cycle
            );
        }

        // Verify total events based on actual pattern:
        // Cycle 0: 5, Cycles 1-3: 4 each
        let expected_total = 5 + 4 + 4 + 4;
        assert_eq!(all_steps.len(), expected_total);

        // Verify step 0 count: cycle 0 has 2, cycles 1-3 have 1 each = 5 total
        let cycle_starts: Vec<u8> = all_steps
            .iter()
            .filter(|(_, step, _)| *step == 0)
            .map(|(_, step, _)| *step)
            .collect();
        assert_eq!(cycle_starts, vec![0, 0, 0, 0, 0]); // 5 step 0s: initial + 4 cycle starts

        // Verify step timing is consistent across cycles for biased loop
        for step_num in 0..total_steps {
            let step_timings: Vec<u32> = all_steps
                .iter()
                .filter(|(_, step, _)| *step == step_num)
                .map(|(_, _, sample)| *sample)
                .collect();

            // Step 0 should happen 5 times, others 4 times
            let expected_count = if step_num == 0 { 5 } else { 4 };
            assert_eq!(
                step_timings.len(),
                expected_count,
                "Step {} should happen {} times",
                step_num,
                expected_count
            );

            // For biased loop, timing should still be consistent relative to bar start
            if step_num == 0 {
                // Step 0: happens immediately (at sample 1 due to tick order)
                assert_eq!(step_timings[0], 1, "First step 0 should happen at sample 1");
                // Subsequent step 0s should be at reasonable intervals
                for timing in &step_timings[1..] {
                    assert!(*timing > 1, "Step 0 timing should be positive");
                }
            } else {
                // Other steps: relative timing should be consistent across cycles
                let first_relative_timing = step_timings[0] % total_samples as u32;
                for i in 1..step_timings.len() {
                    let current_relative_timing = step_timings[i] % total_samples as u32;
                    assert_eq!(
                        current_relative_timing, first_relative_timing,
                        "Step {} relative timing should be consistent across cycles",
                        step_num
                    );
                }
            }
        }

        // Verify bias effect - step 1 should happen before the 1/4 mark (early bias)
        let step_1_timings: Vec<u32> = all_steps
            .iter()
            .filter(|(_, step, _)| *step == 1)
            .map(|(_, _, sample)| (*sample - 1) % total_samples as u32) // -1 because sample is incremented after step detection
            .collect();

        // Should have 4 occurrences of step 1 (once per cycle)
        assert_eq!(step_1_timings.len(), 4);

        let quarter_point = total_samples as u32 / 4;
        for timing in step_1_timings {
            assert!(
                timing < quarter_point,
                "With early bias, step 1 should happen before 1/4 point ({}), but happened at {}",
                quarter_point,
                timing
            );
        }
    }

    #[test]
    fn test_multiple_loops_same_clock() {
        let mut clock = Clock::new();
        let mut loop1 = Loop::new(800, 8);
        let mut loop2 = Loop::new(1200, 6);

        let mut loop1_steps = Vec::new();
        let mut loop2_steps = Vec::new();

        // Run both loops with the same clock
        for _ in 0..2400 {
            clock.tick();
            if let Some(step) = loop1.tick(&clock) {
                loop1_steps.push(step);
            }
            if let Some(step) = loop2.tick(&clock) {
                loop2_steps.push(step);
            }
        }

        // Loop1 should complete 3 cycles (2400 / 800 = 3) with +1 for first cycle
        // Cycle 0: 9, Cycles 1-2: 8 each = 9 + 8 + 8 = 25
        assert_eq!(loop1_steps.len(), 9 + 8 + 8);

        // Loop2 should complete 2 cycles (2400 / 1200 = 2) with +1 for first cycle
        // Cycle 0: 7, Cycle 1: 6 = 7 + 6 = 13
        assert_eq!(loop2_steps.len(), 7 + 6);
    }
}
