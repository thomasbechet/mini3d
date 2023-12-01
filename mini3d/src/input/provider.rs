use alloc::boxed::Box;
use mini3d_derive::Error;

use crate::{
    define_provider_handle,
    feature::input::{action::InputAction, axis::InputAxis},
};

use super::event::InputEvent;

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
        action: &InputAction,
        id: u32,
    ) -> Result<InputProviderHandle, InputProviderError>;
    fn add_axis(
        &mut self,
        axis: &InputAxis,
        id: u32,
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
        _action: &InputAction,
        _id: u32,
    ) -> Result<InputProviderHandle, InputProviderError> {
        Ok(Default::default())
    }
    fn add_axis(
        &mut self,
        _axis: &InputAxis,
        _id: u32,
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
