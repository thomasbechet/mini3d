#![no_std]

use action::{InputAction, InputActionHandle, InputActionState};
use alloc::boxed::Box;
use axis::{InputAxis, InputAxisHandle, InputAxisRange, InputAxisState};
use event::InputEvent;
use mini3d_derive::Error;
use mini3d_utils::slotmap::SlotMap;
use provider::{InputProvider, InputProviderError};
use text::{InputText, InputTextId};

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
    texts: SlotMap<InputTextId, InputText>,
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
                InputEvent::Text(_event) => {
                    todo!()
                }
            }
        }
    }

    pub fn create_action(&mut self, name: &str) -> Result<InputActionHandle, InputError> {
        if self.find_action(name).is_some() {
            return Err(InputError::DuplicatedAction);
        }
        let id = self.actions.add(InputAction {
            name: name.into(),
            state: InputActionState::default(),
            handle: Default::default(),
        });
        let phandle = self
            .provider
            .create_action(name, id)
            .map_err(InputError::ProviderError)?;
        self.actions[id].handle = phandle;
        Ok(id)
    }

    pub fn delete_action(&mut self, handle: InputActionHandle) -> Result<(), InputError> {
        if !self.actions.contains(handle) {
            return Err(InputError::ActionNotFound);
        }
        let phandle = self.actions[handle].handle;
        self.provider
            .delete_action(phandle)
            .map_err(InputError::ProviderError)?;
        self.actions.remove(handle);
        Ok(())
    }

    pub fn action(&self, handle: InputActionHandle) -> Option<&InputAction> {
        self.actions.get(handle)
    }

    pub fn find_action(&self, name: &str) -> Option<(InputActionHandle, &InputAction)> {
        self.actions.iter().find_map(|(id, action)| {
            if action.name == name {
                Some((id, action))
            } else {
                None
            }
        })
    }

    pub fn create_axis(
        &mut self,
        name: &str,
        range: InputAxisRange,
    ) -> Result<InputAxisHandle, InputError> {
        if self.find_axis(name).is_some() {
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
            .create_axis(name, &range, id)
            .map_err(InputError::ProviderError)?;
        self.axis[id].handle = phandle;
        Ok(id)
    }

    pub fn delete_axis(&mut self, handle: InputAxisHandle) -> Result<(), InputError> {
        if !self.axis.contains(handle) {
            return Err(InputError::AxisNotFound);
        }
        let phandle = self.axis[handle].handle;
        self.provider
            .delete_axis(phandle)
            .map_err(InputError::ProviderError)?;
        self.axis.remove(handle);
        Ok(())
    }

    pub fn axis(&self, handle: InputAxisHandle) -> Option<&InputAxis> {
        self.axis.get(handle)
    }

    pub fn find_axis(&self, name: &str) -> Option<(InputAxisHandle, &InputAxis)> {
        self.axis.iter().find_map(|(id, axis)| {
            if axis.name == name {
                Some((id, axis))
            } else {
                None
            }
        })
    }
}
