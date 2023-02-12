use anyhow::Result;

use crate::{input::{InputManager, InputActionState, InputAxisState}, uid::UID};

pub struct InputContext<'a> {
    input: &'a InputManager,
}

impl<'a> InputContext<'a> {

    pub(crate) fn new(input: &InputManager) -> Self {
        Self { input }
    }

    pub fn text(&self) -> &str {
        self.input.text()        
    }

    pub fn action(&self, uid: UID) -> Result<&InputActionState> {
        self.input.action(uid)
    }

    pub fn axis(&self, uid: UID) -> Result<&InputAxisState> {
        self.input.axis(uid)
    }
}