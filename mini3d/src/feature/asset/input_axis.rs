use serde::{Serialize, Deserialize};

#[derive(Default, Clone, Copy, Serialize, Deserialize)]
pub enum InputAxisRange {
    Clamped { min: f32, max: f32 },
    Normalized { norm: f32 },
    ClampedNormalized { min: f32, max: f32, norm: f32 },
    #[default]
    Infinite,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InputAxisAsset {
    pub display_name: String,
    pub description: String,
    pub range: InputAxisRange,
    pub default_value: f32,
}