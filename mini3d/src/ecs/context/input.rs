use crate::{
    feature::component::input::input_table::InputTable,
    input::{InputActionState, InputAxisState, InputError, InputManager, InputTextState},
    utils::uid::UID,
};

pub struct ExclusiveInputContext<'a> {
    pub(crate) manager: &'a mut InputManager,
}

impl<'a> ExclusiveInputContext<'a> {
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

pub struct ParallelInputContext<'a> {
    pub(crate) manager: &'a InputManager,
}

impl<'a> ParallelInputContext<'a> {
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
