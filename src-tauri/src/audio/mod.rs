pub mod oscillators;
pub mod envelopes;
pub mod effects;
pub mod instruments;
pub mod systems;
pub mod modulators;

pub const SAMPLE_RATE: f32 = 44100.0;
pub const PI: f32 = std::f32::consts::PI;
pub const TWO_PI: f32 = 2.0 * PI;