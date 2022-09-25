use self::{input::InputEvent, system::SystemEvent, asset::ImportAssetEvent};

pub mod asset;
pub mod input;
pub mod system;

pub struct FrameEvents {
    pub(crate) assets: Vec<ImportAssetEvent>,
    pub(crate) inputs: Vec<InputEvent>,
    pub(crate) systems: Vec<SystemEvent>,
}

impl FrameEvents {

    pub fn new() -> Self {
        Self {
            assets: Default::default(),
            inputs: Default::default(),
            systems: Default::default(),
        }
    }

    pub fn push_asset(&mut self, event: ImportAssetEvent) -> &mut Self {
        self.assets.push(event);
        self
    }

    pub fn push_input(&mut self, event: InputEvent) -> &mut Self {
        self.inputs.push(event);
        self
    }

    pub fn push_system(&mut self, event: SystemEvent) -> &mut Self {
        self.systems.push(event);
        self
    }

    pub fn clear(&mut self) -> &mut Self {
        self.assets.clear();
        self.inputs.clear();
        self.systems.clear();
        self
    }
}