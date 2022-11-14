use std::{collections::HashMap, fs::File};

use gilrs::GamepadId;
use mini3d::{app::App, event::{AppEvents, input::{InputEvent, InputActionEvent, InputAxisEvent}}, slotmap::{new_key_type, SlotMap, Key}, anyhow::{Result, Context, anyhow}, asset::{input_action::InputAction, input_axis::InputAxis}, uid::UID};
use mini3d_os::input::{CommonAction, CommonAxis};
use serde::{Serialize, Deserialize};
use winit::event::{VirtualKeyCode, MouseButton, ElementState};

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
    Controller { id: GamepadId, axis: gilrs::Axis }
}

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum Button {
    Keyboard { code: VirtualKeyCode },
    Mouse { button: MouseButton },
    Controller { id: GamepadId, button: gilrs::Button }
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

new_key_type! { pub(crate) struct InputProfileId; }

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct InputProfile {
    pub(crate) name: String,
    pub(crate) active: bool,
    pub(crate) actions: HashMap<UID, MapActionInput>,
    pub(crate) axis: HashMap<UID, MapAxisInput>,
}

#[derive(Default)]
pub(crate) struct InputMapper {
    
    pub(crate) profiles: SlotMap<InputProfileId, InputProfile>,
    pub(crate) default_profile: InputProfileId,

    key_to_action: HashMap<VirtualKeyCode, Vec<KeyToAction>>,
    key_to_axis: HashMap<VirtualKeyCode, Vec<KeyToAxis>>,
    mouse_button_to_action: HashMap<MouseButton, Vec<MouseButtonToAction>>,
    mouse_button_to_axis: HashMap<MouseButton, Vec<MouseButtonToAxis>>,
    mouse_motion_x_to_axis: Vec<MouseMotionToAxis>,
    mouse_motion_y_to_axis: Vec<MouseMotionToAxis>,
    mouse_position_x_to_axis: Vec<MousePositionToAxis>,
    mouse_position_y_to_axis: Vec<MousePositionToAxis>,
    controllers_button_to_action: HashMap<gilrs::GamepadId, HashMap<gilrs::Button, Vec<ControllerButtonToAction>>>,
    controllers_button_to_axis: HashMap<gilrs::GamepadId, HashMap<gilrs::Button, Vec<ControllerButtonToAxis>>>,
    controllers_axis_to_axis: HashMap<gilrs::GamepadId, HashMap<gilrs::Axis, Vec<ControllerAxisToAxis>>>,
}

impl InputMapper {

    pub(crate) fn new() -> Self {
        let mut mapper: InputMapper = Default::default();
        // Default inputs
        mapper.default_profile = mapper.profiles.insert(InputProfile { 
            name: "Default".to_string(), 
            active: true,
            actions: HashMap::from([
                (CommonAction::UP.into(), MapActionInput { button: Some(Button::Keyboard { code: VirtualKeyCode::Z }) }),
                (CommonAction::LEFT.into(), MapActionInput { button: Some(Button::Keyboard { code: VirtualKeyCode::Q }) }),
                (CommonAction::DOWN.into(), MapActionInput { button: Some(Button::Keyboard { code: VirtualKeyCode::S }) }),
                (CommonAction::RIGHT.into(), MapActionInput { button: Some(Button::Keyboard { code: VirtualKeyCode::D }) }),
                (CommonAction::CHANGE_CONTROL_MODE.into(), MapActionInput { button: Some(Button::Keyboard { code: VirtualKeyCode::F }) }),
                ("switch_mode".into(), MapActionInput { button: Some(Button::Keyboard { code: VirtualKeyCode::C }) }),
                ("roll_left".into(), MapActionInput { button: Some(Button::Keyboard { code: VirtualKeyCode::A }) }),
                ("roll_right".into(), MapActionInput { button: Some(Button::Keyboard { code: VirtualKeyCode::E }) }),
            ]),
            axis: HashMap::from([
                (CommonAxis::CURSOR_X.into(), MapAxisInput { axis: Some((Axis::MousePositionX, 0.0)), ..Default::default() }),
                (CommonAxis::CURSOR_Y.into(), MapAxisInput { axis: Some((Axis::MousePositionY, 0.0)), ..Default::default() }),
                (CommonAxis::VIEW_X.into(), MapAxisInput { axis: Some((Axis::MouseMotionX, 0.01)), ..Default::default() }),
                (CommonAxis::VIEW_Y.into(), MapAxisInput { axis: Some((Axis::MouseMotionY, 0.01)), ..Default::default() }),
                (CommonAxis::MOVE_FORWARD.into(), MapAxisInput { button: Some((Button::Keyboard { code: VirtualKeyCode::Z }, 1.0)), ..Default::default() }),
                (CommonAxis::MOVE_BACKWARD.into(), MapAxisInput { button: Some((Button::Keyboard { code: VirtualKeyCode::S }, 1.0)), ..Default::default() }),
                (CommonAxis::MOVE_LEFT.into(), MapAxisInput { button: Some((Button::Keyboard { code: VirtualKeyCode::Q }, 1.0)), ..Default::default() }),
                (CommonAxis::MOVE_RIGHT.into(), MapAxisInput { button: Some((Button::Keyboard { code: VirtualKeyCode::D }, 1.0)), ..Default::default() }),
                (CommonAxis::MOVE_UP.into(), MapAxisInput { button: Some((Button::Keyboard { code: VirtualKeyCode::X }, 1.0)), ..Default::default() }),
                (CommonAxis::MOVE_DOWN.into(), MapAxisInput { button: Some((Button::Keyboard { code: VirtualKeyCode::W }, 1.0)), ..Default::default() }),
            ]),
        });

        mapper.load().ok();

        mapper
    }

    pub(crate) fn new_profile(&mut self, app: &App) -> InputProfileId {
        let mut next_index = self.profiles.len() + 1;
        let mut name = format!("Profile {}", next_index);
        while self.profiles.iter().any(|(_, p)| p.name == name) {
            next_index += 1;
            name = format!("Profile {}", next_index); 
        }
        let id = self.profiles.insert(InputProfile { name, active: true, actions: Default::default(), axis: Default::default() });
        self.refresh(app);
        id
    }

    pub(crate) fn duplicate(&mut self, from: InputProfileId, app: &App) -> InputProfileId {
        if let Some(from) = self.profiles.get(from) {
            let mut name = format!("{} Copy", from.name);
            let mut next_index = 1;
            while self.profiles.iter().any(|(_, p)| p.name == name) {
                next_index += 1;
                name = format!("{} Copy {}", from.name, next_index);
            }
            let profile = InputProfile { name, active: true, actions: from.actions.clone(), axis: from.axis.clone() };
            let id = self.profiles.insert(profile);
            self.refresh(app);
            id
        } else {
            InputProfileId::null()
        }
    }

    pub(crate) fn save(&self) -> Result<()> {
        std::fs::create_dir_all("config").unwrap();
        let file = File::create("config/profiles.json")
            .with_context(|| "Failed to open file.")?;
        let profiles = self.profiles.values().collect::<Vec<&_>>();
        serde_json::to_writer_pretty(&file, &profiles)
            .with_context(|| "Failed to write file.")?;
        Ok(())
    }

    pub(crate) fn load(&mut self) -> Result<()> {
        if let Ok(file) = File::open("config/profiles.json") {
            let mut profiles: Vec<InputProfile> = serde_json::from_reader(&file).unwrap();
            for profile in profiles.drain(..) {
                if let Some((_, current)) = self.profiles.iter_mut().find(|(_, p)| p.name == profile.name) {
                    *current = profile;
                } else {
                    self.profiles.insert(profile);
                }
            }
            Ok(())
        } else {
            Err(anyhow!("Failed to open file."))
        }
    }

    pub(crate) fn refresh(&mut self, app: &App) {

        // Update profiles
        for (_, profile) in &mut self.profiles {

            // Update actions
            app.asset().iter::<InputAction>().expect("InputAction asset not found").for_each(|(uid, _)| {
                profile.actions.entry(*uid).or_insert_with(Default::default);
            });
            
            // Update axis
            app.asset().iter::<InputAxis>().expect("InputAxis asset not found").for_each(|(uid, _)| {
                profile.axis.entry(*uid).or_insert_with(Default::default);
            });
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
        self.controllers_button_to_action.clear();
        self.controllers_button_to_axis.clear();
        self.controllers_axis_to_axis.clear();

        // Update caches
        for (_, profile) in &self.profiles {
            if profile.active {
                for (uid, action) in &profile.actions {
                    if let Some(button) = &action.button {
                        match button {
                            Button::Keyboard { code } => {
                                self.key_to_action.entry(*code).or_insert_with(Default::default).push(KeyToAction { uid: *uid });
                            },
                            Button::Mouse { button } => {
                                self.mouse_button_to_action.entry(*button).or_insert_with(Default::default).push(MouseButtonToAction { uid: *uid });
                            },
                            Button::Controller { id, button } => {
                                self.controllers_button_to_action.entry(*id).or_insert_with(Default::default).entry(*button).or_insert_with(Default::default)
                                    .push(ControllerButtonToAction { uid: *uid });
                            },
                        }
                    }
                }
                for (uid, axis) in &profile.axis {
                    if let Some((b, value)) = &axis.button {
                        match b {
                            Button::Keyboard { code } => {
                                self.key_to_axis.entry(*code).or_insert_with(Default::default).push(KeyToAxis { uid: *uid, value: *value });
                            },
                            Button::Mouse { button } => {
                                self.mouse_button_to_axis.entry(*button).or_insert_with(Default::default).push(MouseButtonToAxis { uid: *uid, value: *value });
                            },
                            Button::Controller { id, button } => {
                                self.controllers_button_to_axis.entry(*id).or_insert_with(Default::default).entry(*button).or_insert_with(Default::default)
                                    .push(ControllerButtonToAxis { uid: *uid, value: *value });
                            },
                        }
                    }
                    if let Some((a, scale)) = &axis.axis {
                        match a {
                            Axis::MousePositionX => {
                                self.mouse_position_x_to_axis.push(MousePositionToAxis { uid: *uid });
                            },
                            Axis::MousePositionY => {
                                self.mouse_position_y_to_axis.push(MousePositionToAxis { uid: *uid });
                            },
                            Axis::MouseMotionX => {
                                self.mouse_motion_x_to_axis.push(MouseMotionToAxis { uid: *uid, scale: *scale });
                            },
                            Axis::MouseMotionY => {
                                self.mouse_motion_y_to_axis.push(MouseMotionToAxis { uid: *uid, scale: *scale });
                            },
                            Axis::Controller { id, axis: ax } => {
                                self.controllers_axis_to_axis.entry(*id).or_insert_with(Default::default).entry(*ax).or_insert_with(Default::default)
                                    .push(ControllerAxisToAxis { uid: *uid, scale: *scale })
                            }
                        }
                    }
                }
            }
        }   
    }

    pub(crate) fn dispatch_keyboard(&self, keycode: VirtualKeyCode, state: ElementState, events: &mut AppEvents) {
        if let Some(actions) = self.key_to_action.get(&keycode) {
            let pressed = state == ElementState::Pressed;
            for action in actions {
                events.input.push(InputEvent::Action(InputActionEvent { action: action.uid, pressed }));        
            }
        }
        if let Some(axis) = self.key_to_axis.get(&keycode) {
            for ax in axis {
                let value = match state {
                    ElementState::Pressed => ax.value,
                    ElementState::Released => 0.0,
                };
                events.input.push(InputEvent::Axis(InputAxisEvent { axis: ax.uid, value }));
            }
        }
    }

    pub(crate) fn dispatch_mouse_button(&self, button: MouseButton, state: ElementState, events: &mut AppEvents) {
        let pressed = state == ElementState::Pressed;
        if let Some(actions) = self.mouse_button_to_action.get(&button) {
            for action in actions {
                events.input.push(InputEvent::Action(InputActionEvent { action: action.uid, pressed }));
            }
        }
        if let Some(axis) = self.mouse_button_to_axis.get(&button) {
            for ax in axis {
                events.input.push(InputEvent::Axis(InputAxisEvent { axis: ax.uid, value: ax.value }));
            }
        }
    }

    pub(crate) fn dispatch_mouse_motion(&self, delta: (f64, f64), events: &mut AppEvents) {
        for axis in &self.mouse_motion_x_to_axis {
            events.input.push(InputEvent::Axis(InputAxisEvent { axis: axis.uid, value: delta.0 as f32 * axis.scale }));
        }
        for axis in &self.mouse_motion_y_to_axis {
            events.input.push(InputEvent::Axis(InputAxisEvent { axis: axis.uid, value: delta.1 as f32 * axis.scale }));
        }
    }

    pub(crate) fn dispatch_mouse_cursor(&self, cursor: (f32, f32), events: &mut AppEvents) {
        for axis in &self.mouse_position_x_to_axis {
            events.input.push(InputEvent::Axis(InputAxisEvent { axis: axis.uid, value: cursor.0 }));
        }
        for axis in &self.mouse_position_y_to_axis {
            events.input.push(InputEvent::Axis(InputAxisEvent { axis: axis.uid, value: cursor.1 }));
        }
    }

    pub(crate) fn dispatch_controller_button(&self, id: GamepadId, button: gilrs::Button, pressed: bool, events: &mut AppEvents) {
        if let Some(buttons) = &self.controllers_button_to_action.get(&id) {
            if let Some(actions) = buttons.get(&button) {
                for action in actions {
                    events.input.push(InputEvent::Action(InputActionEvent { action: action.uid, pressed }));
                }
            }
        }
        if let Some(axis) = &self.controllers_button_to_axis.get(&id) {
            if let Some(a) = axis.get(&button) {
                for ax in a {
                    let value = if pressed { ax.value } else { 0.0 };
                    events.input.push(InputEvent::Axis(InputAxisEvent { axis: ax.uid, value }));
                }
            }
        }
    }

    pub(crate) fn dispatch_controller_axis(&self, id: GamepadId, controller_axis: gilrs::Axis, value: f32, events: &mut AppEvents) {

        // Compute value with deadzone
        let value = if f32::abs(value) <= 0.15 { 0.0 } else { value };

        if let Some(axis) = &self.controllers_axis_to_axis.get(&id) {
            if let Some(ax) = axis.get(&controller_axis) {
                for a in ax {
                    events.input.push(InputEvent::Axis(InputAxisEvent { axis: a.uid, value: value * a.scale }));
                }
            }
        }
    }
}