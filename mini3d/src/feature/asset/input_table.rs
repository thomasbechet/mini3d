use serde::{Serialize, Deserialize};

use crate::{uid::UID, registry::asset::Asset};

#[derive(Default, Clone, Copy, Serialize, Deserialize)]
pub enum InputAxisRange {
    Clamped { min: f32, max: f32 },
    Normalized { norm: f32 },
    ClampedNormalized { min: f32, max: f32, norm: f32 },
    #[default]
    Infinite,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InputAxis {
    pub display_name: String,
    pub description: String,
    pub range: InputAxisRange,
    pub default_value: f32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InputAction {
    pub display_name: String,
    pub description: String,
    pub default_pressed: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InputTable {
    pub display_name: String,
    pub description: String,
    pub actions: Vec<InputAction>,
    pub axis: Vec<InputAxis>,
}

impl Asset for InputTable {}

impl InputTable {
    pub const NAME: &'static str = "input_table";
    pub const UID: UID = UID::new(InputTable::NAME);
}