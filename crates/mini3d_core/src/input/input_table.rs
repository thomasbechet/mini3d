use std::collections::HashMap;

use super::event::{InputEvent, ActionState};

pub struct ActionInput {
    pub pressed: bool,
    pub(crate) was_pressed: bool,
}

impl ActionInput {
    pub fn new() -> Self {
        ActionInput { pressed: false, was_pressed: false }
    }

    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    pub fn is_released(&self) -> bool {
        !self.pressed
    }

    pub fn is_just_pressed(&self) -> bool {
        self.pressed && !self.was_pressed
    }

    pub fn is_just_released(&self) -> bool {
        !self.pressed && self.was_pressed
    }
}

pub struct AxisInput {
    pub value: f32,
    pub range: [f32; 2],
}

impl AxisInput {
    pub fn new() -> Self {
        AxisInput { value: 0.0, range: [0f32, 1f32] }
    }
}

pub const ACTION_UP: &'static str = "up";
pub const ACTION_DOWN: &'static str = "down";
pub const ACTION_LEFT: &'static str = "left";
pub const ACTION_RIGHT: &'static str = "right";

pub type InputName = &'static str;

pub struct InputTable {
    pub actions: HashMap<InputName, ActionInput>,
    pub axes: HashMap<InputName, AxisInput>,
}

impl InputTable {
    pub fn dispatch_event(&mut self, event: &InputEvent) {
        match event {
            InputEvent::Action(action_event) => {
                if let Some(action) = self.actions.get_mut(action_event.name) {
                    match action_event.state {
                        ActionState::Pressed => {
                            action.pressed = true;
                        },
                        ActionState::Released => {
                            action.pressed = false;
                        },
                    }
                }
            },
            InputEvent::Axis(_) => {},
        }
    }

    pub fn update_inputs(&mut self) {
        for (_, action) in self.actions.iter_mut() {
            action.was_pressed = action.pressed;
        }
    }
}

impl Default for InputTable {
    fn default() -> Self {
        InputTable { 
            actions: HashMap::from([
                (ACTION_UP, ActionInput::new())
            ]),
            axes: HashMap::from([]),
        }
    }
}