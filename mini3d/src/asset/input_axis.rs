use serde::{Serialize, Deserialize};
use slotmap::new_key_type;

use super::Asset;

new_key_type! { pub struct InputAxisId; }

#[derive(Default, Clone, Serialize, Deserialize)]
pub enum InputAxisKind {
    Clamped { min: f32, max: f32 },
    Normalized { norm: f32 },
    ClampedNormalized { min: f32, max: f32, norm: f32 },
    #[default]
    Infinite,
}

#[derive(Serialize, Deserialize)]
pub struct InputAxis {
    pub display_name: String,
    pub description: String,
    pub kind: InputAxisKind,
    pub default_value: f32,
}

impl Asset for InputAxis {
    type Id = InputAxisId;
    fn typename() -> &'static str { "input_axis" }
}