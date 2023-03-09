use anyhow::Result;
use crate::{input::{InputManager, InputActionState, InputAxisState}, uid::UID, feature::asset::input_table::InputTable};

pub struct InputContext<'a> {
    pub(crate) manager: &'a mut InputManager,
}

impl<'a> InputContext<'a> {

    pub fn add_table(&mut self, table: &InputTable) -> Result<()> {
        self.manager.add_table(table)
    }

    pub fn text(&self) -> &str {
        self.manager.text()
    }

    pub fn action(&self, uid: UID) -> Result<&InputActionState> {
        self.manager.action(uid)
    }

    pub fn axis(&self, uid: UID) -> Result<&InputAxisState> {
        self.manager.axis(uid)
    }
}