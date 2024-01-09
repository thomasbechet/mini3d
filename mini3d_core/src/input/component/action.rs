use mini3d_derive::{Reflect, Serialize};

use crate::{
    ecs::{
        component::{Component, ComponentError, ComponentStorage},
        context::Context,
        entity::Entity,
    },
    input::provider::InputProviderHandle,
    utils::string::AsciiArray,
};

#[derive(Default, Clone, Serialize)]
pub struct InputActionState {
    pub(crate) pressed: bool,
    pub(crate) was_pressed: bool,
}

#[derive(Clone, Reflect, Default, Serialize)]
pub struct InputAction {
    pub(crate) name: AsciiArray<32>,
    pub(crate) state: InputActionState,
    #[serialize(skip)]
    pub(crate) handle: InputProviderHandle,
}

impl InputAction {
    pub const NAME: &'static str = "input_action";

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn is_pressed(&self) -> bool {
        self.state.pressed
    }

    pub fn is_released(&self) -> bool {
        !self.state.pressed
    }

    pub fn is_just_pressed(&self) -> bool {
        self.state.pressed && !self.state.was_pressed
    }

    pub fn is_just_released(&self) -> bool {
        !self.state.pressed && self.state.was_pressed
    }

    pub fn enable(&mut self, ctx: &mut Context) {}
}

impl Component for InputAction {
    const STORAGE: ComponentStorage = ComponentStorage::Single;

    fn on_added(&mut self, entity: Entity, ctx: &mut Context) -> Result<(), ComponentError> {
        self.handle = ctx.input.add_action(self.name.as_str(), entity)?;
        Ok(())
    }

    fn on_removed(&mut self, entity: Entity, ctx: &mut Context) -> Result<(), ComponentError> {
        ctx.input.remove_action(&self.name, self.handle)?;
        Ok(())
    }
}
