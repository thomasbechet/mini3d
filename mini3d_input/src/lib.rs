#![no_std]

use action::{InputAction, InputActionHandle, InputActionState};
use alloc::boxed::Box;
use axis::{InputAxis, InputAxisHandle, InputAxisRange, InputAxisState};
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
    actions: SlotMap<InputActionHandle, InputAction>,
    axis: SlotMap<InputAxisHandle, InputAxis>,
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

    pub(crate) fn add_action(&mut self, name: &str) -> Result<InputActionHandle, InputError> {
        if self.actions.values().any(|action| action.name == name) {
            return Err(InputError::DuplicatedAction);
        }
        let handle = self.actions.add(InputAction {
            name: name.into(),
            state: InputActionState::default(),
            handle: Default::default(),
        });
        let phandle = self
            .provider
            .add_action(name, handle)
            .map_err(|e| InputError::ProviderError(e))?;
        self.actions[handle].handle = phandle;
        Ok(handle)
    }

    pub(crate) fn remove_action(&mut self, handle: InputActionHandle) -> Result<(), InputError> {
        if !self.actions.contains(handle) {
            return Err(InputError::ActionNotFound);
        }
        let phandle = self.actions[handle].handle;
        self.provider
            .remove_action(phandle)
            .map_err(|e| InputError::ProviderError(e))?;
        self.actions.remove(handle);
        Ok(())
    }

    pub(crate) fn add_axis(
        &mut self,
        name: &str,
        range: InputAxisRange,
    ) -> Result<InputAxisHandle, InputError> {
        if self.axis.values().any(|axis: &InputAxis| axis.name == name) {
            return Err(InputError::DuplicatedAxis);
        }
        let handle = self.axis.add(InputAxis {
            name: name.into(),
            range: range.clone(),
            state: InputAxisState::default(),
            handle: Default::default(),
        });
        let phandle = self
            .provider
            .add_axis(name, &range, handle)
            .map_err(|e| InputError::ProviderError(e))?;
        self.axis[handle].handle = phandle;
        Ok(handle)
    }

    pub(crate) fn remove_axis(&mut self, handle: InputAxisHandle) -> Result<(), InputError> {
        if !self.axis.contains(handle) {
            return Err(InputError::AxisNotFound);
        }
        let phandle = self.axis[handle].handle;
        self.provider
            .remove_axis(phandle)
            .map_err(|e| InputError::ProviderError(e))?;
        self.axis.remove(handle);
        Ok(())
    }
}
