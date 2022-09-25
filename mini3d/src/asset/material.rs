use slotmap::new_key_type;
use crate::asset::texture::TextureId;
use super::Asset;

#[derive(Default)]
pub struct Material {
    pub diffuse: TextureId,
}

new_key_type! { pub struct MaterialId; }

impl Asset for Material {
    type Id = MaterialId;

    fn typename() -> &'static str {
        "material"
    }
}