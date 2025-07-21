use crate::audio::delays::FilteredDelayLine;
use crate::audio::instruments::{ClapDrum, KickDrum};
use crate::audio::modulators::SampleAndHold;
use crate::audio::reverbs::DownsampledReverbLite;
use crate::audio::{AudioNode, AudioSystem};
use crate::events::{ServerEvent, ServerEventSender};
use crate::sequencing::{BiasedLoop, Clock, MarkovChain};

// Calculate the number of samples for 4 beats based on BPM and sample rate
fn bpm_to_samples(bpm: f32, sample_rate: f32) -> u32 {
    (60.0 / bpm * sample_rate) as u32 * 4
}

pub struct DrumMachineSystem {
    // Audio nodes
    kick: KickDrum,
    clap: ClapDrum,
    delay: FilteredDelayLine,
    reverb: DownsampledReverbLite,

    // Sequencer
    clock: Clock,
    kick_loop: BiasedLoop,
    clap_loop: BiasedLoop,
    kick_pattern: [bool; 16],
    clap_pattern: [bool; 16],

    // Markov chain for generating patterns
    markov_generator: MarkovChain,

    // Event sender for communicating with UI
    event_sender: ServerEventSender,

    // Track previous steps for event emission
    prev_kick_step: Option<u8>,
    prev_clap_step: Option<u8>,

    // Effects sends and returns
    delay_send: f32,
    reverb_send: f32,
    delay_return: f32,
    reverb_return: f32,

    // Sample and hold modulators
    delay_time_mod: SampleAndHold,
    reverb_size_mod: SampleAndHold,
    reverb_decay_mod: SampleAndHold,
    sample_rate: f32,

    // Pause state
    is_paused: bool,
}

impl DrumMachineSystem {
    pub fn new(sample_rate: f32, event_sender: ServerEventSender) -> Self {
        // Initialize clocks and Markov generator
        let total_samples_in_loop = bpm_to_samples(120.0, sample_rate);
        let clock = Clock::new();
        let kick_loop = BiasedLoop::new(total_samples_in_loop, 16, 0.5);
        let clap_loop = BiasedLoop::new(total_samples_in_loop, 16, 0.5);
        let markov_generator = MarkovChain::new(0.3); // 30% density

        Self {
            // Create audio nodes with default gains
            kick: KickDrum::new(sample_rate),
            clap: ClapDrum::new(sample_rate),
            delay: FilteredDelayLine::new(0.5, sample_rate), // 0.5 seconds max delay
            reverb: DownsampledReverbLite::new(sample_rate),

            clock,
            kick_loop,
            clap_loop,
            kick_pattern: [
                true, false, false, false, false, false, true, false, false, false, false, false,
                false, false, true, false,
            ],
            clap_pattern: [
                false, false, false, false, true, false, false, false, false, false, false, false,
                true, false, false, false,
            ],

            markov_generator,
            event_sender,
            prev_kick_step: None,
            prev_clap_step: None,

            // Default send and return levels
            delay_send: 0.2,
            reverb_send: 0.3,
            delay_return: 0.8,
            reverb_return: 0.6,

            sample_rate,

            // Initialize modulators with slower rates and configurable slew
            delay_time_mod: SampleAndHold::new(0.125, 0.1, 0.5, 150.0, sample_rate), // 8 sec updates, 150ms slew
            reverb_size_mod: SampleAndHold::new(0.165, 0.5, 1.5, 200.0, sample_rate), // 6 sec updates, 200ms slew
            reverb_decay_mod: SampleAndHold::new(0.1, 0.5, 0.95, 100.0, sample_rate), // 10 sec updates, 100ms slew

            // Initialize as paused
            is_paused: true,
        }
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        let total_samples_in_loop = bpm_to_samples(bpm, self.sample_rate);
        self.kick_loop.set_total_samples(total_samples_in_loop);
        self.clap_loop.set_total_samples(total_samples_in_loop);
    }

    pub fn set_kick_pattern(&mut self, pattern: [bool; 16]) {
        self.kick_pattern = pattern;
    }

    pub fn set_clap_pattern(&mut self, pattern: [bool; 16]) {
        self.clap_pattern = pattern;
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.is_paused = paused;
    }

    // Pattern generation methods
    pub fn generate_kick_pattern(&mut self) {
        self.kick_pattern = self
            .markov_generator
            .generate_sequence(16)
            .try_into()
            .unwrap();

        self.send_event(ServerEvent::KickPatternGenerated(self.kick_pattern));
    }

    pub fn generate_clap_pattern(&mut self) {
        self.clap_pattern = self
            .markov_generator
            .generate_sequence(16)
            .try_into()
            .unwrap();

        self.send_event(ServerEvent::ClapPatternGenerated(self.clap_pattern));
    }

    pub fn set_markov_density(&mut self, density: f32) {
        self.markov_generator.set_density(density);
    }

    pub fn set_kick_loop_bias(&mut self, bias: f32) {
        self.kick_loop.set_bias(bias);
    }

    pub fn set_clap_loop_bias(&mut self, bias: f32) {
        self.clap_loop.set_bias(bias);
    }
    
    pub fn set_delay_send(&mut self, send: f32) {
        self.delay_send = send.clamp(0.0, 1.0);
    }
    
    pub fn set_reverb_send(&mut self, send: f32) {
        self.reverb_send = send.clamp(0.0, 1.0);
    }
    
    pub fn set_delay_return(&mut self, return_level: f32) {
        self.delay_return = return_level.clamp(0.0, 1.0);
    }
    
    pub fn set_reverb_return(&mut self, return_level: f32) {
        self.reverb_return = return_level.clamp(0.0, 1.0);
    }

    fn send_event(&self, event: ServerEvent) {
        self.event_sender.send(event);
    }

    fn emit_modulator_values(&self) {
        let delay_time = self.delay_time_mod.get_current_value();
        let reverb_size = self.reverb_size_mod.get_current_value();
        let reverb_decay = self.reverb_decay_mod.get_current_value();
        self.send_event(ServerEvent::ModulatorValues(
            delay_time,
            reverb_size,
            reverb_decay,
        ));
    }
}

impl AudioSystem for DrumMachineSystem {
    fn handle_node_event(
        &mut self,
        node_name: crate::events::NodeName,
        event: crate::events::NodeEvent,
    ) -> Result<(), String> {
        use crate::events::{NodeName, NodeEvent};
        match node_name {
            NodeName::Kick => self.kick.handle_event(event),
            NodeName::Clap => self.clap.handle_event(event),
            NodeName::Delay => self.delay.handle_event(event),
            NodeName::Reverb => self.reverb.handle_event(event),
            NodeName::System => {
                // Handle system events
                match event {
                    NodeEvent::SetBpm(bpm) => {
                        self.set_bpm(bpm);
                        Ok(())
                    }
                    NodeEvent::SetPaused(paused) => {
                        self.set_paused(paused);
                        Ok(())
                    }
                    NodeEvent::SetMarkovDensity(density) => {
                        self.set_markov_density(density);
                        Ok(())
                    }
                    NodeEvent::SetKickLoopBias(bias) => {
                        self.set_kick_loop_bias(bias);
                        Ok(())
                    }
                    NodeEvent::SetClapLoopBias(bias) => {
                        self.set_clap_loop_bias(bias);
                        Ok(())
                    }
                    NodeEvent::GenerateKickPattern => {
                        self.generate_kick_pattern();
                        Ok(())
                    }
                    NodeEvent::GenerateClapPattern => {
                        self.generate_clap_pattern();
                        Ok(())
                    }
                    NodeEvent::SetDelaySend(send) => {
                        self.set_delay_send(send);
                        Ok(())
                    }
                    NodeEvent::SetReverbSend(send) => {
                        self.set_reverb_send(send);
                        Ok(())
                    }
                    NodeEvent::SetDelayReturn(return_level) => {
                        self.set_delay_return(return_level);
                        Ok(())
                    }
                    NodeEvent::SetReverbReturn(return_level) => {
                        self.set_reverb_return(return_level);
                        Ok(())
                    }
                    _ => Err(format!("Unsupported system event: {:?}", event))
                }
            }
        }
    }


    fn next_sample(&mut self) -> (f32, f32) {
        // Only run sequencer when not paused
        if !self.is_paused {
            self.clock.tick();

            // Handle kick drum with biased clock and step sequencing
            if let Some(step) = self.kick_loop.tick(&self.clock) {
                // Check if this is a new step and emit event
                if self.prev_kick_step.map_or(true, |prev| prev != step) {
                    self.prev_kick_step = Some(step);
                    self.send_event(ServerEvent::KickStepChanged(step));
                    self.emit_modulator_values();
                }

                if self.kick_pattern[step as usize] {
                    self.kick
                        .handle_event(crate::events::NodeEvent::Trigger)
                        .ok();
                }
            }

            // Handle clap drum with biased clock and step sequencing
            if let Some(step) = self.clap_loop.tick(&self.clock) {
                // Check if this is a new step and emit event
                if self.prev_clap_step.map_or(true, |prev| prev != step) {
                    self.prev_clap_step = Some(step);
                    self.send_event(ServerEvent::ClapStepChanged(step));
                }

                if self.clap_pattern[step as usize] {
                    self.clap
                        .handle_event(crate::events::NodeEvent::Trigger)
                        .ok();
                }
            }
        }

        // Update modulators
        let modulated_delay_time = self.delay_time_mod.next_sample();
        let modulated_reverb_size = self.reverb_size_mod.next_sample();
        let modulated_reverb_decay = self.reverb_decay_mod.next_sample();

        // Apply modulated parameters
        self.reverb
            .handle_event(crate::events::NodeEvent::SetSize(modulated_reverb_size))
            .ok();
        self.reverb
            .handle_event(crate::events::NodeEvent::SetFeedback(
                modulated_reverb_decay,
            ))
            .ok();
        self.delay
            .handle_event(crate::events::NodeEvent::SetDelaySeconds(
                modulated_delay_time,
            ))
            .ok();
        self.delay
            .handle_event(crate::events::NodeEvent::SetFeedback(0.9))
            .ok();

        // Process through audio node chain
        // Start with silence (no input signal)
        let mut signal = (0.0, 0.0);

        // Add instruments (dry signal)
        signal = self.kick.process(signal.0, signal.1);
        signal = self.clap.process(signal.0, signal.1);

        // Send to delay first
        let delay_input = (signal.0 * self.delay_send, signal.1 * self.delay_send);
        let delay_output = self.delay.process(delay_input.0, delay_input.1);
        
        // Create post-delay signal (dry + delay return)
        let post_delay_signal = (
            signal.0 + delay_output.0 * self.delay_return,
            signal.1 + delay_output.1 * self.delay_return,
        );

        // Send post-delay signal (dry + delay) to reverb
        let reverb_input = (post_delay_signal.0 * self.reverb_send, post_delay_signal.1 * self.reverb_send);
        let reverb_output = self.reverb.process(reverb_input.0, reverb_input.1);

        // Final mix: post-delay signal + reverb return
        (
            post_delay_signal.0 + reverb_output.0 * self.reverb_return,
            post_delay_signal.1 + reverb_output.1 * self.reverb_return,
        )
    }
    

    fn set_sequence(&mut self, sequence_config: &serde_json::Value) -> Result<(), String> {
        // Parse sequence configuration from JSON
        if let Some(kick_pattern) = sequence_config.get("kick_pattern") {
            if let Some(pattern_array) = kick_pattern.as_array() {
                if pattern_array.len() == 16 {
                    let mut new_pattern = [false; 16];
                    for (i, value) in pattern_array.iter().enumerate() {
                        if let Some(bool_val) = value.as_bool() {
                            new_pattern[i] = bool_val;
                        } else {
                            return Err("Invalid kick pattern: non-boolean value".to_string());
                        }
                    }
                    self.set_kick_pattern(new_pattern);
                } else {
                    return Err("Invalid kick pattern: must be 16 steps".to_string());
                }
            } else {
                return Err("Invalid kick pattern: must be an array".to_string());
            }
        }

        if let Some(clap_pattern) = sequence_config.get("clap_pattern") {
            if let Some(pattern_array) = clap_pattern.as_array() {
                if pattern_array.len() == 16 {
                    let mut new_pattern = [false; 16];
                    for (i, value) in pattern_array.iter().enumerate() {
                        if let Some(bool_val) = value.as_bool() {
                            new_pattern[i] = bool_val;
                        } else {
                            return Err("Invalid clap pattern: non-boolean value".to_string());
                        }
                    }
                    self.set_clap_pattern(new_pattern);
                } else {
                    return Err("Invalid clap pattern: must be 16 steps".to_string());
                }
            } else {
                return Err("Invalid clap pattern: must be an array".to_string());
            }
        }

        Ok(())
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.kick.set_sample_rate(sample_rate);
        self.clap.set_sample_rate(sample_rate);
        self.delay.set_sample_rate(sample_rate);
        self.reverb.set_sample_rate(sample_rate);
        self.delay_time_mod.set_sample_rate(sample_rate);
        self.reverb_size_mod.set_sample_rate(sample_rate);
        self.reverb_decay_mod.set_sample_rate(sample_rate);
    }
}
