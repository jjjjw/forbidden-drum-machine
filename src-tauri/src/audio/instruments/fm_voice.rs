use crate::audio::envelopes::{AREEnvelope, AREnvelope};
use crate::audio::oscillators::PMOscillator;
use crate::audio::AudioGenerator;

pub struct FMVoice {
    // 4 operators with their own envelopes
    operators: [PMOscillator; 4],
    op_envelopes: [AREEnvelope; 4],

    // Voice amplitude envelope
    amp_envelope: AREnvelope,

    // Operator frequencies (as multipliers of base frequency)
    op_multipliers: [f32; 4],

    // Modulation matrix parameters
    op2_to_op1_amount: f32, // op2 modulates op1
    op1_to_op0_amount: f32, // op1 modulates op0
    op3_to_op0_amount: f32, // op3 modulates op0

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
        op0_out * amp_env * self.gain
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        for i in 0..4 {
            self.operators[i].set_sample_rate(sample_rate);
            self.op_envelopes[i].set_sample_rate(sample_rate);
        }
        self.amp_envelope.set_sample_rate(sample_rate);
    }
}

