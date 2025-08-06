use super::fm_voice::FMVoice;
use crate::audio::AudioGenerator;

pub struct ChordSynth {
    voices: Vec<FMVoice>,
    chord_ratios: Vec<f32>, // Just intonation ratios
    base_frequency: f32,
    gain: f32,
}

impl ChordSynth {
    pub fn new(sample_rate: f32) -> Self {
        // Create 5 voices for the chord (matching inspiration.gen)
        let mut voices = Vec::new();
        for _ in 0..5 {
            voices.push(FMVoice::new(sample_rate));
        }

        // Just intonation ratios matching the original semitone intervals
        // -5, 2, 5, 9, 10 semitones from inspiration.gen
        let chord_ratios = vec![
            2.0_f32.powf(-5.0 / 12.0), // -5 semitones (minor 4th below)
            9.0 / 8.0,                 // +2 semitones (major 2nd) - just intonation
            4.0 / 3.0,                 // +5 semitones (perfect 4th) - just intonation
            5.0 / 3.0,                 // +9 semitones (major 6th) - just intonation
            15.0 / 8.0,                // +10 semitones (major 7th) - just intonation
        ];

        let mut chord = Self {
            voices,
            chord_ratios,
            base_frequency: 220.0, // A3
            gain: 0.25,
        };

        // Update voice frequencies
        chord.update_frequencies();

        chord
    }

    fn update_frequencies(&mut self) {
        for (i, voice) in self.voices.iter_mut().enumerate() {
            if i < self.chord_ratios.len() {
                let freq = self.base_frequency * self.chord_ratios[i];
                voice.set_base_frequency(freq);
            }
        }
    }

    pub fn trigger(&mut self) {
        for voice in self.voices.iter_mut() {
            voice.trigger();
        }
    }

    pub fn set_base_frequency(&mut self, freq: f32) {
        self.base_frequency = freq;
        self.update_frequencies();
    }

    pub fn set_modulation_index(&mut self, index: f32) {
        for voice in self.voices.iter_mut() {
            voice.set_modulation_index(index);
        }
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        for voice in self.voices.iter_mut() {
            voice.set_feedback(feedback);
        }
    }

    pub fn set_attack(&mut self, time: f32) {
        for voice in self.voices.iter_mut() {
            voice.set_attack(time);
        }
    }

    pub fn set_release(&mut self, time: f32) {
        for voice in self.voices.iter_mut() {
            voice.set_release(time);
        }
    }

    pub fn is_active(&self) -> bool {
        self.voices.iter().any(|v| v.is_active())
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }
}

impl AudioGenerator for ChordSynth {
    fn next_sample(&mut self) -> f32 {
        if !self.is_active() {
            return 0.0;
        }

        let mut output = 0.0;
        for voice in self.voices.iter_mut() {
            output += voice.next_sample();
        }

        // Mix down the voices
        output * 0.2 // Divide by 5 for equal mixing
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        for voice in self.voices.iter_mut() {
            AudioGenerator::set_sample_rate(voice, sample_rate);
        }
    }
}

