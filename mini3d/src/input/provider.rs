use mini3d_derive::Error;

use crate::feature::input::{action::InputAction, axis::InputAxis};

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

    fn add_action(&mut self, id: u32, action: &InputAction);
    fn add_axis(&mut self, id: u32, axis: &InputAxis);
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
