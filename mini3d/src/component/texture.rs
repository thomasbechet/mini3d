use crate::{self as mini3d, api::API, system::SystemParam};
use mini3d_db::database::{ComponentHandle, GetComponentHandle};
use mini3d_renderer::texture::TextureId;

pub struct Texture(ComponentHandle);

impl SystemParam for Texture {
    fn resolve(db: &mini3d_db::database::Database) -> Self {
        Self(db.find_component(Self::NAME).unwrap())
    }
}

impl GetComponentHandle for &Texture {
    fn handle(&self) -> ComponentHandle {
        self.0
    }
}

impl Texture {
    pub const NAME: &'static str = "texture";

    pub fn register(api: &mut API) {
        let handle = api.register_component_key(Self::NAME);
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
    let key = api
        .renderer
        .create_texture(Default::default())
        .unwrap()
        .key();
    api.write_key(api.event_entity(), texture, key);
}

fn delete_texture(api: &mut API, texture: &Texture) {
    let key = TextureId::from_key(api.read_key(api.event_entity(), texture).unwrap());
    api.renderer.delete_texture(key).unwrap();
}
