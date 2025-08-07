use crate::audio::instruments::SupersawSynth;
use crate::audio::{AudioSystem, StereoAudioGenerator};

/// Musical scale definitions
#[derive(Clone, Copy)]
pub enum Scale {
    Major,
    Minor,
    Dorian,
    Phrygian,
    Mixolydian,
    Blues,
}

impl Scale {
    /// Get scale intervals in semitones from root
    pub fn intervals(self) -> &'static [i32] {
        match self {
            Scale::Major => &[0, 2, 4, 5, 7, 9, 11],
            Scale::Minor => &[0, 2, 3, 5, 7, 8, 10],
            Scale::Dorian => &[0, 2, 3, 5, 7, 9, 10],
            Scale::Phrygian => &[0, 1, 3, 5, 7, 8, 10],
            Scale::Mixolydian => &[0, 2, 4, 5, 7, 9, 10],
            Scale::Blues => &[0, 3, 5, 6, 7, 10],
        }
    }
}

/// Simple trance riff generator with predefined patterns
pub struct RiffGenerator {
    scale: Scale,
    root_note: f32, // Root frequency in Hz
    pattern: Vec<(usize, f32)>, // (scale_degree, duration_in_beats)
    pattern_index: usize,
    beat_sample_counter: f32,
    current_note_duration_samples: f32,
    bpm: f32,
    sample_rate: f32,
    samples_per_beat: f32,
}

impl RiffGenerator {
    pub fn new(scale: Scale, root_note: f32, bpm: f32, sample_rate: f32) -> Self {
        let samples_per_beat = (60.0 / bpm) * sample_rate;
        
        // Default trance riff pattern (classic uplifting trance)
        let pattern = vec![
            (0, 0.25), // Root, 1/16 note
            (2, 0.25), // Third, 1/16 note
            (4, 0.25), // Fifth, 1/16 note
            (2, 0.25), // Third, 1/16 note
            (0, 0.5),  // Root, 1/8 note
            (4, 0.25), // Fifth, 1/16 note
            (6, 0.25), // Seventh, 1/16 note
            (4, 0.5),  // Fifth, 1/8 note
        ];
        
        let current_note_duration_samples = pattern[0].1 * samples_per_beat;
        
        Self {
            scale,
            root_note,
            pattern,
            pattern_index: 0,
            beat_sample_counter: 0.0,
            current_note_duration_samples,
            bpm,
            sample_rate,
            samples_per_beat,
        }
    }
    
    pub fn set_scale(&mut self, scale: Scale) {
        self.scale = scale;
    }
    
    pub fn set_root_note(&mut self, root_note: f32) {
        self.root_note = root_note;
    }
    
    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm;
        self.samples_per_beat = (60.0 / bpm) * self.sample_rate;
        self.current_note_duration_samples = self.pattern[self.pattern_index].1 * self.samples_per_beat;
    }
    
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.samples_per_beat = (60.0 / self.bpm) * sample_rate;
        self.current_note_duration_samples = self.pattern[self.pattern_index].1 * self.samples_per_beat;
    }
    
    /// Returns (frequency, should_trigger) for the current step
    pub fn tick(&mut self) -> (f32, bool) {
        self.beat_sample_counter += 1.0;
        
        let mut should_trigger = false;
        
        // Check if we need to advance to the next note
        if self.beat_sample_counter >= self.current_note_duration_samples {
            self.beat_sample_counter = 0.0;
            self.pattern_index = (self.pattern_index + 1) % self.pattern.len();
            self.current_note_duration_samples = self.pattern[self.pattern_index].1 * self.samples_per_beat;
            should_trigger = true;
        }
        
        // Get current note
        let scale_degree = self.pattern[self.pattern_index].0;
        let intervals = self.scale.intervals();
        let semitone_offset = intervals[scale_degree % intervals.len()];
        let frequency = self.root_note * 2.0_f32.powf(semitone_offset as f32 / 12.0);
        
        (frequency, should_trigger)
    }
}

/// Main TranceRiff system
pub struct TranceRiffSystem {
    synth: SupersawSynth,
    riff_generator: RiffGenerator,
    bpm: f32,
    is_paused: bool,
    sample_rate: f32,
}

impl TranceRiffSystem {
    pub fn new(sample_rate: f32) -> Self {
        let bpm = 138.0; // Classic trance BPM
        let root_note = 220.0; // A3
        let scale = Scale::Minor;
        
        Self {
            synth: SupersawSynth::new(sample_rate),
            riff_generator: RiffGenerator::new(scale, root_note, bpm, sample_rate),
            bpm,
            is_paused: false,
            sample_rate,
        }
    }
    
    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm.clamp(60.0, 200.0);
        self.riff_generator.set_bpm(self.bpm);
    }
    
    pub fn set_paused(&mut self, paused: bool) {
        self.is_paused = paused;
    }
    
    pub fn set_scale(&mut self, scale: Scale) {
        self.riff_generator.set_scale(scale);
    }
    
    pub fn set_root_note(&mut self, root_note: f32) {
        self.riff_generator.set_root_note(root_note);
    }
    
    fn handle_synth_event(&mut self, event: &crate::events::ClientEvent) -> Result<(), String> {
        match event.event.as_str() {
            "trigger" => {
                self.synth.trigger();
                Ok(())
            }
            "set_gain" => {
                self.synth.set_gain(event.param());
                Ok(())
            }
            "set_detune" => {
                self.synth.set_detune(event.param());
                Ok(())
            }
            "set_stereo_width" => {
                self.synth.set_stereo_width(event.param());
                Ok(())
            }
            "set_filter_cutoff" => {
                self.synth.set_filter_cutoff(event.param());
                Ok(())
            }
            "set_filter_resonance" => {
                self.synth.set_filter_resonance(event.param());
                Ok(())
            }
            "set_filter_env_amount" => {
                self.synth.set_filter_env_amount(event.param());
                Ok(())
            }
            "set_amp_attack" => {
                self.synth.set_amp_attack(event.param());
                Ok(())
            }
            "set_amp_release" => {
                self.synth.set_amp_release(event.param());
                Ok(())
            }
            "set_filter_attack" => {
                self.synth.set_filter_attack(event.param());
                Ok(())
            }
            "set_filter_release" => {
                self.synth.set_filter_release(event.param());
                Ok(())
            }
            _ => Err(format!("Unknown synth event: {}", event.event)),
        }
    }
    
    fn handle_system_event(&mut self, event: &crate::events::ClientEvent) -> Result<(), String> {
        match event.event.as_str() {
            "set_bpm" => {
                self.set_bpm(event.param());
                Ok(())
            }
            "set_paused" => {
                self.set_paused(event.param() > 0.5);
                Ok(())
            }
            "set_root_note" => {
                self.set_root_note(event.param());
                Ok(())
            }
            "set_scale" => {
                let scale = match event.param() as u32 {
                    0 => Scale::Major,
                    1 => Scale::Minor,
                    2 => Scale::Dorian,
                    3 => Scale::Phrygian,
                    4 => Scale::Mixolydian,
                    5 => Scale::Blues,
                    _ => Scale::Minor,
                };
                self.set_scale(scale);
                Ok(())
            }
            _ => Err(format!("Unknown system event: {}", event.event)),
        }
    }
}

impl AudioSystem for TranceRiffSystem {
    fn next_sample(&mut self) -> (f32, f32) {
        if self.is_paused {
            return (0.0, 0.0);
        }
        
        // Tick the riff generator
        let (frequency, should_trigger) = self.riff_generator.tick();
        
        // Trigger new notes when needed
        if should_trigger {
            self.synth.set_base_frequency(frequency);
            self.synth.trigger();
        }
        
        // Generate audio sample
        self.synth.next_sample()
    }
    
    fn handle_client_event(&mut self, event: &crate::events::ClientEvent) -> Result<(), String> {
        match event.node.as_str() {
            "synth" => self.handle_synth_event(event),
            "system" => self.handle_system_event(event),
            _ => Err(format!("Unknown node '{}' for trance riff system", event.node)),
        }
    }
    
    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.synth.set_sample_rate(sample_rate);
        self.riff_generator.set_sample_rate(sample_rate);
    }
}