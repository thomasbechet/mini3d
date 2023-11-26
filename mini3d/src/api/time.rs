use super::Context;

pub(crate) struct TimeAPI {
    pub(crate) delta: f64,
    pub(crate) global: f64,
    pub(crate) frame: u64,
}

pub struct Time;

impl Time {
    pub fn delta(ctx: &Context) -> f64 {
        ctx.time.delta
    }

    pub fn global(ctx: &Context) -> f64 {
        ctx.time.global
    }

    pub fn frame(ctx: &Context) -> u64 {
        ctx.time.frame
    }
}
