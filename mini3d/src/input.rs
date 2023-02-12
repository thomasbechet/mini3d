use std::collections::HashMap;

use anyhow::{Result, anyhow, Context};
use serde::{Serialize, Deserialize, Serializer, Deserializer, ser::SerializeTuple, de::Visitor};

use crate::{event::input::{InputEvent, InputTextEvent}, uid::UID, feature::asset::{input_axis::{InputAxisRange, InputAxis}, input_action::InputAction}, asset::AssetManager};

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

    pub(crate) fn save_state<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&self.actions)?;
        tuple.serialize_element(&self.axis)?;
        tuple.end()
    }

    pub(crate) fn load_state<'de, D: Deserializer<'de>>(&mut self, deserializer: D) -> Result<(), D::Error> {
        struct InputVisitor<'a> {
            manager: &'a mut InputManager,
        }
        impl<'de, 'a> Visitor<'de> for InputVisitor<'a> {
            type Value = ();
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Input manager data")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where A: serde::de::SeqAccess<'de> {
                use serde::de::Error;
                self.manager.actions = seq.next_element()?.with_context(|| "Expect actions").map_err(Error::custom)?;
                self.manager.axis = seq.next_element()?.with_context(|| "Expect axis").map_err(Error::custom)?;
                Ok(())
            }
        }
        self.reload_input_mapping = true;
        self.text.clear();
        deserializer.deserialize_tuple(2, InputVisitor { manager: self })
    }

    pub(crate) fn reload_input_tables(&mut self, asset: &AssetManager) -> Result<()> {
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

    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    pub(crate) fn action(&self, uid: UID) -> Result<&InputActionState> {
        self.actions.get(&uid).ok_or_else(|| anyhow!("Input action not found"))
    }

    pub(crate) fn axis(&self, uid: UID) -> Result<&InputAxisState> {
        self.axis.get(&uid).ok_or_else(|| anyhow!("Input axis not found"))
    }
}