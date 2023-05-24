use mini3d_derive::Error;

use crate::{feature::asset::input_table::InputTable, uid::UID};

#[derive(Debug, Error)]
pub enum InputBackendError {
    #[error("Unknown error")]
    Unknown,
}

#[allow(unused_variables)]
pub trait InputBackend {
    fn update_table(&mut self, uid: UID, table: Option<&InputTable>) -> Result<(), InputBackendError> { Ok(()) }
}

#[derive(Default)]
pub struct DummyInputBackend;

impl InputBackend for DummyInputBackend {}