use crate::{
    feature::input::{action::InputAction, axis::InputAxis},
    input::{
        handle::{InputActionHandle, InputAxisHandle, InputTextHandle},
        InputActionState, InputAxisState, InputError, InputTextState,
    },
};

use super::context::Context;

pub struct Input;

impl Input {
    pub fn add_action(
        ctx: &mut Context,
        action: InputAction,
    ) -> Result<InputActionHandle, InputError> {
        ctx.input.add_action(action)
    }

    pub fn find_action(ctx: &Context, name: &str) -> Option<InputActionHandle> {
        ctx.input.find_action(name)
    }

    pub fn add_axis(ctx: &mut Context, axis: InputAxis) -> Result<InputAxisHandle, InputError> {
        ctx.input.add_axis(axis)
    }

    pub fn find_axis(ctx: &Context, name: &str) -> Option<InputAxisHandle> {
        ctx.input.find_axis(name)
    }

    pub fn action<'a>(
        ctx: &'a Context,
        handle: InputActionHandle,
    ) -> Result<&'a InputActionState, InputError> {
        ctx.input.action(handle)
    }

    pub fn axis<'a>(
        ctx: &'a Context,
        handle: InputAxisHandle,
    ) -> Result<&'a InputAxisState, InputError> {
        ctx.input.axis(handle)
    }

    pub fn text<'a>(
        ctx: &'a Context,
        handle: InputTextHandle,
    ) -> Result<&'a InputTextState, InputError> {
        ctx.input.text(handle)
    }
}
