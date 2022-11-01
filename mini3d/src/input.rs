use serde::{Serialize, Deserialize};
use slotmap::SecondaryMap;

use crate::{event::input::{InputEvent, InputTextEvent}, asset::{input_action::{InputActionId, InputAction}, input_axis::{InputAxisId, InputAxisKind, InputAxis}, AssetManager, AssetRef, AssetUID}};

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
}

impl InputAxisState {
    
    pub fn set_value(&mut self, value: f32, kind: &InputAxisKind) {
        self.value = match kind {
            InputAxisKind::Clamped { min, max } => {
                value.max(*min).min(*max)
            },
            InputAxisKind::Normalized { norm } => {
                value / norm
            },
            InputAxisKind::ClampedNormalized { min, max, norm } => {
                value.max(*min).min(*max) / norm
            },
            InputAxisKind::Infinite => {
                value
            },
        }
    }
}

pub struct InputManager {
    text: String,
    action_states: SecondaryMap<InputActionId, InputActionState>,
    axis_states: SecondaryMap<InputAxisId, InputAxisState>,
    pub(crate) reload_input_mapping: bool,
}

impl Default for InputManager {
    fn default() -> Self {
        Self {
            text: Default::default(),
            action_states: Default::default(),
            axis_states: Default::default(),
            reload_input_mapping: false,
        }
    }
}

impl InputManager {

    /// Reset action states and mouse motion
    pub(crate) fn prepare_dispatch(&mut self) {

        // Save the previous action state
        for (_, action) in self.action_states.iter_mut() {
            action.was_pressed = action.pressed;
        }

        // Reset text for current frame
        self.text.clear();
    }

    pub(crate) fn reload_states(&mut self, asset: &AssetManager) {
        self.action_states.clear();
        for entry in asset.iter::<InputAction>() {
            self.action_states.insert(entry.id, InputActionState { pressed: entry.asset.default_pressed, was_pressed: false });
        }
        self.axis_states.clear();
        for entry in asset.iter::<InputAxis>() {
            let mut state =  InputAxisState { value: entry.asset.default_value };
            state.set_value(entry.asset.default_value, &entry.asset.kind);
            self.axis_states.insert(entry.id, state);
        }
        self.text.clear();
        self.reload_input_mapping = true;
    }

    /// Process input events
    pub(crate) fn dispatch_event(&mut self, event: &mut InputEvent, asset: &AssetManager) {
        match event {
            InputEvent::Action(event) => {
                if let Some(entry) = event.action.get_or_resolve(asset) {
                    if let Some(state) = self.action_states.get_mut(entry.id) {
                        state.pressed = event.pressed;
                    }
                }
            },
            InputEvent::Axis(event) => {
                if let Some(entry) = event.axis.get_or_resolve(asset) {
                    if let Some(state) = self.axis_states.get_mut(entry.id) {
                        state.set_value(event.value, &entry.asset.kind);
                    }
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

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn action(&self, id: InputActionId) -> Option<&InputActionState> {
        self.action_states.get(id)
    }

    pub fn axis(&self, id: InputAxisId) -> Option<&InputAxisState> {
        self.axis_states.get(id)
    }

    pub fn find_action(&self, uid: AssetUID, asset: &AssetManager, default_pressed: bool) -> InputActionState {
        asset.find::<InputAction>(uid).map_or(InputActionState { pressed: default_pressed, was_pressed: default_pressed }, |entry| {
            *self.action_states.get(entry.id).unwrap_or(&InputActionState { pressed: default_pressed, was_pressed: default_pressed })
        })
    }

    pub fn find_axis(&self, uid: AssetUID, asset: &AssetManager, default_value: f32) -> InputAxisState {
        asset.find::<InputAxis>(uid).map_or(InputAxisState { value: default_value }, |entry| {
            *self.axis_states.get(entry.id).unwrap_or(&InputAxisState { value: default_value })
        })
    }

}

impl AssetRef<InputAction> {
    pub fn state(&mut self, asset: &AssetManager, input: &InputManager, default_pressed: bool) -> InputActionState {
        self.get_or_resolve(asset).map_or(InputActionState { pressed: default_pressed, was_pressed: default_pressed }, |entry| {
            *input.action_states.get(entry.id).unwrap_or(&InputActionState { pressed: default_pressed, was_pressed: default_pressed })
        })
    }
}

impl AssetRef<InputAxis> {
    pub fn state(&mut self, asset: &AssetManager, input: &InputManager, default: f32) -> InputAxisState {
        self.get_or_resolve(asset).map_or(InputAxisState { value: default }, |entry| {
            *input.axis_states.get(entry.id).unwrap_or(&InputAxisState { value: default })
        })
    }
}