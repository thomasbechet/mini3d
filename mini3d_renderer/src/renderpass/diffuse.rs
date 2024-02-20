use mini3d_derive::{Reflect, Serialize};

use crate::{
    ecs::context::Context,
    math::vec::{V2I32, V3I32F16},
    renderer::{
        component::{Camera, Material, Mesh, RenderTransform, Texture},
        provider::RendererProviderHandle,
    },
};

pub enum TextureRenderTarget<'a> {
    Texture(&'a Texture),
    CubeMap(&'a Texture),
}

pub enum DiffusePassCommand {
    DrawMesh {
        mesh: RendererProviderHandle,
        material: RendererProviderHandle,
        transform: RendererProviderHandle,
    },
    DrawMeshSkinned {
        mesh: RendererProviderHandle,
        material: RendererProviderHandle,
        transform: RendererProviderHandle,
    },
    DrawBillboard,
    PushPointLight {
        transform: RendererProviderHandle,
        color: V3I32F16,
    },
}

pub struct DiffusePassRenderInfo<'a> {
    pub camera: &'a Camera,
    pub target: TextureRenderTarget<'a>,
}

pub struct DiffusePassInfo {
    pub per_vertex_lighting: bool,
    pub max_point_lights: u8,
    pub max_spot_lights: u8,
    pub max_directional_lights: u8,
}

#[derive(Default, Reflect, Serialize)]
pub struct DiffusePass {
    #[serialize(skip)]
    pub(crate) handle: RendererProviderHandle,
}

impl DiffusePass {
    pub fn render(&mut self, ctx: &mut Context, info: &DiffusePassRenderInfo) {
        todo!()
    }

    pub fn add_point_light(ctx: &mut Context, position: V2I32) {}

    pub fn draw_mesh(
        ctx: &mut Context,
        mesh: &Mesh,
        material: &Material,
        transform: &RenderTransform,
        sort: u32,
    ) {
    }
}
