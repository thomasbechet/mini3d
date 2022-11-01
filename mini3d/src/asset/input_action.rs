use serde::{Serialize, Deserialize};
use slotmap::new_key_type;

use super::Asset;

new_key_type! { pub struct InputActionId; }

#[derive(Serialize, Deserialize)]
pub struct InputAction {
    pub display_name: String,
    pub description: String,
    pub default_pressed: bool, 
}

impl Asset for InputAction {
    type Id = InputActionId;
    fn typename() -> &'static str { "input_action" }
}