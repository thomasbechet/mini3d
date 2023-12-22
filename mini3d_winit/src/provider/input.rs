use std::{cell::RefCell, rc::Rc};

use mini3d_core::input::{
    provider::{InputProvider, InputProviderError, InputProviderHandle},
    resource::{InputAction, InputActionHandle, InputAxis, InputAxisHandle},
};
use mini3d_input::mapper::{InputMapper, MapperTypes};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct WinitInputMapperTypes;

impl MapperTypes for WinitInputMapperTypes {
    type MouseButton = winit::event::MouseButton;
    type MouseAxis = winit::event::AxisId;
    type KeyboardKeyCode = winit::keyboard::KeyCode;
    type ControllerId = gilrs::GamepadId;
    type ControllerAxis = gilrs::Axis;
    type ControllerButton = gilrs::Button;
}

pub(crate) struct WinitInputProvider(Rc<RefCell<InputMapper<WinitInputMapperTypes>>>);

impl WinitInputProvider {
    pub(crate) fn new(mapper: Rc<RefCell<InputMapper<WinitInputMapperTypes>>>) -> Self {
        Self(mapper)
    }
}

impl InputProvider for WinitInputProvider {
    fn on_connect(&mut self) {}
    fn on_disconnect(&mut self) {}

    fn next_event(&mut self) -> Option<mini3d_core::input::event::InputEvent> {
        self.0.borrow_mut().next_event()
    }

    fn add_action(
        &mut self,
        name: &str,
        action: &InputAction,
        handle: InputActionHandle,
    ) -> Result<InputProviderHandle, InputProviderError> {
        self.0.borrow_mut().add_action(name, action, handle)
    }
    fn add_axis(
        &mut self,
        name: &str,
        axis: &InputAxis,
        handle: InputAxisHandle,
    ) -> Result<InputProviderHandle, InputProviderError> {
        self.0.borrow_mut().add_axis(name, axis, handle)
    }
    fn remove_action(&mut self, handle: InputProviderHandle) -> Result<(), InputProviderError> {
        self.0.borrow_mut().remove_action(handle)
    }
    fn remove_axis(&mut self, handle: InputProviderHandle) -> Result<(), InputProviderError> {
        self.0.borrow_mut().remove_axis(handle)
    }
}
