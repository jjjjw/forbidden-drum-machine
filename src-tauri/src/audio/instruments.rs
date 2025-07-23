use crate::audio::envelopes::{AREEnvelope, AREnvelope, Segment};
use crate::audio::filters::{FilterMode, SVF};
use crate::audio::oscillators::{NoiseGenerator, PMOscillator, SineOscillator};
use crate::audio::{AudioGenerator, AudioNode, AudioProcessor};
use crate::events::NodeEvent;

pub struct KickDrum {
    oscillator: SineOscillator,
    amp_envelope: AREnvelope,
    freq_envelope: AREnvelope,
    base_frequency: f32,
    frequency_ratio: f32,
    gain: f32,
}

impl KickDrum {
    pub fn new(sample_rate: f32) -> Self {
        let mut kick = Self {
            oscillator: SineOscillator::new(60.0, sample_rate),
            amp_envelope: AREnvelope::new(sample_rate),
            freq_envelope: AREnvelope::new(sample_rate),
            base_frequency: 60.0,
            frequency_ratio: 7.0,
            gain: 1.0,
        };

        kick.amp_envelope.set_attack_time(0.005);
        kick.amp_envelope.set_release_time(0.2);
        kick.amp_envelope.set_attack_bias(0.3); // Logarithmic-like
        kick.amp_envelope.set_release_bias(0.7); // Exponential-like

        kick.freq_envelope.set_attack_time(0.002);
        kick.freq_envelope.set_release_time(0.05);
        kick.freq_envelope.set_attack_bias(0.7); // Exponential-like
        kick.freq_envelope.set_release_bias(0.7); // Exponential-like

        kick
    }

    pub fn trigger(&mut self) {
        self.amp_envelope.trigger();
        self.freq_envelope.trigger();
        self.oscillator.reset();
    }

    pub fn set_base_frequency(&mut self, freq: f32) {
        self.base_frequency = freq;
    }

    pub fn set_frequency_ratio(&mut self, ratio: f32) {
        self.frequency_ratio = ratio;
    }

    pub fn set_amp_attack(&mut self, time: f32) {
        self.amp_envelope.set_attack_time(time);
    }

    pub fn set_amp_release(&mut self, time: f32) {
        self.amp_envelope.set_release_time(time);
    }

    pub fn set_freq_attack(&mut self, time: f32) {
        self.freq_envelope.set_attack_time(time);
    }

    pub fn set_freq_release(&mut self, time: f32) {
        self.freq_envelope.set_release_time(time);
    }

    pub fn is_active(&self) -> bool {
        self.amp_envelope.is_active()
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }
}

impl AudioGenerator for KickDrum {
    fn next_sample(&mut self) -> f32 {
        if !self.is_active() {
            return 0.0;
        }

        let amp_env = self.amp_envelope.next_sample();
        let freq_env = self.freq_envelope.next_sample();

        // Use frequency ratio for sharper sweep: starts at base_frequency * ratio, sweeps down to base_frequency
        let start_freq = self.base_frequency * self.frequency_ratio;
        let current_freq = self.base_frequency + (freq_env * (start_freq - self.base_frequency));
        self.oscillator.set_frequency(current_freq);

        let sample = self.oscillator.next_sample();
        sample * amp_env
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.oscillator.set_sample_rate(sample_rate);
        self.amp_envelope.set_sample_rate(sample_rate);
        self.freq_envelope.set_sample_rate(sample_rate);
    }
}

impl AudioNode for KickDrum {
    fn process(&mut self, left_in: f32, right_in: f32) -> (f32, f32) {
        let drum_sample = self.next_sample() * self.gain;
        (left_in + drum_sample, right_in + drum_sample)
    }

    fn handle_event(&mut self, event: NodeEvent) -> Result<(), String> {
        match event {
            NodeEvent::Trigger => {
                self.trigger();
                Ok(())
            }
            NodeEvent::SetGain(gain) => {
                self.set_gain(gain);
                Ok(())
            }
            NodeEvent::SetBaseFrequency(freq) => {
                self.set_base_frequency(freq);
                Ok(())
            }
            NodeEvent::SetFrequencyRatio(ratio) => {
                self.set_frequency_ratio(ratio);
                Ok(())
            }
            NodeEvent::SetAmpAttack(time) => {
                self.set_amp_attack(time);
                Ok(())
            }
            NodeEvent::SetAmpRelease(time) => {
                self.set_amp_release(time);
                Ok(())
            }
            NodeEvent::SetFreqAttack(time) => {
                self.set_freq_attack(time);
                Ok(())
            }
            NodeEvent::SetFreqRelease(time) => {
                self.set_freq_release(time);
                Ok(())
            }
            _ => Err(format!("Unsupported event for KickDrum: {:?}", event)),
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        AudioGenerator::set_sample_rate(self, sample_rate);
    }
}

pub struct SnareDrum {
    noise_generator: NoiseGenerator,
    amp_envelope: AREnvelope,
}

impl SnareDrum {
    pub fn new(sample_rate: f32) -> Self {
        let mut snare = Self {
            noise_generator: NoiseGenerator::new(),
            amp_envelope: AREnvelope::new(sample_rate),
        };

        snare.amp_envelope.set_attack_time(0.001);
        snare.amp_envelope.set_release_time(0.08);
        snare.amp_envelope.set_attack_bias(0.5); // Linear
        snare.amp_envelope.set_release_bias(0.7); // Exponential-like

        snare
    }

    pub fn trigger(&mut self) {
        self.amp_envelope.trigger();
    }

    pub fn set_amp_attack(&mut self, time: f32) {
        self.amp_envelope.set_attack_time(time);
    }

    pub fn set_amp_release(&mut self, time: f32) {
        self.amp_envelope.set_release_time(time);
    }

    pub fn is_active(&self) -> bool {
        self.amp_envelope.is_active()
    }
}

impl AudioGenerator for SnareDrum {
    fn next_sample(&mut self) -> f32 {
        if !self.is_active() {
            return 0.0;
        }

        let amp_env = self.amp_envelope.next_sample();
        let sample = self.noise_generator.next_sample();
        sample * amp_env
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.noise_generator.set_sample_rate(sample_rate);
        self.amp_envelope.set_sample_rate(sample_rate);
    }
}

pub struct ClapDrum {
    noise_generator: NoiseGenerator,

    // Three bandpass filters at different frequencies
    filter_1320: SVF,
    filter_1100: SVF,
    filter_1420: SVF,

    // Multi-segment envelope using individual Segments
    // Pattern: [0, 1, 0, 1, 0, 1, 0] with randomized timing
    envelope_segments: [Segment; 6], // 6 segments for the 7-point envelope
    current_segment: usize,
    envelope_value: f32,
    is_envelope_active: bool,

    sample_rate: f32,
    gain: f32,
}

impl ClapDrum {
    pub fn new(sample_rate: f32) -> Self {
        // Create the multi-segment envelope with randomized timing
        // SuperCollider: [0, 1, 0, 1, 0, 1, 0] with durations [Rand(0.001, 0.01), 0.01, 0.001, 0.01, 0.001, 0.08]
        let envelope_segments = [
            Segment::new(0.0, 1.0, fastrand::f32() * 0.009 + 0.001, 0.9, sample_rate), // 0->1: 0.001-0.01s, fast attack
            Segment::new(1.0, 0.0, 0.01, 0.1, sample_rate), // 1->0: 0.01s, fast decay
            Segment::new(0.0, 1.0, 0.001, 0.9, sample_rate), // 0->1: 0.001s, fast attack
            Segment::new(1.0, 0.0, 0.01, 0.1, sample_rate), // 1->0: 0.01s, fast decay
            Segment::new(0.0, 1.0, 0.001, 0.9, sample_rate), // 0->1: 0.001s, fast attack
            Segment::new(1.0, 0.0, 0.08, 0.3, sample_rate), // 1->0: 0.08s, slow final decay
        ];

        Self {
            noise_generator: NoiseGenerator::new(),

            filter_1320: SVF::new(1320.0, 10.0, FilterMode::Bandpass, sample_rate), // Q=10 for narrow band
            filter_1100: SVF::new(1100.0, 10.0, FilterMode::Bandpass, sample_rate),
            filter_1420: SVF::new(1420.0, 10.0, FilterMode::Bandpass, sample_rate),

            envelope_segments,
            current_segment: 0,
            envelope_value: 0.0,
            is_envelope_active: false,

            sample_rate,
            gain: 1.0,
        }
    }

    pub fn trigger(&mut self) {
        // Randomize the first segment timing (like SuperCollider Rand)
        self.envelope_segments[0].set_duration_seconds(fastrand::f32() * 0.009 + 0.001);

        // Start the envelope sequence
        self.current_segment = 0;
        self.envelope_value = 0.0;
        self.is_envelope_active = true;
        self.envelope_segments[0].trigger();
    }

    pub fn is_active(&self) -> bool {
        self.is_envelope_active
    }

    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }

    fn update_envelope(&mut self) {
        if !self.is_envelope_active {
            self.envelope_value = 0.0;
            return;
        }

        if self.current_segment >= self.envelope_segments.len() {
            self.is_envelope_active = false;
            self.envelope_value = 0.0;
            return;
        }

        // Get current segment value
        if self.envelope_segments[self.current_segment].is_active() {
            self.envelope_value = self.envelope_segments[self.current_segment].next_sample();
        } else if self.envelope_segments[self.current_segment].is_finished() {
            // Move to next segment
            self.current_segment += 1;
            if self.current_segment < self.envelope_segments.len() {
                self.envelope_segments[self.current_segment].trigger();
                self.envelope_value = self.envelope_segments[self.current_segment].next_sample();
            } else {
                self.is_envelope_active = false;
                self.envelope_value = 0.0;
            }
        }
    }
}

impl AudioGenerator for ClapDrum {
    fn next_sample(&mut self) -> f32 {
        if !self.is_active() {
            return 0.0;
        }

        // Update the multi-segment envelope
        self.update_envelope();

        // Generate noise and process through three bandpass filters
        let noise = self.noise_generator.next_sample();

        // Process through all three bandpass filters and sum
        let filtered_1320 = self.filter_1320.process(noise);
        let filtered_1100 = self.filter_1100.process(noise);
        let filtered_1420 = self.filter_1420.process(noise);

        // Sum the filtered signals and apply 10dB gain (10.dbamp ≈ 3.16)
        let filtered_sum = (filtered_1320 + filtered_1100 + filtered_1420) * 3.16;

        // Apply envelope and tanh saturation
        (filtered_sum * self.envelope_value).tanh()
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.noise_generator.set_sample_rate(sample_rate);
        self.filter_1320.set_sample_rate(sample_rate);
        self.filter_1100.set_sample_rate(sample_rate);
        self.filter_1420.set_sample_rate(sample_rate);

        // Update all envelope segments
        for segment in &mut self.envelope_segments {
            segment.set_sample_rate(sample_rate);
        }
    }
}

impl AudioNode for ClapDrum {
    fn process(&mut self, left_in: f32, right_in: f32) -> (f32, f32) {
        let clap_sample = self.next_sample() * self.gain;
        (left_in + clap_sample, right_in + clap_sample)
    }

    fn handle_event(&mut self, event: NodeEvent) -> Result<(), String> {
        match event {
            NodeEvent::Trigger => {
                self.trigger();
                Ok(())
            }
            NodeEvent::SetGain(gain) => {
                self.set_gain(gain);
                Ok(())
            }
            _ => Err(format!("Unsupported event for ClapDrum: {:?}", event)),
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        AudioGenerator::set_sample_rate(self, sample_rate);
    }
}

pub struct FMVoice {
    // 4 operators with their own envelopes
    operators: [PMOscillator; 4],
    op_envelopes: [AREEnvelope; 4],
    
    // Voice amplitude envelope
    amp_envelope: AREnvelope,
    
    // Operator frequencies (as multipliers of base frequency)
    op_multipliers: [f32; 4],
    
    // Modulation matrix parameters
    op2_to_op1_amount: f32,  // op2 modulates op1
    op1_to_op0_amount: f32,  // op1 modulates op0
    op3_to_op0_amount: f32,  // op3 modulates op0
    
    // Global parameters
    base_frequency: f32,
    gain: f32,
}

impl FMVoice {
    pub fn new(sample_rate: f32) -> Self {
        let mut voice = Self {
            operators: [
                PMOscillator::new(220.0, sample_rate),
                PMOscillator::new(440.0, sample_rate),
                PMOscillator::new(660.0, sample_rate),
                PMOscillator::new(2640.0, sample_rate),
            ],
            op_envelopes: [
                AREEnvelope::new(sample_rate),
                AREEnvelope::new(sample_rate),
                AREEnvelope::new(sample_rate),
                AREEnvelope::new(sample_rate),
            ],
            amp_envelope: AREnvelope::new(sample_rate),
            op_multipliers: [1.0, 2.0, 3.0, 12.0],
            op2_to_op1_amount: 0.5,
            op1_to_op0_amount: 0.5,
            op3_to_op0_amount: 0.5,
            base_frequency: 220.0,
            gain: 0.5,
        };
        
        // Set up operator envelopes based on inspiration.gen
        // op0: carrier (no decay, stays at 1.0)
        voice.op_envelopes[0].set_attack_time(0.001);
        voice.op_envelopes[0].set_release_time(0.0);
        voice.op_envelopes[0].set_end_level(1.0);
        
        // op1: modulator (decay to 0.25)
        voice.op_envelopes[1].set_attack_time(0.001);
        voice.op_envelopes[1].set_release_time(1.0);
        voice.op_envelopes[1].set_end_level(0.25);
        
        // op2: modulator (decay to 0)
        voice.op_envelopes[2].set_attack_time(0.001);
        voice.op_envelopes[2].set_release_time(4.0);
        voice.op_envelopes[2].set_end_level(0.0);
        
        // op3: modulator (decay to 0)
        voice.op_envelopes[3].set_attack_time(0.001);
        voice.op_envelopes[3].set_release_time(8.0);
        voice.op_envelopes[3].set_end_level(0.0);
        
        // Voice amplitude envelope
        voice.amp_envelope.set_attack_time(0.5);
        voice.amp_envelope.set_release_time(4.0);
        voice.amp_envelope.set_attack_bias(0.3);
        voice.amp_envelope.set_release_bias(0.7);
        
        voice
    }
    
    pub fn trigger(&mut self) {
        self.amp_envelope.trigger();
        for i in 0..4 {
            self.op_envelopes[i].trigger();
            self.operators[i].reset();
        }
    }
    
    pub fn set_base_frequency(&mut self, freq: f32) {
        self.base_frequency = freq;
        for i in 0..4 {
            self.operators[i].set_frequency(freq * self.op_multipliers[i]);
        }
    }
    
    pub fn set_op_multiplier(&mut self, op_index: usize, multiplier: f32) {
        if op_index < 4 {
            self.op_multipliers[op_index] = multiplier;
            self.operators[op_index].set_frequency(self.base_frequency * multiplier);
        }
    }
    
    pub fn set_modulation_index(&mut self, index: f32) {
        // Scale the modulation amounts together
        let scale = index.clamp(0.0, 2.0);
        self.op2_to_op1_amount = 0.5 * scale;
        self.op1_to_op0_amount = 0.5 * scale;
        self.op3_to_op0_amount = 0.5 * scale;
    }
    
    pub fn set_feedback(&mut self, feedback: f32) {
        // Apply feedback to all operators
        for op in self.operators.iter_mut() {
            op.set_feedback(feedback);
        }
    }
    
    pub fn set_attack(&mut self, time: f32) {
        self.amp_envelope.set_attack_time(time);
    }
    
    pub fn set_release(&mut self, time: f32) {
        self.amp_envelope.set_release_time(time);
    }
    
    pub fn is_active(&self) -> bool {
        self.amp_envelope.is_active()
    }
    
    pub fn set_gain(&mut self, gain: f32) {
        self.gain = gain;
    }
}

impl AudioGenerator for FMVoice {
    fn next_sample(&mut self) -> f32 {
        if !self.is_active() {
            return 0.0;
        }
        
        // Get envelope values
        let amp_env = self.amp_envelope.next_sample();
        let op_envs: [f32; 4] = [
            self.op_envelopes[0].next_sample(),
            self.op_envelopes[1].next_sample(),
            self.op_envelopes[2].next_sample(),
            self.op_envelopes[3].next_sample(),
        ];
        
        // Generate operators with modulation routing
        // op3: pure (no modulation input)
        let op3_out = self.operators[3].next_sample_with_pm(0.0) * op_envs[3];
        
        // op2: pure (no modulation input)
        let op2_out = self.operators[2].next_sample_with_pm(0.0) * op_envs[2];
        
        // op1: modulated by op2
        let op1_pm = op2_out * self.op2_to_op1_amount;
        let op1_out = self.operators[1].next_sample_with_pm(op1_pm) * op_envs[1];
        
        // op0: carrier modulated by op1 and op3
        let op0_pm = op1_out * self.op1_to_op0_amount + op3_out * self.op3_to_op0_amount;
        let op0_out = self.operators[0].next_sample_with_pm(op0_pm) * op_envs[0];
        
        // Output is op0 with amplitude envelope
        op0_out * amp_env
    }
    
    fn set_sample_rate(&mut self, sample_rate: f32) {
        for i in 0..4 {
            self.operators[i].set_sample_rate(sample_rate);
            self.op_envelopes[i].set_sample_rate(sample_rate);
        }
        self.amp_envelope.set_sample_rate(sample_rate);
    }
}

impl AudioNode for FMVoice {
    fn process(&mut self, left_in: f32, right_in: f32) -> (f32, f32) {
        let voice_sample = self.next_sample() * self.gain;
        (left_in + voice_sample, right_in + voice_sample)
    }
    
    fn handle_event(&mut self, event: NodeEvent) -> Result<(), String> {
        match event {
            NodeEvent::Trigger => {
                self.trigger();
                Ok(())
            }
            NodeEvent::SetGain(gain) => {
                self.set_gain(gain);
                Ok(())
            }
            NodeEvent::SetBaseFrequency(freq) => {
                self.set_base_frequency(freq);
                Ok(())
            }
            NodeEvent::SetModulationIndex(index) => {
                self.set_modulation_index(index);
                Ok(())
            }
            NodeEvent::SetFeedback(feedback) => {
                self.set_feedback(feedback);
                Ok(())
            }
            NodeEvent::SetAmpAttack(time) => {
                self.set_attack(time);
                Ok(())
            }
            NodeEvent::SetAmpRelease(time) => {
                self.set_release(time);
                Ok(())
            }
            _ => Err(format!("Unsupported event for FMVoice: {:?}", event)),
        }
    }
    
    fn set_sample_rate(&mut self, sample_rate: f32) {
        AudioGenerator::set_sample_rate(self, sample_rate);
    }
}

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
            2.0_f32.powf(-5.0/12.0),  // -5 semitones (minor 4th below)
            9.0/8.0,                  // +2 semitones (major 2nd) - just intonation
            4.0/3.0,                  // +5 semitones (perfect 4th) - just intonation  
            5.0/3.0,                  // +9 semitones (major 6th) - just intonation
            15.0/8.0,                 // +10 semitones (major 7th) - just intonation
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

impl AudioNode for ChordSynth {
    fn process(&mut self, left_in: f32, right_in: f32) -> (f32, f32) {
        let chord_sample = self.next_sample() * self.gain;
        (left_in + chord_sample, right_in + chord_sample)
    }
    
    fn handle_event(&mut self, event: NodeEvent) -> Result<(), String> {
        match event {
            NodeEvent::Trigger => {
                self.trigger();
                Ok(())
            }
            NodeEvent::SetGain(gain) => {
                self.set_gain(gain);
                Ok(())
            }
            NodeEvent::SetBaseFrequency(freq) => {
                self.set_base_frequency(freq);
                Ok(())
            }
            NodeEvent::SetModulationIndex(index) => {
                self.set_modulation_index(index);
                Ok(())
            }
            NodeEvent::SetFeedback(feedback) => {
                self.set_feedback(feedback);
                Ok(())
            }
            NodeEvent::SetAmpAttack(time) => {
                self.set_attack(time);
                Ok(())
            }
            NodeEvent::SetAmpRelease(time) => {
                self.set_release(time);
                Ok(())
            }
            _ => Err(format!("Unsupported event for ChordSynth: {:?}", event)),
        }
    }
    
    fn set_sample_rate(&mut self, sample_rate: f32) {
        AudioGenerator::set_sample_rate(self, sample_rate);
    }
}
