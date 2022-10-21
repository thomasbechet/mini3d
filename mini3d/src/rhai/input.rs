use rhai::plugin::*;

use crate::program::ProgramContext;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub(crate) struct InputManagerHandle(usize);

impl From<&mut ProgramContext<'_>> for InputManagerHandle {
    fn from(input: &mut ProgramContext) -> Self {
        Self::new(input)
    }
}

impl<'a> AsMut<ProgramContext<'a>> for InputManagerHandle {
    fn as_mut(&mut self) -> &mut ProgramContext<'a> {
        unsafe { std::mem::transmute(self.0) }
    }
}

impl InputManagerHandle {

    fn new(input: &mut ProgramContext) -> Self {
        let handle = unsafe { std::mem::transmute(input) };
        Self(handle)
    }
}

#[export_module]
pub mod rhai_input_api {

    #[rhai_fn(pure)]
    pub(crate) fn action_pressed(input: &mut InputManagerHandle, group: &str, name: &str) -> bool {
        let context: &mut ProgramContext = input.as_mut();
        let group = context.input.find_group(group).unwrap().id;
        context.input.find_action(group, name).unwrap().is_pressed()
    }

    #[rhai_fn(pure)]
    pub(crate) fn action_released(input: &mut InputManagerHandle, group: &str, name: &str) -> bool {
        let context: &mut ProgramContext = input.as_mut();
        let group = context.input.find_group(group).unwrap().id;
        context.input.find_action(group, name).unwrap().is_released()
    }

    #[rhai_fn(pure)]
    pub(crate) fn action_just_pressed(input: &mut InputManagerHandle, group: &str, name: &str) -> bool {
        let context: &mut ProgramContext = input.as_mut();
        let group = context.input.find_group(group).unwrap().id;
        context.input.find_action(group, name).unwrap().is_just_pressed()
    }

    #[rhai_fn(pure)]
    pub(crate) fn action_just_released(input: &mut InputManagerHandle, group: &str, name: &str) -> bool {
        let context: &mut ProgramContext = input.as_mut();
        let group = context.input.find_group(group).unwrap().id;
        context.input.find_action(group, name).unwrap().is_just_released()
    }

    #[rhai_fn(pure)]
    pub(crate) fn axis_value(input: &mut InputManagerHandle, group: &str, name: &str) -> f32 {
        let context: &mut ProgramContext = input.as_mut();
        let group = context.input.find_group(group).unwrap().id;
        context.input.find_axis(group, name).unwrap().value
    }
}

