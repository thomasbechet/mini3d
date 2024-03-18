use std::path::{Path, PathBuf};

use mini3d::import::{AssetImportEntry, ImportAssetEvent};
use mini3d_renderer::texture::{TextureData, TextureFormat};

pub struct ImageImport {
    texture: AssetImportEntry<TextureData>,
}

impl ImageImport {
    pub fn push(self, events: &mut Vec<ImportAssetEvent>) {
        events.push(ImportAssetEvent::Texture(self.texture));
    }
}

#[derive(Default)]
pub struct ImageImporter {
    name: Option<String>,
    path: PathBuf,
}

impl ImageImporter {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn from_source(&mut self, path: &Path) -> &mut Self {
        self.path = path.into();
        self
    }

    pub fn with_name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn import(&self) -> Result<ImageImport, String> {
        // Find the resource name either from the user defined name or the source
        let filename = self
            .name
            .clone()
            .or_else(|| {
                self.path
                    .file_stem()
                    .map(|n| n.to_owned().into_string().unwrap())
            })
            .ok_or_else(|| "Failed to get name from path (no name provided)".to_string())?;
        // Load the image
        let image = image::open(&self.path).map_err(|err| format!("Failed open image: {err}"))?;
        // Convert to rgba8
        let data = image.to_rgba8();
        // Build the texture
        let texture = TextureData::new(
            TextureFormat::Color,
            data.to_vec(),
            image.width() as u16,
            image.height() as u16,
        );
        // Return the texture import
        Ok(ImageImport {
            texture: AssetImportEntry::<TextureData> {
                data: texture,
                name: filename.into(),
            },
        })
    }
}
