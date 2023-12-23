use std::{cell::RefCell, rc::Rc};

use mini3d_core::input::{
    provider::{InputProvider, InputProviderError, InputProviderHandle},
    resource::{InputAction, InputActionHandle, InputAxis, InputAxisHandle},
};
use mini3d_input::mapper::{InputMapper, InputMapperAxis, InputMapperButton};
use serde::{Deserialize, Serialize};

use crate::mouse::{MouseAxis, MouseButton};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) enum Button {
    Keyboard(u32),
    Mouse(MouseButton),
    Controller(gilrs::GamepadId, gilrs::Button),
}

impl InputMapperButton for Button {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub(crate) enum Axis {
    Mouse(MouseAxis),
    Controller(gilrs::GamepadId, gilrs::Axis),
}

impl InputMapperAxis for Axis {}

pub(crate) struct Win32InputProvider(Rc<RefCell<InputMapper<Button, Axis>>>);

impl Default for Win32InputProvider {
    fn default() -> Self {
        let mut mapper = InputMapper::new();
        let profile = mapper.default_profile();
        // mapper.bind_button_to_action(
        //     profile,
        //     CommonAction::UP.to_uid(),
        //     Some(Button::Keyboard(native_windows_gui::keys::_Z)),
        // );
        // TODO: add common mapping
        // mapper.profiles.insert(
        //     mapper.default_profile,
        //     InputProfile {
        //         name: "Default".to_string(),
        //         active: true,
        //         actions: vec![
        //             MapActionInput {
        //                 name: CommonAction::UP.to_string(),
        //                 handle: None,
        //                 button: Some(Button::Keyboard {
        //                     code: VirtualKeyCode::Z,
        //                 }),
        //             },
        //             MapActionInput {
        //                 name: CommonAction::LEFT.to_string(),
        //                 handle: None,
        //                 button: Some(Button::Keyboard {
        //                     code: VirtualKeyCode::Q,
        //                 }),
        //             },
        //             MapActionInput {
        //                 name: CommonAction::DOWN.to_string(),
        //                 handle: None,
        //                 button: Some(Button::Keyboard {
        //                     code: VirtualKeyCode::S,
        //                 }),
        //             },
        //             MapActionInput {
        //                 name: CommonAction::RIGHT.to_string(),
        //                 handle: None,
        //                 button: Some(Button::Keyboard {
        //                     code: VirtualKeyCode::D,
        //                 }),
        //             },
        //             MapActionInput {
        //                 name: CommonAction::CHANGE_CONTROL_MODE.to_string(),
        //                 handle: None,
        //                 button: Some(Button::Keyboard {
        //                     code: VirtualKeyCode::F,
        //                 }),
        //             },
        //             MapActionInput {
        //                 name: "switch_mode".to_string(),
        //                 handle: None,
        //                 button: Some(Button::Keyboard {
        //                     code: VirtualKeyCode::C,
        //                 }),
        //             },
        //             MapActionInput {
        //                 name: "roll_left".to_string(),
        //                 handle: None,
        //                 button: Some(Button::Keyboard {
        //                     code: VirtualKeyCode::A,
        //                 }),
        //             },
        //             MapActionInput {
        //                 name: "roll_right".to_string(),
        //                 handle: None,
        //                 button: Some(Button::Keyboard {
        //                     code: VirtualKeyCode::E,
        //                 }),
        //             },
        //         ],
        //         axis: vec![
        //             MapAxisInput {
        //                 name: CommonAxis::CURSOR_X.to_string(),
        //                 axis: Some((Axis::MousePositionX, 0.0)),
        //                 ..Default::default()
        //             },
        //             MapAxisInput {
        //                 name: CommonAxis::CURSOR_Y.to_string(),
        //                 axis: Some((Axis::MousePositionY, 0.0)),
        //                 ..Default::default()
        //             },
        //             MapAxisInput {
        //                 name: CommonAxis::VIEW_X.to_string(),
        //                 axis: Some((Axis::MouseMotionX, 0.01)),
        //                 ..Default::default()
        //             },
        //             MapAxisInput {
        //                 name: CommonAxis::VIEW_Y.to_string(),
        //                 axis: Some((Axis::MouseMotionY, 0.01)),
        //                 ..Default::default()
        //             },
        //             MapAxisInput {
        //                 name: CommonAxis::MOVE_FORWARD.to_string(),
        //                 button: Some((
        //                     Button::Keyboard {
        //                         code: VirtualKeyCode::Z,
        //                     },
        //                     1.0,
        //                 )),
        //                 ..Default::default()
        //             },
        //             MapAxisInput {
        //                 name: CommonAxis::MOVE_BACKWARD.to_string(),
        //                 button: Some((
        //                     Button::Keyboard {
        //                         code: VirtualKeyCode::S,
        //                     },
        //                     1.0,
        //                 )),
        //                 ..Default::default()
        //             },
        //             MapAxisInput {
        //                 name: CommonAxis::MOVE_LEFT.to_string(),
        //                 button: Some((
        //                     Button::Keyboard {
        //                         code: VirtualKeyCode::Q,
        //                     },
        //                     1.0,
        //                 )),
        //                 ..Default::default()
        //             },
        //             MapAxisInput {
        //                 name: CommonAxis::MOVE_RIGHT.to_string(),
        //                 button: Some((
        //                     Button::Keyboard {
        //                         code: VirtualKeyCode::D,
        //                     },
        //                     1.0,
        //                 )),
        //                 ..Default::default()
        //             },
        //             MapAxisInput {
        //                 name: CommonAxis::MOVE_UP.to_string(),
        //                 button: Some((
        //                     Button::Keyboard {
        //                         code: VirtualKeyCode::X,
        //                     },
        //                     1.0,
        //                 )),
        //                 ..Default::default()
        //             },
        //             MapAxisInput {
        //                 name: CommonAxis::MOVE_DOWN.to_string(),
        //                 button: Some((
        //                     Button::Keyboard {
        //                         code: VirtualKeyCode::W,
        //                     },
        //                     1.0,
        //                 )),
        //                 ..Default::default()
        //             },
        //         ],
        //     },
        // );
        // mapper.load().ok();
        Self(Rc::new(RefCell::new(mapper)))
    }
}

impl InputProvider for Win32InputProvider {
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
