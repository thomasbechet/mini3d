use mini3d_derive::Error;

use crate::{feature::component::input::input_table::InputTable, utils::uid::UID};

use super::event::InputEvent;

#[derive(Debug, Error)]
pub enum InputServerError {
    #[error("Unknown error")]
    Unknown,
}

#[allow(unused_variables)]
pub trait InputServer {
    fn poll_event(&self) -> Option<InputEvent> {
        None
    }

    fn update_table(
        &mut self,
        uid: UID,
        table: Option<&InputTable>,
    ) -> Result<(), InputServerError> {
        Ok(())
    }
}

#[derive(Default)]
pub struct DummyInputServer;

impl InputServer for DummyInputServer {}
