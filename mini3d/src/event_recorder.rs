use crate::{input::event::InputEvent, asset::AssetImportEvent};

pub enum SystemEvent {
    CloseRequested,
    AssetImport(AssetImportEvent),
}

#[derive(Default)]
pub struct EventRecorder {
    pub input_events: Vec<InputEvent>,
    pub system_events: Vec<SystemEvent>,
}

impl EventRecorder {
    pub fn push_input_event(&mut self, event: InputEvent) {
        self.input_events.push(event);
    }
    
    pub fn push_system_event(&mut self, event: SystemEvent) {
        self.system_events.push(event);
    }

    pub fn reset(&mut self) {
        self.input_events.clear();
        self.system_events.clear();
    }
}