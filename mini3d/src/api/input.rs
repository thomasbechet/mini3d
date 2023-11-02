use crate::{
    input::{
        handle::{InputActionHandle, InputAxisHandle},
        resource::action::InputAction,
    },
    resource::error::ResourceError,
};

use super::Context;

pub struct Input;

impl Input {
    pub fn find_action(ctx: &Context, name: &str) -> Option<InputActionHandle> {
        ctx.input.find_action(name, &ctx.resource)
    }

    pub fn find_axis(ctx: &Context, name: &str) -> Option<InputAxisHandle> {
        ctx.input.find_axis(name, &ctx.resource)
    }

    pub fn action<'a>(
        ctx: &'a Context,
        action: InputActionHandle,
    ) -> Result<&'a InputActionState, ResourceError> {
        ctx.resource
            .get::<InputAction>(action.0)
            .map(|action| &action.state)
    }

    pub fn axis<'a>(
        ctx: &'a Context,
        axis: InputAxisHandle,
    ) -> Result<&'a InputAxisState, ResourceError> {
        ctx.resource
            .get::<InputAxis>(axis.0)
            .map(|axis| &axis.state)
    }
}
