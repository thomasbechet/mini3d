use std::{collections::{HashMap, HashSet}, error::Error, fmt::Display};

use serde::{Serialize, Deserialize, Serializer, Deserializer, ser::SerializeTuple, de::Visitor};

use crate::{event::input::{InputEvent}, uid::UID, feature::asset::input_table::{InputAxisRange, InputTable}};

use self::backend::{InputBackend, InputBackendError};

pub mod backend;

#[derive(Debug)]
pub enum InputError {
    ActionNotFound { uid: UID },
    AxisNotFound { uid: UID },
    TextNotFound { uid: UID },
    DuplicatedTable { name: String },
    DuplicatedAction { name: String },
    DuplicatedAxis { name: String },
    TableValidationError,
}

impl Error for InputError {}

impl Display for InputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputError::ActionNotFound { uid } => write!(f, "Action with UID {} not found", uid),
            InputError::AxisNotFound { uid } => write!(f, "Axis with UID {} not found", uid),
            InputError::TextNotFound { uid } => write!(f, "Text with UID {} not found", uid),
            InputError::DuplicatedTable { name } => write!(f, "Duplicated table: {}", name),
            InputError::DuplicatedAction { name } => write!(f, "Duplicated action: {}", name),
            InputError::DuplicatedAxis { name } => write!(f, "Duplicated axis: {}", name),
            InputError::TableValidationError => write!(f, "Table validation error"),
        }
    }
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
    notify_tables: HashSet<UID>, // None means table removed
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
                self.manager.tables = seq.next_element()?.ok_or_else(|| Error::custom("Expect tables"))?;
                self.manager.actions = seq.next_element()?.ok_or_else(|| Error::custom("Expect actions"))?;
                self.manager.axis = seq.next_element()?.ok_or_else(|| Error::custom("Expect axis"))?;
                Ok(())
            }
        }
        deserializer.deserialize_tuple(3, InputVisitor { manager: self })
    }

    pub(crate) fn synchronize_backend(&mut self, backend: &mut impl InputBackend) -> Result<(), InputBackendError> {
        for uid in self.notify_tables.drain() {
            if let Some(table) = self.tables.get(&uid) {
                backend.update_table(uid, Some(table))?;
            } else {
                backend.update_table(uid, None)?;
            }
        }
        Ok(())
    }

    pub(crate) fn add_table(&mut self, table: &InputTable) -> Result<(), InputError> {
        // Check table validity
        table.validate().map_err(|_| InputError::TableValidationError)?;
        // Check duplicated table
        if self.tables.contains_key(&table.uid()) {
            return Err(InputError::DuplicatedTable { name: table.name.to_string() });
        }
        // Check duplicated actions
        for action in table.actions.iter() {
            if self.actions.contains_key(&action.uid()) {
                return Err(InputError::DuplicatedAction { name: action.name.to_string() });
            }
        }
        // Check duplicated axis
        for axis in table.axis.iter() {
            if self.axis.contains_key(&axis.uid()) {
                return Err(InputError::DuplicatedAxis { name: axis.name.to_string() });
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
        // Notify input mapping
        self.notify_tables.insert(table.uid());
        Ok(())
    }

    pub(crate) fn action(&self, uid: UID) -> Result<&InputActionState, InputError> {
        self.actions.get(&uid).ok_or(InputError::ActionNotFound { uid })
    }

    pub(crate) fn axis(&self, uid: UID) -> Result<&InputAxisState, InputError> {
        self.axis.get(&uid).ok_or(InputError::AxisNotFound { uid })
    }

    pub(crate) fn text(&self, uid: UID) -> Result<&InputTextState, InputError> {
        self.texts.get(&uid).ok_or(InputError::TextNotFound { uid })
    }
}