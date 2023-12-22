use alloc::boxed::Box;
use mini3d_derive::Error;

use crate::define_provider_handle;

use super::{
    event::InputEvent,
    resource::{action::InputAction, axis::InputAxis, InputActionHandle, InputAxisHandle},
};

#[derive(Debug, Error)]
pub enum InputProviderError {
    #[error("Unknown error")]
    Unknown,
}

define_provider_handle!(InputProviderHandle);

#[allow(unused_variables)]
pub trait InputProvider {
    fn on_connect(&mut self);
    fn on_disconnect(&mut self);

    fn next_event(&mut self) -> Option<InputEvent>;

    fn add_action(
        &mut self,
        name: &str,
        action: &InputAction,
        handle: InputActionHandle,
    ) -> Result<InputProviderHandle, InputProviderError>;
    fn add_axis(
        &mut self,
        name: &str,
        axis: &InputAxis,
        handle: InputAxisHandle,
    ) -> Result<InputProviderHandle, InputProviderError>;
    fn remove_action(&mut self, handle: InputProviderHandle) -> Result<(), InputProviderError>;
    fn remove_axis(&mut self, handle: InputProviderHandle) -> Result<(), InputProviderError>;
}

#[derive(Default)]
pub struct PassiveInputProvider;

impl InputProvider for PassiveInputProvider {
    fn on_connect(&mut self) {}
    fn on_disconnect(&mut self) {}

    fn next_event(&mut self) -> Option<InputEvent> {
        None
    }

    fn add_action(
        &mut self,
        _name: &str,
        _action: &InputAction,
        _handle: InputActionHandle,
    ) -> Result<InputProviderHandle, InputProviderError> {
        Ok(Default::default())
    }
    fn add_axis(
        &mut self,
        _name: &str,
        _axis: &InputAxis,
        _handle: InputAxisHandle,
    ) -> Result<InputProviderHandle, InputProviderError> {
        Ok(Default::default())
    }
    fn remove_action(&mut self, _handle: InputProviderHandle) -> Result<(), InputProviderError> {
        Ok(())
    }
    fn remove_axis(&mut self, _handle: InputProviderHandle) -> Result<(), InputProviderError> {
        Ok(())
    }
}

impl Default for Box<dyn InputProvider> {
    fn default() -> Self {
        Box::<PassiveInputProvider>::default()
    }
}
