use std::{collections::HashSet, error::Error, fmt::Display};

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

    pub fn validate(&self) -> Result<(), InputTableValidationError> {
        let mut unique = HashSet::new();
        if !self.actions.iter().all(move |action| unique.insert(action.uid())) {
            return Err(InputTableValidationError::DuplicatedAction);
        }
        let mut unique = HashSet::new();
        if !self.axis.iter().all(move |axis| unique.insert(axis.uid())) {
            return Err(InputTableValidationError::DuplicatedAxis);
        }
        Ok(())
    }

    pub fn uid(&self) -> UID {
        UID::new(&self.name)
    }
}

#[derive(Debug)]
pub enum InputTableValidationError {
    DuplicatedAction,
    DuplicatedAxis,
}

impl Error for InputTableValidationError {}

impl Display for InputTableValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputTableValidationError::DuplicatedAction => write!(f, "Duplicated action"),
            InputTableValidationError::DuplicatedAxis => write!(f, "Duplicated axis"),
        }
    }
}