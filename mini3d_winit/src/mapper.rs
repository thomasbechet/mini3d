use std::{collections::HashMap, fs::File};

use gilrs::GamepadId;
use mini3d_core::{
    feature::input::{
        action::InputAction,
        axis::{InputAxis, InputAxisRange},
    },
    input::{
        event::{InputActionEvent, InputAxisEvent, InputEvent, InputTextEvent},
        provider::InputProvider,
    },
    utils::uid::UID,
};
use mini3d_os::input::{CommonAction, CommonAxis};
use serde::{Deserialize, Serialize};
use winit::event::{ElementState, MouseButton, VirtualKeyCode};

struct KeyToAction {
    id: u32,
}
struct KeyToAxis {
    id: u32,
    value: f32,
}
struct MouseButtonToAction {
    id: u32,
}
struct MouseButtonToAxis {
    id: u32,
    value: f32,
}
struct MouseMotionToAxis {
    id: u32,
    scale: f32,
}
struct MousePositionToAxis {
    id: u32,
}
struct MouseWheelToAxis {
    id: u32,
    scale: f32,
}
struct ControllerButtonToAction {
    id: u32,
}
struct ControllerButtonToAxis {
    id: u32,
    value: f32,
}
struct ControllerAxisToAxis {
    id: u32,
    scale: f32,
}

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum Axis {
    MousePositionX,
    MousePositionY,
    MouseMotionX,
    MouseMotionY,
    MouseWheelX,
    MouseWheelY,
    Controller { id: GamepadId, axis: gilrs::Axis },
}

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum Button {
    Keyboard {
        code: VirtualKeyCode,
    },
    Mouse {
        button: MouseButton,
    },
    Controller {
        id: GamepadId,
        button: gilrs::Button,
    },
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub(crate) struct MapActionInput {
    pub(crate) name: String,
    #[serde(skip)]
    id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) button: Option<Button>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub(crate) struct MapAxisInput {
    pub(crate) name: String,
    #[serde(skip)]
    pub(crate) range: InputAxisRange,
    #[serde(skip)]
    id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) button: Option<(Button, f32)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) axis: Option<(Axis, f32)>,
}

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct InputProfile {
    pub(crate) name: String,
    pub(crate) active: bool,
    pub(crate) actions: Vec<MapActionInput>,
    pub(crate) axis: Vec<MapAxisInput>,
}

#[derive(Default)]
pub(crate) struct InputMapper {
    pub(crate) profiles: HashMap<UID, InputProfile>,
    pub(crate) default_profile: UID,

    events: Vec<InputEvent>,

    key_to_action: HashMap<VirtualKeyCode, Vec<KeyToAction>>,
    key_to_axis: HashMap<VirtualKeyCode, Vec<KeyToAxis>>,
    mouse_button_to_action: HashMap<MouseButton, Vec<MouseButtonToAction>>,
    mouse_button_to_axis: HashMap<MouseButton, Vec<MouseButtonToAxis>>,
    mouse_motion_x_to_axis: Vec<MouseMotionToAxis>,
    mouse_motion_y_to_axis: Vec<MouseMotionToAxis>,
    mouse_position_x_to_axis: Vec<MousePositionToAxis>,
    mouse_position_y_to_axis: Vec<MousePositionToAxis>,
    mouse_wheel_x_to_axis: Vec<MouseWheelToAxis>,
    mouse_wheel_y_to_axis: Vec<MouseWheelToAxis>,
    controllers_button_to_action:
        HashMap<gilrs::GamepadId, HashMap<gilrs::Button, Vec<ControllerButtonToAction>>>,
    controllers_button_to_axis:
        HashMap<gilrs::GamepadId, HashMap<gilrs::Button, Vec<ControllerButtonToAxis>>>,
    controllers_axis_to_axis:
        HashMap<gilrs::GamepadId, HashMap<gilrs::Axis, Vec<ControllerAxisToAxis>>>,
}

impl InputMapper {
    pub(crate) fn new() -> Self {
        let mut mapper: InputMapper = InputMapper {
            default_profile: UID::from("Default"),
            ..Default::default()
        };
        // Default inputs
        mapper.profiles.insert(
            mapper.default_profile,
            InputProfile {
                name: "Default".to_string(),
                active: true,
                actions: vec![
                    MapActionInput {
                        name: CommonAction::UP.to_string(),
                        id: None,
                        button: Some(Button::Keyboard {
                            code: VirtualKeyCode::Z,
                        }),
                    },
                    MapActionInput {
                        name: CommonAction::LEFT.to_string(),
                        id: None,
                        button: Some(Button::Keyboard {
                            code: VirtualKeyCode::Q,
                        }),
                    },
                    MapActionInput {
                        name: CommonAction::DOWN.to_string(),
                        id: None,
                        button: Some(Button::Keyboard {
                            code: VirtualKeyCode::S,
                        }),
                    },
                    MapActionInput {
                        name: CommonAction::RIGHT.to_string(),
                        id: None,
                        button: Some(Button::Keyboard {
                            code: VirtualKeyCode::D,
                        }),
                    },
                    MapActionInput {
                        name: CommonAction::CHANGE_CONTROL_MODE.to_string(),
                        id: None,
                        button: Some(Button::Keyboard {
                            code: VirtualKeyCode::F,
                        }),
                    },
                    MapActionInput {
                        name: "switch_mode".to_string(),
                        id: None,
                        button: Some(Button::Keyboard {
                            code: VirtualKeyCode::C,
                        }),
                    },
                    MapActionInput {
                        name: "roll_left".to_string(),
                        id: None,
                        button: Some(Button::Keyboard {
                            code: VirtualKeyCode::A,
                        }),
                    },
                    MapActionInput {
                        name: "roll_right".to_string(),
                        id: None,
                        button: Some(Button::Keyboard {
                            code: VirtualKeyCode::E,
                        }),
                    },
                ],
                axis: vec![
                    MapAxisInput {
                        name: CommonAxis::CURSOR_X.to_string(),
                        axis: Some((Axis::MousePositionX, 0.0)),
                        ..Default::default()
                    },
                    MapAxisInput {
                        name: CommonAxis::CURSOR_Y.to_string(),
                        axis: Some((Axis::MousePositionY, 0.0)),
                        ..Default::default()
                    },
                    MapAxisInput {
                        name: CommonAxis::VIEW_X.to_string(),
                        axis: Some((Axis::MouseMotionX, 0.01)),
                        ..Default::default()
                    },
                    MapAxisInput {
                        name: CommonAxis::VIEW_Y.to_string(),
                        axis: Some((Axis::MouseMotionY, 0.01)),
                        ..Default::default()
                    },
                    MapAxisInput {
                        name: CommonAxis::MOVE_FORWARD.to_string(),
                        button: Some((
                            Button::Keyboard {
                                code: VirtualKeyCode::Z,
                            },
                            1.0,
                        )),
                        ..Default::default()
                    },
                    MapAxisInput {
                        name: CommonAxis::MOVE_BACKWARD.to_string(),
                        button: Some((
                            Button::Keyboard {
                                code: VirtualKeyCode::S,
                            },
                            1.0,
                        )),
                        ..Default::default()
                    },
                    MapAxisInput {
                        name: CommonAxis::MOVE_LEFT.to_string(),
                        button: Some((
                            Button::Keyboard {
                                code: VirtualKeyCode::Q,
                            },
                            1.0,
                        )),
                        ..Default::default()
                    },
                    MapAxisInput {
                        name: CommonAxis::MOVE_RIGHT.to_string(),
                        button: Some((
                            Button::Keyboard {
                                code: VirtualKeyCode::D,
                            },
                            1.0,
                        )),
                        ..Default::default()
                    },
                    MapAxisInput {
                        name: CommonAxis::MOVE_UP.to_string(),
                        button: Some((
                            Button::Keyboard {
                                code: VirtualKeyCode::X,
                            },
                            1.0,
                        )),
                        ..Default::default()
                    },
                    MapAxisInput {
                        name: CommonAxis::MOVE_DOWN.to_string(),
                        button: Some((
                            Button::Keyboard {
                                code: VirtualKeyCode::W,
                            },
                            1.0,
                        )),
                        ..Default::default()
                    },
                ],
            },
        );

        mapper.load().ok();

        mapper
    }

    pub(crate) fn new_profile(&mut self) -> UID {
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

    pub(crate) fn duplicate(&mut self, from: UID) -> UID {
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

    pub(crate) fn save(&self) -> Result<(), std::io::Error> {
        std::fs::create_dir_all("config").unwrap();
        let file = File::create("config/profiles.json")?;
        let profiles = self.profiles.values().collect::<Vec<&_>>();
        serde_json::to_writer_pretty(&file, &profiles)?;
        Ok(())
    }

    pub(crate) fn load(&mut self) -> Result<(), std::io::Error> {
        let file = File::open("config/profiles.json")?;
        let mut profiles: Vec<InputProfile> = serde_json::from_reader(&file).unwrap();
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

    pub(crate) fn rebuild_cache(&mut self) {
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
                        if let Some(action_id) = action.id {
                            match button {
                                Button::Keyboard { code } => {
                                    self.key_to_action
                                        .entry(*code)
                                        .or_insert_with(Default::default)
                                        .push(KeyToAction { id: action_id });
                                }
                                Button::Mouse { button } => {
                                    self.mouse_button_to_action
                                        .entry(*button)
                                        .or_insert_with(Default::default)
                                        .push(MouseButtonToAction { id: action_id });
                                }
                                Button::Controller { id, button } => {
                                    self.controllers_button_to_action
                                        .entry(*id)
                                        .or_insert_with(Default::default)
                                        .entry(*button)
                                        .or_insert_with(Default::default)
                                        .push(ControllerButtonToAction { id: action_id });
                                }
                            }
                        }
                    }
                }
                for axis in &profile.axis {
                    if let Some((b, value)) = &axis.button {
                        if let Some(axis_id) = axis.id {
                            match b {
                                Button::Keyboard { code } => {
                                    self.key_to_axis
                                        .entry(*code)
                                        .or_insert_with(Default::default)
                                        .push(KeyToAxis {
                                            id: axis_id,
                                            value: *value,
                                        });
                                }
                                Button::Mouse { button } => {
                                    self.mouse_button_to_axis
                                        .entry(*button)
                                        .or_insert_with(Default::default)
                                        .push(MouseButtonToAxis {
                                            id: axis_id,
                                            value: *value,
                                        });
                                }
                                Button::Controller { id, button } => {
                                    self.controllers_button_to_axis
                                        .entry(*id)
                                        .or_insert_with(Default::default)
                                        .entry(*button)
                                        .or_insert_with(Default::default)
                                        .push(ControllerButtonToAxis {
                                            id: axis_id,
                                            value: *value,
                                        });
                                }
                            }
                        }
                    }
                    if let Some((a, scale)) = &axis.axis {
                        if let Some(axis_id) = axis.id {
                            match a {
                                Axis::MousePositionX => {
                                    self.mouse_position_x_to_axis
                                        .push(MousePositionToAxis { id: axis_id });
                                }
                                Axis::MousePositionY => {
                                    self.mouse_position_y_to_axis
                                        .push(MousePositionToAxis { id: axis_id });
                                }
                                Axis::MouseMotionX => {
                                    self.mouse_motion_x_to_axis.push(MouseMotionToAxis {
                                        id: axis_id,
                                        scale: *scale,
                                    });
                                }
                                Axis::MouseMotionY => {
                                    self.mouse_motion_y_to_axis.push(MouseMotionToAxis {
                                        id: axis_id,
                                        scale: *scale,
                                    });
                                }
                                Axis::MouseWheelX => {
                                    self.mouse_wheel_x_to_axis.push(MouseWheelToAxis {
                                        id: axis_id,
                                        scale: *scale,
                                    });
                                }
                                Axis::MouseWheelY => {
                                    self.mouse_wheel_y_to_axis.push(MouseWheelToAxis {
                                        id: axis_id,
                                        scale: *scale,
                                    });
                                }
                                Axis::Controller { id, axis: ax } => self
                                    .controllers_axis_to_axis
                                    .entry(*id)
                                    .or_insert_with(Default::default)
                                    .entry(*ax)
                                    .or_insert_with(Default::default)
                                    .push(ControllerAxisToAxis {
                                        id: axis_id,
                                        scale: *scale,
                                    }),
                            }
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn dispatch_text(&mut self, value: String) {
        self.events
            .push(InputEvent::Text(InputTextEvent { id: 0, value }));
    }

    pub(crate) fn dispatch_keyboard(&mut self, keycode: VirtualKeyCode, state: ElementState) {
        if let Some(actions) = self.key_to_action.get(&keycode) {
            let pressed = state == ElementState::Pressed;
            for action in actions {
                self.events.push(InputEvent::Action(InputActionEvent {
                    id: action.id,
                    pressed,
                }));
            }
        }
        if let Some(axis) = self.key_to_axis.get(&keycode) {
            for ax in axis {
                let value = match state {
                    ElementState::Pressed => ax.value,
                    ElementState::Released => 0.0,
                };
                self.events
                    .push(InputEvent::Axis(InputAxisEvent { id: ax.id, value }));
            }
        }
    }

    pub(crate) fn dispatch_mouse_button(&mut self, button: MouseButton, state: ElementState) {
        let pressed = state == ElementState::Pressed;
        if let Some(actions) = self.mouse_button_to_action.get(&button) {
            for action in actions {
                self.events.push(InputEvent::Action(InputActionEvent {
                    id: action.id,
                    pressed,
                }));
            }
        }
        if let Some(axis) = self.mouse_button_to_axis.get(&button) {
            for ax in axis {
                self.events.push(InputEvent::Axis(InputAxisEvent {
                    id: ax.id,
                    value: ax.value,
                }));
            }
        }
    }

    pub(crate) fn dispatch_mouse_motion(&mut self, delta: (f64, f64)) {
        for axis in &self.mouse_motion_x_to_axis {
            self.events.push(InputEvent::Axis(InputAxisEvent {
                id: axis.id,
                value: delta.0 as f32 * axis.scale,
            }));
        }
        for axis in &self.mouse_motion_y_to_axis {
            self.events.push(InputEvent::Axis(InputAxisEvent {
                id: axis.id,
                value: delta.1 as f32 * axis.scale,
            }));
        }
    }

    pub(crate) fn dispatch_mouse_cursor(&mut self, cursor: (f32, f32)) {
        for axis in &self.mouse_position_x_to_axis {
            self.events.push(InputEvent::Axis(InputAxisEvent {
                id: axis.id,
                value: cursor.0,
            }));
        }
        for axis in &self.mouse_position_y_to_axis {
            self.events.push(InputEvent::Axis(InputAxisEvent {
                id: axis.id,
                value: cursor.1,
            }));
        }
    }

    pub(crate) fn dispatch_mouse_wheel(&mut self, delta: (f32, f32)) {
        for axis in &self.mouse_wheel_x_to_axis {
            self.events.push(InputEvent::Axis(InputAxisEvent {
                id: axis.id,
                value: delta.0 * axis.scale,
            }));
        }
        for axis in &self.mouse_wheel_y_to_axis {
            self.events.push(InputEvent::Axis(InputAxisEvent {
                id: axis.id,
                value: delta.1 * axis.scale,
            }));
        }
    }

    pub(crate) fn dispatch_controller_button(
        &mut self,
        id: GamepadId,
        button: gilrs::Button,
        pressed: bool,
    ) {
        if let Some(buttons) = &self.controllers_button_to_action.get(&id) {
            if let Some(actions) = buttons.get(&button) {
                for action in actions {
                    self.events.push(InputEvent::Action(InputActionEvent {
                        id: action.id,
                        pressed,
                    }));
                }
            }
        }
        if let Some(axis) = &self.controllers_button_to_axis.get(&id) {
            if let Some(a) = axis.get(&button) {
                for ax in a {
                    let value = if pressed { ax.value } else { 0.0 };
                    self.events
                        .push(InputEvent::Axis(InputAxisEvent { id: ax.id, value }));
                }
            }
        }
    }

    pub(crate) fn dispatch_controller_axis(
        &mut self,
        id: GamepadId,
        controller_axis: gilrs::Axis,
        value: f32,
    ) {
        // Compute value with deadzone
        let value = if f32::abs(value) <= 0.15 { 0.0 } else { value };

        if let Some(axis) = &self.controllers_axis_to_axis.get(&id) {
            if let Some(ax) = axis.get(&controller_axis) {
                for a in ax {
                    self.events.push(InputEvent::Axis(InputAxisEvent {
                        id: a.id,
                        value: value * a.scale,
                    }));
                }
            }
        }
    }
}

impl InputProvider for InputMapper {
    fn on_connect(&mut self) {}
    fn on_disconnect(&mut self) {}

    fn next_event(&mut self) -> Option<InputEvent> {
        self.events.pop()
    }

    fn add_action(&mut self, id: u32, action: &InputAction) {
        for profile in self.profiles.values_mut() {
            profile
                .actions
                .iter_mut()
                .find(|a| a.name == action.name.to_string())
                .map(|a| a.id = Some(id));
            profile.actions.push(MapActionInput {
                name: action.name.to_string(),
                id: Some(id),
                button: None,
            });
        }
        self.rebuild_cache();
    }

    fn add_axis(&mut self, id: u32, axis: &InputAxis) {
        for profile in self.profiles.values_mut() {
            profile
                .axis
                .iter_mut()
                .find(|a| a.name == axis.name.to_string())
                .map(|a| a.id = Some(id));
            profile.axis.push(MapAxisInput {
                name: axis.name.to_string(),
                range: axis.range,
                id: Some(id),
                button: None,
                axis: None,
            });
        }
        self.rebuild_cache();
    }
}
