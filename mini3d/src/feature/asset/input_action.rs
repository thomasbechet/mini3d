use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct InputActionAsset {
    pub display_name: String,
    pub description: String,
    pub default_pressed: bool,
}