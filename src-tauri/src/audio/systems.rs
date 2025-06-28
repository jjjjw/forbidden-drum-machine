use crate::audio::delays::FilteredDelayLine;
use crate::audio::instruments::{ClapDrum, KickDrum};
use crate::audio::modulators::SampleAndHold;
use crate::audio::reverbs::FDNReverb;
use crate::audio::{AudioGenerator, AudioProcessor, StereoAudioProcessor};
use crate::commands::AudioCommand;
use crate::events::{AudioEvent, AudioEventSender};
use crate::sequencing::{BiasedClock, MarkovChain};

// Calculate the number of samples for 4 beats based on BPM and sample rate
fn bpm_to_samples(bpm: f32, sample_rate: f32) -> u32 {
    (60.0 / bpm * sample_rate) as u32 * 4
}

pub struct DrumMachine {
    kick: KickDrum,
    clap: ClapDrum,
    kick_clock: BiasedClock,
    clap_clock: BiasedClock,
    kick_pattern: [bool; 16],
    clap_pattern: [bool; 16],

    // Markov chain for generating patterns
    markov_generator: MarkovChain,

    // Event sender for communicating with UI
    event_sender: AudioEventSender,

    // Track previous steps for event emission
    prev_kick_step: Option<u8>,
    prev_clap_step: Option<u8>,

    // Effects chain
    delay: FilteredDelayLine,
    reverb: FDNReverb,

    // Effects sends
    delay_send: f32,
    reverb_send: f32,

    // Sample and hold modulators
    delay_time_mod: SampleAndHold,
    reverb_size_mod: SampleAndHold,
    reverb_decay_mod: SampleAndHold,
    sample_rate: f32,
}

impl DrumMachine {
    pub fn new(sample_rate: f32, event_sender: AudioEventSender) -> Self {
        // Initialize clocks and Markov generator
        let total_samples_in_loop = bpm_to_samples(120.0, sample_rate);
        let kick_clock = BiasedClock::new(total_samples_in_loop, 16, 0.5); // Neutral bias initially
        let clap_clock = BiasedClock::new(total_samples_in_loop, 16, 0.5); // Neutral bias initially
        let markov_generator = MarkovChain::new(0.3); // 30% density

        Self {
            kick: KickDrum::new(sample_rate),
            clap: ClapDrum::new(sample_rate),
            kick_clock,
            clap_clock,
            kick_pattern: [
                true, false, false, false, false, false, true, false, false, false, false, false,
                false, false, true, false,
            ],
            clap_pattern: [
                false, false, false, false, true, false, false, false, false, false, false, false,
                true, false, false, false,
            ],

            // Markov generator
            markov_generator,

            // Event sender
            event_sender,

            // Initialize step tracking
            prev_kick_step: None,
            prev_clap_step: None,

            // Initialize effects
            delay: FilteredDelayLine::new(0.5, sample_rate), // 0.5 seconds max delay
            reverb: FDNReverb::new(sample_rate),

            // Default send levels
            delay_send: 0.2,
            reverb_send: 0.3,

            sample_rate,

            // Initialize modulators with slower rates and configurable slew
            delay_time_mod: SampleAndHold::new(0.125, 0.1, 0.5, 150.0, sample_rate), // 8 sec updates, 150ms slew
            reverb_size_mod: SampleAndHold::new(0.165, 0.5, 1.5, 200.0, sample_rate), // 6 sec updates, 200ms slew
            reverb_decay_mod: SampleAndHold::new(0.1, 0.5, 0.95, 100.0, sample_rate), // 10 sec updates, 100ms slew
        }
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        let total_samples_in_loop = bpm_to_samples(bpm, self.sample_rate);
        self.kick_clock.set_total_samples(total_samples_in_loop);
        self.clap_clock.set_total_samples(total_samples_in_loop);
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        // Update all audio components to use the new sample rate
        self.kick.set_sample_rate(sample_rate);
        self.clap.set_sample_rate(sample_rate);
        self.delay.set_sample_rate(sample_rate);
        self.reverb.set_sample_rate(sample_rate);
        self.delay_time_mod.set_sample_rate(sample_rate);
        self.reverb_size_mod.set_sample_rate(sample_rate);
        self.reverb_decay_mod.set_sample_rate(sample_rate);
    }

    pub fn set_kick_pattern(&mut self, pattern: [bool; 16]) {
        self.kick_pattern = pattern;
    }

    pub fn set_clap_pattern(&mut self, pattern: [bool; 16]) {
        self.clap_pattern = pattern;
    }

    pub fn next_sample(&mut self) -> (f32, f32) {
        // Handle kick drum with biased clock and step sequencing
        if let Some(step) = self.kick_clock.tick() {
            // Check if this is a new step and emit event
            if self.prev_kick_step.map_or(true, |prev| prev != step) {
                self.prev_kick_step = Some(step);
                self.send_event(AudioEvent::KickStepChanged(step));
                self.emit_modulator_values();
            }

            if self.kick_pattern[step as usize] {
                self.kick.trigger();
            }
        }

        // Handle clap drum with biased clock and step sequencing
        if let Some(step) = self.clap_clock.tick() {
            // Check if this is a new step and emit event
            if self.prev_clap_step.map_or(true, |prev| prev != step) {
                self.prev_clap_step = Some(step);
                self.send_event(AudioEvent::ClapStepChanged(step));
            }

            if self.clap_pattern[step as usize] {
                self.clap.trigger();
            }
        }

        // Update modulators
        let modulated_delay_time = self.delay_time_mod.next_sample();
        let modulated_reverb_size = self.reverb_size_mod.next_sample();
        let modulated_reverb_decay = self.reverb_decay_mod.next_sample();

        // Apply modulated parameters
        self.reverb.set_size(modulated_reverb_size);
        self.reverb.set_feedback(modulated_reverb_decay);

        // Generate dry drum samples
        let kick_sample = self.kick.next_sample();
        let clap_sample = self.clap.next_sample();
        let dry_mixed = kick_sample + clap_sample;

        // Create sends to effects
        let delay_input = dry_mixed * self.delay_send;
        let reverb_input = dry_mixed * self.reverb_send;

        // Process through effects with modulated delay time
        self.delay.set_delay_seconds(modulated_delay_time);
        self.delay.set_feedback(0.9);
        let delay_output = self.delay.process(delay_input);
        let (reverb_output_l, reverb_output_r) =
            self.reverb.process_stereo(reverb_input, reverb_input);

        // Mix dry and wet signals with proper level management
        // Scale dry signal down since we're adding effects
        let dry_level = 0.6; // Leave headroom for effects
        let wet_level = 0.4; // Effects contribution

        let output_l = dry_mixed * dry_level + (delay_output + reverb_output_l) * wet_level;
        let output_r = dry_mixed * dry_level + (delay_output + reverb_output_r) * wet_level;

        (output_l, output_r)
    }

    /// Process a block of samples (more efficient than sample-by-sample)
    /// Returns stereo interleaved samples [L, R, L, R, ...]
    pub fn process_block(&mut self, block_size: usize) -> Vec<f32> {
        let mut output = Vec::with_capacity(block_size * 2);

        for _ in 0..block_size {
            let (left, right) = self.next_sample();
            output.push(left);
            output.push(right);
        }

        output
    }

    /// Apply a single command to the drum machine
    pub fn apply_command(&mut self, command: AudioCommand) {
        match command {
            AudioCommand::SetBpm(bpm) => self.set_bpm(bpm),
            AudioCommand::SetKickPattern(pattern) => self.set_kick_pattern(pattern),
            AudioCommand::SetClapPattern(pattern) => self.set_clap_pattern(pattern),
            AudioCommand::SetKickAmpAttack(time) => self.kick.set_amp_attack(time),
            AudioCommand::SetKickAmpRelease(time) => self.kick.set_amp_release(time),
            AudioCommand::SetDelaySend(send) => self.set_delay_send(send),
            AudioCommand::SetReverbSend(send) => self.set_reverb_send(send),
            AudioCommand::SetDelayFreeze(freeze) => self.set_delay_freeze(freeze),
            AudioCommand::SetDelayHighpass(freq) => self.set_delay_highpass(freq),
            AudioCommand::SetDelayLowpass(freq) => self.set_delay_lowpass(freq),
            AudioCommand::SetReverbSize(size) => self.set_reverb_size(size),
            AudioCommand::SetReverbDecay(decay) => self.set_reverb_decay(decay),
            AudioCommand::SetClapDensity(density) => self.set_markov_density(density),
            AudioCommand::SetKickClockBias(bias) => self.set_kick_clock_bias(bias),
            AudioCommand::SetClapClockBias(bias) => self.set_clap_clock_bias(bias),
            AudioCommand::GenerateKickPattern => self.generate_kick_pattern(),
            AudioCommand::GenerateClapPattern => self.generate_clap_pattern(),
        }
    }

    pub fn get_kick(&mut self) -> &mut KickDrum {
        &mut self.kick
    }

    pub fn get_clap(&mut self) -> &mut ClapDrum {
        &mut self.clap
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
        self.reverb.set_feedback(decay);
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
        // Use kick clock as the main step reference
        self.kick_clock.get_current_step()
    }

    fn emit_modulator_values(&self) {
        let delay_time = self.get_current_delay_time();
        let reverb_size = self.get_current_reverb_size();
        let reverb_decay = self.get_current_reverb_decay();
        self.send_event(AudioEvent::ModulatorValues(
            delay_time,
            reverb_size,
            reverb_decay,
        ));
    }

    // Markov generation controls
    pub fn set_markov_density(&mut self, density: f32) {
        self.markov_generator.set_density(density);
    }

    pub fn generate_kick_pattern(&mut self) {
        // Generate new kick pattern using Markov chain
        self.kick_pattern = self
            .markov_generator
            .generate_sequence(16)
            .try_into()
            .unwrap();

        // Send event to UI
        self.send_event(AudioEvent::KickPatternGenerated(self.kick_pattern));
    }

    pub fn generate_clap_pattern(&mut self) {
        // Generate new clap pattern using Markov chain
        self.clap_pattern = self
            .markov_generator
            .generate_sequence(16)
            .try_into()
            .unwrap();

        // Send event to UI
        self.send_event(AudioEvent::ClapPatternGenerated(self.clap_pattern));
    }

    fn send_event(&self, event: AudioEvent) {
        self.event_sender.send(event);
    }

    pub fn get_kick_pattern(&self) -> [bool; 16] {
        self.kick_pattern
    }

    pub fn get_clap_pattern(&self) -> [bool; 16] {
        self.clap_pattern
    }

    // Clock bias controls
    pub fn set_kick_clock_bias(&mut self, bias: f32) {
        self.kick_clock.set_bias(bias);
    }

    pub fn set_clap_clock_bias(&mut self, bias: f32) {
        self.clap_clock.set_bias(bias);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_output_one_bar() {
        let sample_rate = 44100.0;
        // Create a mock event queue for testing
        let event_queue = crate::events::AudioEventQueue::new();
        let event_sender = event_queue.sender();
        let mut drum_machine = DrumMachine::new(sample_rate, event_sender);
        drum_machine.set_bpm(120.0);

        // Calculate samples for one bar (16 steps at 120 BPM)
        let beats_per_second = 120.0 / 60.0;
        let steps_per_second = beats_per_second * 4.0; // 16 steps per 4 beats
        let samples_per_step = sample_rate / steps_per_second;
        let total_samples = (samples_per_step * 16.0) as usize;

        println!("Testing drum machine audio output for one bar");
        println!(
            "BPM: 120, Sample rate: {}, Samples per step: {:.0}, Total samples: {}",
            sample_rate, samples_per_step, total_samples
        );

        let mut max_amplitude = 0.0f32;
        let mut rms_sum = 0.0f32;
        let mut step_triggers = Vec::new();

        for i in 0..total_samples {
            let (left, right) = drum_machine.next_sample();

            // Track maximum amplitude
            let amplitude = left.abs().max(right.abs());
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
