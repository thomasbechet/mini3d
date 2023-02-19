use serde::{Serialize, Deserialize};

use crate::{uid::UID, registry::asset::Asset};

#[derive(Clone, Serialize, Deserialize)]
pub struct InputTable {
    pub display_name: String,
    pub description: String,
    pub actions: Vec<UID>,
    pub axis: Vec<UID>,
}

impl Asset for InputTable {}

impl InputTable {
    pub const NAME: &'static str = "input_table";
    pub const UID: UID = InputTable::NAME.into();
}