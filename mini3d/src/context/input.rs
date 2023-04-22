use crate::{input::{InputManager, InputActionState, InputAxisState, InputTextState, InputError}, uid::UID, feature::asset::input_table::InputTable};

pub struct InputContext<'a> {
    pub(crate) manager: &'a mut InputManager,
}

impl<'a> InputContext<'a> {

    pub fn add_table(&mut self, table: &InputTable) -> Result<(), InputError> {
        self.manager.add_table(table)
    }

    pub fn action(&self, uid: UID) -> Result<&InputActionState, InputError> {
        self.manager.action(uid)
    }

    pub fn axis(&self, uid: UID) -> Result<&InputAxisState, InputError> {
        self.manager.axis(uid)
    }

    pub fn text(&self, uid: UID) -> Result<&InputTextState, InputError> {
        self.manager.text(uid)
    }
}