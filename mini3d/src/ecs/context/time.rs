pub struct TimeContext {
    pub(crate) delta: f64,
    pub(crate) global: f64,
}

impl TimeContext {
    pub fn delta(&self) -> f64 {
        self.delta
    }

    pub fn global(&self) -> f64 {
        self.global
    }
}
