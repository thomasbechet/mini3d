use super::event::SystemEvent;

pub trait SystemBackend {
    fn events(&self) -> &[SystemEvent] {
        &[]
    }
}
