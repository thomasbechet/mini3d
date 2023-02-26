use anyhow::Result;

use crate::{input::{InputManager, InputActionState, InputAxisState}, uid::UID};

pub struct InputContext<'a> {
    pub(crate) manager: &'a InputManager,
}

impl<'a> InputContext<'a> {

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