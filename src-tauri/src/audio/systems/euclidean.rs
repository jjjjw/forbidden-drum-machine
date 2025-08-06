// use crate::audio::instruments::{ChordSynth, ClapDrum, HiHat, KickDrum};
// use crate::audio::{AudioGenerator, AudioSystem};
// use crate::events::{ServerEvent, ServerEventSender};
// use crate::sequencing::{Clock, EuclideanSequencer, Loop};

// // Calculate the number of samples for 4 beats based on BPM and sample rate
// fn bpm_to_samples(bpm: f32, sample_rate: f32) -> u32 {
//     (60.0 / bpm * sample_rate) as u32 * 4
// }

// pub struct EuclideanSystem {
//     // Audio nodes
//     kick: KickDrum,
//     clap: ClapDrum,
//     hihat: HiHat,
//     chord: ChordSynth,

//     // Main clock and sequencer loop
//     clock: Clock,
//     main_loop: Loop,

//     // Euclidean sequencers for each instrument
//     kick_sequencer: EuclideanSequencer,
//     clap_sequencer: EuclideanSequencer,
//     hihat_sequencer: EuclideanSequencer,
//     chord_sequencer: EuclideanSequencer,

//     // Event sender for communicating with UI
//     event_sender: ServerEventSender,

//     // Track previous steps for event emission
//     prev_kick_step: Option<u32>,
//     prev_clap_step: Option<u32>,
//     prev_hihat_step: Option<u32>,
//     prev_chord_step: Option<u32>,

//     sample_rate: f32,

//     // System parameters
//     bpm: f32,
//     is_paused: bool,
// }

// impl EuclideanSystem {
//     pub fn new(sample_rate: f32, event_sender: ServerEventSender) -> Self {
//         let clock = Clock::new();
//         let total_samples = bpm_to_samples(120.0, sample_rate);
//         let main_loop = Loop::new(total_samples, 16); // 16 steps per 4 beats

//         Self {
//             // Create audio nodes
//             kick: KickDrum::new(sample_rate),
//             clap: ClapDrum::new(sample_rate),
//             hihat: HiHat::new(sample_rate),
//             chord: ChordSynth::new(sample_rate),

//             clock,
//             main_loop,

//             // Initialize Euclidean sequencers with default patterns
//             kick_sequencer: EuclideanSequencer::new(8, 3, 1.0),      // Classic 3/8 kick pattern
//             clap_sequencer: EuclideanSequencer::new(8, 2, 1.0),     // Simple 2/8 clap pattern
//             hihat_sequencer: EuclideanSequencer::new(16, 7, 2.0),   // Busy hihat pattern, double tempo
//             chord_sequencer: EuclideanSequencer::new(8, 1, 0.5),    // Sparse chord pattern, half tempo

//             event_sender,

//             prev_kick_step: None,
//             prev_clap_step: None,
//             prev_hihat_step: None,
//             prev_chord_step: None,

//             sample_rate,
//             bpm: 120.0,
//             is_paused: false,
//         }
//     }

//     fn update_clock_for_bpm(&mut self) {
//         let total_samples = bpm_to_samples(self.bpm, self.sample_rate);
//         self.main_loop.set_total_samples(total_samples);
//     }

//     fn handle_kick_event(&mut self, event: &crate::events::ClientEvent) -> Result<(), String> {
//         match event.event.as_str() {
//             "trigger" => {
//                 self.kick.trigger();
//                 Ok(())
//             }
//             "set_gain" => {
//                 self.kick.set_gain(event.parameter);
//                 Ok(())
//             }
//             "set_base_frequency" => {
//                 self.kick.set_base_frequency(event.parameter);
//                 Ok(())
//             }
//             "set_frequency_ratio" => {
//                 self.kick.set_frequency_ratio(event.parameter);
//                 Ok(())
//             }
//             "set_modulation_index" => {
//                 self.kick.set_modulation_index(event.parameter);
//                 Ok(())
//             }
//             "set_amp_attack" => {
//                 self.kick.set_amp_attack(event.parameter);
//                 Ok(())
//             }
//             "set_amp_release" => {
//                 self.kick.set_amp_release(event.parameter);
//                 Ok(())
//             }
//             "set_freq_attack" => {
//                 self.kick.set_freq_attack(event.parameter);
//                 Ok(())
//             }
//             "set_freq_release" => {
//                 self.kick.set_freq_release(event.parameter);
//                 Ok(())
//             }
//             _ => Err(format!("Unknown kick event: {}", event.event)),
//         }
//     }

//     fn handle_clap_event(&mut self, event: &crate::events::ClientEvent) -> Result<(), String> {
//         match event.event.as_str() {
//             "trigger" => {
//                 self.clap.trigger();
//                 Ok(())
//             }
//             "set_gain" => {
//                 self.clap.set_gain(event.parameter);
//                 Ok(())
//             }
//             "set_base_frequency" => {
//                 self.clap.set_base_frequency(event.parameter);
//                 Ok(())
//             }
//             "set_frequency_ratio" => {
//                 self.clap.set_frequency_ratio(event.parameter);
//                 Ok(())
//             }
//             "set_modulation_index" => {
//                 self.clap.set_modulation_index(event.parameter);
//                 Ok(())
//             }
//             "set_amp_attack" => {
//                 self.clap.set_amp_attack(event.parameter);
//                 Ok(())
//             }
//             "set_amp_release" => {
//                 self.clap.set_amp_release(event.parameter);
//                 Ok(())
//             }
//             "set_freq_attack" => {
//                 self.clap.set_freq_attack(event.parameter);
//                 Ok(())
//             }
//             "set_freq_release" => {
//                 self.clap.set_freq_release(event.parameter);
//                 Ok(())
//             }
//             _ => Err(format!("Unknown clap event: {}", event.event)),
//         }
//     }

//     fn handle_hihat_event(&mut self, event: &crate::events::ClientEvent) -> Result<(), String> {
//         match event.event.as_str() {
//             "trigger" => {
//                 self.hihat.trigger();
//                 Ok(())
//             }
//             "set_gain" => {
//                 self.hihat.set_gain(event.parameter);
//                 Ok(())
//             }
//             "set_base_frequency" => {
//                 self.hihat.set_base_frequency(event.parameter);
//                 Ok(())
//             }
//             "set_frequency_ratio" => {
//                 self.hihat.set_frequency_ratio(event.parameter);
//                 Ok(())
//             }
//             "set_modulation_index" => {
//                 self.hihat.set_modulation_index(event.parameter);
//                 Ok(())
//             }
//             "set_amp_attack" => {
//                 self.hihat.set_amp_attack(event.parameter);
//                 Ok(())
//             }
//             "set_amp_release" => {
//                 self.hihat.set_amp_release(event.parameter);
//                 Ok(())
//             }
//             "set_freq_attack" => {
//                 self.hihat.set_freq_attack(event.parameter);
//                 Ok(())
//             }
//             "set_freq_release" => {
//                 self.hihat.set_freq_release(event.parameter);
//                 Ok(())
//             }
//             _ => Err(format!("Unknown hihat event: {}", event.event)),
//         }
//     }

//     fn handle_chord_event(&mut self, event: &crate::events::ClientEvent) -> Result<(), String> {
//         match event.event.as_str() {
//             "trigger" => {
//                 self.chord.trigger();
//                 Ok(())
//             }
//             "set_gain" => {
//                 self.chord.set_gain(event.parameter);
//                 Ok(())
//             }
//             "set_base_frequency" => {
//                 self.chord.set_base_frequency(event.parameter);
//                 Ok(())
//             }
//             "set_frequency_ratio" => {
//                 self.chord.set_frequency_ratio(event.parameter);
//                 Ok(())
//             }
//             "set_modulation_index" => {
//                 self.chord.set_modulation_index(event.parameter);
//                 Ok(())
//             }
//             "set_amp_attack" => {
//                 self.chord.set_amp_attack(event.parameter);
//                 Ok(())
//             }
//             "set_amp_release" => {
//                 self.chord.set_amp_release(event.parameter);
//                 Ok(())
//             }
//             "set_freq_attack" => {
//                 self.chord.set_freq_attack(event.parameter);
//                 Ok(())
//             }
//             "set_freq_release" => {
//                 self.chord.set_freq_release(event.parameter);
//                 Ok(())
//             }
//             _ => Err(format!("Unknown chord event: {}", event.event)),
//         }
//     }

//     fn handle_system_event(&mut self, event: &crate::events::ClientEvent) -> Result<(), String> {
//         match event.event.as_str() {
//             "set_bpm" => {
//                 self.bpm = event.parameter.max(30.0).min(300.0);
//                 self.update_clock_for_bpm();
//                 Ok(())
//             }
//             "set_paused" => {
//                 self.is_paused = event.as_bool();
//                 Ok(())
//             }
//             "set_kick_steps" => {
//                 self.kick_sequencer.set_steps(event.parameter as u32);
//                 Ok(())
//             }
//             "set_kick_beats" => {
//                 self.kick_sequencer.set_beats(event.parameter as u32);
//                 Ok(())
//             }
//             "set_kick_tempo_mult" => {
//                 self.kick_sequencer.set_tempo_multiplier(event.parameter);
//                 Ok(())
//             }
//             "set_clap_steps" => {
//                 self.clap_sequencer.set_steps(event.parameter as u32);
//                 Ok(())
//             }
//             "set_clap_beats" => {
//                 self.clap_sequencer.set_beats(event.parameter as u32);
//                 Ok(())
//             }
//             "set_clap_tempo_mult" => {
//                 self.clap_sequencer.set_tempo_multiplier(event.parameter);
//                 Ok(())
//             }
//             "set_hihat_steps" => {
//                 self.hihat_sequencer.set_steps(event.parameter as u32);
//                 Ok(())
//             }
//             "set_hihat_beats" => {
//                 self.hihat_sequencer.set_beats(event.parameter as u32);
//                 Ok(())
//             }
//             "set_hihat_tempo_mult" => {
//                 self.hihat_sequencer.set_tempo_multiplier(event.parameter);
//                 Ok(())
//             }
//             "set_chord_steps" => {
//                 self.chord_sequencer.set_steps(event.parameter as u32);
//                 Ok(())
//             }
//             "set_chord_beats" => {
//                 self.chord_sequencer.set_beats(event.parameter as u32);
//                 Ok(())
//             }
//             "set_chord_tempo_mult" => {
//                 self.chord_sequencer.set_tempo_multiplier(event.parameter);
//                 Ok(())
//             }
//             _ => Err(format!("Unknown system event: {}", event.event)),
//         }
//     }
// }

// impl AudioSystem for EuclideanSystem {
//     fn next_sample(&mut self) -> (f32, f32) {
//         if self.is_paused {
//             return (0.0, 0.0);
//         }

//         // Tick the main clock
//         self.clock.tick();

//         // Check if we've hit a step boundary (every 1/16th note)
//         if let Some(_step) = self.main_loop.tick(&self.clock) {
//             // Process each sequencer

//             // Kick
//             if self.kick_sequencer.tick() {
//                 self.kick.trigger();
//             }
//             let kick_step = self.kick_sequencer.get_current_step();
//             if self.prev_kick_step != Some(kick_step) {
//                 self.event_sender.send(ServerEvent::KickStepChanged(kick_step as u8));
//                 self.prev_kick_step = Some(kick_step);
//             }

//             // Clap
//             if self.clap_sequencer.tick() {
//                 self.clap.trigger();
//             }
//             let clap_step = self.clap_sequencer.get_current_step();
//             if self.prev_clap_step != Some(clap_step) {
//                 self.event_sender.send(ServerEvent::ClapStepChanged(clap_step as u8));
//                 self.prev_clap_step = Some(clap_step);
//             }

//             // HiHat
//             if self.hihat_sequencer.tick() {
//                 self.hihat.trigger();
//             }
//             let hihat_step = self.hihat_sequencer.get_current_step();
//             if self.prev_hihat_step != Some(hihat_step) {
//                 self.prev_hihat_step = Some(hihat_step);
//             }

//             // Chord
//             if self.chord_sequencer.tick() {
//                 self.chord.trigger();
//             }
//             let chord_step = self.chord_sequencer.get_current_step();
//             if self.prev_chord_step != Some(chord_step) {
//                 self.prev_chord_step = Some(chord_step);
//             }
//         }

//         // Generate audio from instruments (using stereo process method)
//         let (kick_left, kick_right) = self.kick.process(0.0, 0.0);
//         let (clap_left, clap_right) = self.clap.process(0.0, 0.0);
//         let (hihat_left, hihat_right) = self.hihat.process(0.0, 0.0);
//         let (chord_left, chord_right) = self.chord.process(0.0, 0.0);

//         // Mix signals
//         let final_left = kick_left + clap_left + hihat_left + chord_left;
//         let final_right = kick_right + clap_right + hihat_right + chord_right;

//         (final_left * 0.7, final_right * 0.7) // Master volume
//     }

//     fn handle_client_event(&mut self, event: &crate::events::ClientEvent) -> Result<(), String> {
//         match event.node.as_str() {
//             "kick" => self.handle_kick_event(event),
//             "clap" => self.handle_clap_event(event),
//             "hihat" => self.handle_hihat_event(event),
//             "chord" => self.handle_chord_event(event),
//             "system" => self.handle_system_event(event),
//             _ => Err(format!("Unknown node '{}' for euclidean system", event.node)),
//         }
//     }

//     fn set_sample_rate(&mut self, sample_rate: f32) {
//         self.sample_rate = sample_rate;

//         // Update all instruments
//         self.kick.set_sample_rate(sample_rate);
//         self.clap.set_sample_rate(sample_rate);
//         self.hihat.set_sample_rate(sample_rate);
//         self.chord.set_sample_rate(sample_rate);

//         // Update clock
//         self.update_clock_for_bpm();
//     }
// }
