use serde::{Serialize, Deserialize};

use super::{texture::Texture, AssetRef, Asset};

#[derive(Default, Serialize, Deserialize)]
pub struct Material {
    pub diffuse: AssetRef<Texture>,
}

impl Asset for Material {
    fn typename() -> &'static str { "material" }
}