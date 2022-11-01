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
    pub(crate) fn action_pressed(input: &mut InputManagerHandle, name: &str) -> bool {
        let ctx: &mut ProgramContext = input.as_mut();
        ctx.input.find_action(name.into(), &ctx.asset, false).is_pressed()
    }

    #[rhai_fn(pure)]
    pub(crate) fn action_released(input: &mut InputManagerHandle, name: &str) -> bool {
        let ctx: &mut ProgramContext = input.as_mut();
        ctx.input.find_action(name.into(), &ctx.asset, false).is_released()
    }

    #[rhai_fn(pure)]
    pub(crate) fn action_just_pressed(input: &mut InputManagerHandle, name: &str) -> bool {
        let ctx: &mut ProgramContext = input.as_mut();
        ctx.input.find_action(name.into(), &ctx.asset, false).is_just_pressed()
    }

    #[rhai_fn(pure)]
    pub(crate) fn action_just_released(input: &mut InputManagerHandle, name: &str) -> bool {
        let ctx: &mut ProgramContext = input.as_mut();
        ctx.input.find_action(name.into(), &ctx.asset, false).is_just_released()
    }

    #[rhai_fn(pure)]
    pub(crate) fn axis_value(input: &mut InputManagerHandle, name: &str) -> f32 {
        let ctx: &mut ProgramContext = input.as_mut();
        ctx.input.find_axis(name.into(), &ctx.asset, 0.0).value
    }
}

