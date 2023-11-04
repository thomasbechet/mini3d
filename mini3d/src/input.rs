use mini3d_derive::Error;

use crate::feature::core::resource::ResourceTypeHandle;
use crate::feature::input::action::{InputAction, InputActionHandle};
use crate::feature::input::axis::{InputAxis, InputAxisHandle};
use crate::feature::input::text::{InputText, InputTextHandle};
use crate::resource::handle::ResourceHandle;
use crate::resource::ResourceManager;
use crate::serialize::{Decoder, DecoderError};
use crate::serialize::{Encoder, EncoderError};
use crate::utils::uid::ToUID;

use self::event::InputEvent;
use self::provider::InputProvider;

pub mod event;
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
pub(crate) struct InputTypes {
    pub(crate) action: ResourceTypeHandle,
    pub(crate) axis: ResourceTypeHandle,
    pub(crate) text: ResourceTypeHandle,
}

#[derive(Default)]
pub struct InputManager {
    provider: Box<dyn InputProvider>,
    pub(crate) types: InputTypes,
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
        for action in resources.iter_native_values_mut::<InputAction>(self.types.action) {
            action.state.was_pressed = action.state.pressed;
        }
        // Reset text for current frame
        for text in resources.iter_native_values_mut::<InputText>(self.types.text) {
            text.state.value.clear();
        }
    }

    /// Process input events
    pub(crate) fn dispatch_events(&mut self, resource: &mut ResourceManager) {
        while let Some(event) = self.provider.next_event() {
            match event {
                InputEvent::Action(event) => {
                    let action = resource
                        .get_mut_unchecked::<InputAction>(ResourceHandle::from_raw(event.id));
                    action.state.pressed = event.pressed;
                }
                InputEvent::Axis(event) => {
                    let axis =
                        resource.get_mut_unchecked::<InputAxis>(ResourceHandle::from_raw(event.id));
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

    pub(crate) fn on_action_added(
        &mut self,
        handle: InputActionHandle,
        resources: &mut ResourceManager,
    ) {
        let action = resources.get_mut_unchecked::<InputAction>(handle);
        action.state.handle = self
            .provider
            .add_action(action, handle.0 .0.raw())
            .expect("Input provider failed to add action");
    }

    pub(crate) fn on_axis_added(
        &mut self,
        handle: InputAxisHandle,
        resources: &mut ResourceManager,
    ) {
        let axis = resources.get_mut_unchecked::<InputAxis>(handle);
        axis.state.handle = self
            .provider
            .add_axis(axis, handle.0 .0.raw())
            .expect("Input provider failed to add axis");
    }

    pub(crate) fn on_action_removed(
        &mut self,
        handle: InputActionHandle,
        resources: &ResourceManager,
    ) {
        let action = resources.get_unchecked::<InputAction>(handle);
        self.provider
            .remove_action(action.state.handle)
            .expect("Input provider failed to remove action");
    }

    pub(crate) fn on_axis_removed(&mut self, handle: InputAxisHandle, resources: &ResourceManager) {
        let axis = resources.get_unchecked::<InputAxis>(handle);
        self.provider
            .remove_axis(axis.state.handle)
            .expect("Input provider failed to remove axis");
    }

    pub(crate) fn find_action(
        &self,
        key: impl ToUID,
        resource: &ResourceManager,
    ) -> Option<InputActionHandle> {
        resource.find_typed(key, self.types.action)
    }

    pub(crate) fn find_axis(
        &self,
        key: impl ToUID,
        resource: &ResourceManager,
    ) -> Option<InputAxisHandle> {
        resource.find_typed(key, self.types.axis)
    }

    pub(crate) fn find_text(
        &self,
        key: impl ToUID,
        resource: &ResourceManager,
    ) -> Option<InputTextHandle> {
        resource.find_typed(key, self.types.text)
    }
}
