use std::collections::HashSet;

use anyhow::{Result, anyhow};
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
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub range: InputAxisRange,
    pub default_value: f32,
}

impl InputAxis {
    pub fn uid(&self) -> UID {
        UID::new(&self.name)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InputAction {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub default_pressed: bool,
}

impl InputAction {
    pub fn uid(&self) -> UID {
        UID::new(&self.name)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InputTable {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub actions: Vec<InputAction>,
    pub axis: Vec<InputAxis>,
}

impl Asset for InputTable {}

impl InputTable {
    pub const NAME: &'static str = "input_table";
    pub const UID: UID = UID::new(InputTable::NAME);

    pub fn check_valid(&self) -> Result<()> {
        let mut unique = HashSet::new();
        if !self.actions.iter().all(move |action| unique.insert(action.uid())) {
            return Err(anyhow!("Duplicated action name"));
        }
        let mut unique = HashSet::new();
        if !self.axis.iter().all(move |axis| unique.insert(axis.uid())) {
            return Err(anyhow!("Duplicated axis name"));
        }
        Ok(())
    }

    pub fn uid(&self) -> UID {
        UID::new(&self.name)
    }
}