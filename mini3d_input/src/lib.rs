#![no_std]

use action::{InputAction, InputActionId, InputActionState};
use alloc::boxed::Box;
use axis::{InputAxis, InputAxisId, InputAxisRange, InputAxisState};
use event::InputEvent;
use mini3d_derive::Error;
use mini3d_utils::slotmap::SlotMap;
use provider::{InputProvider, InputProviderError};
use text::{InputText, InputTextHandle};

pub mod action;
pub mod axis;
pub mod event;
pub mod provider;
pub mod text;

extern crate alloc;

#[cfg(test)]
extern crate std;

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
    #[error("Provider error: {0}")]
    ProviderError(InputProviderError),
}

#[derive(Default)]
pub struct InputManager {
    provider: Box<dyn InputProvider>,
    actions: SlotMap<InputActionId, InputAction>,
    axis: SlotMap<InputAxisId, InputAxis>,
    texts: SlotMap<InputTextHandle, InputText>,
}

impl InputManager {
    pub fn set_provider(&mut self, provider: Box<dyn InputProvider>) {
        self.provider.on_disconnect();
        self.provider = provider;
        self.provider.on_connect();
    }

    /// Reset action states and mouse motion
    pub fn prepare_dispatch(&mut self) {
        // Save the previous action state
        for action in self.actions.values_mut() {
            action.state.was_pressed = action.state.pressed;
        }
        // Reset text for current frame
        for text in self.texts.values_mut() {
            text.state.value.clear();
        }
    }

    /// Process input events
    pub fn dispatch_events(&mut self) {
        while let Some(event) = self.provider.next_event() {
            match event {
                InputEvent::Action(event) => {
                    if let Some(action) = self.actions.get_mut(event.action) {
                        action.state.pressed = event.pressed;
                    }
                }
                InputEvent::Axis(event) => {
                    if let Some(axis) = self.axis.get_mut(event.axis) {
                        axis.set_value(event.value);
                    }
                }
                InputEvent::Text(event) => {
                    todo!()
                }
            }
        }
    }

    pub fn add_action(&mut self, name: &str) -> Result<InputActionId, InputError> {
        if self.actions.values().any(|action| action.name == name) {
            return Err(InputError::DuplicatedAction);
        }
        let id = self.actions.add(InputAction {
            name: name.into(),
            state: InputActionState::default(),
            handle: Default::default(),
        });
        let phandle = self
            .provider
            .add_action(name, id)
            .map_err(InputError::ProviderError)?;
        self.actions[id].handle = phandle;
        Ok(id)
    }

    pub fn remove_action(&mut self, id: InputActionId) -> Result<(), InputError> {
        if !self.actions.contains(id) {
            return Err(InputError::ActionNotFound);
        }
        let phandle = self.actions[id].handle;
        self.provider
            .remove_action(phandle)
            .map_err(InputError::ProviderError)?;
        self.actions.remove(id);
        Ok(())
    }

    pub fn action(&self, id: InputActionId) -> Option<&InputAction> {
        self.actions.get(id)
    }

    pub fn add_axis(
        &mut self,
        name: &str,
        range: InputAxisRange,
    ) -> Result<InputAxisId, InputError> {
        if self.axis.values().any(|axis: &InputAxis| axis.name == name) {
            return Err(InputError::DuplicatedAxis);
        }
        let id = self.axis.add(InputAxis {
            name: name.into(),
            range,
            state: InputAxisState::default(),
            handle: Default::default(),
        });
        let phandle = self
            .provider
            .add_axis(name, &range, id)
            .map_err(InputError::ProviderError)?;
        self.axis[id].handle = phandle;
        Ok(id)
    }

    pub fn remove_axis(&mut self, id: InputAxisId) -> Result<(), InputError> {
        if !self.axis.contains(id) {
            return Err(InputError::AxisNotFound);
        }
        let phandle = self.axis[id].handle;
        self.provider
            .remove_axis(phandle)
            .map_err(InputError::ProviderError)?;
        self.axis.remove(id);
        Ok(())
    }

    pub fn axis(&self, id: InputAxisId) -> Option<&InputAxis> {
        self.axis.get(id)
    }
}
