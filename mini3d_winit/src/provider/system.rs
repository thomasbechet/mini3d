use std::{cell::RefCell, rc::Rc};

use mini3d::runtime::provider::RuntimeProvider;

use crate::WinitSystemStatus;

pub(crate) struct WinitSystemProvider(Rc<RefCell<WinitSystemStatus>>);

impl WinitSystemProvider {
    pub(crate) fn new(status: Rc<RefCell<WinitSystemStatus>>) -> Self {
        Self(status)
    }
}

impl RuntimeProvider for WinitSystemProvider {
    fn on_connect(&mut self) {}
    fn on_disconnect(&mut self) {}

    fn next_import(&mut self) -> Option<mini3d::runtime::event::ImportAssetEvent> {
        self.0.borrow_mut().next_import()
    }

    fn next_event(&mut self) -> Option<mini3d::runtime::event::RuntimeEvent> {
        if self.0.borrow().stop_event {
            self.0.borrow_mut().stop_event = false;
            return Some(mini3d::runtime::event::RuntimeEvent::RequestStop);
        }
        None
    }

    fn request_stop(&mut self) {
        self.0.borrow_mut().request_stop();
    }
}
