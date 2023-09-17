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

    fn update_table(
        &mut self,
        uid: mini3d::utils::uid::UID,
        table: Option<&mini3d::feature::component::input::input_table::InputTable>,
    ) -> Result<(), mini3d::input::provider::InputProviderError> {
        self.0.borrow_mut().update_table(uid, table)
    }
}
