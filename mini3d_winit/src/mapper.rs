use std::{collections::HashMap, fs::File};

use gilrs::GamepadId;
use mini3d::{input::{InputGroupId, axis::AxisInputId, action::{ActionInputId, ActionState}, InputDatabase}, app::App, event::{AppEvents, input::{InputEvent, ActionEvent, AxisEvent}}, slotmap::{new_key_type, SlotMap, Key}, anyhow::{Result, Context, anyhow}};
use mini3d_os::input::{CommonAction, CommonAxis, CommonInput};
use serde::{Serialize, Deserialize};
use winit::event::{VirtualKeyCode, MouseButton, ElementState};

struct KeyToAction {
    id: ActionInputId,
}
struct KeyToAxis {
    id: AxisInputId,
    value: f32,
}
struct MouseButtonToAction {
    id: ActionInputId,
}
struct MouseButtonToAxis {
    id: AxisInputId,
    value: f32,
}
struct MouseMotionToAxis {
    id: AxisInputId,
    scale: f32,
}
struct MousePositionToAxis {
    id: AxisInputId,
}
struct ControllerButtonToAction {
    id: ActionInputId,
}
struct ControllerButtonToAxis {
    id: AxisInputId,
    value: f32,
}
struct ControllerAxisToAxis {
    id: AxisInputId,
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
    #[serde(skip)]
    pub(crate) id: ActionInputId,
    pub(crate) name: String,
    pub(crate) button: Option<Button>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub(crate) struct MapAxisInput {
    #[serde(skip)]
    pub(crate) id: AxisInputId,
    pub(crate) name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) button: Option<(Button, f32)>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) axis: Option<(Axis, f32)>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub(crate) struct MapGroupInput {
    #[serde(skip)]
    pub(crate) id: InputGroupId,
    pub(crate) name: String,
    pub(crate) actions: Vec<MapActionInput>,
    pub(crate) axis: Vec<MapAxisInput>,
}

new_key_type! { pub(crate) struct InputProfileId; }

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct InputProfile {
    pub(crate) name: String,
    pub(crate) active: bool,
    pub(crate) groups: Vec<MapGroupInput>,
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
            groups: vec![
                MapGroupInput {
                    name: CommonInput::GROUP.to_owned(),
                    actions: vec![
                        MapActionInput { name: CommonAction::UP.to_string(), button: Some(Button::Keyboard { code: VirtualKeyCode::Z }), ..Default::default() },
                        MapActionInput { name: CommonAction::LEFT.to_string(), button: Some(Button::Keyboard { code: VirtualKeyCode::Q }), ..Default::default() },
                        MapActionInput { name: CommonAction::DOWN.to_string(), button: Some(Button::Keyboard { code: VirtualKeyCode::S }), ..Default::default() },
                        MapActionInput { name: CommonAction::RIGHT.to_string(), button: Some(Button::Keyboard { code: VirtualKeyCode::D }), ..Default::default() },
                        MapActionInput { name: CommonAction::CHANGE_CONTROL_MODE.to_string(), button: Some(Button::Keyboard { code: VirtualKeyCode::F }), ..Default::default() },
                    ],
                    axis: vec![
                        MapAxisInput { name: CommonAxis::CURSOR_X.to_string(), axis: Some((Axis::MousePositionX, 0.0)), ..Default::default() },
                        MapAxisInput { name: CommonAxis::CURSOR_Y.to_string(), axis: Some((Axis::MousePositionY, 0.0)), ..Default::default() },
                        MapAxisInput { name: CommonAxis::VIEW_X.to_string(), axis: Some((Axis::MouseMotionX, 0.01)), ..Default::default() },
                        MapAxisInput { name: CommonAxis::VIEW_Y.to_string(), axis: Some((Axis::MouseMotionY, 0.01)), ..Default::default() },
                        MapAxisInput { name: CommonAxis::MOVE_FORWARD.to_string(), button: Some((Button::Keyboard { code: VirtualKeyCode::Z }, 1.0)), ..Default::default() },
                        MapAxisInput { name: CommonAxis::MOVE_BACKWARD.to_string(), button: Some((Button::Keyboard { code: VirtualKeyCode::S }, 1.0)), ..Default::default() },
                        MapAxisInput { name: CommonAxis::MOVE_LEFT.to_string(), button: Some((Button::Keyboard { code: VirtualKeyCode::Q }, 1.0)), ..Default::default() },
                        MapAxisInput { name: CommonAxis::MOVE_RIGHT.to_string(), button: Some((Button::Keyboard { code: VirtualKeyCode::D }, 1.0)), ..Default::default() },
                        MapAxisInput { name: CommonAxis::MOVE_UP.to_string(), button: Some((Button::Keyboard { code: VirtualKeyCode::X }, 1.0)), ..Default::default() },
                        MapAxisInput { name: CommonAxis::MOVE_DOWN.to_string(), button: Some((Button::Keyboard { code: VirtualKeyCode::W }, 1.0)), ..Default::default() },
                    ],
                    ..Default::default()
                },
                MapGroupInput {
                    name: "test".to_owned(),
                    actions: vec![
                        MapActionInput { name: "switch_mode".to_string(), button: Some(Button::Keyboard { code: VirtualKeyCode::C }), ..Default::default() },
                        MapActionInput { name: "roll_left".to_string(), button: Some(Button::Keyboard { code: VirtualKeyCode::A }), ..Default::default() },
                        MapActionInput { name: "roll_right".to_string(), button: Some(Button::Keyboard { code: VirtualKeyCode::E }), ..Default::default() },
                    ],
                    axis: vec![],
                    ..Default::default()
                }
            ],
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
        let id = self.profiles.insert(InputProfile { name, active: true, groups: Default::default() });
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
            let profile = InputProfile { name, active: true, groups: from.groups.clone() };
            let id = self.profiles.insert(profile);
            self.refresh(app);
            id
        } else {
            InputProfileId::null()
        }
    }

    pub(crate) fn save(&self) -> Result<()> {
        std::fs::create_dir_all("config").unwrap();
        let file = File::create(format!("config/profiles.json"))
            .context("Failed to open file.")?;
        let profiles = self.profiles.values().collect::<Vec<&_>>();
        serde_json::to_writer(&file, &profiles)
            .context("Failed to write file.")?;
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

            // Update groups
            for id in InputDatabase::iter_groups(app) {
                let group = InputDatabase::group(app, id).unwrap();
                // Find existing group
                if let Some(g) = profile.groups.iter_mut().find(|g| g.name == group.name) {
                    g.id = group.id;
                } else {
                    profile.groups.push(MapGroupInput { 
                        id: group.id, 
                        name: group.name.clone(), 
                        actions: Default::default(), 
                        axis: Default::default() 
                    });
                }
            }

            // Update actions
            for id in InputDatabase::iter_actions(app) {
                let action = InputDatabase::action(app, id).unwrap();
                let group = profile.groups.iter_mut().find(|g| g.id == action.group).unwrap();
                if let Some(a) = group.actions.iter_mut().find(|a| a.name == action.descriptor.name) {
                    // Update action info
                    a.id = action.id;
                } else {
                    // Insert new action
                    group.actions.push(MapActionInput { 
                        id: action.id,
                        name: action.descriptor.name.clone(),
                        button: Default::default(),
                    });
                }
            }

            // Update axis
            for id in InputDatabase::iter_axis(app) {
                let axis = InputDatabase::axis(app, id).unwrap();
                let group = profile.groups.iter_mut().find(|g| g.id == axis.group).unwrap();
                if let Some(a) = group.axis.iter_mut().find(|a| a.name == axis.descriptor.name) {
                    // Update axis info
                    a.id = axis.id;
                } else {
                    // Insert new axis
                    group.axis.push(MapAxisInput { 
                        id: axis.id, 
                        name: axis.descriptor.name.clone(), 
                        button: Default::default(),
                        axis: Default::default(),
                    });
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
        self.controllers_button_to_action.clear();
        self.controllers_button_to_axis.clear();
        self.controllers_axis_to_axis.clear();

        // Update caches
        for (_, profile) in &self.profiles {
            if profile.active {
                for group in &profile.groups {
                    for action in &group.actions {
                        if let Some(button) = &action.button {
                            match button {
                                Button::Keyboard { code } => {
                                    self.key_to_action.entry(*code).or_insert(Default::default()).push(KeyToAction { id: action.id });
                                },
                                Button::Mouse { button } => {
                                    self.mouse_button_to_action.entry(*button).or_insert(Default::default()).push(MouseButtonToAction { id: action.id });
                                },
                                Button::Controller { id, button } => {
                                    self.controllers_button_to_action.entry(*id).or_insert(Default::default()).entry(*button).or_insert(Default::default())
                                        .push(ControllerButtonToAction { id: action.id });
                                },
                            }
                        }
                    }
                    for axis in &group.axis {
                        if let Some((b, value)) = &axis.button {
                            match b {
                                Button::Keyboard { code } => {
                                    self.key_to_axis.entry(*code).or_insert(Default::default()).push(KeyToAxis { id: axis.id, value: *value });
                                },
                                Button::Mouse { button } => {
                                    self.mouse_button_to_axis.entry(*button).or_insert(Default::default()).push(MouseButtonToAxis { id: axis.id, value: *value });
                                },
                                Button::Controller { id, button } => {
                                    self.controllers_button_to_axis.entry(*id).or_insert(Default::default()).entry(*button).or_insert(Default::default())
                                        .push(ControllerButtonToAxis { id: axis.id, value: *value });
                                },
                            }
                        }
                        if let Some((a, scale)) = &axis.axis {
                            match a {
                                Axis::MousePositionX => {
                                    self.mouse_position_x_to_axis.push(MousePositionToAxis { id: axis.id });
                                },
                                Axis::MousePositionY => {
                                    self.mouse_position_y_to_axis.push(MousePositionToAxis { id: axis.id });
                                },
                                Axis::MouseMotionX => {
                                    self.mouse_motion_x_to_axis.push(MouseMotionToAxis { id: axis.id, scale: *scale });
                                },
                                Axis::MouseMotionY => {
                                    self.mouse_motion_y_to_axis.push(MouseMotionToAxis { id: axis.id, scale: *scale });
                                },
                                Axis::Controller { id, axis: ax } => {
                                    self.controllers_axis_to_axis.entry(*id).or_insert(Default::default()).entry(*ax).or_insert(Default::default())
                                        .push(ControllerAxisToAxis { id: axis.id, scale: *scale })
                                }
                            }
                        }
                    }
                }
            }
        }   
    }

    pub(crate) fn dispatch_keyboard(&self, keycode: VirtualKeyCode, state: ElementState, events: &mut AppEvents) {
        if let Some(actions) = self.key_to_action.get(&keycode) {
            let state = match state {
                ElementState::Pressed => ActionState::Pressed,
                ElementState::Released => ActionState::Released,
            };
            for action in actions {
                events.push_input(InputEvent::Action(ActionEvent { id: action.id, state }));        
            }
        }
        if let Some(axis) = self.key_to_axis.get(&keycode) {
            for ax in axis {
                let value = match state {
                    ElementState::Pressed => ax.value,
                    ElementState::Released => 0.0,
                };
                events.push_input(InputEvent::Axis(AxisEvent { id: ax.id, value }));
            }
        }
    }

    pub(crate) fn dispatch_mouse_button(&self, button: MouseButton, state: ElementState, events: &mut AppEvents) {
        let state = match state {
            ElementState::Pressed => ActionState::Pressed,
            ElementState::Released => ActionState::Released,
        };
        if let Some(actions) = self.mouse_button_to_action.get(&button) {
            for action in actions {
                events.push_input(InputEvent::Action(ActionEvent { id: action.id, state }));
            }
        }
        if let Some(axis) = self.mouse_button_to_axis.get(&button) {
            for ax in axis {
                events.push_input(InputEvent::Axis(AxisEvent { id: ax.id, value: ax.value }));
            }
        }
    }

    pub(crate) fn dispatch_mouse_motion(&self, delta: (f64, f64), events: &mut AppEvents) {
        for axis in &self.mouse_motion_x_to_axis {
            events.push_input(InputEvent::Axis(AxisEvent { id: axis.id, value: delta.0 as f32 * axis.scale }));
        }
        for axis in &self.mouse_motion_y_to_axis {
            events.push_input(InputEvent::Axis(AxisEvent { id: axis.id, value: delta.1 as f32 * axis.scale }));
        }
    }

    pub(crate) fn dispatch_mouse_cursor(&self, cursor: (f32, f32), events: &mut AppEvents) {
        for axis in &self.mouse_position_x_to_axis {
            events.push_input(InputEvent::Axis(AxisEvent { id: axis.id, value: cursor.0 }));
        }
        for axis in &self.mouse_position_y_to_axis {
            events.push_input(InputEvent::Axis(AxisEvent { id: axis.id, value: cursor.1 }));
        }
    }

    pub(crate) fn dispatch_controller_button(&self, id: GamepadId, button: gilrs::Button, state: ActionState, events: &mut AppEvents) {
        if let Some(buttons) = &self.controllers_button_to_action.get(&id) {
            if let Some(actions) = buttons.get(&button) {
                for action in actions {
                    events.push_input(InputEvent::Action(ActionEvent { id: action.id, state }));
                }
            }
        }
        if let Some(axis) = &self.controllers_button_to_axis.get(&id) {
            if let Some(a) = axis.get(&button) {
                for ax in a {
                    let value = match state {
                        ActionState::Pressed => ax.value,
                        ActionState::Released => 0.0,
                    };
                    events.push_input(InputEvent::Axis(AxisEvent { id: ax.id, value }));
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
                    events.push_input(InputEvent::Axis(AxisEvent { id: a.id, value: value * a.scale }));
                }
            }
        }
    }
}