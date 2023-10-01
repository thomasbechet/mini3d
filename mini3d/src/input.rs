use mini3d_derive::{Error, Serialize};

use crate::feature::input::action::InputAction;
use crate::feature::input::axis::{InputAxis, InputAxisRange};
use crate::serialize::{Decoder, DecoderError};
use crate::serialize::{Encoder, EncoderError};
use crate::utils::slotmap::{SlotId, SlotMap};
use crate::utils::uid::ToUID;

use self::event::InputEvent;
use self::handle::{InputActionHandle, InputAxisHandle, InputTextHandle};
use self::provider::InputProvider;

pub mod event;
pub mod handle;
pub mod provider;

pub const MAX_INPUT_NAME_LEN: usize = 64;
pub const MAX_INPUT_DISPLAY_NAME_LEN: usize = 64;

#[derive(Debug, Error)]
pub enum InputError {
    #[error("Action not found")]
    ActionNotFound,
    #[error("Axis not found")]
    AxisNotFound,
    #[error("Text not found")]
    TextNotFound,
    #[error("Duplicated action")]
    DuplicatedAction,
    #[error("Duplicated axis")]
    DuplicatedAxis,
    #[error("Table validation error")]
    TableValidationError,
}

#[derive(Serialize, Clone)]
pub struct InputActionState {
    action: InputAction,
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

#[derive(Serialize, Clone)]
pub struct InputAxisState {
    axis: InputAxis,
    pub value: f32,
}

impl InputAxisState {
    pub fn set_value(&mut self, value: f32) {
        self.value = match &self.axis.range {
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
    provider: Box<dyn InputProvider>,
    actions: SlotMap<InputActionState>,
    axis: SlotMap<InputAxisState>,
    texts: SlotMap<InputTextState>,
}

impl InputManager {
    pub(crate) fn set_provider(&mut self, provider: Box<dyn InputProvider>) {
        self.provider.on_disconnect();
        self.provider = provider;
        self.provider.on_connect();
    }

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
    pub(crate) fn dispatch_events(&mut self) {
        while let Some(event) = self.provider.next_event() {
            match event {
                InputEvent::Action(event) => {
                    if let Some(action) = self.actions.get_mut(SlotId::from_raw(event.id)) {
                        action.pressed = event.pressed;
                    }
                }
                InputEvent::Axis(event) => {
                    if let Some(axis) = self.axis.get_mut(SlotId::from_raw(event.id)) {
                        axis.set_value(event.value);
                    }
                }
                InputEvent::Text(event) => {
                    if let Some(text) = self.texts.get_mut(SlotId::from_raw(event.id)) {
                        text.value = text.value.clone();
                    }
                }
            }
        }
    }

    pub(crate) fn save_state(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        // self.actions.serialize(encoder)?;
        // self.axis.serialize(encoder)?;
        // self.texts.serialize(encoder)?;
        Ok(())
    }

    pub(crate) fn load_state(&mut self, decoder: &mut impl Decoder) -> Result<(), DecoderError> {
        // self.actions = HashMap::deserialize(decoder, &Default::default())?;
        // self.axis = HashMap::deserialize(decoder, &Default::default())?;
        // self.texts = HashMap::deserialize(decoder, &Default::default())?;
        Ok(())
    }

    pub fn add_action(&mut self, action: InputAction) -> Result<InputActionHandle, InputError> {
        if self.find_action(&action.name).is_some() {
            return Err(InputError::DuplicatedAction);
        }
        let id = self.actions.add(InputActionState {
            action: action.clone(),
            pressed: action.default_pressed,
            was_pressed: false,
        });
        let handle = InputActionHandle {
            id,
            uid: action.name.to_uid(),
        };
        self.provider.add_action(handle.id.raw(), &action);
        Ok(handle)
    }

    pub fn find_action(&self, name: &str) -> Option<InputActionHandle> {
        let uid = name.into();
        self.actions
            .iter()
            .find(|(_, state)| state.action.name.to_uid() == uid)
            .map(|(id, state)| InputActionHandle {
                id,
                uid: state.action.name.to_uid(),
            })
    }

    pub fn add_axis(&mut self, axis: InputAxis) -> Result<InputAxisHandle, InputError> {
        if self.find_axis(&axis.name).is_some() {
            return Err(InputError::DuplicatedAxis);
        }
        let mut state = InputAxisState {
            axis: axis.clone(),
            value: 0.0,
        };
        state.set_value(axis.default_value);
        let id = self.axis.add(state);
        let handle = InputAxisHandle {
            id,
            uid: axis.name.to_uid(),
        };
        self.provider.add_axis(handle.id.raw(), &axis);
        Ok(handle)
    }

    pub fn find_axis(&self, name: &str) -> Option<InputAxisHandle> {
        let uid = name.into();
        self.axis
            .iter()
            .find(|(_, state)| state.axis.name.to_uid() == uid)
            .map(|(id, state)| InputAxisHandle {
                id,
                uid: state.axis.name.to_uid(),
            })
    }

    pub fn action(&self, handle: InputActionHandle) -> Result<&InputActionState, InputError> {
        self.actions
            .get(handle.id)
            .ok_or(InputError::ActionNotFound)
    }

    pub fn axis(&self, handle: InputAxisHandle) -> Result<&InputAxisState, InputError> {
        self.axis.get(handle.id).ok_or(InputError::AxisNotFound)
    }

    pub fn text(&self, handle: InputTextHandle) -> Result<&InputTextState, InputError> {
        self.texts.get(handle.id).ok_or(InputError::TextNotFound)
    }
}
