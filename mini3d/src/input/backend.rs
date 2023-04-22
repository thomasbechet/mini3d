use std::{error::Error, fmt::Display};

use crate::{feature::asset::input_table::InputTable, uid::UID};

#[derive(Debug)]
pub enum InputBackendError {
    Unknown,
}

impl Error for InputBackendError {}

impl Display for InputBackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputBackendError::Unknown => write!(f, "Unknown error"),
        }
    }
}

#[allow(unused_variables)]
pub trait InputBackend {
    fn update_table(&mut self, uid: UID, table: Option<&InputTable>) -> Result<(), InputBackendError> { Ok(()) }
}

#[derive(Default)]
pub struct DummyInputBackend;

impl InputBackend for DummyInputBackend {}