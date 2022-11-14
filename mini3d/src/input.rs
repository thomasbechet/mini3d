use std::collections::HashMap;

use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};

use crate::{event::input::{InputEvent, InputTextEvent}, asset::{input_action::InputAction, input_axis::{InputAxisRange, InputAxis}, AssetManager}, uid::UID};

pub mod control_layout;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct InputActionState {
    pressed: bool,
    was_pressed: bool,
}

impl InputActionState {
        
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

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct InputAxisState {
    pub value: f32,
    pub range: InputAxisRange,
}

impl InputAxisState {
    
    pub fn set_value(&mut self, value: f32) {
        self.value = match &self.range {
            InputAxisRange::Clamped { min, max } => {
                value.max(*min).min(*max)
            },
            InputAxisRange::Normalized { norm } => {
                value / norm
            },
            InputAxisRange::ClampedNormalized { min, max, norm } => {
                value.max(*min).min(*max) / norm
            },
            InputAxisRange::Infinite => {
                value
            },
        }
    }
}

#[derive(Default)]
pub struct InputManager {
    text: String,
    actions: HashMap<UID, InputActionState>,
    axis: HashMap<UID, InputAxisState>,
    pub(crate) reload_input_mapping: bool,
}

impl InputManager {

    /// Reset action states and mouse motion
    pub(crate) fn prepare_dispatch(&mut self) {

        // Save the previous action state
        for (_, action) in self.actions.iter_mut() {
            action.was_pressed = action.pressed;
        }

        // Reset text for current frame
        self.text.clear();
    }

    /// Process input events
    pub(crate) fn dispatch_event(&mut self, event: &InputEvent) {
        match event {
            InputEvent::Action(event) => {
                if let Some(action) = self.actions.get_mut(&event.action) {
                    action.pressed = event.pressed;
                }
            },
            InputEvent::Axis(event) => {
                if let Some(axis) = self.axis.get_mut(&event.axis) {
                    axis.set_value(event.value);
                }
            },
            InputEvent::Text(text_event) => {
                match text_event {
                    InputTextEvent::Character(char) => {
                        self.text.push(*char);
                    },
                    InputTextEvent::String(string) => {
                        self.text += string;
                    },
                }
            },
        }
    }

    pub fn reload_input_tables(&mut self, asset: &AssetManager) -> Result<()> {
        self.actions.clear();
        for (uid, entry) in asset.iter::<InputAction>()? {
            self.actions.insert(*uid, InputActionState { pressed: entry.asset.default_pressed, was_pressed: false });
        }
        self.axis.clear();
        for (uid, entry) in asset.iter::<InputAxis>()? {
            let mut state =  InputAxisState { value: entry.asset.default_value, range: entry.asset.range };
            state.set_value(entry.asset.default_value);
            self.axis.insert(*uid, state);
        }
        self.text.clear();
        self.reload_input_mapping = true;
        Ok(())
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn action(&self, uid: UID) -> Result<&InputActionState> {
        self.actions.get(&uid).ok_or_else(|| anyhow!("Input action not found"))
    }

    pub fn axis(&self, uid: UID) -> Result<&InputAxisState> {
        self.axis.get(&uid).ok_or_else(|| anyhow!("Input axis not found"))
    }
}