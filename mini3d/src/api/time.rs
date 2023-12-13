use crate::{ecs::ECSCommand, math::fixed::U32F16};

use super::Context;

pub(crate) struct TimeAPI {
    pub(crate) delta: U32F16,
    pub(crate) frame: u64,
    pub(crate) target_tps: u16,
}

pub struct Time;

impl Time {
    pub fn delta(ctx: &Context) -> U32F16 {
        ctx.time.delta
    }

    pub fn frame(ctx: &Context) -> u64 {
        ctx.time.frame
    }

    pub fn target_fps(ctx: &Context) -> u16 {
        ctx.time.target_tps
    }

    pub fn set_target_fps(ctx: &mut Context, tps: u16) {
        ctx.commands.push(ECSCommand::SetTargetTPS(tps));
    }
}
