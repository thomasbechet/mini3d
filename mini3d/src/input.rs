use mini3d_derive::{Error, Serialize};
use std::collections::{HashMap, HashSet};

use crate::event::input::InputEvent;
use crate::feature::component::input::input_table::{InputAxisRange, InputTable};
use crate::serialize::{Decoder, DecoderError, Serialize};
use crate::{
    serialize::{Encoder, EncoderError},
    uid::UID,
};

use self::backend::{InputBackend, InputBackendError};

pub mod backend;

#[derive(Debug, Error)]
pub enum InputError {
    #[error("Action with UID {uid} not found")]
    ActionNotFound { uid: UID },
    #[error("Axis with UID {uid} not found")]
    AxisNotFound { uid: UID },
    #[error("Text with UID {uid} not found")]
    TextNotFound { uid: UID },
    #[error("Duplicated table: {name}")]
    DuplicatedTable { name: String },
    #[error("Duplicated action: {name}")]
    DuplicatedAction { name: String },
    #[error("Duplicated axis: {name}")]
    DuplicatedAxis { name: String },
    #[error("Table validation error")]
    TableValidationError,
}

#[derive(Serialize, Clone, Copy)]
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

#[derive(Serialize, Clone, Copy)]
pub struct InputAxisState {
    pub value: f32,
    pub range: InputAxisRange,
}

impl InputAxisState {
    pub fn set_value(&mut self, value: f32) {
        self.value = match &self.range {
            InputAxisRange::Clamped { min, max } => value.max(*min).min(*max),
            InputAxisRange::Normalized { norm } => value / norm,
            InputAxisRange::ClampedNormalized { min, max, norm } => {
                value.max(*min).min(*max) / norm
            }
            InputAxisRange::Infinite => value,
        }
    }
}

#[derive(Serialize, Clone)]
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
            }
            InputEvent::Axis(event) => {
                if let Some(axis) = self.axis.get_mut(&event.axis) {
                    axis.set_value(event.value);
                }
            }
            InputEvent::Text(text) => {
                if let Some(text) = self.texts.get_mut(&text.stream) {
                    text.value = text.value.clone();
                }
            }
        }
    }

    pub(crate) fn save_state(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        self.tables.serialize(encoder)?;
        self.actions.serialize(encoder)?;
        self.axis.serialize(encoder)?;
        Ok(())
    }

    pub(crate) fn load_state(&mut self, decoder: &mut impl Decoder) -> Result<(), DecoderError> {
        self.tables = HashMap::deserialize(decoder, &Default::default())?;
        self.actions = HashMap::deserialize(decoder, &Default::default())?;
        self.axis = HashMap::deserialize(decoder, &Default::default())?;
        Ok(())
    }

    pub(crate) fn synchronize_backend(
        &mut self,
        backend: &mut impl InputBackend,
    ) -> Result<(), InputBackendError> {
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
        table
            .validate()
            .map_err(|_| InputError::TableValidationError)?;
        // Check duplicated table
        if self.tables.contains_key(&table.uid()) {
            return Err(InputError::DuplicatedTable {
                name: table.name.to_string(),
            });
        }
        // Check duplicated actions
        for action in table.actions.iter() {
            if self.actions.contains_key(&action.uid()) {
                return Err(InputError::DuplicatedAction {
                    name: action.name.to_string(),
                });
            }
        }
        // Check duplicated axis
        for axis in table.axis.iter() {
            if self.axis.contains_key(&axis.uid()) {
                return Err(InputError::DuplicatedAxis {
                    name: axis.name.to_string(),
                });
            }
        }
        // We can safely insert table, actions and axis
        for action in table.actions.iter() {
            self.actions.insert(
                action.uid(),
                InputActionState {
                    pressed: action.default_pressed,
                    was_pressed: false,
                },
            );
        }
        for axis in table.axis.iter() {
            let mut state = InputAxisState {
                value: axis.default_value,
                range: axis.range,
            };
            state.set_value(axis.default_value);
            self.axis.insert(axis.uid(), state);
        }
        self.tables.insert(table.uid(), table.clone());
        // Notify input mapping
        self.notify_tables.insert(table.uid());
        Ok(())
    }

    pub(crate) fn action(&self, uid: UID) -> Result<&InputActionState, InputError> {
        self.actions
            .get(&uid)
            .ok_or(InputError::ActionNotFound { uid })
    }

    pub(crate) fn axis(&self, uid: UID) -> Result<&InputAxisState, InputError> {
        self.axis.get(&uid).ok_or(InputError::AxisNotFound { uid })
    }

    pub(crate) fn text(&self, uid: UID) -> Result<&InputTextState, InputError> {
        self.texts.get(&uid).ok_or(InputError::TextNotFound { uid })
    }
}
