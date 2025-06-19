use crate::audio::{sec_to_samples, AudioGenerator};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CurveType {
    Linear,
    Exponential,
    Logarithmic,
}

pub struct AREnvelope {
    attack_time: f32,
    release_time: f32,
    attack_curve: CurveType,
    pub(crate) release_curve: CurveType,

    pub(crate) state: EnvelopeState,
    pub(crate) current_level: f32,
    attack_increment: f32,
    release_decrement: f32,
    pub(crate) attack_samples: u32,
    pub(crate) release_samples: u32,
    pub(crate) current_sample: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum EnvelopeState {
    Idle,
    Attack,
    Release,
}

impl AREnvelope {
    pub fn new() -> Self {
        Self {
            attack_time: 0.01,
            release_time: 0.1,
            attack_curve: CurveType::Logarithmic,
            release_curve: CurveType::Exponential,

            state: EnvelopeState::Idle,
            current_level: 0.0,
            attack_increment: 0.0,
            release_decrement: 0.0,
            attack_samples: 0,
            release_samples: 0,
            current_sample: 0,
        }
    }

    pub fn set_attack_time(&mut self, time: f32) {
        self.attack_time = time.max(0.001);
        self.calculate_parameters();
    }

    pub fn set_release_time(&mut self, time: f32) {
        self.release_time = time.max(0.001);
        self.calculate_parameters();
    }

    pub fn set_attack_curve(&mut self, curve: CurveType) {
        self.attack_curve = curve;
    }

    pub fn set_release_curve(&mut self, curve: CurveType) {
        self.release_curve = curve;
    }

    fn calculate_parameters(&mut self) {
        self.attack_samples = sec_to_samples(self.attack_time) as u32;
        self.release_samples = sec_to_samples(self.release_time) as u32;

        self.attack_increment = if self.attack_samples > 0 {
            1.0 / self.attack_samples as f32
        } else {
            1.0
        };

        self.release_decrement = if self.release_samples > 0 {
            1.0 / self.release_samples as f32
        } else {
            1.0
        };
    }

    pub fn trigger(&mut self) {
        self.state = EnvelopeState::Attack;
        self.current_sample = 0;
        self.calculate_parameters();
    }

    pub fn release(&mut self) {
        if self.state != EnvelopeState::Idle {
            self.state = EnvelopeState::Release;
            self.current_sample = 0;
        }
    }

    pub(crate) fn apply_curve(&self, progress: f32, curve_type: CurveType) -> f32 {
        match curve_type {
            CurveType::Linear => progress,
            CurveType::Exponential => progress * progress,
            CurveType::Logarithmic => 1.0 - (1.0 - progress).powi(2),
        }
    }

    pub fn is_active(&self) -> bool {
        self.state != EnvelopeState::Idle
    }
}

impl AudioGenerator for AREnvelope {
    fn next_sample(&mut self) -> f32 {
        match self.state {
            EnvelopeState::Idle => 0.0,

            EnvelopeState::Attack => {
                if self.current_sample >= self.attack_samples {
                    self.state = EnvelopeState::Release;
                    self.current_sample = 0;
                    self.current_level = 1.0;
                } else {
                    let progress = self.current_sample as f32 / self.attack_samples as f32;
                    self.current_level = self.apply_curve(progress, self.attack_curve);
                    self.current_sample += 1;
                }
                self.current_level
            }

            EnvelopeState::Release => {
                if self.current_sample >= self.release_samples {
                    self.state = EnvelopeState::Idle;
                    self.current_level = 0.0;
                } else {
                    let progress = self.current_sample as f32 / self.release_samples as f32;
                    self.current_level = 1.0 - self.apply_curve(progress, self.release_curve);
                    self.current_sample += 1;
                }
                self.current_level
            }
        }
    }
}

// AREEnvelope - Attack-Release-End envelope (extends AR with configurable end level)
pub struct AREEnvelope {
    ar_envelope: AREnvelope,
    end_level: f32,
}

impl AREEnvelope {
    pub fn new() -> Self {
        Self {
            ar_envelope: AREnvelope::new(),
            end_level: 0.0,
        }
    }

    pub fn set_attack_time(&mut self, time: f32) {
        self.ar_envelope.set_attack_time(time);
    }

    pub fn set_release_time(&mut self, time: f32) {
        self.ar_envelope.set_release_time(time);
    }

    pub fn set_end_level(&mut self, level: f32) {
        self.end_level = level.clamp(0.0, 1.0);
    }

    pub fn set_attack_curve(&mut self, curve: CurveType) {
        self.ar_envelope.set_attack_curve(curve);
    }

    pub fn set_release_curve(&mut self, curve: CurveType) {
        self.ar_envelope.set_release_curve(curve);
    }

    pub fn trigger(&mut self) {
        self.ar_envelope.trigger();
    }

    pub fn release(&mut self) {
        self.ar_envelope.release();
    }

    pub fn is_active(&self) -> bool {
        self.ar_envelope.is_active()
    }
}

impl AudioGenerator for AREEnvelope {
    fn next_sample(&mut self) -> f32 {
        match self.ar_envelope.state {
            EnvelopeState::Idle => self.end_level,
            EnvelopeState::Attack => {
                // Use AR envelope for attack phase
                self.ar_envelope.next_sample()
            }
            EnvelopeState::Release => {
                // Modified release phase that goes to end_level instead of 0
                if self.ar_envelope.current_sample >= self.ar_envelope.release_samples {
                    self.ar_envelope.state = EnvelopeState::Idle;
                    self.ar_envelope.current_level = self.end_level;
                } else {
                    let progress = self.ar_envelope.current_sample as f32
                        / self.ar_envelope.release_samples as f32;
                    let release_progress = self
                        .ar_envelope
                        .apply_curve(progress, self.ar_envelope.release_curve);
                    self.ar_envelope.current_level =
                        1.0 - (1.0 - self.end_level) * release_progress;
                    self.ar_envelope.current_sample += 1;
                }
                self.ar_envelope.current_level
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::SAMPLE_RATE;

    #[test]
    fn test_ar_envelope_basic_operation() {
        let mut env = AREnvelope::new();
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
        while env.state == EnvelopeState::Attack {
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
        let mut env = AREnvelope::new();
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
    fn test_curve_types_preserve_timing_and_amplitude() {
        let attack_time = 0.05; // 50ms
        let release_time = 0.1; // 100ms

        let curves = [
            CurveType::Linear,
            CurveType::Exponential,
            CurveType::Logarithmic,
        ];

        for &attack_curve in &curves {
            for &release_curve in &curves {
                let mut env = AREnvelope::new();
                env.set_attack_time(attack_time);
                env.set_release_time(release_time);
                env.set_attack_curve(attack_curve);
                env.set_release_curve(release_curve);

                env.trigger();

                let mut max_level = 0.0f32;
                let mut samples_in_attack = 0;
                let mut samples_in_release = 0;
                let mut levels = Vec::new();

                // Collect attack phase
                while env.state == EnvelopeState::Attack {
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

                let expected_attack_samples = (attack_time * SAMPLE_RATE) as u32;
                let expected_release_samples = (release_time * SAMPLE_RATE) as u32;

                println!("Curve {:?}/{:?}: attack {} samples (expected {}), release {} samples (expected {}), max level {}",
                    attack_curve, release_curve, samples_in_attack, expected_attack_samples,
                    samples_in_release, expected_release_samples, max_level);

                // Timing should be consistent regardless of curve type
                assert!(
                    (samples_in_attack as i32 - expected_attack_samples as i32).abs() <= 1,
                    "Attack timing should be consistent for curve {:?}",
                    attack_curve
                );
                assert!(
                    (samples_in_release as i32 - expected_release_samples as i32).abs() <= 1,
                    "Release timing should be consistent for curve {:?}",
                    release_curve
                );

                // Maximum amplitude should always reach 1.0 regardless of curve
                assert!(
                    (max_level - 1.0f32).abs() < 0.001,
                    "Max level should be 1.0 for all curves, got {} with curve {:?}",
                    max_level,
                    attack_curve
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
}
