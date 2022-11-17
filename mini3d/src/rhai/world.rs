use rhai::plugin::*;

use crate::process::ProcessContext;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub(crate) struct WorldHandle(usize);

impl From<&mut ProcessContext<'_, '_>> for WorldHandle {
    fn from(input: &mut ProcessContext) -> Self {
        Self::new(input)
    }
}

impl<'a, 'b> AsMut<ProcessContext<'a, 'b>> for WorldHandle {
    fn as_mut(&mut self) -> &mut ProcessContext<'a, 'b> {
        unsafe { std::mem::transmute(self.0) }
    }
}

impl WorldHandle {

    fn new(input: &mut ProcessContext) -> Self {
        let handle = unsafe { std::mem::transmute(input) };
        Self(handle)
    }
}

#[export_module]
pub mod rhai_world_api {

    #[rhai_fn(pure)]
    pub(crate) fn action_pressed(_input: &mut WorldHandle) -> bool {
        // let context: &mut ProgramContext = input.as_mut();
        // let group = context.input.find_group(group).unwrap().id;
        // context.input.find_action(group, name).unwrap().is_pressed()
        false
    }
}

