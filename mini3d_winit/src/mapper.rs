use std::{collections::HashMap, fs::File};

use gilrs::GamepadId;
use mini3d::{
    feature::component::input::input_table::InputTable,
    input::server::{InputServer, InputServerError},
    utils::uid::UID,
};
use mini3d_os::input::{CommonAction, CommonAxis};
use serde::{Deserialize, Serialize};
use winit::event::{ElementState, MouseButton, VirtualKeyCode};

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub(crate) struct InputUID(UID);

impl InputUID {
    fn new(name: &str) -> Self {
        Self(UID::new(name))
    }
}

impl From<UID> for InputUID {
    fn from(uid: UID) -> Self {
        Self(uid)
    }
}

impl From<&InputUID> for UID {
    fn from(uid: &InputUID) -> Self {
        uid.0
    }
}

impl Serialize for InputUID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u64(self.0.into())
    }
}

impl<'de> Deserialize<'de> for InputUID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        u64::deserialize(deserializer).map(|uid| Self(UID::from(uid)))
    }
}

struct KeyToAction {
    uid: UID,
}
struct KeyToAxis {
    uid: UID,
    value: f32,
}
struct MouseButtonToAction {
    uid: UID,
}
struct MouseButtonToAxis {
    uid: UID,
    value: f32,
}
struct MouseMotionToAxis {
    uid: UID,
    scale: f32,
}
struct MousePositionToAxis {
    uid: UID,
}
struct MouseWheelToAxis {
    uid: UID,
    scale: f32,
}
struct ControllerButtonToAction {
    uid: UID,
}
struct ControllerButtonToAxis {
    uid: UID,
    value: f32,
}
struct ControllerAxisToAxis {
    uid: UID,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) button: Option<Button>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub(crate) struct MapAxisInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) button: Option<(Button, f32)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) axis: Option<(Axis, f32)>,
}

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct InputProfile {
    pub(crate) name: String,
    pub(crate) active: bool,
    pub(crate) actions: HashMap<InputUID, MapActionInput>,
    pub(crate) axis: HashMap<InputUID, MapAxisInput>,
}

#[derive(Default)]
pub(crate) struct InputMapper {
    pub(crate) profiles: HashMap<UID, InputProfile>,
    pub(crate) default_profile: UID,
    pub(crate) tables: HashMap<UID, InputTable>,

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
                actions: HashMap::from([
                    (
                        InputUID::new(CommonAction::UP),
                        MapActionInput {
                            button: Some(Button::Keyboard {
                                code: VirtualKeyCode::Z,
                            }),
                        },
                    ),
                    (
                        InputUID::new(CommonAction::LEFT),
                        MapActionInput {
                            button: Some(Button::Keyboard {
                                code: VirtualKeyCode::Q,
                            }),
                        },
                    ),
                    (
                        InputUID::new(CommonAction::DOWN),
                        MapActionInput {
                            button: Some(Button::Keyboard {
                                code: VirtualKeyCode::S,
                            }),
                        },
                    ),
                    (
                        InputUID::new(CommonAction::RIGHT),
                        MapActionInput {
                            button: Some(Button::Keyboard {
                                code: VirtualKeyCode::D,
                            }),
                        },
                    ),
                    (
                        InputUID::new(CommonAction::CHANGE_CONTROL_MODE),
                        MapActionInput {
                            button: Some(Button::Keyboard {
                                code: VirtualKeyCode::F,
                            }),
                        },
                    ),
                    (
                        InputUID::new("switch_mode"),
                        MapActionInput {
                            button: Some(Button::Keyboard {
                                code: VirtualKeyCode::C,
                            }),
                        },
                    ),
                    (
                        InputUID::new("roll_left"),
                        MapActionInput {
                            button: Some(Button::Keyboard {
                                code: VirtualKeyCode::A,
                            }),
                        },
                    ),
                    (
                        InputUID::new("roll_right"),
                        MapActionInput {
                            button: Some(Button::Keyboard {
                                code: VirtualKeyCode::E,
                            }),
                        },
                    ),
                ]),
                axis: HashMap::from([
                    (
                        InputUID::new(CommonAxis::CURSOR_X),
                        MapAxisInput {
                            axis: Some((Axis::MousePositionX, 0.0)),
                            ..Default::default()
                        },
                    ),
                    (
                        InputUID::new(CommonAxis::CURSOR_Y),
                        MapAxisInput {
                            axis: Some((Axis::MousePositionY, 0.0)),
                            ..Default::default()
                        },
                    ),
                    (
                        InputUID::new(CommonAxis::VIEW_X),
                        MapAxisInput {
                            axis: Some((Axis::MouseMotionX, 0.01)),
                            ..Default::default()
                        },
                    ),
                    (
                        InputUID::new(CommonAxis::VIEW_Y),
                        MapAxisInput {
                            axis: Some((Axis::MouseMotionY, 0.01)),
                            ..Default::default()
                        },
                    ),
                    (
                        InputUID::new(CommonAxis::MOVE_FORWARD),
                        MapAxisInput {
                            button: Some((
                                Button::Keyboard {
                                    code: VirtualKeyCode::Z,
                                },
                                1.0,
                            )),
                            ..Default::default()
                        },
                    ),
                    (
                        InputUID::new(CommonAxis::MOVE_BACKWARD),
                        MapAxisInput {
                            button: Some((
                                Button::Keyboard {
                                    code: VirtualKeyCode::S,
                                },
                                1.0,
                            )),
                            ..Default::default()
                        },
                    ),
                    (
                        InputUID::new(CommonAxis::MOVE_LEFT),
                        MapAxisInput {
                            button: Some((
                                Button::Keyboard {
                                    code: VirtualKeyCode::Q,
                                },
                                1.0,
                            )),
                            ..Default::default()
                        },
                    ),
                    (
                        InputUID::new(CommonAxis::MOVE_RIGHT),
                        MapAxisInput {
                            button: Some((
                                Button::Keyboard {
                                    code: VirtualKeyCode::D,
                                },
                                1.0,
                            )),
                            ..Default::default()
                        },
                    ),
                    (
                        InputUID::new(CommonAxis::MOVE_UP),
                        MapAxisInput {
                            button: Some((
                                Button::Keyboard {
                                    code: VirtualKeyCode::X,
                                },
                                1.0,
                            )),
                            ..Default::default()
                        },
                    ),
                    (
                        InputUID::new(CommonAxis::MOVE_DOWN),
                        MapAxisInput {
                            button: Some((
                                Button::Keyboard {
                                    code: VirtualKeyCode::W,
                                },
                                1.0,
                            )),
                            ..Default::default()
                        },
                    ),
                ]),
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
        self.refresh();
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
            self.refresh();
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

    pub(crate) fn refresh(&mut self) {
        // Update profiles
        for profile in self.profiles.values_mut() {
            for table in self.tables.values() {
                for action in table.actions.iter() {
                    profile
                        .actions
                        .entry(action.uid().into())
                        .or_insert_with(Default::default);
                }
                for axis in table.axis.iter() {
                    profile
                        .axis
                        .entry(axis.uid().into())
                        .or_insert_with(Default::default);
                }
            }
        }

        // Rebuild cache
        self.rebuild_cache();
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
                for (uid, action) in &profile.actions {
                    if let Some(button) = &action.button {
                        match button {
                            Button::Keyboard { code } => {
                                self.key_to_action
                                    .entry(*code)
                                    .or_insert_with(Default::default)
                                    .push(KeyToAction { uid: uid.into() });
                            }
                            Button::Mouse { button } => {
                                self.mouse_button_to_action
                                    .entry(*button)
                                    .or_insert_with(Default::default)
                                    .push(MouseButtonToAction { uid: uid.into() });
                            }
                            Button::Controller { id, button } => {
                                self.controllers_button_to_action
                                    .entry(*id)
                                    .or_insert_with(Default::default)
                                    .entry(*button)
                                    .or_insert_with(Default::default)
                                    .push(ControllerButtonToAction { uid: uid.into() });
                            }
                        }
                    }
                }
                for (uid, axis) in &profile.axis {
                    if let Some((b, value)) = &axis.button {
                        match b {
                            Button::Keyboard { code } => {
                                self.key_to_axis
                                    .entry(*code)
                                    .or_insert_with(Default::default)
                                    .push(KeyToAxis {
                                        uid: uid.into(),
                                        value: *value,
                                    });
                            }
                            Button::Mouse { button } => {
                                self.mouse_button_to_axis
                                    .entry(*button)
                                    .or_insert_with(Default::default)
                                    .push(MouseButtonToAxis {
                                        uid: uid.into(),
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
                                        uid: uid.into(),
                                        value: *value,
                                    });
                            }
                        }
                    }
                    if let Some((a, scale)) = &axis.axis {
                        match a {
                            Axis::MousePositionX => {
                                self.mouse_position_x_to_axis
                                    .push(MousePositionToAxis { uid: uid.into() });
                            }
                            Axis::MousePositionY => {
                                self.mouse_position_y_to_axis
                                    .push(MousePositionToAxis { uid: uid.into() });
                            }
                            Axis::MouseMotionX => {
                                self.mouse_motion_x_to_axis.push(MouseMotionToAxis {
                                    uid: uid.into(),
                                    scale: *scale,
                                });
                            }
                            Axis::MouseMotionY => {
                                self.mouse_motion_y_to_axis.push(MouseMotionToAxis {
                                    uid: uid.into(),
                                    scale: *scale,
                                });
                            }
                            Axis::MouseWheelX => {
                                self.mouse_wheel_x_to_axis.push(MouseWheelToAxis {
                                    uid: uid.into(),
                                    scale: *scale,
                                });
                            }
                            Axis::MouseWheelY => {
                                self.mouse_wheel_y_to_axis.push(MouseWheelToAxis {
                                    uid: uid.into(),
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
                                    uid: uid.into(),
                                    scale: *scale,
                                }),
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn dispatch_keyboard(
        &self,
        keycode: VirtualKeyCode,
        state: ElementState,
        events: &mut Events,
    ) {
        if let Some(actions) = self.key_to_action.get(&keycode) {
            let pressed = state == ElementState::Pressed;
            for action in actions {
                events.input.push(InputEvent::Action(InputActionEvent {
                    action: action.uid,
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
                events.input.push(InputEvent::Axis(InputAxisEvent {
                    axis: ax.uid,
                    value,
                }));
            }
        }
    }

    pub(crate) fn dispatch_mouse_button(
        &self,
        button: MouseButton,
        state: ElementState,
        events: &mut Events,
    ) {
        let pressed = state == ElementState::Pressed;
        if let Some(actions) = self.mouse_button_to_action.get(&button) {
            for action in actions {
                events.input.push(InputEvent::Action(InputActionEvent {
                    action: action.uid,
                    pressed,
                }));
            }
        }
        if let Some(axis) = self.mouse_button_to_axis.get(&button) {
            for ax in axis {
                events.input.push(InputEvent::Axis(InputAxisEvent {
                    axis: ax.uid,
                    value: ax.value,
                }));
            }
        }
    }

    pub(crate) fn dispatch_mouse_motion(&self, delta: (f64, f64), events: &mut Events) {
        for axis in &self.mouse_motion_x_to_axis {
            events.input.push(InputEvent::Axis(InputAxisEvent {
                axis: axis.uid,
                value: delta.0 as f32 * axis.scale,
            }));
        }
        for axis in &self.mouse_motion_y_to_axis {
            events.input.push(InputEvent::Axis(InputAxisEvent {
                axis: axis.uid,
                value: delta.1 as f32 * axis.scale,
            }));
        }
    }

    pub(crate) fn dispatch_mouse_cursor(&self, cursor: (f32, f32), events: &mut Events) {
        for axis in &self.mouse_position_x_to_axis {
            events.input.push(InputEvent::Axis(InputAxisEvent {
                axis: axis.uid,
                value: cursor.0,
            }));
        }
        for axis in &self.mouse_position_y_to_axis {
            events.input.push(InputEvent::Axis(InputAxisEvent {
                axis: axis.uid,
                value: cursor.1,
            }));
        }
    }

    pub(crate) fn dispatch_mouse_wheel(&self, delta: (f32, f32), events: &mut Events) {
        for axis in &self.mouse_wheel_x_to_axis {
            events.input.push(InputEvent::Axis(InputAxisEvent {
                axis: axis.uid,
                value: delta.0 * axis.scale,
            }));
        }
        for axis in &self.mouse_wheel_y_to_axis {
            events.input.push(InputEvent::Axis(InputAxisEvent {
                axis: axis.uid,
                value: delta.1 * axis.scale,
            }));
        }
    }

    pub(crate) fn dispatch_controller_button(
        &self,
        id: GamepadId,
        button: gilrs::Button,
        pressed: bool,
        events: &mut Events,
    ) {
        if let Some(buttons) = &self.controllers_button_to_action.get(&id) {
            if let Some(actions) = buttons.get(&button) {
                for action in actions {
                    events.input.push(InputEvent::Action(InputActionEvent {
                        action: action.uid,
                        pressed,
                    }));
                }
            }
        }
        if let Some(axis) = &self.controllers_button_to_axis.get(&id) {
            if let Some(a) = axis.get(&button) {
                for ax in a {
                    let value = if pressed { ax.value } else { 0.0 };
                    events.input.push(InputEvent::Axis(InputAxisEvent {
                        axis: ax.uid,
                        value,
                    }));
                }
            }
        }
    }

    pub(crate) fn dispatch_controller_axis(
        &self,
        id: GamepadId,
        controller_axis: gilrs::Axis,
        value: f32,
        events: &mut Events,
    ) {
        // Compute value with deadzone
        let value = if f32::abs(value) <= 0.15 { 0.0 } else { value };

        if let Some(axis) = &self.controllers_axis_to_axis.get(&id) {
            if let Some(ax) = axis.get(&controller_axis) {
                for a in ax {
                    events.input.push(InputEvent::Axis(InputAxisEvent {
                        axis: a.uid,
                        value: value * a.scale,
                    }));
                }
            }
        }
    }
}

impl InputServer for InputMapper {
    fn update_table(
        &mut self,
        uid: UID,
        table: Option<&InputTable>,
    ) -> Result<(), InputServerError> {
        if let Some(table) = table {
            self.tables.insert(uid, table.clone());
        } else {
            self.tables.remove(&uid);
        }
        self.refresh();
        Ok(())
    }
}
