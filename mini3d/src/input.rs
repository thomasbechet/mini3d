use std::collections::HashMap;

use anyhow::{Result, anyhow, Context};
use serde::{Serialize, Deserialize, Serializer, Deserializer, ser::SerializeTuple, de::Visitor};

use crate::{event::input::{InputEvent}, uid::UID, feature::asset::input_table::{InputAxisRange, InputTable, InputAction, InputAxis}};

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

#[derive(Serialize, Deserialize, Clone)]
pub struct InputTextState {
    pub value: String,
}

#[derive(Default)]
pub struct InputManager {
    tables: HashMap<UID, InputTable>,
    actions: HashMap<UID, InputActionState>,
    axis: HashMap<UID, InputAxisState>,
    texts: HashMap<UID, InputTextState>,
    pub(crate) reload_input_mapping: bool,
}

impl InputManager {

    /// Reset action states and mouse motion
    pub(crate) fn prepare_dispatch(&mut self) {

        // Save the previous action state
        for action in self.actions.values_mut() {
            action.was_pressed = action.pressed;
        }

        // Reset text for current frame
        for text in self.texts.values_mut() {
            text.value.clear();
        }
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
            InputEvent::Text(text) => {
                if let Some(text) = self.texts.get_mut(&text.stream) {
                    text.value = text.value.clone();
                }
            },
        }
    }

    pub(crate) fn save_state<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut tuple = serializer.serialize_tuple(3)?;
        tuple.serialize_element(&self.tables)?;
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
                self.manager.tables = seq.next_element()?.with_context(|| "Expect tables").map_err(Error::custom)?;
                self.manager.actions = seq.next_element()?.with_context(|| "Expect actions").map_err(Error::custom)?;
                self.manager.axis = seq.next_element()?.with_context(|| "Expect axis").map_err(Error::custom)?;
                Ok(())
            }
        }
        self.reload_input_mapping = true;
        deserializer.deserialize_tuple(3, InputVisitor { manager: self })
    }

    pub(crate) fn add_table(&mut self, table: &InputTable) -> Result<()> {
        // Check table validity
        table.check_valid()?;
        // Check duplicated table
        if self.tables.contains_key(&table.uid()) {
            return Err(anyhow!("Input table already exists"));
        }
        // Check duplicated actions
        for action in table.actions.iter() {
            if self.actions.contains_key(&action.uid()) {
                return Err(anyhow!("Input action already exists: {}", action.name));
            }
        }
        // Check duplicated axis
        for axis in table.axis.iter() {
            if self.axis.contains_key(&axis.uid()) {
                return Err(anyhow!("Input axis already exists: {}", axis.name));
            }
        }
        // We can safely insert table, actions and axis
        for action in table.actions.iter() {
            self.actions.insert(action.uid(), InputActionState { pressed: action.default_pressed, was_pressed: false });
        }
        for axis in table.axis.iter() {
            let mut state =  InputAxisState { value: axis.default_value, range: axis.range };
            state.set_value(axis.default_value);
            self.axis.insert(axis.uid(), state);
        }
        self.tables.insert(table.uid(), table.clone());
        // Reload input mapping
        self.reload_input_mapping = true;
        Ok(())
    }

    pub(crate) fn iter_tables(&self) -> impl Iterator<Item = &InputTable> {
        self.tables.values()
    }

    pub(crate) fn iter_actions(&self) -> impl Iterator<Item = (&InputAction, &InputActionState)> {
        self.tables.values().flat_map(|table| table.actions.iter().map(|action| {
            let state = self.actions.get(&action.uid()).unwrap();
            (action, state)
        }))
    }

    pub(crate) fn iter_axis(&self) -> impl Iterator<Item = (&InputAxis, &InputAxisState)> {
        self.tables.values().flat_map(|table| table.axis.iter().map(|axis| {
            let state = self.axis.get(&axis.uid()).unwrap();
            (axis, state)
        }))
    }

    pub(crate) fn action(&self, uid: UID) -> Result<&InputActionState> {
        self.actions.get(&uid).ok_or_else(|| anyhow!("Input action not found"))
    }

    pub(crate) fn axis(&self, uid: UID) -> Result<&InputAxisState> {
        self.axis.get(&uid).ok_or_else(|| anyhow!("Input axis not found"))
    }

    pub(crate) fn text(&self, uid: UID) -> Result<&InputTextState> {
        self.texts.get(&uid).ok_or_else(|| anyhow!("Input text not found"))
    }
}