use self::{input::InputEvent, system::SystemEvent, asset::ImportAssetEvent};

pub mod asset;
pub mod input;
pub mod system;

#[derive(Default)]
pub struct Events {
    pub asset: Vec<ImportAssetEvent>,
    pub input: Vec<InputEvent>,
    pub system: Vec<SystemEvent>,
}

impl Events {

    pub fn new() -> Self {
        Self {
            asset: Default::default(),
            input: Default::default(),
            system: Default::default(),
        }
    }

    pub fn clear(&mut self) -> &mut Self {
        self.asset.clear();
        self.input.clear();
        self.system.clear();
        self
    }
}