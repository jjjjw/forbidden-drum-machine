/// Tatum clock that provides timing signals for all sequencers
pub struct TatumClock {
    bpm: f32,
    tatums_per_beat: u32,
    sample_rate: f32,
    samples_per_tatum: u32,
    sample_counter: u32,
}

impl TatumClock {
    pub fn new(sample_rate: f32) -> Self {
        let mut clock = Self {
            bpm: 120.0,
            tatums_per_beat: 8, // 8 tatums per beat = 32nd note resolution
            sample_rate,
            samples_per_tatum: 0,
            sample_counter: 0,
        };
        clock.recalculate_timing();
        clock
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm.clamp(60.0, 200.0);
        self.recalculate_timing();
    }

    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.recalculate_timing();
    }

    fn recalculate_timing(&mut self) {
        let calculated = ((60.0 * self.sample_rate) / (self.bpm * self.tatums_per_beat as f32)) as u32;
        // Ensure we never get 0 samples per tatum
        self.samples_per_tatum = calculated.max(1);
        println!("TatumClock: BPM={}, sample_rate={}, tatums_per_beat={}, samples_per_tatum={}", 
                 self.bpm, self.sample_rate, self.tatums_per_beat, self.samples_per_tatum);
    }

    /// Call this once per audio sample. Returns true when a new tatum begins.
    pub fn tick(&mut self) -> bool {
        let is_new_tatum = self.sample_counter % self.samples_per_tatum == 0;
        self.sample_counter = self.sample_counter.wrapping_add(1);
        is_new_tatum
    }

    pub fn reset(&mut self) {
        self.sample_counter = 0;
    }
}

/// A sequencer that plays through a list of frequencies and durations
pub struct TonalSequencer {
    /// List of notes: (frequency_hz, duration_tatums, velocity)
    sequence: Vec<(f32, u32, f32)>,
    /// Current position in the sequence
    current_index: usize,
    /// Tatums remaining for current note
    tatums_remaining: u32,
    /// Current frequency being played
    current_frequency: f32,
    /// Current velocity being played
    current_velocity: f32,
}

impl TonalSequencer {
    pub fn new() -> Self {
        Self {
            sequence: Vec::new(),
            current_index: 0,
            tatums_remaining: 0,
            current_frequency: 0.0,
            current_velocity: 0.0,
        }
    }

    /// Set a new sequence
    pub fn set_sequence(&mut self, sequence: Vec<(f32, u32, f32)>) {
        self.sequence = sequence;
        self.reset();
    }

    /// Push a new note to the end of the sequence
    pub fn push(&mut self, frequency: f32, duration_tatums: u32, velocity: f32) {
        self.sequence.push((frequency, duration_tatums, velocity));
    }

    /// Pop the last note from the sequence
    pub fn pop(&mut self) -> Option<(f32, u32, f32)> {
        let result = self.sequence.pop();

        // Adjust current index if needed
        if !self.sequence.is_empty() && self.current_index >= self.sequence.len() {
            self.current_index = 0;
            self.tatums_remaining = 0;
        }

        result
    }

    /// Replace a note at the given index
    pub fn replace(&mut self, index: usize, frequency: f32, duration_tatums: u32, velocity: f32) {
        if index < self.sequence.len() {
            self.sequence[index] = (frequency, duration_tatums, velocity);
        }
    }

    /// Swap two elements in the sequence
    pub fn swap(&mut self, index_a: usize, index_b: usize) {
        if index_a < self.sequence.len() && index_b < self.sequence.len() {
            self.sequence.swap(index_a, index_b);
        }
    }

    /// Reset to the beginning of the sequence
    pub fn reset(&mut self) {
        self.current_index = 0;
        self.tatums_remaining = 0;
        self.current_frequency = 0.0;
        self.current_velocity = 0.0;
    }

    /// Get the current frequency
    pub fn current_frequency(&self) -> f32 {
        self.current_frequency
    }

    /// Get the current velocity
    pub fn current_velocity(&self) -> f32 {
        self.current_velocity
    }

    /// Process a tatum event from the master clock
    /// Returns (should_trigger_note, frequency, velocity)
    pub fn on_tatum(&mut self) -> (bool, f32, f32) {
        if self.sequence.is_empty() {
            return (false, 0.0, 0.0);
        }

        // Check if we need to move to the next note
        if self.tatums_remaining == 0 {
            // Get the next note in the sequence
            if let Some(&(freq, duration_tatums, velocity)) = self.sequence.get(self.current_index)
            {
                self.current_frequency = freq;
                self.current_velocity = velocity;
                self.tatums_remaining = duration_tatums;

                // Move to next index for next time
                self.current_index = (self.current_index + 1) % self.sequence.len();

                return (true, freq, velocity);
            }
        }

        // Decrement tatum counter
        if self.tatums_remaining > 0 {
            self.tatums_remaining -= 1;
        }

        (false, self.current_frequency, self.current_velocity)
    }

    /// Get current state (frequency, velocity) - call every audio sample
    pub fn current_state(&self) -> (f32, f32) {
        (self.current_frequency, self.current_velocity)
    }

    /// Set the playback position (0.0 to 1.0)
    pub fn set_position(&mut self, position: f32) {
        if self.sequence.is_empty() {
            return;
        }

        let position = position.clamp(0.0, 1.0);

        // Calculate total duration in tatums
        let total_tatums: u32 = self
            .sequence
            .iter()
            .map(|(_, duration_tatums, _)| *duration_tatums)
            .sum();
        let target_tatum = (position * total_tatums as f32) as u32;

        // Find which note we should be at
        let mut accumulated = 0u32;
        for (index, &(freq, duration_tatums, velocity)) in self.sequence.iter().enumerate() {
            if accumulated + duration_tatums > target_tatum {
                self.current_index = index;
                self.tatums_remaining = duration_tatums - (target_tatum - accumulated);
                self.current_frequency = freq;
                self.current_velocity = velocity;
                return;
            }
            accumulated += duration_tatums;
        }
    }
}
