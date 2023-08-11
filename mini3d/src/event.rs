use self::{
    asset::ImportAssetEvent, input::InputEvent, network::NetworkEvent, storage::StorageEvent,
    system::SystemEvent,
};

pub mod asset;
pub mod input;
pub mod network;
pub mod storage;
pub mod system;

#[derive(Default)]
pub struct Events {
    pub asset: Vec<ImportAssetEvent>,
    pub input: Vec<InputEvent>,
    pub system: Vec<SystemEvent>,
    pub network: Vec<NetworkEvent>,
    pub disk: Vec<StorageEvent>,
}

impl Events {
    pub fn new() -> Self {
        Self {
            asset: Default::default(),
            input: Default::default(),
            system: Default::default(),
            network: Default::default(),
            disk: Default::default(),
        }
    }

    pub fn clear(&mut self) -> &mut Self {
        self.asset.clear();
        self.input.clear();
        self.system.clear();
        self.network.clear();
        self.disk.clear();
        self
    }
}
