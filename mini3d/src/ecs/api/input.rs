use std::ops::Deref;

use crate::{
    feature::component::input::input_table::InputTable,
    input::{
        server::InputServer, InputActionState, InputAxisState, InputError, InputManager,
        InputTextState,
    },
    utils::uid::UID,
};

pub struct ExclusiveInputAPI<'a> {
    pub(crate) manager: &'a mut InputManager,
    pub(crate) server: &'a mut dyn InputServer,
}

impl<'a> ExclusiveInputAPI<'a> {
    pub fn add_table(&mut self, table: &InputTable) -> Result<(), InputError> {
        self.manager.add_table(self.server, table)
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

impl<'a> Deref for ExclusiveInputAPI<'a> {
    type Target = InputManager;

    fn deref(&self) -> &Self::Target {
        self.manager
    }
}

pub struct ParallelInputAPI<'a> {
    pub(crate) manager: &'a InputManager,
}

impl<'a> ParallelInputAPI<'a> {
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
