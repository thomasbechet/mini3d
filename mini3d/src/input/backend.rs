use mini3d_derive::Error;

use crate::{feature::component::input::input_table::InputTable, utils::uid::UID};

use super::event::InputEvent;

#[derive(Debug, Error)]
pub enum InputBackendError {
    #[error("Unknown error")]
    Unknown,
}

#[allow(unused_variables)]
pub trait InputBackend {
    fn events(&self) -> &[InputEvent] {
        &[]
    }

    fn update_table(
        &mut self,
        uid: UID,
        table: Option<&InputTable>,
    ) -> Result<(), InputBackendError> {
        Ok(())
    }
}

#[derive(Default)]
pub struct DummyInputBackend;

impl InputBackend for DummyInputBackend {}
