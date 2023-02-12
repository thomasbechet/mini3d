use rhai::plugin::*;

use crate::context::SystemContext;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub(crate) struct InputManagerHandle(usize);

impl From<&mut SystemContext<'_>> for InputManagerHandle {
    fn from(input: &mut SystemContext) -> Self {
        Self::new(input)
    }
}

impl<'a> AsMut<SystemContext<'a>> for InputManagerHandle {
    fn as_mut(&mut self) -> &mut SystemContext<'a> {
        unsafe { std::mem::transmute(self.0) }
    }
}

impl InputManagerHandle {

    fn new(input: &mut SystemContext) -> Self {
        let handle = unsafe { std::mem::transmute(input) };
        Self(handle)
    }
}

#[export_module]
pub mod rhai_input_api {

    #[rhai_fn(pure, return_raw)]
    pub(crate) fn action_pressed(input: &mut InputManagerHandle, name: &str) -> Result<bool, Box<EvalAltResult>> {
        let ctx: &mut SystemContext = input.as_mut();
        Ok(ctx.input().action(name.into()).map_err(|err| err.to_string())?.is_pressed())
    }

    #[rhai_fn(pure, return_raw)]
    pub(crate) fn action_released(input: &mut InputManagerHandle, name: &str) -> Result<bool, Box<EvalAltResult>> {
        let ctx: &mut SystemContext = input.as_mut();
        Ok(ctx.input().action(name.into()).map_err(|err| err.to_string())?.is_released())
    }

    #[rhai_fn(pure, return_raw)]
    pub(crate) fn action_just_pressed(input: &mut InputManagerHandle, name: &str) -> Result<bool, Box<EvalAltResult>> {
        let ctx: &mut SystemContext = input.as_mut();
        Ok(ctx.input().action(name.into()).map_err(|err| err.to_string())?.is_just_pressed())
    }

    #[rhai_fn(pure, return_raw)]
    pub(crate) fn action_just_released(input: &mut InputManagerHandle, name: &str) -> Result<bool, Box<EvalAltResult>> {
        let ctx: &mut SystemContext = input.as_mut();
        Ok(ctx.input().action(name.into()).map_err(|err| err.to_string())?.is_just_released())
    }

    #[rhai_fn(pure, return_raw)]
    pub(crate) fn axis_value(input: &mut InputManagerHandle, name: &str) -> Result<f32, Box<EvalAltResult>> {
        let ctx: &mut SystemContext = input.as_mut();
        Ok(ctx.input().axis(name.into()).map_err(|err| err.to_string())?.value)
    }
}

