use crate::{self as mini3d, api::API};
use mini3d_derive::component;
use mini3d_renderer::texture::TextureId;

#[component]
pub struct Texture {
    handle: u32,
}

impl Texture {
    pub fn register_callbacks(api: &mut API) {
        let handle = api.find_component(Self::NAME).unwrap();
        api.register_system(
            "create_texture",
            api.on_component_added_stage(handle),
            Default::default(),
            create_texture,
        );
        api.register_system(
            "delete_texture",
            api.on_component_removed_stage(handle),
            Default::default(),
            delete_texture,
        );
    }
}

fn create_texture(api: &mut API, texture: &Texture) {
    let handle = api.renderer.create_texture(Default::default()).unwrap();
    api.write(api.event_entity(), texture.handle, handle.raw());
}

fn delete_texture(api: &mut API, texture: &Texture) {
    let handle = api.read(api.event_entity(), texture.handle).unwrap();
    if handle != 0 {
        api.renderer
            .delete_texture(TextureId::from_raw(handle))
            .unwrap();
    }
}
