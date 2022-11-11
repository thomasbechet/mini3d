use serde::{Serialize, Deserialize};

use super::Asset;

#[derive(Clone, Serialize, Deserialize)]
pub struct InputAction {
    pub display_name: String,
    pub description: String,
    pub default_pressed: bool,
}

impl Asset for InputAction {}