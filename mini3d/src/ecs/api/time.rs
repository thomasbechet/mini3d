use super::context::Context;

pub(crate) struct TimeAPI {
    pub(crate) delta: f64,
    pub(crate) global: f64,
}

pub struct Time;

impl Time {
    pub fn delta(ctx: &Context) -> f64 {
        ctx.time.delta
    }

    pub fn global(ctx: &Context) -> f64 {
        ctx.time.global
    }
}
