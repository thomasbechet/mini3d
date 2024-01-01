use alloc::boxed::Box;
use alloc::vec::Vec;
use mini3d_derive::Error;

use crate::ecs::component::ComponentError;
use crate::ecs::entity::Entity;
use crate::ecs::view::native::single::NativeSingleViewMut;
use crate::serialize::{Decoder, DecoderError};
use crate::serialize::{Encoder, EncoderError};
use crate::utils::uid::{ToUID, UID};

use self::component::{InputAction, InputAxis, InputAxisRange, InputText};
use self::event::InputEvent;
use self::provider::{InputProvider, InputProviderHandle};

pub mod component;
pub mod event;
pub mod provider;

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

#[derive(Default)]
pub(crate) struct InputViews {
    pub(crate) action: NativeSingleViewMut<InputAction>,
    pub(crate) axis: NativeSingleViewMut<InputAxis>,
    pub(crate) text: NativeSingleViewMut<InputText>,
}

#[derive(Default)]
pub struct InputManager {
    provider: Box<dyn InputProvider>,
    pub(crate) views: InputViews,
    pub(crate) active_actions: Vec<(UID, Entity)>,
    pub(crate) active_axis: Vec<(UID, Entity)>,
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
        for action in self.views.action.iter_mut() {
            action.state.was_pressed = action.state.pressed;
        }
        // Reset text for current frame
        for text in self.views.text.iter_mut() {
            text.state.value.clear();
        }
    }

    /// Process input events
    pub(crate) fn dispatch_events(&mut self) {
        while let Some(event) = self.provider.next_event() {
            match event {
                InputEvent::Action(event) => {
                    if let Some(action) = self.views.action.get_mut(event.action) {
                        action.state.pressed = event.pressed;
                    }
                }
                InputEvent::Axis(event) => {
                    if let Some(axis) = self.views.axis.get_mut(event.axis) {
                        axis.set_value(event.value);
                    }
                }
                InputEvent::Text(event) => {
                    todo!()
                }
            }
        }
    }

    pub(crate) fn save_state(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        Ok(())
    }

    pub(crate) fn load_state(&mut self, decoder: &mut impl Decoder) -> Result<(), DecoderError> {
        Ok(())
    }

    pub(crate) fn add_action(
        &mut self,
        name: &str,
        entity: Entity,
    ) -> Result<InputProviderHandle, ComponentError> {
        let uid = name.to_uid();
        if self.active_actions.iter().any(|(x, _)| *x == uid) {
            return Err(ComponentError::DuplicatedEntry);
        }
        let handle = self
            .provider
            .add_action(name, entity.raw())
            .map_err(|_| ComponentError::ProviderError)?;
        self.active_actions.push((uid, entity));
        Ok(handle)
    }

    pub(crate) fn remove_action(
        &mut self,
        name: &str,
        handle: InputProviderHandle,
    ) -> Result<(), ComponentError> {
        let uid = name.to_uid();
        if !self.active_actions.iter().any(|(x, _)| *x == uid) {
            return Err(ComponentError::UnresolvedReference);
        }
        self.provider
            .remove_action(handle)
            .map_err(|_| ComponentError::ProviderError)?;
        self.active_actions.retain(|&(x, _)| x != uid);
        Ok(())
    }

    pub(crate) fn add_axis(
        &mut self,
        name: &str,
        entity: Entity,
        range: &InputAxisRange,
    ) -> Result<InputProviderHandle, ComponentError> {
        let uid = name.to_uid();
        if self.active_axis.iter().any(|(x, _)| *x == uid) {
            return Err(ComponentError::DuplicatedEntry);
        }
        let handle = self
            .provider
            .add_axis(name, range, entity.raw())
            .map_err(|_| ComponentError::ProviderError)?;
        self.active_axis.push((uid, entity));
        Ok(handle)
    }

    pub(crate) fn remove_axis(
        &mut self,
        name: &str,
        handle: InputProviderHandle,
    ) -> Result<(), ComponentError> {
        let uid = name.to_uid();
        if !self.active_axis.iter().any(|(x, _)| *x == uid) {
            return Err(ComponentError::UnresolvedReference);
        }
        self.provider
            .remove_axis(handle)
            .map_err(|_| ComponentError::ProviderError)?;
        self.active_axis.retain(|&(x, _)| x != uid);
        Ok(())
    }
}
