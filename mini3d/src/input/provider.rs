use mini3d_derive::Error;

use crate::define_provider_handle;

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
}

#[derive(Default)]
pub struct PassiveInputProvider;

impl InputProvider for PassiveInputProvider {
    fn on_connect(&mut self) {}
    fn on_disconnect(&mut self) {}

    fn next_event(&mut self) -> Option<InputEvent> {
        None
    }

    fn add_action(&mut self, _id: u32, _action: &InputAction) {}
    fn add_axis(&mut self, _id: u32, _axis: &InputAxis) {}
}

impl Default for Box<dyn InputProvider> {
    fn default() -> Self {
        Box::<PassiveInputProvider>::default()
    }
}
