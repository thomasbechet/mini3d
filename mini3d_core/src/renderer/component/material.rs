use mini3d_derive::{Reflect, Serialize};

use crate::{
    ecs::{
        component::{
            Component, ComponentContext, ComponentError, ComponentStorage, EntityResolver,
        },
        context::Context,
        entity::Entity,
    },
    math::vec::V2I32F16,
    renderer::provider::RendererProviderHandle,
};

use super::Texture;

#[derive(Default, Reflect, Serialize, Clone)]
pub(crate) enum MaterialType {
    #[default]
    Opaque,
    Transparent,
}

#[derive(Default, Clone, Serialize, Reflect)]
pub(crate) struct MaterialData {
    pub(crate) ty: MaterialType,
    pub(crate) tex0: Entity,
    pub(crate) tex1: Entity,
    pub(crate) uv0_offset: V2I32F16,
    pub(crate) uv0_scale: V2I32F16,
    pub(crate) uv1_offset: V2I32F16,
    pub(crate) uv1_scale: V2I32F16,
}

#[derive(Default, Clone, Serialize, Reflect)]
pub struct Material {
    pub(crate) data: MaterialData,
    #[serialize(skip)]
    pub(crate) handle: RendererProviderHandle,
}

impl Material {
    pub const NAME: &'static str = "material";

    pub fn set_texture0(&mut self, ctx: &mut Context, texture: &Texture) {
        self.tex0 = texture;
    }
}

impl Component for Material {
    const STORAGE: ComponentStorage = ComponentStorage::Single;
    fn resolve_entities(&mut self, resolver: &mut EntityResolver) -> Result<(), ComponentError> {
        Ok(())
    }
    fn on_added(&mut self, entity: Entity, ctx: ComponentContext) -> Result<(), ComponentError> {
        self.handle = ctx.renderer.add_material(entity, &self.data)?;
        Ok(())
    }
    fn on_removed(&mut self, entity: Entity, ctx: ComponentContext) -> Result<(), ComponentError> {
        ctx.renderer.remove_material(self.handle)
    }
}
