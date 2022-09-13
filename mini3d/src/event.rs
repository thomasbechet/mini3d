use self::{asset::AssetEvent, input::InputEvent, system::SystemEvent};

pub mod asset;
pub mod input;
pub mod system;

#[derive(Default)]
pub struct EventManager {
    pub(crate) assets: Vec<AssetEvent>,
    pub(crate) inputs: Vec<InputEvent>,
    pub(crate) systems: Vec<SystemEvent>,
}

impl EventManager {
    
    pub fn push_asset(&mut self, event: AssetEvent) {
        self.assets.push(event);
    }
    
    pub fn push_input(&mut self, event: InputEvent) {
        self.inputs.push(event);
    }
    
    pub fn push_system(&mut self, event: SystemEvent) {
        self.systems.push(event);
    }
}