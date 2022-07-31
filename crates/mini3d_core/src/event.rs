use crate::asset::AssetImportEvent;
pub use crate::input::event::InputEvent;

pub enum Event {
    CloseRequested,
    Input(InputEvent),
    AssetImport(AssetImportEvent),
}