use crate::audio::buffers::DelayBuffer;
use crate::audio::{AudioProcessor, PI, SAMPLE_RATE};

// Tan approximation function
fn tan_a(x: f32) -> f32 {
    let x2 = x * x;
    x * (0.999999492001 + x2 * -0.096524608111)
        / (1.0 + x2 * (-0.429867256894 + x2 * 0.009981877999))
}

#[derive(Clone, Copy)]
pub enum FilterMode {
    Lowpass,
    Highpass,
    Bandpass,
}

// SVF implementation matching Emilie Gillet's stmlib version
pub struct SVF {
    // State variables
    y0: f32,
    y1: f32,

    // Filter outputs
    lp: f32,
    hp: f32,
    bp: f32,

    // Filter parameters
    mode: FilterMode,
    cf: f32, // Cutoff frequency
    q: f32,  // Resonance

    // Precomputed coefficients
    g: f32,
    r: f32,
    h: f32,
    rpg: f32,

    coeffs_dirty: bool,
}

impl SVF {
    pub fn new(cf: f32, q: f32, mode: FilterMode) -> Self {
        let mut svf = Self {
            y0: 0.0,
            y1: 0.0,
            lp: 0.0,
            hp: 0.0,
            bp: 0.0,
            mode,
            cf,
            q,
            g: 0.0,
            r: 0.0,
            h: 0.0,
            rpg: 0.0,
            coeffs_dirty: true,
        };
        svf.update_coefficients();
        svf
    }

    fn update_coefficients(&mut self) {
        if self.coeffs_dirty {
            self.g = tan_a(self.cf * PI / SAMPLE_RATE);
            self.r = 1.0 / self.q;
            self.h = 1.0 / (1.0 + self.r * self.g + self.g * self.g);
            self.rpg = self.r + self.g;
            self.coeffs_dirty = false;
        }
    }

    pub fn set_cutoff_frequency(&mut self, cf: f32) {
        if (self.cf - cf).abs() > f32::EPSILON {
            self.cf = cf;
            self.coeffs_dirty = true;
        }
    }

    pub fn set_resonance(&mut self, q: f32) {
        if (self.q - q).abs() > f32::EPSILON {
            self.q = q;
            self.coeffs_dirty = true;
        }
    }

    pub fn set_mode(&mut self, mode: FilterMode) {
        self.mode = mode;
    }

    pub fn reset(&mut self) {
        self.y0 = 0.0;
        self.y1 = 0.0;
        self.lp = 0.0;
        self.hp = 0.0;
        self.bp = 0.0;
    }
}

impl AudioProcessor for SVF {
    fn process(&mut self, input: f32) -> f32 {
        self.update_coefficients();

        self.hp = (input - self.rpg * self.y0 - self.y1) * self.h;
        self.bp = self.g * self.hp + self.y0;
        self.y0 = self.g * self.hp + self.bp;
        self.lp = self.g * self.bp + self.y1;
        self.y1 = self.g * self.bp + self.lp;

        match self.mode {
            FilterMode::Lowpass => self.lp,
            FilterMode::Highpass => self.hp,
            FilterMode::Bandpass => self.bp,
        }
    }
}

#[derive(Clone, Copy)]
pub enum OnePoleMode {
    Lowpass,
    Highpass,
}

pub struct OnePoleFilter {
    state: f32,
    cutoff: f32,
    mode: OnePoleMode,
    a0: f32,
    b1: f32,
    coeffs_dirty: bool,
}

impl OnePoleFilter {
    pub fn new(cutoff: f32, mode: OnePoleMode) -> Self {
        let mut filter = Self {
            state: 0.0,
            cutoff,
            mode,
            a0: 0.0,
            b1: 0.0,
            coeffs_dirty: true,
        };
        filter.update_coefficients();
        filter
    }

    fn update_coefficients(&mut self) {
        if self.coeffs_dirty {
            let omega = 2.0 * PI * self.cutoff / SAMPLE_RATE;
            self.b1 = (-omega).exp();
            self.a0 = 1.0 - self.b1;
            self.coeffs_dirty = false;
        }
    }

    pub fn set_cutoff_frequency(&mut self, cutoff: f32) {
        if (self.cutoff - cutoff).abs() > f32::EPSILON {
            self.cutoff = cutoff;
            self.coeffs_dirty = true;
        }
    }

    pub fn set_mode(&mut self, mode: OnePoleMode) {
        self.mode = mode;
    }

    pub fn reset(&mut self) {
        self.state = 0.0;
    }
}

impl AudioProcessor for OnePoleFilter {
    fn process(&mut self, input: f32) -> f32 {
        self.update_coefficients();
        let lowpass = self.b1 * self.state + self.a0 * input;
        self.state = lowpass;

        match self.mode {
            OnePoleMode::Lowpass => lowpass,
            OnePoleMode::Highpass => input - lowpass,
        }
    }
}

// Allpass filter
pub struct Allpass {
    delay: DelayBuffer,
    g: f32, // Feedback gain
}

impl Allpass {
    pub fn new(max_delay_samples: usize) -> Self {
        Self {
            delay: DelayBuffer::new(max_delay_samples),
            g: 0.0, // Default feedback gain
        }
    }

    pub fn set_delay_seconds(&mut self, seconds: f32) {
        self.delay.set_delay_seconds(seconds);
    }

    pub fn set_feedback(&mut self, g: f32) {
        self.g = g.clamp(-0.99, 0.99); // Clamp to avoid instability
    }
}

impl AudioProcessor for Allpass {
    fn process(&mut self, input: f32) -> f32 {
        let z = self.delay.read();
        let x = input + z * self.g;
        let y = z + x * -self.g;
        self.delay.write(x);
        y
    }
}
