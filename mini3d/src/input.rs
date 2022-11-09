use std::collections::HashMap;

use anyhow::{Result, anyhow};
use serde::{Serialize, Deserialize};
use slotmap::{new_key_type, SlotMap};

use crate::{event::input::{InputEvent, InputTextEvent}, asset::{input_action::InputAction, input_axis::{InputAxisRange, InputAxis}, AssetManager}, uid::UID};

pub mod control_layout;

new_key_type! {
    pub struct InputActionId;
    pub struct InputAxisId;
}

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
    actions: SlotMap<InputActionId, InputActionState>,
    uid_to_action: HashMap<UID, InputActionId>,
    axis: SlotMap<InputAxisId, InputAxisState>,
    uid_to_axis: HashMap<UID, InputAxisId>,
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
                if let Some(action) = self.uid_to_action.get(&event.action).and_then(|handle| self.actions.get_mut(*handle)) {
                    action.pressed = event.pressed;
                }
            },
            InputEvent::Axis(event) => {
                if let Some(axis) = self.uid_to_axis.get(&event.axis).and_then(|handle| self.axis.get_mut(*handle)) {
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

    pub fn reload_input_tables(&mut self, asset: &AssetManager) {
        self.actions.clear();
        for entry in asset.iter::<InputAction>() {
            let id = self.actions.insert(InputActionState { pressed: entry.asset.default_pressed, was_pressed: false });
            self.uid_to_action.insert(entry.uid, id);
        }
        self.axis.clear();
        for entry in asset.iter::<InputAxis>() {
            let mut state =  InputAxisState { value: entry.asset.default_value, range: entry.asset.range };
            state.set_value(entry.asset.default_value);
            let id = self.axis.insert(state);
            self.uid_to_axis.insert(entry.uid, id);
        }
        self.text.clear();
        self.reload_input_mapping = true;
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn action(&self, id: InputActionId) -> Result<&InputActionState> {
        self.actions.get(id).ok_or_else(|| anyhow!("Input action not found"))
    }

    pub fn axis(&self, id: InputAxisId) -> Result<&InputAxisState> {
        self.axis.get(id).ok_or_else(|| anyhow!("Input axis not found"))
    }

    pub fn find_action(&self, uid: UID) -> Result<InputActionId> {
        self.uid_to_action.get(&uid).copied().ok_or_else(|| anyhow!("Input action not found"))
    }

    pub fn find_axis(&self, uid: UID) -> Result<InputAxisId> {
        self.uid_to_axis.get(&uid).copied().ok_or_else(|| anyhow!("Input axis not found"))
    }
}