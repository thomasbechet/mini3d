use mini3d_derive::Error;

use crate::{feature::component::input::input_table::InputTable, utils::uid::UID};

use super::event::InputEvent;

#[derive(Debug, Error)]
pub enum InputProviderError {
    #[error("Unknown error")]
    Unknown,
}

#[allow(unused_variables)]
pub trait InputProvider {
    fn on_connect(&mut self);
    fn on_disconnect(&mut self);

    fn next_event(&mut self) -> Option<InputEvent>;

    fn update_table(
        &mut self,
        uid: UID,
        table: Option<&InputTable>,
    ) -> Result<(), InputProviderError>;
}

#[derive(Default)]
pub struct PassiveInputProvider;

impl InputProvider for PassiveInputProvider {
    fn on_connect(&mut self) {}
    fn on_disconnect(&mut self) {}

    fn next_event(&mut self) -> Option<InputEvent> {
        None
    }

    fn update_table(
        &mut self,
        uid: UID,
        table: Option<&InputTable>,
    ) -> Result<(), InputProviderError> {
        Ok(())
    }
}

impl Default for Box<dyn InputProvider> {
    fn default() -> Self {
        Box::<PassiveInputProvider>::default()
    }
}
