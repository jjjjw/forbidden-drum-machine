/// A sequencer that plays through a list of frequencies and durations
pub struct TonalSequencer {
    /// List of notes: (frequency_hz, duration_seconds, velocity)
    sequence: Vec<(f32, f32, f32)>,
    /// Current position in the sequence
    current_index: usize,
    /// Samples remaining for current note
    samples_remaining: u32,
    /// Current frequency being played
    current_frequency: f32,
    /// Current velocity being played
    current_velocity: f32,
    /// Sample rate for timing calculations
    sample_rate: f32,
}

impl TonalSequencer {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sequence: Vec::new(),
            current_index: 0,
            samples_remaining: 0,
            current_frequency: 0.0,
            current_velocity: 0.0,
            sample_rate,
        }
    }

    /// Update the sample rate
    pub fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    /// Set a new sequence
    pub fn set_sequence(&mut self, sequence: Vec<(f32, f32, f32)>) {
        self.sequence = sequence;
        self.reset();
    }

    /// Push a new note to the end of the sequence
    pub fn push(&mut self, frequency: f32, duration_seconds: f32, velocity: f32) {
        self.sequence.push((frequency, duration_seconds, velocity));
    }

    /// Pop the last note from the sequence
    pub fn pop(&mut self) -> Option<(f32, f32, f32)> {
        let result = self.sequence.pop();
        
        // Adjust current index if needed
        if !self.sequence.is_empty() && self.current_index >= self.sequence.len() {
            self.current_index = 0;
            self.samples_remaining = 0;
        }
        
        result
    }

    /// Replace a note at the given index
    pub fn replace(&mut self, index: usize, frequency: f32, duration_seconds: f32, velocity: f32) {
        if index < self.sequence.len() {
            self.sequence[index] = (frequency, duration_seconds, velocity);
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
        self.samples_remaining = 0;
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

    /// Advance the sequencer by one sample
    /// Returns (should_trigger_note, frequency, velocity)
    pub fn tick(&mut self) -> (bool, f32, f32) {
        if self.sequence.is_empty() {
            return (false, 0.0, 0.0);
        }

        // Check if we need to move to the next note
        if self.samples_remaining == 0 {
            // Get the next note in the sequence
            if let Some(&(freq, duration_seconds, velocity)) = self.sequence.get(self.current_index) {
                self.current_frequency = freq;
                self.current_velocity = velocity;
                // Convert duration from seconds to samples
                self.samples_remaining = (duration_seconds * self.sample_rate) as u32;

                // Move to next index for next time
                self.current_index = (self.current_index + 1) % self.sequence.len();

                return (true, freq, velocity);
            }
        }

        // Decrement sample counter
        if self.samples_remaining > 0 {
            self.samples_remaining -= 1;
        }

        (false, self.current_frequency, self.current_velocity)
    }

    /// Set the playback position (0.0 to 1.0)
    pub fn set_position(&mut self, position: f32) {
        if self.sequence.is_empty() {
            return;
        }

        let position = position.clamp(0.0, 1.0);
        
        // Calculate total duration in samples
        let total_samples: u32 = self.sequence
            .iter()
            .map(|(_, dur_secs, _)| (dur_secs * self.sample_rate) as u32)
            .sum();
        let target_sample = (position * total_samples as f32) as u32;
        
        // Find which note we should be at
        let mut accumulated = 0u32;
        for (index, &(freq, duration_seconds, velocity)) in self.sequence.iter().enumerate() {
            let duration_samples = (duration_seconds * self.sample_rate) as u32;
            if accumulated + duration_samples > target_sample {
                self.current_index = index;
                self.samples_remaining = duration_samples - (target_sample - accumulated);
                self.current_frequency = freq;
                self.current_velocity = velocity;
                return;
            }
            accumulated += duration_samples;
        }
    }
}