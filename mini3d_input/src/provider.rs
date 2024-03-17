use alloc::boxed::Box;
use mini3d_derive::Error;
use mini3d_utils::define_provider_handle;

use crate::{
    action::InputActionHandle,
    axis::{InputAxisHandle, InputAxisRange},
    event::InputEvent,
};

#[derive(Debug, Error)]
pub enum InputProviderError {
    #[error("Unknown error")]
    Unknown,
}

define_provider_handle!(InputProviderHandle);

#[allow(unused_variables)]
pub trait InputProvider {
    fn on_connect(&mut self) {}
    fn on_disconnect(&mut self) {}

    fn next_event(&mut self) -> Option<InputEvent> {
        None
    }

    fn create_action(
        &mut self,
        name: &str,
        handle: InputActionHandle,
    ) -> Result<InputProviderHandle, InputProviderError> {
        Ok(Default::default())
    }
    fn create_axis(
        &mut self,
        name: &str,
        range: &InputAxisRange,
        handle: InputAxisHandle,
    ) -> Result<InputProviderHandle, InputProviderError> {
        Ok(Default::default())
    }
    fn delete_action(&mut self, handle: InputProviderHandle) -> Result<(), InputProviderError> {
        Ok(())
    }
    fn delete_axis(&mut self, handle: InputProviderHandle) -> Result<(), InputProviderError> {
        Ok(())
    }
}

#[derive(Default)]
pub struct PassiveInputProvider;

impl InputProvider for PassiveInputProvider {}

impl Default for Box<dyn InputProvider> {
    fn default() -> Self {
        Box::<PassiveInputProvider>::default()
    }
}
