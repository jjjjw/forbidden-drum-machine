/// Euclidean rhythm generator using Bjorklund's algorithm
/// Generates rhythms by distributing k beats as evenly as possible across n steps
pub struct EuclideanSequencer {
    /// Total number of steps in the pattern
    steps: u32,
    /// Number of beats to distribute
    beats: u32,
    /// Current step position (0-based)
    current_step: u32,
    /// Generated pattern as a boolean array
    pattern: Vec<bool>,
    /// Tempo multiplier for this sequencer
    tempo_multiplier: f32,
    /// Internal counter for tempo multiplication
    tempo_counter: f32,
}

impl EuclideanSequencer {
    /// Create a new Euclidean sequencer with given steps and beats
    pub fn new(steps: u32, beats: u32, tempo_multiplier: f32) -> Self {
        let mut sequencer = Self {
            steps,
            beats,
            current_step: 0,
            pattern: Vec::new(),
            tempo_multiplier,
            tempo_counter: 0.0,
        };
        sequencer.generate_pattern();
        sequencer
    }

    /// Generate the Euclidean pattern using Bjorklund's algorithm
    fn generate_pattern(&mut self) {
        self.pattern = bjorklund_algorithm(self.steps, self.beats);
    }

    /// Advance the sequencer by one tick and return whether a beat should trigger
    pub fn tick(&mut self) -> bool {
        // Apply tempo multiplier
        self.tempo_counter += self.tempo_multiplier;
        
        if self.tempo_counter < 1.0 {
            return false;
        }
        
        // Reset counter and advance step
        self.tempo_counter -= 1.0;
        
        let should_trigger = self.pattern[self.current_step as usize];
        
        // Advance to next step
        self.current_step = (self.current_step + 1) % self.steps;
        
        should_trigger
    }

    /// Get the current step position
    pub fn get_current_step(&self) -> u32 {
        self.current_step
    }

    /// Update the number of steps and regenerate pattern
    pub fn set_steps(&mut self, steps: u32) {
        if steps > 0 && steps != self.steps {
            self.steps = steps;
            self.current_step = self.current_step % steps; // Clamp current step
            self.generate_pattern();
        }
    }

    /// Update the number of beats and regenerate pattern
    pub fn set_beats(&mut self, beats: u32) {
        if beats != self.beats {
            self.beats = beats.min(self.steps); // Clamp beats to steps
            self.generate_pattern();
        }
    }

    /// Update the tempo multiplier
    pub fn set_tempo_multiplier(&mut self, multiplier: f32) {
        self.tempo_multiplier = multiplier.max(0.0); // Prevent negative multipliers
    }

    /// Get the current pattern
    pub fn get_pattern(&self) -> &[bool] {
        &self.pattern
    }

    /// Reset the sequencer to the beginning
    pub fn reset(&mut self) {
        self.current_step = 0;
        self.tempo_counter = 0.0;
    }
}

/// Bjorklund's algorithm for generating Euclidean rhythms
/// Distributes k beats as evenly as possible across n steps
fn bjorklund_algorithm(steps: u32, beats: u32) -> Vec<bool> {
    if steps == 0 || beats == 0 {
        return vec![false; steps as usize];
    }
    
    if beats >= steps {
        return vec![true; steps as usize];
    }

    let mut pattern = vec![false; steps as usize];
    
    // Simple distribution algorithm
    let mut remainder = 0u32;
    
    for i in 0..steps {
        remainder += beats;
        if remainder >= steps {
            remainder -= steps;
            pattern[i as usize] = true;
        }
    }
    
    pattern
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_euclidean_pattern_generation() {
        // Test classic Euclidean rhythms
        
        // Son clave (3,8) - should be [x..x..x.]
        let pattern = bjorklund_algorithm(8, 3);
        let expected_beats: Vec<usize> = pattern.iter()
            .enumerate()
            .filter(|(_, &beat)| beat)
            .map(|(i, _)| i)
            .collect();
        assert_eq!(expected_beats.len(), 3);
        
        // Ensure beats are distributed
        assert!(expected_beats[1] - expected_beats[0] >= 2);
        assert!(expected_beats[2] - expected_beats[1] >= 2);
    }

    #[test]
    fn test_sequencer_tick() {
        let mut seq = EuclideanSequencer::new(8, 3, 1.0);
        
        // Count triggers over a full cycle
        let mut trigger_count = 0;
        for _ in 0..8 {
            if seq.tick() {
                trigger_count += 1;
            }
        }
        
        assert_eq!(trigger_count, 3);
    }

    #[test]
    fn test_tempo_multiplier() {
        let mut seq = EuclideanSequencer::new(4, 2, 0.5); // Half speed
        
        // Should not trigger on first tick
        assert!(!seq.tick());
        // Should trigger on second tick (if pattern has beat at position 0)
        let second_tick = seq.tick();
        
        // Verify tempo multiplier affects timing
        assert_eq!(seq.tempo_counter, 0.0); // Counter should reset after trigger
    }

    #[test]
    fn test_pattern_updates() {
        let mut seq = EuclideanSequencer::new(8, 3, 1.0);
        let original_pattern = seq.get_pattern().to_vec();
        
        // Change beats and verify pattern changes
        seq.set_beats(4);
        let new_pattern = seq.get_pattern().to_vec();
        
        assert_ne!(original_pattern, new_pattern);
        assert_eq!(new_pattern.iter().filter(|&&x| x).count(), 4);
    }
}