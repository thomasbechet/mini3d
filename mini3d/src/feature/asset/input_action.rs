use serde::{Serialize, Deserialize};

use crate::{registry::asset::Asset, uid::UID};

#[derive(Clone, Serialize, Deserialize)]
pub struct InputAction {
    pub display_name: String,
    pub description: String,
    pub default_pressed: bool,
}

impl Asset for InputAction {}

impl InputAction {
    pub const NAME: &'static str = "input_action";
    pub const UID: UID = UID::new(InputAction::NAME);
}