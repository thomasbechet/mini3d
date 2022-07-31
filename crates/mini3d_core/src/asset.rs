use std::collections::HashMap;

use self::font::Font;

pub mod font;

type AssetName = &'static str;
type AssetID = u16;

pub struct Asset<R> {
    name: AssetName,
    id: AssetID,
    resource: R,
}

pub struct AssetManager {
    fonts: HashMap<AssetID, Asset<Font>>,
}

impl AssetManager {
    pub fn new() -> Self {
        AssetManager { fonts: HashMap::new() }
    }

    pub fn dispatch_event(&mut self, event: &AssetImportEvent) {
        
    }
}

pub trait AssetImporter<R> {
    fn get_asset(&self) -> R;
    fn get_name(&self) -> &'static str;
} 

pub enum AssetImportEvent {
    Font(Box<dyn AssetImporter<Font>>),
}