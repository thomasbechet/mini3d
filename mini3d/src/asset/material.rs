use super::{Asset, AssetId};

#[derive(Default)]
pub struct Material {
    pub diffuse: AssetId,
}

impl Asset for Material {
    fn typename() -> &'static str {
        "material"
    }

    fn default() -> Self {
        Default::default()
    }
}