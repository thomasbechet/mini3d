use std::{cell::RefCell, rc::Rc};

use mini3d::input::provider::InputProvider;

use crate::mapper::InputMapper;

pub(crate) struct WinitInputProvider(Rc<RefCell<InputMapper>>);

impl WinitInputProvider {
    pub(crate) fn new(mapper: Rc<RefCell<InputMapper>>) -> Self {
        Self(mapper)
    }
}

impl InputProvider for WinitInputProvider {
    fn on_connect(&mut self) {}
    fn on_disconnect(&mut self) {}

    fn next_event(&mut self) -> Option<mini3d::input::event::InputEvent> {
        self.0.borrow_mut().next_event()
    }

    fn add_action(&mut self, id: u32, action: &mini3d::feature::input::action::InputAction) {
        self.0.borrow_mut().add_action(id, action);
    }

    fn add_axis(&mut self, id: u32, axis: &mini3d::feature::input::axis::InputAxis) {
        self.0.borrow_mut().add_axis(id, axis);
    }
}
