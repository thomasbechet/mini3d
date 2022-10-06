use std::collections::HashMap;

use mini3d::{input::{InputGroupId, axis::{AxisInputId, AxisDescriptor}, action::{ActionInputId, ActionState, ActionDescriptor}, InputDatabase}, app::App, event::{AppEvents, input::{InputEvent, ActionEvent, AxisEvent}}, slotmap::{new_key_type, SlotMap}};
use mini3d_os::input::{OSGroup, OSAction, OSAxis};
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
    sensibility: f32,
}
struct MouseCursorToAxis {
    id: AxisInputId,
}

#[derive(PartialEq, Clone, Copy)]
pub(crate) enum Axis {
    CursorX, 
    CursorY, 
    MotionX, 
    MotionY,
}

#[derive(PartialEq, Clone, Copy)]
pub(crate) enum Button {
    Keyboard { code: VirtualKeyCode },
    Mouse { button: MouseButton },
    // Add controller button
}

#[derive(Default)]
pub(crate) struct MapActionInput {
    pub(crate) id: ActionInputId,
    pub(crate) descriptor: ActionDescriptor,
    pub(crate) button: Option<Button>,
}
#[derive(Default)]
pub(crate) struct MapAxisInput {
    pub(crate) id: AxisInputId,
    pub(crate) descriptor: AxisDescriptor,
    pub(crate) button: Option<Button>,
    pub(crate) button_value: f32,
    pub(crate) axis: Option<Axis>,
    pub(crate) axis_sensibility: f32,
}

#[derive(Default)]
pub(crate) struct MapGroupInput {
    pub(crate) id: InputGroupId,
    pub(crate) name: String,
    pub(crate) actions: Vec<MapActionInput>,
    pub(crate) axis: Vec<MapAxisInput>,
}

new_key_type! { pub(crate) struct InputConfigId; }

#[derive(Default)]
pub(crate) struct InputConfig {
    pub(crate) name: String,
    pub(crate) groups: Vec<MapGroupInput>,
}

#[derive(Default)]
pub(crate) struct InputMapper {
    
    pub(crate) configs: SlotMap<InputConfigId, InputConfig>,
    pub(crate) default_config: InputConfigId,

    key_to_action: HashMap<VirtualKeyCode, Vec<KeyToAction>>,
    key_to_axis: HashMap<VirtualKeyCode, Vec<KeyToAxis>>,
    mouse_button_to_action: HashMap<MouseButton, Vec<MouseButtonToAction>>,
    mouse_button_to_axis: HashMap<MouseButton, Vec<MouseButtonToAxis>>,
    mouse_motion_x_to_axis: Vec<MouseMotionToAxis>,
    mouse_motion_y_to_axis: Vec<MouseMotionToAxis>,
    mouse_cursor_x_to_axis: Vec<MouseCursorToAxis>,
    mouse_cursor_y_to_axis: Vec<MouseCursorToAxis>,
}

impl InputMapper {

    pub(crate) fn new() -> Self {
        let mut mapper: InputMapper = Default::default();

        // Default inputs
        mapper.default_config = mapper.configs.insert(InputConfig { 
            name: "Default".to_string(), 
            groups: vec![
                MapGroupInput {
                    name: OSGroup::INPUT.to_owned(),
                    actions: vec![
                        MapActionInput { descriptor: ActionDescriptor { name: OSAction::UP.to_string(), ..Default::default() }, button: Some(Button::Keyboard { code: VirtualKeyCode::Z }), ..Default::default() },
                        MapActionInput { descriptor: ActionDescriptor { name: OSAction::LEFT.to_string(), ..Default::default() }, button: Some(Button::Keyboard { code: VirtualKeyCode::Q }), ..Default::default() },
                        MapActionInput { descriptor: ActionDescriptor { name: OSAction::DOWN.to_string(), ..Default::default() }, button: Some(Button::Keyboard { code: VirtualKeyCode::S }), ..Default::default() },
                        MapActionInput { descriptor: ActionDescriptor { name: OSAction::RIGHT.to_string(), ..Default::default() }, button: Some(Button::Keyboard { code: VirtualKeyCode::D }), ..Default::default() },
                    ],
                    axis: vec![
                        MapAxisInput { descriptor: AxisDescriptor { name: OSAxis::CURSOR_X.to_string(), ..Default::default() }, axis: Some(Axis::CursorX), ..Default::default() },
                        MapAxisInput { descriptor: AxisDescriptor { name: OSAxis::CURSOR_Y.to_string(), ..Default::default() }, axis: Some(Axis::CursorY), ..Default::default() },
                        MapAxisInput { descriptor: AxisDescriptor { name: OSAxis::MOTION_X.to_string(), ..Default::default() }, axis: Some(Axis::MotionX), axis_sensibility: 0.01, ..Default::default() },
                        MapAxisInput { descriptor: AxisDescriptor { name: OSAxis::MOTION_Y.to_string(), ..Default::default() }, axis: Some(Axis::MotionY), axis_sensibility: 0.01, ..Default::default() },
                    ],
                    ..Default::default()
                },
                MapGroupInput {
                    name: "test".to_owned(),
                    actions: vec![
                        MapActionInput { descriptor: ActionDescriptor { name: "switch_mode".to_string(), ..Default::default() }, button: Some(Button::Keyboard { code: VirtualKeyCode::C }), ..Default::default() },
                        MapActionInput { descriptor: ActionDescriptor { name: "roll_left".to_string(), ..Default::default() }, button: Some(Button::Keyboard { code: VirtualKeyCode::A }), ..Default::default() },
                        MapActionInput { descriptor: ActionDescriptor { name: "roll_right".to_string(), ..Default::default() }, button: Some(Button::Keyboard { code: VirtualKeyCode::E }), ..Default::default() },
                        MapActionInput { descriptor: ActionDescriptor { name: "toggle_layout".to_string(), ..Default::default() }, button: Some(Button::Keyboard { code: VirtualKeyCode::F }), ..Default::default() },
                    ],
                    axis: vec![
                        MapAxisInput { descriptor: AxisDescriptor { name: "move_forward".to_string(), ..Default::default() }, button: Some(Button::Keyboard { code: VirtualKeyCode::Z }), button_value: 1.0, ..Default::default() },
                        MapAxisInput { descriptor: AxisDescriptor { name: "move_backward".to_string(), ..Default::default() }, button: Some(Button::Keyboard { code: VirtualKeyCode::S }), button_value: 1.0, ..Default::default() },
                        MapAxisInput { descriptor: AxisDescriptor { name: "move_left".to_string(), ..Default::default() }, button: Some(Button::Keyboard { code: VirtualKeyCode::Q }), button_value: 1.0, ..Default::default() },
                        MapAxisInput { descriptor: AxisDescriptor { name: "move_right".to_string(), ..Default::default() }, button: Some(Button::Keyboard { code: VirtualKeyCode::D }), button_value: 1.0, ..Default::default() },
                        MapAxisInput { descriptor: AxisDescriptor { name: "move_up".to_string(), ..Default::default() }, button: Some(Button::Keyboard { code: VirtualKeyCode::X }), button_value: 1.0, ..Default::default() },
                        MapAxisInput { descriptor: AxisDescriptor { name: "move_down".to_string(), ..Default::default() }, button: Some(Button::Keyboard { code: VirtualKeyCode::W }), button_value: 1.0, ..Default::default() },
                    ],
                    ..Default::default()
                }
            ],
        });

        mapper
    }

    pub(crate) fn reload(&mut self, app: &App) {

        // Update configs
        for (_, config) in &mut self.configs {

            // Update groups
            for id in InputDatabase::iter_groups(app) {
                let group = InputDatabase::group(app, id).unwrap();
                // Find existing group
                if let Some(g) = config.groups.iter_mut().find(|g| g.name == group.name) {
                    g.id = group.id;
                } else {
                    config.groups.push(MapGroupInput { 
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
                let group = config.groups.iter_mut().find(|g| g.id == action.group).unwrap();
                if let Some(a) = group.actions.iter_mut().find(|a| a.descriptor.name == action.descriptor.name) {
                    // Update action info
                    a.id = action.id;
                    a.descriptor = action.descriptor.clone();
                } else {
                    // Insert new action
                    group.actions.push(MapActionInput { 
                        id: action.id, 
                        descriptor: action.descriptor.clone(), 
                        button: Default::default(),
                    });
                }
            }

            // Update axis
            for id in InputDatabase::iter_axis(app) {
                let axis = InputDatabase::axis(app, id).unwrap();
                let group = config.groups.iter_mut().find(|g| g.id == axis.group).unwrap();
                if let Some(a) = group.axis.iter_mut().find(|a| a.descriptor.name == axis.descriptor.name) {
                    // Update axis info
                    a.id = axis.id;
                    a.descriptor = axis.descriptor.clone();
                } else {
                    // Insert new axis
                    group.axis.push(MapAxisInput { 
                        id: axis.id, 
                        descriptor: axis.descriptor.clone(), 
                        button: Default::default(),
                        button_value: 1.0,
                        axis: Default::default(),
                        axis_sensibility: 1.0,
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
        self.mouse_cursor_x_to_axis.clear();
        self.mouse_cursor_y_to_axis.clear();
        self.mouse_motion_x_to_axis.clear();
        self.mouse_motion_y_to_axis.clear();

        // Update caches
        for (_, config) in &self.configs {
            for group in &config.groups {
                for action in &group.actions {
                    if let Some(button) = &action.button {
                        match button {
                            Button::Keyboard { code } => {
                                self.key_to_action.entry(*code).or_insert(Default::default()).push(KeyToAction { id: action.id });
                            },
                            Button::Mouse { button } => {
                                self.mouse_button_to_action.entry(*button).or_insert(Default::default()).push(MouseButtonToAction { id: action.id });
                            },
                        }
                    }
                }
                for axis in &group.axis {
                    if let Some(b) = &axis.button {
                        match b {
                            Button::Keyboard { code } => {
                                self.key_to_axis.entry(*code).or_insert(Default::default()).push(KeyToAxis { id: axis.id, value: axis.button_value });
                            },
                            Button::Mouse { button } => {
                                self.mouse_button_to_axis.entry(*button).or_insert(Default::default()).push(MouseButtonToAxis { id: axis.id, value: axis.button_value });
                            },
                        }
                    }
                    if let Some(a) = &axis.axis {
                        match a {
                            Axis::CursorX => {
                                self.mouse_cursor_x_to_axis.push(MouseCursorToAxis { id: axis.id });
                            },
                            Axis::CursorY => {
                                self.mouse_cursor_y_to_axis.push(MouseCursorToAxis { id: axis.id });
                            },
                            Axis::MotionX => {
                                self.mouse_motion_x_to_axis.push(MouseMotionToAxis { id: axis.id, sensibility: axis.axis_sensibility });
                            },
                            Axis::MotionY => {
                                self.mouse_motion_y_to_axis.push(MouseMotionToAxis { id: axis.id, sensibility: axis.axis_sensibility });
                            },
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
            events.push_input(InputEvent::Axis(AxisEvent { id: axis.id, value: delta.0 as f32 * axis.sensibility }));
        }
        for axis in &self.mouse_motion_y_to_axis {
            events.push_input(InputEvent::Axis(AxisEvent { id: axis.id, value: delta.1 as f32 * axis.sensibility }));
        }
    }

    pub(crate) fn dispatch_mouse_cursor(&self, cursor: (f32, f32), events: &mut AppEvents) {
        for axis in &self.mouse_cursor_x_to_axis {
            events.push_input(InputEvent::Axis(AxisEvent { id: axis.id, value: cursor.0 }));
        }
        for axis in &self.mouse_cursor_y_to_axis {
            events.push_input(InputEvent::Axis(AxisEvent { id: axis.id, value: cursor.1 }));
        }
    }
}