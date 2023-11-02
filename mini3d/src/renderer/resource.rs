use crate::resource::handle::ResourceTypeHandle;

pub mod buffer;
pub mod camera;
pub mod canvas;
pub mod font;
pub mod graph;
pub mod material;
pub mod mesh;
pub mod pass;
pub mod pipeline;
pub mod static_mesh;
pub mod system;
pub mod texture;
pub mod tilemap;
pub mod tileset;
pub mod viewport;

pub(crate) struct RendererResources {
    pub(crate) texture: ResourceTypeHandle,
    pub(crate) material: ResourceTypeHandle,
    pub(crate) mesh: ResourceTypeHandle,
    pub(crate) font: ResourceTypeHandle,
}
