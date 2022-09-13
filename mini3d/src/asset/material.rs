use super::{Asset, TextureId};

#[derive(Default)]
pub struct Material {
    pub diffuse: TextureId,
}

impl Asset for Material {
    fn typename() -> &'static str {
        "material"
    }

    fn default() -> Self {
        Default::default()
    }
}