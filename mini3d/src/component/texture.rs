use crate::{self as mini3d, api::API};
use mini3d_db::entity::Entity;
use mini3d_derive::component;
use mini3d_renderer::texture::{TextureData, TextureHandle};

#[component]
pub struct Texture {
    handle: TextureHandle,
}

impl Texture {
    pub fn create_callbacks(&self, api: &mut API) {
        api.create_system(
            "delete_texture",
            api.on_component_removed_stage(self.id()),
            Default::default(),
            delete_texture,
        ).unwrap();
    }

    pub fn add_default(&self, api: &mut API, e: Entity) {
        let texture = api.create_texture(TextureData::default());
        api.add_default(e, self.id());
        api.write(e, self.handle, texture);
    }
}

fn delete_texture(api: &mut API) {
    let texture = Texture::meta(api);
    let handle = api.read(api.event_entity(), texture.handle).unwrap();
    api.delete_texture(handle);
}
