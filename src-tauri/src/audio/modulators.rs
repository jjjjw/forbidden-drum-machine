// Modulators module - using SineOscillator for LFOs
use fastrand::Rng;

pub struct SampleAndHold {
    rng: Rng,
    current_value: f32,
    target_value: f32,
    rate_hz: f32,
    samples_per_update: u32,
    sample_counter: u32,
    min_value: f32,
    max_value: f32,
    slew_rate: f32, // Max change per sample
    sample_rate: f32,
}

impl SampleAndHold {
    pub fn new(rate_hz: f32, min_value: f32, max_value: f32, slew_time_ms: f32, sample_rate: f32) -> Self {
        let samples_per_update = (sample_rate / rate_hz) as u32;
        let mut rng = Rng::new();
        let initial_value = min_value + rng.f32() * (max_value - min_value);
        
        // Calculate slew rate for smooth transitions
        let slew_samples = (slew_time_ms / 1000.0) * sample_rate;
        let slew_rate = (max_value - min_value) / slew_samples;
        
        Self {
            rng,
            current_value: initial_value,
            target_value: initial_value,
            rate_hz,
            samples_per_update,
            sample_counter: 0,
            min_value,
            max_value,
            slew_rate,
            sample_rate,
        }
    }
    
    pub fn next_sample(&mut self) -> f32 {
        self.sample_counter += 1;
        
        // Generate new target value when timer expires
        if self.sample_counter >= self.samples_per_update {
            self.sample_counter = 0;
            self.target_value = self.min_value + self.rng.f32() * (self.max_value - self.min_value);
        }
        
        // Slew towards target value
        let diff = self.target_value - self.current_value;
        if diff.abs() > self.slew_rate {
            if diff > 0.0 {
                self.current_value += self.slew_rate;
            } else {
                self.current_value -= self.slew_rate;
            }
        } else {
            self.current_value = self.target_value;
        }
        
        self.current_value
    }
    
    pub fn get_current_value(&self) -> f32 {
        self.current_value
    }
    
    pub fn set_rate(&mut self, rate_hz: f32) {
        self.rate_hz = rate_hz;
        self.samples_per_update = (self.sample_rate / rate_hz).max(1.0) as u32;
    }
    
    pub fn set_range(&mut self, min_value: f32, max_value: f32) {
        self.min_value = min_value;
        self.max_value = max_value;
        
        // Clamp current values to new range
        self.current_value = self.current_value.clamp(min_value, max_value);
        self.target_value = self.target_value.clamp(min_value, max_value);
        
        // Recalculate slew rate for new range
        let slew_time_ms = (self.slew_rate * self.sample_rate * 1000.0) / (self.max_value - self.min_value);
        self.set_slew_time(slew_time_ms);
    }
    
    pub fn set_slew_time(&mut self, slew_time_ms: f32) {
        let slew_samples = (slew_time_ms / 1000.0) * self.sample_rate;
        self.slew_rate = (self.max_value - self.min_value) / slew_samples;
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        // Recalculate dependent values
        self.samples_per_update = (sample_rate / self.rate_hz).max(1.0) as u32;
        // Preserve slew time in milliseconds when sample rate changes
        let current_slew_time_ms = (self.slew_rate * self.sample_rate * 1000.0) / (self.max_value - self.min_value);
        self.set_slew_time(current_slew_time_ms);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sample_and_hold_basic_operation() {
        let sample_rate = 44100.0;
        let mut sh = SampleAndHold::new(1.0, 0.0, 1.0, 100.0, sample_rate); // 1Hz rate, 0-1 range, 100ms slew
        
        // Initial value should be within range
        let initial_value = sh.get_current_value();
        assert!(initial_value >= 0.0 && initial_value <= 1.0, "Initial value {} should be within range [0.0, 1.0]", initial_value);
        
        // Process some samples
        let mut values = Vec::new();
        for _ in 0..100 {
            values.push(sh.next_sample());
        }
        
        // All values should be within range
        for (i, &value) in values.iter().enumerate() {
            assert!(value >= 0.0 && value <= 1.0, "Value {} at sample {} should be within range [0.0, 1.0]", value, i);
        }
        
        println!("Sample-and-hold basic test: {} samples processed, range maintained", values.len());
    }
    
    #[test]
    fn test_sample_and_hold_rate_changes() {
        let sample_rate = 44100.0;
        let rate_hz = 1.0; // 1Hz = every 44100 samples at 44.1kHz
        let mut sh = SampleAndHold::new(rate_hz, 0.0, 1.0, 10.0, sample_rate); // Short slew time
        
        let expected_samples_per_update = (sample_rate / rate_hz) as u32;
        let mut target_changes = 0;
        let mut sample_count = 0;
        
        // Track when new targets are generated (not slewed values)
        for _ in 0..3 {
            // Process one full period
            for _ in 0..expected_samples_per_update {
                sh.next_sample();
                sample_count += 1;
            }
            target_changes += 1;
            println!("Target change {} after {} samples", target_changes, sample_count);
        }
        
        // Should have generated 3 new targets
        assert!(target_changes == 3, "Should have seen 3 target changes, got {}", target_changes);
        println!("Rate test: {} target changes over {} samples (expected every {} samples)", 
            target_changes, sample_count, expected_samples_per_update);
    }
    
    #[test]
    fn test_sample_and_hold_slew_limiting() {
        let sample_rate = 44100.0;
        let mut sh = SampleAndHold::new(0.1, 0.0, 1.0, 200.0, sample_rate); // Very slow rate, 200ms slew
        
        // Force a target change by processing past the update time
        let samples_per_update = (sample_rate / 0.1) as usize;
        let _initial_value = sh.get_current_value();
        
        // Process samples to trigger target change
        for _ in 0..samples_per_update + 10 {
            sh.next_sample();
        }
        
        // Now track slewing behavior
        let mut values = Vec::new();
        let mut max_change_per_sample = 0.0f32;
        
        for _ in 0..1000 {
            let prev_value = sh.get_current_value();
            let new_value = sh.next_sample();
            let change = (new_value - prev_value).abs();
            max_change_per_sample = max_change_per_sample.max(change);
            values.push(new_value);
        }
        
        // Calculate expected maximum change per sample based on slew time
        let slew_samples = (200.0 / 1000.0) * sample_rate; // 200ms in samples
        let expected_max_change = 1.0 / slew_samples; // Max range / slew samples
        
        println!("Slew test: max change per sample = {:.6}, expected max = {:.6}", 
            max_change_per_sample, expected_max_change);
        
        // Allow some tolerance for floating point precision
        assert!(max_change_per_sample <= expected_max_change * 1.1, 
            "Max change per sample {} should not exceed expected rate {}", 
            max_change_per_sample, expected_max_change);
        
        // Values should change gradually, not jump instantly
        assert!(max_change_per_sample > 0.0, "Should see gradual changes due to slewing");
        assert!(max_change_per_sample < 0.1, "Changes should be gradual, not instantaneous");
    }
    
    #[test]
    fn test_sample_and_hold_range_limits() {
        let sample_rate = 44100.0;
        let min_val = 0.2;
        let max_val = 0.8;
        let mut sh = SampleAndHold::new(5.0, min_val, max_val, 10.0, sample_rate); // Fast slew for quicker settling
        
        // Process many samples to see various random values
        // Need enough samples to see multiple target changes and full range exploration
        let mut all_values = Vec::new();
        for _ in 0..50000 { // More samples for better range coverage
            all_values.push(sh.next_sample());
        }
        
        let actual_min = all_values.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let actual_max = all_values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        
        println!("Range test: min = {:.3}, max = {:.3}, expected [{:.3}, {:.3}]", 
            actual_min, actual_max, min_val, max_val);
        
        // Values should stay within specified range
        assert!(actual_min >= min_val - 0.001, "Minimum value {} should be >= {}", actual_min, min_val);
        assert!(actual_max <= max_val + 0.001, "Maximum value {} should be <= {}", actual_max, max_val);
        
        // Should explore a reasonable portion of the range over time
        let range_coverage = (actual_max - actual_min) / (max_val - min_val);
        assert!(range_coverage > 0.25, "Should cover at least 25% of range, got {:.1}%", range_coverage * 100.0);
    }
    
    #[test]
    fn test_sample_and_hold_set_methods() {
        let sample_rate = 44100.0;
        let mut sh = SampleAndHold::new(10.0, 0.0, 1.0, 10.0, sample_rate); // High rate, fast slew
        
        // Test that current value is initially in range
        let initial_value = sh.get_current_value();
        assert!(initial_value >= 0.0 && initial_value <= 1.0, "Initial value should be in range");
        
        // Test range change - this should clamp current values
        sh.set_range(0.3, 0.7);
        let clamped_value = sh.get_current_value();
        assert!(clamped_value >= 0.3 && clamped_value <= 0.7, 
            "Value should be clamped to new range, got {}", clamped_value);
        
        // Test slew time change
        sh.set_slew_time(5.0); // Very fast slew
        
        // Process enough samples to see multiple target changes
        let mut all_values = Vec::new();
        for _ in 0..20000 { // Process many samples
            all_values.push(sh.next_sample());
        }
        
        let min_val = all_values.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_val = all_values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        
        // Should respect new range
        assert!(min_val >= 0.29, "New minimum range should be respected, got {}", min_val);
        assert!(max_val <= 0.71, "New maximum range should be respected, got {}", max_val);
        
        println!("Parameter update test: range [{:.3}, {:.3}] respected, clamped to [{:.3}, {:.3}]", 
            0.3, 0.7, min_val, max_val);
    }
}