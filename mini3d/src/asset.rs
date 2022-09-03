use std::collections::HashMap;

use self::font::Font;

pub mod font;

pub type AssetName = &'static str;
pub type AssetID = u16;

pub struct Asset<R> {
    pub name: AssetName,
    pub id: AssetID,
    pub resource: R,
}

#[derive(Default)]
pub struct AssetManager {
    #[allow(dead_code)]
    pub fonts: HashMap<AssetID, Asset<Font>>,
}

impl AssetManager {
    pub fn dispatch_event(&mut self, _event: &AssetImportEvent) {
        
    }
}

pub trait AssetImporter<R> {
    fn get_asset(&self) -> R;
    fn get_name(&self) -> &'static str;
} 

pub enum AssetImportEvent {
    Font(Box<dyn AssetImporter<Font>>),
}