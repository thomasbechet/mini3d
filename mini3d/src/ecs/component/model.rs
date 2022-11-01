use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use slotmap::Key;

use crate::{backend::renderer::{RendererModelId, RendererBackend, RendererMaterialId, RendererModelDescriptor}, asset::{AssetRef, model::Model, AssetManager}};

#[derive(Serialize, Deserialize)]
pub struct ModelComponent {
    pub model: AssetRef<Model>,
    #[serde(skip)]
    pub renderer_id: RendererModelId,
}

impl ModelComponent {
    pub fn submit(&mut self, asset: &AssetManager, renderer: &mut dyn RendererBackend) -> Result<()> {
        let model = self.model.get(asset)
            .context("Model asset not found")?;
        let mut materials: Vec<RendererMaterialId> = Vec::with_capacity(model.asset.materials.len());
        for material in &model.asset.materials {
            let material = material.get(asset).context("Material asset not found")?;
            materials.push(material.asset.renderer_id);
        }
        let mesh = model.asset.mesh.get(asset).context("Mesh asset not found")?;
        self.renderer_id = renderer.add_model(&RendererModelDescriptor {
            mesh: mesh.asset.renderer_id,
            materials: &materials,
        })?;
        Ok(())
    }
    pub fn release(&mut self, renderer: &mut dyn RendererBackend) -> Result<()> {
        renderer.remove_model(self.renderer_id)?;
        self.renderer_id = RendererModelId::null();
        Ok(())
    }
}

impl From<AssetRef<Model>> for ModelComponent {
    fn from(model: AssetRef<Model>) -> Self {
        Self { model, renderer_id: RendererModelId::null() }
    }
}