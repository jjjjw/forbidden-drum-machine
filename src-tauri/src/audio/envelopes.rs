use crate::audio::AudioGenerator;

fn bias_curve(bias: f32, x: f32) -> f32 {
    x / (((1.0 / bias) - 2.0) * (1.0 - x) + 1.0)
}

fn bias_clip(bias: f32) -> f32 {
    bias.clamp(0.03, 0.97)
}

pub struct Segment {
    start_value: f32,
    end_value: f32,
    duration_seconds: f32,
    bias: f32,
    sample_rate: f32,

    // Runtime state
    current_value: f32,
    current_sample: u32,
    total_samples: u32,
    is_active: bool,
}

impl Segment {
    pub fn new(
        start_value: f32,
        end_value: f32,
        duration_seconds: f32,
        bias: f32,
        sample_rate: f32,
    ) -> Self {
        let total_samples = (duration_seconds * sample_rate).max(1.0) as u32;

        Self {
            start_value,
            end_value,
            duration_seconds,
            bias: bias_clip(bias),
            sample_rate,
            current_value: start_value,
            current_sample: 0,
            total_samples,
            is_active: false,
        }
    }

    pub fn trigger(&mut self) {
        self.current_value = self.start_value;
        self.current_sample = 0;
        self.is_active = true;
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn is_finished(&self) -> bool {
        self.current_sample >= self.total_samples
    }

    pub fn set_bias(&mut self, bias: f32) {
        self.bias = bias_clip(bias);
    }

    pub fn set_duration_seconds(&mut self, duration_seconds: f32) {
        self.duration_seconds = duration_seconds;
        self.total_samples = (duration_seconds * self.sample_rate).max(1.0) as u32;
    }

    pub fn set_start_value(&mut self, start_value: f32) {
        self.start_value = start_value;
    }

    pub fn set_end_value(&mut self, end_value: f32) {
        self.end_value = end_value;
    }

    pub fn get_current_value(&self) -> f32 {
        self.current_value
    }

    pub fn get_end_level(&self) -> f32 {
        self.end_value
    }
}

impl AudioGenerator for Segment {
    fn next_sample(&mut self) -> f32 {
        if !self.is_active {
            return self.current_value;
        }

        if self.current_sample >= self.total_samples {
            self.is_active = false;
            self.current_value = self.end_value;
            return self.current_value;
        }

        // Calculate progress (0.0 to 1.0)
        let progress = self.current_sample as f32 / self.total_samples as f32;

        // Apply bias curve to progress
        // Beware divide-by-zero if start and end are the same
        let curved_progress = bias_curve(self.bias, progress);

        // Interpolate between start and end values
        self.current_value =
            self.start_value + (self.end_value - self.start_value) * curved_progress;

        self.current_sample += 1;

        self.current_value
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.total_samples = (self.duration_seconds * sample_rate).max(1.0) as u32;
    }
}

pub struct AREnvelope {
    attack_segment: Segment,
    release_segment: Segment,
    sample_rate: f32,

    state: AREnvelopeState,
    current_level: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum AREnvelopeState {
    Idle,
    Attack,
    Release,
}

impl AREnvelope {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            attack_segment: Segment::new(0.0, 1.0, 0.01, 0.3, sample_rate), // 10ms attack, logarithmic-like
            release_segment: Segment::new(1.0, 0.0, 0.1, 0.7, sample_rate), // 100ms release, exponential-like
            sample_rate,
            state: AREnvelopeState::Idle,
            current_level: 0.0,
        }
    }

    pub fn set_attack_time(&mut self, time: f32) {
        let time = time.max(0.001); // Minimum 1ms
        self.attack_segment.set_duration_seconds(time);
    }

    pub fn set_release_time(&mut self, time: f32) {
        let time = time.max(0.001); // Minimum 1ms
        self.release_segment.set_duration_seconds(time);
    }

    pub fn set_attack_bias(&mut self, bias: f32) {
        self.attack_segment.set_bias(bias);
    }

    pub fn set_release_bias(&mut self, bias: f32) {
        self.release_segment.set_bias(bias);
    }

    pub fn trigger(&mut self) {
        self.state = AREnvelopeState::Attack;
        // Start attack from current level to avoid pops
        self.attack_segment.set_start_value(self.current_level);
        self.attack_segment.trigger();
    }

    pub fn is_active(&self) -> bool {
        self.state != AREnvelopeState::Idle
    }
}

impl AudioGenerator for AREnvelope {
    fn next_sample(&mut self) -> f32 {
        match self.state {
            AREnvelopeState::Idle => {
                self.current_level = 0.0;
                0.0
            }
            AREnvelopeState::Attack => {
                if self.attack_segment.is_finished() {
                    self.current_level = 1.0;
                    self.state = AREnvelopeState::Release;
                    self.release_segment.trigger();
                } else {
                    self.current_level = self.attack_segment.next_sample();
                }
                self.current_level
            }
            AREnvelopeState::Release => {
                if self.release_segment.is_finished() {
                    self.current_level = 0.0;
                    self.state = AREnvelopeState::Idle;
                } else {
                    self.current_level = self.release_segment.next_sample();
                }
                self.current_level
            }
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.attack_segment.set_sample_rate(sample_rate);
        self.release_segment.set_sample_rate(sample_rate);
    }
}

// AREEnvelope - Attack-Release-End envelope (extends AR with configurable end level)
pub struct AREEnvelope {
    attack_segment: Segment,
    release_segment: Segment,
    sample_rate: f32,

    state: AREnvelopeState,
    current_level: f32,
}

impl AREEnvelope {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            attack_segment: Segment::new(0.0, 1.0, 0.01, 0.3, sample_rate), // 10ms attack, logarithmic-like
            release_segment: Segment::new(1.0, 0.0, 0.1, 0.7, sample_rate), // 100ms release to configurable end, exponential-like
            sample_rate,
            state: AREnvelopeState::Idle,
            current_level: 0.0,
        }
    }

    pub fn set_attack_time(&mut self, time: f32) {
        let time = time.max(0.001);
        self.attack_segment.set_duration_seconds(time);
    }

    pub fn set_release_time(&mut self, time: f32) {
        let time = time.max(0.001);
        self.release_segment.set_duration_seconds(time);
    }

    pub fn set_end_level(&mut self, level: f32) {
        let level = level.clamp(0.0, 1.0);
        self.release_segment.set_end_value(level);
    }

    pub fn set_attack_bias(&mut self, bias: f32) {
        self.attack_segment.set_bias(bias);
    }

    pub fn set_release_bias(&mut self, bias: f32) {
        self.release_segment.set_bias(bias);
    }

    pub fn trigger(&mut self) {
        self.state = AREnvelopeState::Attack;
        // Start attack from current level to avoid pops
        self.attack_segment.set_start_value(self.current_level);
        self.attack_segment.trigger();
    }

    pub fn is_active(&self) -> bool {
        self.state != AREnvelopeState::Idle
    }
}

impl AudioGenerator for AREEnvelope {
    fn next_sample(&mut self) -> f32 {
        match self.state {
            AREnvelopeState::Idle => {
                self.current_level = self.release_segment.get_end_level();
                self.current_level
            }
            AREnvelopeState::Attack => {
                if self.attack_segment.is_finished() {
                    self.current_level = 1.0;
                    self.state = AREnvelopeState::Release;
                    self.release_segment.trigger();
                } else {
                    self.current_level = self.attack_segment.next_sample();
                }
                self.current_level
            }
            AREnvelopeState::Release => {
                if self.release_segment.is_finished() {
                    self.current_level = self.release_segment.get_end_level();
                    self.state = AREnvelopeState::Idle;
                } else {
                    self.current_level = self.release_segment.next_sample();
                }
                self.current_level
            }
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.attack_segment.set_sample_rate(sample_rate);
        self.release_segment.set_sample_rate(sample_rate);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ar_envelope_basic_operation() {
        let sample_rate = 44100.0;
        let mut env = AREnvelope::new(sample_rate);
        env.set_attack_time(0.1); // 100ms attack
        env.set_release_time(0.2); // 200ms release

        // Test initial state
        assert_eq!(env.next_sample(), 0.0);
        assert!(!env.is_active());

        // Trigger envelope
        env.trigger();
        assert!(env.is_active());

        let mut max_level = 0.0f32;
        let mut samples_in_attack = 0;
        let mut samples_in_release = 0;

        // Process through attack phase
        while env.state == AREnvelopeState::Attack {
            let level = env.next_sample();
            max_level = max_level.max(level);
            samples_in_attack += 1;
            if samples_in_attack > 10000 {
                // Safety break
                break;
            }
        }

        println!(
            "Attack phase: {} samples, max level: {}",
            samples_in_attack, max_level
        );

        // Process through release phase
        while env.is_active() {
            let _level = env.next_sample();
            samples_in_release += 1;
            if samples_in_release > 10000 {
                // Safety break
                break;
            }
        }

        println!("Release phase: {} samples", samples_in_release);

        // Verify envelope behavior
        assert!(max_level > 0.0, "Envelope should reach some positive level");
        assert!(samples_in_attack > 0, "Should have attack samples");
        assert!(samples_in_release > 0, "Should have release samples");

        // Final level should be 0
        assert_eq!(env.next_sample(), 0.0);
        assert!(!env.is_active());
    }

    #[test]
    fn test_ar_envelope_levels() {
        let sample_rate = 44100.0;
        let mut env = AREnvelope::new(sample_rate);
        env.set_attack_time(0.01); // 10ms attack (441 samples at 44.1kHz)
        env.set_release_time(0.01); // 10ms release

        env.trigger();

        let mut all_levels = Vec::new();

        // Collect all envelope levels
        while env.is_active() {
            all_levels.push(env.next_sample());
            if all_levels.len() > 2000 {
                // Safety break
                break;
            }
        }

        let max_level = all_levels.iter().fold(0.0f32, |a, &b| a.max(b));
        let min_level = all_levels.iter().fold(f32::INFINITY, |a, &b| a.min(b));

        println!(
            "Envelope levels - min: {}, max: {}, total samples: {}",
            min_level,
            max_level,
            all_levels.len()
        );
        println!(
            "First 10 levels: {:?}",
            &all_levels[..all_levels.len().min(10)]
        );
        println!(
            "Last 10 levels: {:?}",
            &all_levels[all_levels.len().saturating_sub(10)..]
        );

        assert!(max_level <= 1.0, "Envelope should not exceed 1.0");
        assert!(min_level >= 0.0, "Envelope should not go below 0.0");
    }

    #[test]
    fn test_bias_curves_preserve_timing_and_amplitude() {
        let attack_time = 0.05; // 50ms
        let release_time = 0.1; // 100ms

        let bias_values = [0.3, 0.5, 0.7]; // Different bias curves

        for &attack_bias in &bias_values {
            for &release_bias in &bias_values {
                let sample_rate = 44100.0;
                let mut env = AREnvelope::new(sample_rate);
                env.set_attack_time(attack_time);
                env.set_release_time(release_time);
                env.set_attack_bias(attack_bias);
                env.set_release_bias(release_bias);

                env.trigger();

                let mut max_level = 0.0f32;
                let mut samples_in_attack = 0;
                let mut samples_in_release = 0;
                let mut levels = Vec::new();

                // Collect attack phase
                while env.state == AREnvelopeState::Attack {
                    let level = env.next_sample();
                    levels.push(level);
                    max_level = max_level.max(level);
                    samples_in_attack += 1;
                    if samples_in_attack > 5000 {
                        break;
                    }
                }

                // Collect release phase
                while env.is_active() {
                    let level = env.next_sample();
                    levels.push(level);
                    samples_in_release += 1;
                    if samples_in_release > 10000 {
                        break;
                    }
                }

                let expected_attack_samples = (attack_time * sample_rate) as u32;
                let expected_release_samples = (release_time * sample_rate) as u32;

                println!("Bias {:.1}/{:.1}: attack {} samples (expected {}), release {} samples (expected {}), max level {}",
                    attack_bias, release_bias, samples_in_attack, expected_attack_samples,
                    samples_in_release, expected_release_samples, max_level);

                // Timing should be consistent regardless of bias type
                assert!(
                    (samples_in_attack as i32 - expected_attack_samples as i32).abs() <= 1,
                    "Attack timing should be consistent for bias {:.1}",
                    attack_bias
                );
                assert!(
                    (samples_in_release as i32 - expected_release_samples as i32).abs() <= 1,
                    "Release timing should be consistent for bias {:.1}",
                    release_bias
                );

                // Maximum amplitude should always reach 1.0 regardless of bias
                assert!(
                    (max_level - 1.0f32).abs() < 0.001,
                    "Max level should be 1.0 for all bias curves, got {} with bias {:.1}",
                    max_level,
                    attack_bias
                );

                // Envelope should end at 0
                assert!(
                    !env.is_active(),
                    "Envelope should be inactive after completion"
                );
                assert_eq!(env.next_sample(), 0.0, "Final level should be 0.0");
            }
        }
    }

    #[test]
    fn test_segment_basic_operation() {
        let sample_rate = 44100.0;
        let mut segment = Segment::new(0.0, 1.0, 0.1, 0.5, sample_rate); // 0 to 1 over 100ms, neutral bias

        // Should start inactive
        assert!(!segment.is_active());
        assert_eq!(segment.get_current_value(), 0.0);

        // Trigger the segment
        segment.trigger();
        assert!(segment.is_active());

        let mut values = Vec::new();
        let mut sample_count = 0;

        // Collect all values from the segment
        while segment.is_active() && sample_count < 10000 {
            values.push(segment.next_sample());
            sample_count += 1;
        }

        println!("Segment generated {} samples", values.len());
        println!("First 5 values: {:?}", &values[..5.min(values.len())]);
        println!(
            "Last 5 values: {:?}",
            &values[values.len().saturating_sub(5)..]
        );

        // Should have completed
        assert!(!segment.is_active());
        assert!(segment.is_finished());

        // Should have taken approximately 100ms worth of samples
        let expected_samples = (0.1 * sample_rate) as usize;
        assert!(
            (values.len() as i32 - expected_samples as i32).abs() <= 1,
            "Expected ~{} samples, got {}",
            expected_samples,
            values.len()
        );

        // First value should be close to start value
        assert!(
            (values[0] - 0.0).abs() < 0.1,
            "First value should be near start"
        );

        // Last value should be close to end value
        assert!(
            (values[values.len() - 1] - 1.0).abs() < 0.1,
            "Last value should be near end"
        );

        // Values should generally increase (with neutral bias)
        let increases = values.windows(2).filter(|w| w[1] > w[0]).count();
        let total_windows = values.len() - 1;
        assert!(
            increases as f32 / total_windows as f32 > 0.8,
            "Most values should increase with neutral bias"
        );
    }

    #[test]
    fn test_segment_bias_curves() {
        let sample_rate = 44100.0;
        let duration = 0.05; // 50ms

        // Test different bias values
        let bias_values = [0.1, 0.3, 0.5, 0.7, 0.9];

        for &bias in &bias_values {
            let mut segment = Segment::new(0.0, 1.0, duration, bias, sample_rate);
            segment.trigger();

            let mut values = Vec::new();
            while segment.is_active() {
                values.push(segment.next_sample());
            }

            // All segments should start and end at the same values
            assert!(
                (values[0] - 0.0).abs() < 0.01,
                "Start value should be 0 for bias {}",
                bias
            );
            assert!(
                (values[values.len() - 1] - 1.0).abs() < 0.01,
                "End value should be 1 for bias {}",
                bias
            );

            // Check midpoint behavior based on bias
            let midpoint_idx = values.len() / 2;
            let midpoint_value = values[midpoint_idx];

            if bias < 0.5 {
                // Low bias should be below 0.5 at midpoint (logarithmic-like)
                assert!(
                    midpoint_value < 0.5,
                    "Low bias {} should be < 0.5 at midpoint, got {}",
                    bias,
                    midpoint_value
                );
            } else if bias > 0.5 {
                // High bias should be above 0.5 at midpoint (exponential-like)
                assert!(
                    midpoint_value > 0.5,
                    "High bias {} should be > 0.5 at midpoint, got {}",
                    bias,
                    midpoint_value
                );
            }

            println!("Bias {}: midpoint value = {:.3}", bias, midpoint_value);
        }
    }

    #[test]
    fn test_segment_descending() {
        let sample_rate = 44100.0;
        let mut segment = Segment::new(1.0, 0.0, 0.05, 0.5, sample_rate); // 1 to 0 over 50ms

        segment.trigger();

        let mut values = Vec::new();
        while segment.is_active() {
            values.push(segment.next_sample());
        }

        // Should start at 1 and end at 0
        assert!((values[0] - 1.0).abs() < 0.01, "Should start at 1.0");
        assert!(
            (values[values.len() - 1] - 0.0).abs() < 0.01,
            "Should end at 0.0"
        );

        // Values should generally decrease
        let decreases = values.windows(2).filter(|w| w[1] < w[0]).count();
        let total_windows = values.len() - 1;
        assert!(
            decreases as f32 / total_windows as f32 > 0.8,
            "Most values should decrease"
        );
    }
}
