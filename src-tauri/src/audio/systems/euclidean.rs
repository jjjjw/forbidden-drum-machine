use crate::audio::instruments::{ChordSynth, ClapDrum, HiHat, KickDrum};
use crate::audio::{AudioGenerator, AudioNode, AudioSystem};
use crate::events::{NodeEvent, NodeName, ServerEvent, ServerEventSender};
use crate::sequencing::{Clock, EuclideanSequencer, Loop};

// Calculate the number of samples for 4 beats based on BPM and sample rate
fn bpm_to_samples(bpm: f32, sample_rate: f32) -> u32 {
    (60.0 / bpm * sample_rate) as u32 * 4
}

pub struct EuclideanSystem {
    // Audio nodes
    kick: KickDrum,
    clap: ClapDrum,
    hihat: HiHat,
    chord: ChordSynth,

    // Main clock and sequencer loop
    clock: Clock,
    main_loop: Loop,
    
    // Euclidean sequencers for each instrument
    kick_sequencer: EuclideanSequencer,
    clap_sequencer: EuclideanSequencer,
    hihat_sequencer: EuclideanSequencer,
    chord_sequencer: EuclideanSequencer,

    // Event sender for communicating with UI
    event_sender: ServerEventSender,

    // Track previous steps for event emission
    prev_kick_step: Option<u32>,
    prev_clap_step: Option<u32>,
    prev_hihat_step: Option<u32>,
    prev_chord_step: Option<u32>,

    sample_rate: f32,
    
    // System parameters
    bpm: f32,
    is_paused: bool,
}

impl EuclideanSystem {
    pub fn new(sample_rate: f32, event_sender: ServerEventSender) -> Self {
        let clock = Clock::new();
        let total_samples = bpm_to_samples(120.0, sample_rate);
        let main_loop = Loop::new(total_samples, 16); // 16 steps per 4 beats

        Self {
            // Create audio nodes
            kick: KickDrum::new(sample_rate),
            clap: ClapDrum::new(sample_rate),
            hihat: HiHat::new(sample_rate),
            chord: ChordSynth::new(sample_rate),

            clock,
            main_loop,

            // Initialize Euclidean sequencers with default patterns
            kick_sequencer: EuclideanSequencer::new(8, 3, 1.0),      // Classic 3/8 kick pattern
            clap_sequencer: EuclideanSequencer::new(8, 2, 1.0),     // Simple 2/8 clap pattern
            hihat_sequencer: EuclideanSequencer::new(16, 7, 2.0),   // Busy hihat pattern, double tempo
            chord_sequencer: EuclideanSequencer::new(8, 1, 0.5),    // Sparse chord pattern, half tempo

            event_sender,
            
            prev_kick_step: None,
            prev_clap_step: None,
            prev_hihat_step: None,
            prev_chord_step: None,

            sample_rate,
            bpm: 120.0,
            is_paused: false,
        }
    }

    fn update_clock_for_bpm(&mut self) {
        let total_samples = bpm_to_samples(self.bpm, self.sample_rate);
        self.main_loop.set_total_samples(total_samples);
    }
}

impl AudioSystem for EuclideanSystem {
    fn next_sample(&mut self) -> (f32, f32) {
        if self.is_paused {
            return (0.0, 0.0);
        }

        // Tick the main clock
        self.clock.tick();
        
        // Check if we've hit a step boundary (every 1/16th note)
        if let Some(_step) = self.main_loop.tick(&self.clock) {
            // Process each sequencer
            
            // Kick
            if self.kick_sequencer.tick() {
                self.kick.handle_event(NodeEvent::Trigger).ok();
            }
            let kick_step = self.kick_sequencer.get_current_step();
            if self.prev_kick_step != Some(kick_step) {
                self.event_sender.send(ServerEvent::KickStepChanged(kick_step as u8));
                self.prev_kick_step = Some(kick_step);
            }

            // Clap
            if self.clap_sequencer.tick() {
                self.clap.handle_event(NodeEvent::Trigger).ok();
            }
            let clap_step = self.clap_sequencer.get_current_step();
            if self.prev_clap_step != Some(clap_step) {
                self.event_sender.send(ServerEvent::ClapStepChanged(clap_step as u8));
                self.prev_clap_step = Some(clap_step);
            }

            // HiHat
            if self.hihat_sequencer.tick() {
                self.hihat.handle_event(NodeEvent::Trigger).ok();
            }
            let hihat_step = self.hihat_sequencer.get_current_step();
            if self.prev_hihat_step != Some(hihat_step) {
                self.prev_hihat_step = Some(hihat_step);
            }

            // Chord
            if self.chord_sequencer.tick() {
                self.chord.handle_event(NodeEvent::Trigger).ok();
            }
            let chord_step = self.chord_sequencer.get_current_step();
            if self.prev_chord_step != Some(chord_step) {
                self.prev_chord_step = Some(chord_step);
            }
        }

        // Generate audio from instruments (using stereo process method)
        let (kick_left, kick_right) = self.kick.process(0.0, 0.0);
        let (clap_left, clap_right) = self.clap.process(0.0, 0.0);
        let (hihat_left, hihat_right) = self.hihat.process(0.0, 0.0);
        let (chord_left, chord_right) = self.chord.process(0.0, 0.0);

        // Mix signals
        let final_left = kick_left + clap_left + hihat_left + chord_left;
        let final_right = kick_right + clap_right + hihat_right + chord_right;

        (final_left * 0.7, final_right * 0.7) // Master volume
    }

    fn handle_node_event(&mut self, node_name: NodeName, event: NodeEvent) -> Result<(), String> {
        match node_name {
            NodeName::Kick => self.kick.handle_event(event),
            NodeName::Clap => self.clap.handle_event(event),
            NodeName::HiHat => self.hihat.handle_event(event),
            NodeName::Chord => self.chord.handle_event(event),
            NodeName::System => {
                match event {
                    NodeEvent::SetBpm(bpm) => {
                        self.bpm = bpm.max(30.0).min(300.0);
                        self.update_clock_for_bpm();
                        Ok(())
                    }
                    NodeEvent::SetPaused(paused) => {
                        self.is_paused = paused;
                        Ok(())
                    }
                    _ => Err(format!("Unsupported system event: {:?}", event)),
                }
            }
            _ => Err(format!("Unsupported node: {:?}", node_name)),
        }
    }

    fn set_sequence(&mut self, sequence: &serde_json::Value) -> Result<(), String> {
        // Expected JSON format:
        // {
        //   "kick": { "steps": 8, "beats": 3, "tempo_mult": 1.0 },
        //   "clap": { "steps": 8, "beats": 2, "tempo_mult": 1.0 },
        //   "hihat": { "steps": 16, "beats": 7, "tempo_mult": 2.0 },
        //   "chord": { "steps": 8, "beats": 1, "tempo_mult": 0.5 }
        // }
        
        if let Some(kick_config) = sequence.get("kick") {
            if let (Some(steps), Some(beats), Some(tempo_mult)) = (
                kick_config.get("steps").and_then(|v| v.as_u64()),
                kick_config.get("beats").and_then(|v| v.as_u64()),
                kick_config.get("tempo_mult").and_then(|v| v.as_f64()),
            ) {
                self.kick_sequencer.set_steps(steps as u32);
                self.kick_sequencer.set_beats(beats as u32);
                self.kick_sequencer.set_tempo_multiplier(tempo_mult as f32);
            }
        }

        if let Some(clap_config) = sequence.get("clap") {
            if let (Some(steps), Some(beats), Some(tempo_mult)) = (
                clap_config.get("steps").and_then(|v| v.as_u64()),
                clap_config.get("beats").and_then(|v| v.as_u64()),
                clap_config.get("tempo_mult").and_then(|v| v.as_f64()),
            ) {
                self.clap_sequencer.set_steps(steps as u32);
                self.clap_sequencer.set_beats(beats as u32);
                self.clap_sequencer.set_tempo_multiplier(tempo_mult as f32);
            }
        }

        if let Some(hihat_config) = sequence.get("hihat") {
            if let (Some(steps), Some(beats), Some(tempo_mult)) = (
                hihat_config.get("steps").and_then(|v| v.as_u64()),
                hihat_config.get("beats").and_then(|v| v.as_u64()),
                hihat_config.get("tempo_mult").and_then(|v| v.as_f64()),
            ) {
                self.hihat_sequencer.set_steps(steps as u32);
                self.hihat_sequencer.set_beats(beats as u32);
                self.hihat_sequencer.set_tempo_multiplier(tempo_mult as f32);
            }
        }

        if let Some(chord_config) = sequence.get("chord") {
            if let (Some(steps), Some(beats), Some(tempo_mult)) = (
                chord_config.get("steps").and_then(|v| v.as_u64()),
                chord_config.get("beats").and_then(|v| v.as_u64()),
                chord_config.get("tempo_mult").and_then(|v| v.as_f64()),
            ) {
                self.chord_sequencer.set_steps(steps as u32);
                self.chord_sequencer.set_beats(beats as u32);
                self.chord_sequencer.set_tempo_multiplier(tempo_mult as f32);
            }
        }

        Ok(())
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        
        // Update all audio nodes
        AudioNode::set_sample_rate(&mut self.kick, sample_rate);
        AudioNode::set_sample_rate(&mut self.clap, sample_rate);
        AudioNode::set_sample_rate(&mut self.hihat, sample_rate);
        AudioNode::set_sample_rate(&mut self.chord, sample_rate);
        
        // Update clock
        self.update_clock_for_bpm();
    }
}