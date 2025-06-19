use crate::audio::effects::{BloomReverb, DelayLine};
use crate::audio::instruments::{KickDrum, SnareDrum};
use crate::audio::modulators::SampleAndHold;
use crate::audio::{AudioGenerator, AudioProcessor, SAMPLE_RATE};

pub struct Clock {
    bpm: f32,
    samples_per_beat: u32,
    current_sample: u32,
    step: u8,
    steps_per_bar: u8,
}

impl Clock {
    pub fn new(bpm: f32) -> Self {
        let mut clock = Self {
            bpm,
            samples_per_beat: 0,
            current_sample: 0,
            step: 0,
            steps_per_bar: 16,
        };
        clock.calculate_timing();
        clock
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm;
        self.calculate_timing();
    }

    fn calculate_timing(&mut self) {
        let beats_per_second = self.bpm / 60.0;
        let steps_per_second = beats_per_second * (self.steps_per_bar as f32 / 4.0);
        self.samples_per_beat = (SAMPLE_RATE / steps_per_second) as u32;
    }

    pub fn tick(&mut self) -> Option<u8> {
        self.current_sample += 1;

        if self.current_sample >= self.samples_per_beat {
            self.current_sample = 0;
            let current_step = self.step;
            self.step = (self.step + 1) % self.steps_per_bar;
            Some(current_step)
        } else {
            None
        }
    }

    pub fn get_current_step(&self) -> u8 {
        self.step
    }

    pub fn reset(&mut self) {
        self.current_sample = 0;
        self.step = 0;
    }
}

pub struct DrumMachine {
    kick: KickDrum,
    snare: SnareDrum,
    clock: Clock,
    kick_pattern: [bool; 16],
    snare_pattern: [bool; 16],

    // Effects chain
    delay: DelayLine,
    reverb: BloomReverb,

    // Effects sends
    delay_send: f32,
    reverb_send: f32,

    // Sample and hold modulators
    delay_time_mod: SampleAndHold,
    reverb_size_mod: SampleAndHold,
    reverb_decay_mod: SampleAndHold,
}

impl DrumMachine {
    pub fn new() -> Self {
        let delay_samples = (0.5 * SAMPLE_RATE) as usize; // 0.5 seconds max delay

        Self {
            kick: KickDrum::new(),
            snare: SnareDrum::new(),
            clock: Clock::new(120.0),
            kick_pattern: [
                true, false, false, false, false, false, true, false, false, false, false, false,
                false, false, true, false,
            ],
            snare_pattern: [
                false, false, false, false, true, false, false, false, false, false, false, false,
                true, false, false, false,
            ],

            // Initialize effects
            delay: DelayLine::new(delay_samples),
            reverb: BloomReverb::new(),

            // Default send levels
            delay_send: 0.2,
            reverb_send: 0.3,

            // Initialize modulators with slower rates and configurable slew
            delay_time_mod: SampleAndHold::new(0.125, 0.1, 0.5, 150.0), // 8 sec updates, 150ms slew
            reverb_size_mod: SampleAndHold::new(0.165, 0.1, 0.3, 200.0), // 6 sec updates, 200ms slew
            reverb_decay_mod: SampleAndHold::new(0.1, 0.3, 0.7, 100.0), // 10 sec updates, 100ms slew
        }
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        self.clock.set_bpm(bpm);
    }

    pub fn set_kick_pattern(&mut self, pattern: [bool; 16]) {
        self.kick_pattern = pattern;
    }

    pub fn set_snare_pattern(&mut self, pattern: [bool; 16]) {
        self.snare_pattern = pattern;
    }

    pub fn next_sample(&mut self) -> (f32, f32) {
        if let Some(step) = self.clock.tick() {
            if self.kick_pattern[step as usize] {
                self.kick.trigger();
            }
            if self.snare_pattern[step as usize] {
                self.snare.trigger();
            }
        }

        // Update modulators
        let modulated_delay_time = self.delay_time_mod.next_sample();
        let modulated_reverb_size = self.reverb_size_mod.next_sample();
        let modulated_reverb_decay = self.reverb_decay_mod.next_sample();

        // Apply modulated parameters
        self.reverb.set_size(modulated_reverb_size);
        self.reverb.set_decay(modulated_reverb_decay);

        // Generate dry drum samples
        let kick_sample = self.kick.next_sample();
        let snare_sample = self.snare.next_sample();
        let dry_mixed = kick_sample + snare_sample;

        // Create sends to effects
        let delay_input = dry_mixed * self.delay_send;
        let reverb_input = dry_mixed * self.reverb_send;

        // Process through effects with modulated delay time
        self.delay.set_delay_seconds(modulated_delay_time);
        self.delay.set_feedback(0.4);
        let delay_output = self.delay.process(delay_input);
        let reverb_output = self.reverb.process(reverb_input);

        // Mix dry and wet signals with proper level management
        // Scale dry signal down since we're adding effects
        let dry_level = 0.6; // Leave headroom for effects
        let wet_level = 0.4; // Effects contribution

        let output_l = dry_mixed * dry_level + (delay_output + reverb_output) * wet_level;
        let output_r = dry_mixed * dry_level + (delay_output + reverb_output) * wet_level;

        (output_l, output_r)
    }

    pub fn get_kick(&mut self) -> &mut KickDrum {
        &mut self.kick
    }

    pub fn get_snare(&mut self) -> &mut SnareDrum {
        &mut self.snare
    }

    // Effects control methods
    pub fn set_delay_send(&mut self, send: f32) {
        self.delay_send = send.clamp(0.0, 1.0);
    }

    pub fn set_reverb_send(&mut self, send: f32) {
        self.reverb_send = send.clamp(0.0, 1.0);
    }

    pub fn set_delay_freeze(&mut self, freeze: bool) {
        self.delay.set_freeze(freeze);
    }

    pub fn set_delay_highpass(&mut self, freq: f32) {
        self.delay.set_highpass_freq(freq);
    }

    pub fn set_delay_lowpass(&mut self, freq: f32) {
        self.delay.set_lowpass_freq(freq);
    }

    pub fn set_reverb_size(&mut self, size: f32) {
        self.reverb.set_size(size);
    }

    pub fn set_reverb_decay(&mut self, decay: f32) {
        self.reverb.set_decay(decay);
    }

    pub fn set_reverb_wet_mix(&mut self, wet: f32) {
        // Mix is handled at the system level as a send effect
        // This method is kept for API compatibility but doesn't do anything
        let _ = wet; // Suppress unused parameter warning
    }

    pub fn set_reverb_highpass(&mut self, freq: f32) {
        self.reverb.set_feedback_highpass(freq);
    }

    pub fn set_reverb_lowpass(&mut self, freq: f32) {
        self.reverb.set_feedback_lowpass(freq);
    }

    // Modulator value getters
    pub fn get_current_delay_time(&self) -> f32 {
        self.delay_time_mod.get_current_value()
    }

    pub fn get_current_reverb_size(&self) -> f32 {
        self.reverb_size_mod.get_current_value()
    }

    pub fn get_current_reverb_decay(&self) -> f32 {
        self.reverb_decay_mod.get_current_value()
    }

    pub fn get_current_step(&self) -> u8 {
        self.clock.get_current_step()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::SAMPLE_RATE;

    #[test]
    fn test_audio_output_one_bar() {
        let mut drum_machine = DrumMachine::new();
        drum_machine.set_bpm(120.0);

        // Calculate samples for one bar (16 steps at 120 BPM)
        let beats_per_second = 120.0 / 60.0;
        let steps_per_second = beats_per_second * 4.0; // 16 steps per 4 beats
        let samples_per_step = SAMPLE_RATE / steps_per_second;
        let total_samples = (samples_per_step * 16.0) as usize;

        println!("Testing drum machine audio output for one bar");
        println!(
            "BPM: 120, Sample rate: {}, Samples per step: {:.0}, Total samples: {}",
            SAMPLE_RATE, samples_per_step, total_samples
        );

        let mut max_amplitude = 0.0f32;
        let mut rms_sum = 0.0f32;
        let mut step_triggers = Vec::new();

        for i in 0..total_samples {
            let (left, right) = drum_machine.next_sample();

            // Track maximum amplitude
            let amplitude = (left.abs().max(right.abs()));
            if amplitude > max_amplitude {
                max_amplitude = amplitude;
            }

            // Track RMS for volume analysis
            rms_sum += left * left + right * right;

            // Log when drums trigger (look for sudden amplitude increases)
            if amplitude > 0.1 && (i == 0 || amplitude > max_amplitude * 0.8) {
                let step = (i as f32 / samples_per_step).floor() as usize;
                if !step_triggers.contains(&step) {
                    step_triggers.push(step);
                }
            }
        }

        let rms = (rms_sum / (total_samples as f32 * 2.0)).sqrt();

        println!("Audio test results:");
        println!("- Max amplitude: {:.3}", max_amplitude);
        println!("- RMS level: {:.3}", rms);
        println!("- Steps with drum hits: {:?}", step_triggers);

        // Verify audio was generated
        assert!(max_amplitude > 0.0, "No audio output detected");
        assert!(
            max_amplitude < 1.0,
            "Audio clipping detected (amplitude >= 1.0)"
        );
        assert!(rms > 0.0, "No RMS level detected");

        // Verify expected drum pattern triggers
        assert!(!step_triggers.is_empty(), "No drum triggers detected");

        println!("✓ Audio output test passed - one bar played successfully");
    }
}
