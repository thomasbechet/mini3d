use crate::{api::API, event::Event, handle_component};
use mini3d_db::entity::Entity;
use mini3d_renderer::texture::TextureId;

handle_component!(Texture, TextureId, "texture");

impl Texture {
    pub fn register(api: &mut API) {
        let handle = api.register_component_handle(Self::NAME);
        api.register_system(
            "delete_texture",
            Event::ComponentRemoved(handle),
            Default::default(),
            delete_texture,
        );
    }

    pub fn add(&self, api: &mut API, e: Entity) {
        let handle = api.renderer.create_texture(Default::default()).unwrap();
        api.add_default(e, self);
        api.write_handle(e, self, handle);
    }
}

fn delete_texture(api: &mut API, texture: &Texture) {
    if let Some(handle) = texture.handle(api, api.event_entity()) {
        api.renderer.delete_texture(handle).unwrap();
    }
}
