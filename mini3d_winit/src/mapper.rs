use std::collections::HashMap;

use mini3d::{input::{InputGroupId, axis::AxisInputId, action::{ActionInputId, ActionState}, InputDatabase}, app::App, event::{AppEvents, input::{InputEvent, ActionEvent, AxisEvent}}};
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

enum MouseAxis { 
    CursorX, 
    CursorY, 
    MotionX { sensibility: f32 }, 
    MotionY { sensibility: f32 },
}

#[derive(Default)]
struct MapActionInput {
    id: ActionInputId,
    name: String,
    keys: Vec<VirtualKeyCode>,
    mouse_buttons: Vec<MouseButton>,
}
#[derive(Default)]
struct MapAxisInput {
    id: AxisInputId,
    name: String,
    keys: Vec<(VirtualKeyCode, f32)>, // With value
    mouse_buttons: Vec<(MouseButton, f32)>, // With value
    mouse_axis: Option<MouseAxis>, // With sensibility
}
#[derive(Default)]
struct MapGroupInput {
    id: InputGroupId,
    name: String,
    actions: Vec<MapActionInput>,
    axis: Vec<MapAxisInput>,
}

#[derive(Default)]
pub(crate) struct InputMapper {
    
    groups: Vec<MapGroupInput>,

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
        mapper.groups.push(MapGroupInput {
            name: OSGroup::INPUT.to_owned(),
            actions: vec![
                MapActionInput { name: OSAction::UP.to_string(), keys: vec![VirtualKeyCode::Z], ..Default::default() },
                MapActionInput { name: OSAction::DOWN.to_string(), keys: vec![VirtualKeyCode::S], ..Default::default() },
                MapActionInput { name: OSAction::LEFT.to_string(), keys: vec![VirtualKeyCode::Q], ..Default::default() },
                MapActionInput { name: OSAction::RIGHT.to_string(), keys: vec![VirtualKeyCode::D], ..Default::default() },

                MapActionInput { name: "switch_mode".to_string(), keys: vec![VirtualKeyCode::C], ..Default::default() },
                MapActionInput { name: "roll_left".to_string(), keys: vec![VirtualKeyCode::A], ..Default::default() },
                MapActionInput { name: "roll_right".to_string(), keys: vec![VirtualKeyCode::E], ..Default::default() },
                MapActionInput { name: "toggle_layout".to_string(), keys: vec![VirtualKeyCode::F], ..Default::default() },
            ],
            axis: vec![
                MapAxisInput { name: OSAxis::CURSOR_X.to_string(), mouse_axis: Some(MouseAxis::CursorX), ..Default::default() },
                MapAxisInput { name: OSAxis::CURSOR_Y.to_string(), mouse_axis: Some(MouseAxis::CursorY), ..Default::default() },
                MapAxisInput { name: OSAxis::MOTION_X.to_string(), mouse_axis: Some(MouseAxis::MotionX { sensibility: 0.01 }), ..Default::default() },
                MapAxisInput { name: OSAxis::MOTION_Y.to_string(), mouse_axis: Some(MouseAxis::MotionY { sensibility: 0.01 }), ..Default::default() },
                
                MapAxisInput { name: "move_forward".to_string(), keys: vec![(VirtualKeyCode::Z, 1.0)], ..Default::default() },
                MapAxisInput { name: "move_backward".to_string(), keys: vec![(VirtualKeyCode::S, 1.0)], ..Default::default() },
                MapAxisInput { name: "move_left".to_string(), keys: vec![(VirtualKeyCode::Q, 1.0)], ..Default::default() },
                MapAxisInput { name: "move_right".to_string(), keys: vec![(VirtualKeyCode::D, 1.0)], ..Default::default() },
                MapAxisInput { name: "move_up".to_string(), keys: vec![(VirtualKeyCode::X, 1.0)], ..Default::default() },
                MapAxisInput { name: "move_down".to_string(), keys: vec![(VirtualKeyCode::W, 1.0)], ..Default::default() },
            ],
            ..Default::default()
        });

        mapper
    }

    pub(crate) fn reload(&mut self, app: &App) {

        // Update groups
        for id in InputDatabase::iter_groups(app) {
            let group = InputDatabase::group(app, id).unwrap();
            // Find existing group
            if let Some(g) = self.groups.iter_mut().find(|g| g.name == group.name) {
                g.id = group.id;
            } else {
                self.groups.push(MapGroupInput { 
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
            let group = self.groups.iter_mut().find(|g| g.id == action.group).unwrap();
            if let Some(a) = group.actions.iter_mut().find(|a| a.name == action.name) {
                a.id = action.id;
            } else {
                group.actions.push(MapActionInput { 
                    id: action.id, 
                    name: action.name.clone(), 
                    keys: Default::default(),
                    mouse_buttons: Default::default(),
                });
            }
        }

        // Update axis
        for id in InputDatabase::iter_axis(app) {
            let axis = InputDatabase::axis(app, id).unwrap();
            let group = self.groups.iter_mut().find(|g| g.id == axis.group).unwrap();
            if let Some(a) = group.axis.iter_mut().find(|a| a.name == axis.name) {
                a.id = axis.id;
            } else {
                group.axis.push(MapAxisInput { 
                    id: axis.id, 
                    name: axis.name.clone(), 
                    keys: Default::default(),
                    mouse_buttons: Default::default(),
                    mouse_axis: Default::default(),
                });
            }
        }

        // Update cache
        self.key_to_action.clear();
        self.key_to_axis.clear();
        self.mouse_button_to_action.clear();
        self.mouse_button_to_axis.clear();
        for group in &self.groups {
            for action in &group.actions {
                for key in action.keys.iter() {
                    self.key_to_action.entry(*key).or_insert(Default::default()).push(KeyToAction { id: action.id });
                }
                for mouse_button in action.mouse_buttons.iter() {
                    self.mouse_button_to_action.entry(*mouse_button).or_insert(Default::default()).push(MouseButtonToAction { id: action.id });
                }
            }
            for axis in &group.axis {
                for (key, value) in axis.keys.iter() {
                    self.key_to_axis.entry(*key).or_insert(Default::default()).push(KeyToAxis { id: axis.id, value: *value });
                }
                for (mouse_button, value) in axis.mouse_buttons.iter() {
                    self.mouse_button_to_axis.entry(*mouse_button).or_insert(Default::default()).push(MouseButtonToAxis { id: axis.id, value: *value });
                }
                if let Some(mouse_axis) = &axis.mouse_axis {
                    match mouse_axis {
                        MouseAxis::CursorX => self.mouse_cursor_x_to_axis.push(MouseCursorToAxis { id: axis.id }),
                        MouseAxis::CursorY => self.mouse_cursor_y_to_axis.push(MouseCursorToAxis { id: axis.id }),
                        MouseAxis::MotionX { sensibility } => self.mouse_motion_x_to_axis.push(MouseMotionToAxis { id: axis.id, sensibility: *sensibility }),
                        MouseAxis::MotionY { sensibility } => self.mouse_motion_y_to_axis.push(MouseMotionToAxis { id: axis.id, sensibility: *sensibility }),
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