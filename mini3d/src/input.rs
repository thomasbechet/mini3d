use mini3d_derive::Error;

use crate::feature::input::action::InputAction;
use crate::feature::input::axis::InputAxis;
use crate::feature::input::text::InputText;
use crate::resource::handle::{ResourceHandle, ResourceTypeHandle};
use crate::resource::hook::InputResourceHook;
use crate::resource::ResourceManager;
use crate::serialize::{Decoder, DecoderError};
use crate::serialize::{Encoder, EncoderError};
use crate::utils::uid::ToUID;

use self::event::InputEvent;
use self::handle::{InputActionHandle, InputAxisHandle};
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

#[derive(Default)]
pub struct InputManager {
    provider: Box<dyn InputProvider>,
    action_type: ResourceTypeHandle,
    axis_type: ResourceTypeHandle,
    text_type: ResourceTypeHandle,
}

impl InputManager {
    pub(crate) fn set_provider(&mut self, provider: Box<dyn InputProvider>) {
        self.provider.on_disconnect();
        self.provider = provider;
        self.provider.on_connect();
    }

    /// Reset action states and mouse motion
    pub(crate) fn prepare_dispatch(&mut self, resources: &mut ResourceManager) {
        // Save the previous action state
        for action in resources.iter_mut::<InputAction>(self.action_type) {
            action.state.was_pressed = action.state.pressed;
        }
        // Reset text for current frame
        for text in resources.iter_mut::<InputText>(self.text_type) {
            text.state.value.clear();
        }
    }

    /// Process input events
    pub(crate) fn dispatch_events(&mut self, resources: &mut ResourceManager) {
        while let Some(event) = self.provider.next_event() {
            match event {
                InputEvent::Action(event) => {
                    let action = resources.get_mut_unchecked::<InputAction>(
                        self.action_type,
                        ResourceHandle::from_raw(event.id),
                    );
                    action.state.pressed = event.pressed;
                }
                InputEvent::Axis(event) => {
                    let axis = resources.get_mut_unchecked::<InputAxis>(
                        self.axis_type,
                        ResourceHandle::from_raw(event.id),
                    );
                    axis.set_value(event.value);
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

    pub(crate) fn on_resource_added_hook(
        &mut self,
        hook: InputResourceHook,
        handle: ResourceHandle,
        resources: &mut ResourceManager,
    ) {
        match hook {
            InputResourceHook::Action => {
                let action = resources.get_mut_unchecked::<InputAction>(self.action_type, handle);
                action.state.handle = self.provider.add_action(action, handle.0.raw());
            }
            InputResourceHook::Axis => {
                let axis = resources.get_mut_unchecked::<InputAxis>(self.axis_type, handle);
                axis.state.handle = self.provider.add_axis(axis, handle.0.raw());
            }
            InputResourceHook::Text => todo!(),
        }
    }

    pub(crate) fn on_resource_removed_hook(
        &mut self,
        hook: InputResourceHook,
        handle: ResourceHandle,
        resources: &mut ResourceManager,
    ) {
    }

    pub(crate) fn find_action(
        &self,
        key: impl ToUID,
        resource: &ResourceManager,
    ) -> Option<InputActionHandle, InputError> {
        resource
            .find(self.action_type, key)
            .map(|handle| handle.into())
    }

    pub(crate) fn find_axis(
        &self,
        key: impl ToUID,
        resource: &ResourceManager,
    ) -> Option<InputAxisHandle, InputError> {
        resource
            .find(self.axis_type, key)
            .map(|handle| handle.into())
    }

    pub(crate) fn find_text(
        &self,
        key: impl ToUID,
        resource: &ResourceManager,
    ) -> Option<&InputText> {
        resource
            .find(self.text_type, key)
            .map(|handle| handle.into())
    }
}
