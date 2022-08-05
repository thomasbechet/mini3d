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

pub enum RangeType {
    Clamped { min: f32, max: f32 },
    Normalized { norm: f32 },
    ClampedNormalized { min: f32, max: f32, norm: f32 },
    Infinite,
}

pub struct RangeInput {
    pub value: f32,
    pub range: RangeType,
}

impl RangeInput {
    pub fn new(range: RangeType) -> Self {
        RangeInput { value: 0.0, range: range }
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = match self.range {
            RangeType::Clamped { min, max } => {
                value.max(min).min(max)
            },
            RangeType::Normalized { norm } => {
                value / norm
            },
            RangeType::ClampedNormalized { min, max, norm } => {
                value.max(min).min(max) / norm
            },
            RangeType::Infinite => {
                value
            },
        }
    }
}

pub const ACTION_UP: &'static str = "up";
pub const ACTION_DOWN: &'static str = "down";
pub const ACTION_LEFT: &'static str = "left";
pub const ACTION_RIGHT: &'static str = "right";
pub const AXIS_VIEW_X: &'static str = "view_x";
pub const AXIS_VIEW_Y: &'static str = "view_y";

pub type InputName = &'static str;

pub struct InputTable {
    pub actions: HashMap<InputName, ActionInput>,
    pub axes: HashMap<InputName, RangeInput>,
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
            InputEvent::Axis(axis_event) => {
                if let Some(axis) = self.axes.get_mut(axis_event.name) {
                    axis.set_value(axis_event.value);
                }
            },
            InputEvent::Text(text_event) => {
                match text_event {
                    super::event::TextEvent::Character(char) => {
                        
                    },
                    super::event::TextEvent::String(String) => {
                        
                    },
                }
            },
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
            axes: HashMap::from([
                (AXIS_VIEW_X, RangeInput::new(RangeType::Infinite)),
                (AXIS_VIEW_Y, RangeInput::new(RangeType::Infinite)),
            ]),
        }
    }
}