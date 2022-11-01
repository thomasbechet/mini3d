use anyhow::Result;
use serde::{Serialize, Deserialize};
use slotmap::new_key_type;

use super::{Asset, AssetRef, input_action::InputAction, input_axis::InputAxis, AssetManager};

new_key_type! { pub struct InputTableId; }

#[derive(Serialize, Deserialize)]
pub struct InputTable {
    pub display_name: String,
    pub description: String,
    pub actions: Vec<AssetRef<InputAction>>,
    pub axis: Vec<AssetRef<InputAxis>>,
}

impl Asset for InputTable {
    type Id = InputTableId;
    fn typename() -> &'static str { "input_table" }
}

impl InputTable {
    pub fn resolve(&mut self, asset: &AssetManager) -> Result<()> {
        for action in &mut self.actions {
            action.resolve(asset)?;
        }
        for axis in &mut self.axis {
            axis.resolve(asset)?;
        }
        Ok(())
    }
}