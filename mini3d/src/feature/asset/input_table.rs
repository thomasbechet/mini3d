use std::collections::HashSet;

use mini3d_derive::{Serialize, Asset, Error};

use crate::uid::UID;

#[derive(Default, Clone, Copy, Serialize)]
pub enum InputAxisRange {
    Clamped { min: f32, max: f32 },
    Normalized { norm: f32 },
    ClampedNormalized { min: f32, max: f32, norm: f32 },
    #[default]
    Infinite,
}

#[derive(Clone, Serialize)]
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

#[derive(Clone, Serialize)]
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

#[derive(Clone, Asset)]
pub struct InputTable {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub actions: Vec<InputAction>,
    pub axis: Vec<InputAxis>,
}

impl InputTable {

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

#[derive(Debug, Error)]
pub enum InputTableValidationError {
    #[error("Duplicated action")]
    DuplicatedAction,
    #[error("Duplicated axis")]
    DuplicatedAxis,
}