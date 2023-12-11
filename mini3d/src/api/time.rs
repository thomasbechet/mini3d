use crate::math::fixed::I32F16;

use super::Context;

pub(crate) struct TimeAPI {
    pub(crate) delta: I32F16,
    pub(crate) frame: u64,
}

pub struct Time;

impl Time {
    pub fn delta(ctx: &Context) -> I32F16 {
        ctx.time.delta
    }

    pub fn frame(ctx: &Context) -> u64 {
        ctx.time.frame
    }
}
