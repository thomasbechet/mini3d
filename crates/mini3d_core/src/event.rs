use crate::asset::AssetImportEvent;
pub use crate::input::event::InputEvent;

pub enum PlatformEvent {
    CloseRequested,
    Input(InputEvent),
    AssetImport(AssetImportEvent),
}

pub enum AppEvent {
    CloseRequested,
    TextRequested,
}