use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use slotmap::{new_key_type, Key};

use crate::backend::renderer::{RendererMaterialId, RendererBackend, RendererMaterialDescriptor};

use super::{Asset, AssetManager, AssetUID};

new_key_type! { pub struct MaterialId; }

#[derive(Default, Serialize, Deserialize)]
pub struct Material {
    pub diffuse: AssetUID,
}

impl Asset for Material {
    type Id = MaterialId;
    fn typename() -> &'static str { "material" }
}

impl Material {
    pub fn submit(&mut self, asset: &AssetManager, renderer: &mut dyn RendererBackend) -> Result<()> {
        let diffuse = self.diffuse.get(asset).context("Diffuse texture not found")?;
        self.renderer_id = renderer.add_material(&RendererMaterialDescriptor {
            diffuse: diffuse.asset.renderer_id,
        })?;
        Ok(())
    }
    pub fn release(&mut self, renderer: &mut dyn RendererBackend) -> Result<()> {
        renderer.remove_material(self.renderer_id)?;
        self.renderer_id = RendererMaterialId::null();
        Ok(())
    }
    pub fn resolve(&mut self, asset: &AssetManager) -> Result<()> {
        self.diffuse.resolve(asset)?;
        Ok(())
    }
}