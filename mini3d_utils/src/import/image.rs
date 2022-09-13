use std::path::{Path, PathBuf};

use mini3d::{event::asset::{AssetImport, AssetEvent, ImportAssetEvent}, asset::{texture::Texture, AssetName}, application::Application};

pub struct ImageImport {
    texture: AssetImport<Texture>
}

impl ImageImport {
    pub fn push_events(self, app: &mut Application) {
        app.events.push_asset(AssetEvent::Import(ImportAssetEvent::Texture(self.texture)));
    }
}

#[derive(Default)]
pub struct ImageImporter {
    name: Option<AssetName>,
    path: PathBuf,
}

impl ImageImporter {

    pub fn new() -> Self {
        Self { ..Default::default() }
    }

    pub fn from_source(&mut self, path: &Path) -> &mut Self {
        self.path = path.into();
        self
    }

    pub fn with_name(&mut self, name: AssetName) -> &mut Self {
        self.name = Some(name);
        self
    }

    pub fn import(&self) -> Result<ImageImport, String> {
        // Find the asset name either from the user defined name or the source
        let filename = self.name.clone().or(
            self.path.file_stem().map(|n| n.to_owned().into_string().unwrap().into())
        ).ok_or("Failed to get name from path (no name provided)".to_string())?;
        // Load the image
        let image = image::open(&self.path)
            .map_err(|err| format!("Failed open image: {err}").to_string())?;
        // Convert to rgba8
        let data = image.to_rgb8();
        // Build the texture
        let texture = Box::new(Texture {
            data: data.to_vec(),
            width: image.width(),
            height: image.height(),
        });
        // Return the texture import
        Ok(ImageImport {
            texture: AssetImport::<Texture> {
                data: texture,
                name: filename
            }
        })
    }
}