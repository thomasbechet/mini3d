use mini3d_derive::{Reflect, Serialize};

use crate::{
    ecs::{
        component::{Component, ComponentError, ComponentStorage},
        context::Context,
        entity::Entity,
    },
    input::component::InputAxis,
    math::mat::M4I32F16,
    renderer::provider::RendererProviderHandle,
};

#[derive(Clone, Serialize, Default, Reflect)]
pub struct RenderTransform {
    #[serialize(skip)]
    pub(crate) handle: RendererProviderHandle,
}

impl RenderTransform {
    pub fn update(ctx: &mut Context, value: M4I32F16, teleport: bool) {
        todo!()
    }

    pub fn bind_axis(&mut self, ctx: &mut Context, x_axis: &InputAxis, y_axis: &InputAxis) {
        todo!()
    }
}

impl Component for RenderTransform {
    const STORAGE: ComponentStorage = ComponentStorage::List;
    fn on_added(&mut self, entity: Entity, ctx: &mut Context) -> Result<(), ComponentError> {
        self.handle = ctx.renderer.add_transform(entity)?;
        Ok(())
    }
    fn on_removed(&mut self, entity: Entity, ctx: &mut Context) -> Result<(), ComponentError> {
        ctx.renderer.remove_transform(self.handle)?;
        Ok(())
    }
}
