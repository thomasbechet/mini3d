use std::fmt::Debug;
use std::hash::Hash;
use std::{collections::HashMap, fs::File};

use mini3d_core::input::provider::{InputProviderError, InputProviderHandle};
use mini3d_core::input::resource::{
    self, InputActionHandle, InputAxis, InputAxisHandle, InputAxisRange,
};
use mini3d_core::math::fixed::I32F16;
use mini3d_core::utils::uid::ToUID;
use mini3d_core::{
    input::{
        event::{InputActionEvent, InputAxisEvent, InputEvent, InputTextEvent},
        provider::InputProvider,
    },
    utils::uid::UID,
};
use serde::{Deserialize, Deserializer, Serialize};

pub trait MapperTypes: Clone + Serialize + for<'de> Deserialize<'de> {
    type MouseButton: Sized
        + Copy
        + Clone
        + Hash
        + Eq
        + Debug
        + Serialize
        + for<'de> Deserialize<'de>;
    type MouseAxis: Sized + Copy + Clone + Hash + Eq + Debug + Serialize + for<'de> Deserialize<'de>;
    type KeyboardKeyCode: Sized
        + Copy
        + Clone
        + Hash
        + Eq
        + Debug
        + Serialize
        + for<'de> Deserialize<'de>;
    type ControllerId: Sized
        + Copy
        + Clone
        + Hash
        + Eq
        + Debug
        + Serialize
        + for<'de> Deserialize<'de>;
    type ControllerAxis: Sized
        + Copy
        + Clone
        + Hash
        + Eq
        + Debug
        + Serialize
        + for<'de> Deserialize<'de>;
    type ControllerButton: Sized
        + Copy
        + Clone
        + Hash
        + Eq
        + Debug
        + Serialize
        + for<'de> Deserialize<'de>;
}

struct KeyToAction {
    handle: InputActionHandle,
    was_pressed: bool,
}
struct KeyToAxis {
    handle: InputAxisHandle,
    value: f32,
}
struct MouseButtonToAction {
    handle: InputActionHandle,
    was_pressed: bool,
}
struct MouseButtonToAxis {
    handle: InputAxisHandle,
    value: f32,
}
struct MouseMotionToAxis {
    handle: InputAxisHandle,
    scale: f32,
}
struct MousePositionToAxis {
    handle: InputAxisHandle,
}
struct MouseWheelToAxis {
    handle: InputAxisHandle,
    scale: f32,
}
struct ControllerButtonToAction {
    handle: InputActionHandle,
    was_pressed: bool,
}
struct ControllerButtonToAxis {
    handle: InputAxisHandle,
    value: f32,
}
struct ControllerAxisToAxis {
    handle: InputAxisHandle,
    scale: f32,
}

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Axis<T: MapperTypes> {
    MousePositionX,
    MousePositionY,
    MouseMotionX,
    MouseMotionY,
    MouseWheelX,
    MouseWheelY,
    Controller {
        id: T::ControllerId,
        axis: T::ControllerAxis,
    },
}

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Button<T: MapperTypes> {
    Keyboard {
        code: T::KeyboardKeyCode,
    },
    Mouse {
        button: T::MouseButton,
    },
    Controller {
        id: T::ControllerId,
        button: T::ControllerButton,
    },
}

#[derive(Default, Clone, Serialize)]
pub struct MapActionInput<T: MapperTypes> {
    name: String,
    #[serde(skip)]
    handle: Option<InputActionHandle>,
    button: Option<Button<T>>,
}

impl<'de, T: MapperTypes> serde::Deserialize<'de> for MapActionInput<T> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let (name, button) = <(String, Option<Button<T>>)>::deserialize(d)?;
        Ok(Self {
            name,
            handle: None,
            button,
        })
    }
}

#[derive(Default, Clone, Serialize)]
pub struct MapAxisInput<T: MapperTypes> {
    pub name: String,
    #[serde(skip)]
    pub handle: Option<(InputAxisHandle, InputAxisRange)>,
    pub button: Option<(Button<T>, f32)>,
    pub axis: Option<(Axis<T>, f32)>,
}

impl<'de, T: MapperTypes> serde::Deserialize<'de> for MapAxisInput<T> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let (name, button, axis) =
            <(String, Option<(Button<T>, f32)>, Option<(Axis<T>, f32)>)>::deserialize(d)?;
        Ok(Self {
            name,
            handle: None,
            button,
            axis,
        })
    }
}

#[derive(Default, Clone, Serialize)]
pub struct InputProfile<T: MapperTypes> {
    pub name: String,
    pub active: bool,
    pub actions: Vec<MapActionInput<T>>,
    pub axis: Vec<MapAxisInput<T>>,
}

impl<'de, T: MapperTypes> serde::Deserialize<'de> for InputProfile<T> {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let (name, active, actions, axis) =
            <(String, bool, Vec<MapActionInput<T>>, Vec<MapAxisInput<T>>)>::deserialize(d)?;
        Ok(Self {
            name,
            active,
            actions,
            axis,
        })
    }
}

#[derive(Default)]
pub struct InputMapper<T: MapperTypes> {
    profiles: HashMap<UID, InputProfile<T>>,
    default_profile: UID,

    events: Vec<InputEvent>,

    key_to_action: HashMap<T::KeyboardKeyCode, Vec<KeyToAction>>,
    key_to_axis: HashMap<T::KeyboardKeyCode, Vec<KeyToAxis>>,
    mouse_button_to_action: HashMap<T::MouseButton, Vec<MouseButtonToAction>>,
    mouse_button_to_axis: HashMap<T::MouseButton, Vec<MouseButtonToAxis>>,
    mouse_motion_x_to_axis: Vec<MouseMotionToAxis>,
    mouse_motion_y_to_axis: Vec<MouseMotionToAxis>,
    mouse_position_x_to_axis: Vec<MousePositionToAxis>,
    mouse_position_y_to_axis: Vec<MousePositionToAxis>,
    mouse_wheel_x_to_axis: Vec<MouseWheelToAxis>,
    mouse_wheel_y_to_axis: Vec<MouseWheelToAxis>,
    controllers_button_to_action:
        HashMap<T::ControllerId, HashMap<T::ControllerButton, Vec<ControllerButtonToAction>>>,
    controllers_button_to_axis:
        HashMap<T::ControllerId, HashMap<T::ControllerButton, Vec<ControllerButtonToAxis>>>,
    controllers_axis_to_axis:
        HashMap<T::ControllerId, HashMap<T::ControllerAxis, Vec<ControllerAxisToAxis>>>,
}

impl<T: MapperTypes> InputMapper<T> {
    pub fn new() -> Self {
        let mut mapper = InputMapper {
            default_profile: UID::from("Default"),
            profiles: Default::default(),
            events: Default::default(),
            key_to_action: Default::default(),
            key_to_axis: Default::default(),
            mouse_button_to_action: Default::default(),
            mouse_button_to_axis: Default::default(),
            mouse_motion_x_to_axis: Default::default(),
            mouse_motion_y_to_axis: Default::default(),
            mouse_position_x_to_axis: Default::default(),
            mouse_position_y_to_axis: Default::default(),
            mouse_wheel_x_to_axis: Default::default(),
            mouse_wheel_y_to_axis: Default::default(),
            controllers_button_to_action: Default::default(),
            controllers_button_to_axis: Default::default(),
            controllers_axis_to_axis: Default::default(),
        };
        // Default inputs
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

        mapper
    }

    pub fn new_profile(&mut self) -> UID {
        let mut next_index = self.profiles.len() + 1;
        let mut name = format!("Profile {}", next_index);
        let uid = UID::from(&name);
        while self.profiles.iter().any(|(_, p)| p.name == name) {
            next_index += 1;
            name = format!("Profile {}", next_index);
        }
        self.profiles.insert(
            uid,
            InputProfile {
                name,
                active: true,
                actions: Default::default(),
                axis: Default::default(),
            },
        );
        self.rebuild_cache();
        uid
    }

    pub fn remove_profile(&mut self, profile: UID) {
        self.profiles.remove(&profile);
        self.rebuild_cache();
    }

    pub fn default_profile(&self) -> UID {
        self.default_profile
    }

    pub fn iter_profiles(&self) -> impl Iterator<Item = (UID, &InputProfile<T>)> {
        self.profiles.iter().map(|(k, v)| (*k, v))
    }

    pub fn iter_actions(&self, profile: UID) -> impl Iterator<Item = &MapActionInput<T>> {
        self.profiles
            .get(&profile)
            .map(|p| p.actions.iter())
            .unwrap_or_default()
    }

    pub fn iter_axis(&self, profile: UID) -> impl Iterator<Item = &MapAxisInput<T>> {
        self.profiles
            .get(&profile)
            .map(|p| p.axis.iter())
            .unwrap_or_default()
    }

    pub fn bind_button_to_action(
        &mut self,
        profile: UID,
        entry: UID,
        button: Option<Button<T>>,
    ) -> bool {
        if let Some(profile) = self.profiles.get_mut(&profile) {
            if let Some(action) = profile
                .actions
                .iter_mut()
                .find(|a| a.name.to_uid() == entry)
            {
                action.button = button;
                self.rebuild_cache();
                return true;
            }
        }
        false
    }

    pub fn bind_axis_to_axis(
        &mut self,
        profile: UID,
        entry: UID,
        value: Option<(Axis<T>, f32)>,
    ) -> bool {
        if let Some(profile) = self.profiles.get_mut(&profile) {
            if let Some(ax) = profile.axis.iter_mut().find(|a| a.name.to_uid() == entry) {
                ax.axis = value;
                self.rebuild_cache();
                return true;
            }
        }
        false
    }

    pub fn bind_button_to_axis(
        &mut self,
        profile: UID,
        entry: UID,
        value: Option<(Button<T>, f32)>,
    ) -> bool {
        if let Some(profile) = self.profiles.get_mut(&profile) {
            if let Some(axis) = profile.axis.iter_mut().find(|a| a.name.to_uid() == entry) {
                axis.button = value;
                self.rebuild_cache();
                return true;
            }
        }
        false
    }

    pub fn duplicate_profile(&mut self, from: UID) -> UID {
        if let Some(from) = self.profiles.get(&from) {
            let mut name = format!("{} Copy", from.name);
            let mut next_index = 1;
            while self.profiles.iter().any(|(_, p)| p.name == name) {
                next_index += 1;
                name = format!("{} Copy {}", from.name, next_index);
            }
            let uid = UID::from(&name);
            let profile = InputProfile {
                name,
                active: true,
                actions: from.actions.clone(),
                axis: from.axis.clone(),
            };
            self.profiles.insert(uid, profile);
            self.rebuild_cache();
            uid
        } else {
            UID::null()
        }
    }

    pub fn set_default_profile(&mut self, profile: UID) {
        self.default_profile = profile;
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        std::fs::create_dir_all("config").unwrap();
        let file = File::create("config/profiles.json")?;
        let profiles = self.profiles.values().collect::<Vec<&_>>();
        serde_json::to_writer_pretty(&file, &profiles)?;
        Ok(())
    }

    pub fn load(&mut self) -> Result<(), std::io::Error> {
        let file = File::open("config/profiles.json")?;
        let mut profiles: Vec<InputProfile<T>> = serde_json::from_reader(&file).unwrap();
        for profile in profiles.drain(..) {
            if let Some((_, current)) = self
                .profiles
                .iter_mut()
                .find(|(_, p)| p.name == profile.name)
            {
                *current = profile;
            } else {
                let uid = UID::from(&profile.name);
                self.profiles.insert(uid, profile);
            }
        }
        Ok(())
    }

    pub fn rebuild_cache(&mut self) {
        // Clear caches
        self.key_to_action.clear();
        self.key_to_axis.clear();
        self.mouse_button_to_action.clear();
        self.mouse_button_to_axis.clear();
        self.mouse_position_x_to_axis.clear();
        self.mouse_position_y_to_axis.clear();
        self.mouse_motion_x_to_axis.clear();
        self.mouse_motion_y_to_axis.clear();
        self.mouse_wheel_x_to_axis.clear();
        self.mouse_wheel_y_to_axis.clear();
        self.controllers_button_to_action.clear();
        self.controllers_button_to_axis.clear();
        self.controllers_axis_to_axis.clear();

        // Update caches
        for profile in self.profiles.values() {
            if profile.active {
                for action in &profile.actions {
                    if let Some(button) = &action.button {
                        if let Some(action_handle) = action.handle {
                            match button {
                                Button::Keyboard { code } => {
                                    self.key_to_action.entry(*code).or_default().push(
                                        KeyToAction {
                                            handle: action_handle,
                                            was_pressed: false,
                                        },
                                    );
                                }
                                Button::Mouse { button } => {
                                    self.mouse_button_to_action
                                        .entry(*button)
                                        .or_default()
                                        .push(MouseButtonToAction {
                                            handle: action_handle,
                                            was_pressed: false,
                                        });
                                }
                                Button::Controller { id, button } => {
                                    self.controllers_button_to_action
                                        .entry(*id)
                                        .or_default()
                                        .entry(*button)
                                        .or_default()
                                        .push(ControllerButtonToAction {
                                            handle: action_handle,
                                            was_pressed: false,
                                        });
                                }
                            }
                        }
                    }
                }
                for axis in &profile.axis {
                    if let Some((b, value)) = &axis.button {
                        if let Some((axis_handle, _)) = axis.handle {
                            match b {
                                Button::Keyboard { code } => {
                                    self.key_to_axis.entry(*code).or_default().push(KeyToAxis {
                                        handle: axis_handle,
                                        value: *value,
                                    });
                                }
                                Button::Mouse { button } => {
                                    self.mouse_button_to_axis.entry(*button).or_default().push(
                                        MouseButtonToAxis {
                                            handle: axis_handle,
                                            value: *value,
                                        },
                                    );
                                }
                                Button::Controller { id, button } => {
                                    self.controllers_button_to_axis
                                        .entry(*id)
                                        .or_default()
                                        .entry(*button)
                                        .or_default()
                                        .push(ControllerButtonToAxis {
                                            handle: axis_handle,
                                            value: *value,
                                        });
                                }
                            }
                        }
                    }
                    if let Some((a, scale)) = &axis.axis {
                        if let Some((axis_handle, _)) = axis.handle {
                            match a {
                                Axis::MousePositionX => {
                                    self.mouse_position_x_to_axis.push(MousePositionToAxis {
                                        handle: axis_handle,
                                    });
                                }
                                Axis::MousePositionY => {
                                    self.mouse_position_y_to_axis.push(MousePositionToAxis {
                                        handle: axis_handle,
                                    });
                                }
                                Axis::MouseMotionX => {
                                    self.mouse_motion_x_to_axis.push(MouseMotionToAxis {
                                        handle: axis_handle,
                                        scale: *scale,
                                    });
                                }
                                Axis::MouseMotionY => {
                                    self.mouse_motion_y_to_axis.push(MouseMotionToAxis {
                                        handle: axis_handle,
                                        scale: *scale,
                                    });
                                }
                                Axis::MouseWheelX => {
                                    self.mouse_wheel_x_to_axis.push(MouseWheelToAxis {
                                        handle: axis_handle,
                                        scale: *scale,
                                    });
                                }
                                Axis::MouseWheelY => {
                                    self.mouse_wheel_y_to_axis.push(MouseWheelToAxis {
                                        handle: axis_handle,
                                        scale: *scale,
                                    });
                                }
                                Axis::Controller { id, axis: ax } => self
                                    .controllers_axis_to_axis
                                    .entry(*id)
                                    .or_default()
                                    .entry(*ax)
                                    .or_default()
                                    .push(ControllerAxisToAxis {
                                        handle: axis_handle,
                                        scale: *scale,
                                    }),
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn dispatch_text(&mut self, value: String) {
        self.events.push(InputEvent::Text(InputTextEvent {
            handle: Default::default(),
            value,
        }));
    }

    pub fn dispatch_keyboard(&mut self, keycode: T::KeyboardKeyCode, pressed: bool) {
        if let Some(actions) = self.key_to_action.get_mut(&keycode) {
            for action in actions {
                // Prevent repeating events
                if action.was_pressed != pressed {
                    self.events.push(InputEvent::Action(InputActionEvent {
                        handle: action.handle,
                        pressed,
                    }));
                    action.was_pressed = pressed;
                }
            }
        }
        if let Some(axis) = self.key_to_axis.get(&keycode) {
            for ax in axis {
                let value = if pressed { ax.value } else { 0.0 };
                self.events.push(InputEvent::Axis(InputAxisEvent {
                    handle: ax.handle,
                    value: I32F16::from_f32(value),
                }));
            }
        }
    }

    pub fn dispatch_mouse_button(&mut self, button: T::MouseButton, pressed: bool) {
        if let Some(actions) = self.mouse_button_to_action.get_mut(&button) {
            for action in actions {
                // Prevent repeating events
                if action.was_pressed != pressed {
                    self.events.push(InputEvent::Action(InputActionEvent {
                        handle: action.handle,
                        pressed,
                    }));
                    action.was_pressed = pressed;
                }
            }
        }
        if let Some(axis) = self.mouse_button_to_axis.get(&button) {
            for ax in axis {
                self.events.push(InputEvent::Axis(InputAxisEvent {
                    handle: ax.handle,
                    value: I32F16::from_f32(ax.value),
                }));
            }
        }
    }

    pub fn dispatch_mouse_motion(&mut self, delta: (f64, f64)) {
        for axis in &self.mouse_motion_x_to_axis {
            self.events.push(InputEvent::Axis(InputAxisEvent {
                handle: axis.handle,
                value: I32F16::from_f32(delta.0 as f32 * axis.scale),
            }));
        }
        for axis in &self.mouse_motion_y_to_axis {
            self.events.push(InputEvent::Axis(InputAxisEvent {
                handle: axis.handle,
                value: I32F16::from_f32(delta.1 as f32 * axis.scale),
            }));
        }
    }

    pub fn dispatch_mouse_cursor(&mut self, cursor: (f32, f32)) {
        for axis in &self.mouse_position_x_to_axis {
            self.events.push(InputEvent::Axis(InputAxisEvent {
                handle: axis.handle,
                value: I32F16::from_f32(cursor.0),
            }));
        }
        for axis in &self.mouse_position_y_to_axis {
            self.events.push(InputEvent::Axis(InputAxisEvent {
                handle: axis.handle,
                value: I32F16::from_f32(cursor.1),
            }));
        }
    }

    pub fn dispatch_mouse_wheel(&mut self, delta: (f32, f32)) {
        for axis in &self.mouse_wheel_x_to_axis {
            self.events.push(InputEvent::Axis(InputAxisEvent {
                handle: axis.handle,
                value: I32F16::from_f32(delta.0 * axis.scale),
            }));
        }
        for axis in &self.mouse_wheel_y_to_axis {
            self.events.push(InputEvent::Axis(InputAxisEvent {
                handle: axis.handle,
                value: I32F16::from_f32(delta.1 * axis.scale),
            }));
        }
    }

    pub fn dispatch_controller_button(
        &mut self,
        id: T::ControllerId,
        button: T::ControllerButton,
        pressed: bool,
    ) {
        if let Some(buttons) = self.controllers_button_to_action.get_mut(&id) {
            if let Some(actions) = buttons.get_mut(&button) {
                for action in actions {
                    if action.was_pressed != pressed {
                        self.events.push(InputEvent::Action(InputActionEvent {
                            handle: action.handle,
                            pressed,
                        }));
                        action.was_pressed = pressed;
                    }
                }
            }
        }
        if let Some(axis) = &self.controllers_button_to_axis.get(&id) {
            if let Some(a) = axis.get(&button) {
                for ax in a {
                    let value = if pressed { ax.value } else { 0.0 };
                    self.events.push(InputEvent::Axis(InputAxisEvent {
                        handle: ax.handle,
                        value: I32F16::from_f32(value),
                    }));
                }
            }
        }
    }

    pub fn dispatch_controller_axis(
        &mut self,
        id: T::ControllerId,
        controller_axis: T::ControllerAxis,
        value: f32,
    ) {
        // Compute value with deadzone
        let value = if f32::abs(value) <= 0.15 { 0.0 } else { value };

        if let Some(axis) = &self.controllers_axis_to_axis.get(&id) {
            if let Some(ax) = axis.get(&controller_axis) {
                for a in ax {
                    self.events.push(InputEvent::Axis(InputAxisEvent {
                        handle: a.handle,
                        value: I32F16::from_f32(value * a.scale),
                    }));
                }
            }
        }
    }
}

impl<T: MapperTypes> InputProvider for InputMapper<T> {
    fn on_connect(&mut self) {}
    fn on_disconnect(&mut self) {}

    fn next_event(&mut self) -> Option<InputEvent> {
        self.events.pop()
    }

    fn add_action(
        &mut self,
        name: &str,
        action: &resource::InputAction,
        handle: InputActionHandle,
    ) -> Result<InputProviderHandle, InputProviderError> {
        for profile in self.profiles.values_mut() {
            if let Some(a) = profile.actions.iter_mut().find(|a| a.name == name) {
                a.handle = Some(handle);
            }
            profile.actions.push(MapActionInput {
                name: name.to_string(),
                handle: Some(handle),
                button: None,
            });
        }
        self.rebuild_cache();
        Ok(Default::default())
    }

    fn remove_action(&mut self, handle: InputProviderHandle) -> Result<(), InputProviderError> {
        Ok(())
    }

    fn add_axis(
        &mut self,
        name: &str,
        axis: &InputAxis,
        handle: InputAxisHandle,
    ) -> Result<InputProviderHandle, InputProviderError> {
        for profile in self.profiles.values_mut() {
            if let Some(a) = profile.axis.iter_mut().find(|a| a.name == name) {
                a.handle = Some((handle, axis.range));
            }
            profile.axis.push(MapAxisInput {
                name: name.to_string(),
                handle: Some((handle, axis.range)),
                button: None,
                axis: None,
            });
        }
        self.rebuild_cache();
        Ok(Default::default())
    }

    fn remove_axis(&mut self, handle: InputProviderHandle) -> Result<(), InputProviderError> {
        Ok(())
    }
}
