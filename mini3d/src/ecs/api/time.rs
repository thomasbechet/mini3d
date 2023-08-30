pub struct TimeAPI {
    pub(crate) delta: f64,
    pub(crate) fixed: Option<f64>,
    pub(crate) global: f64,
}

impl TimeAPI {
    pub fn delta(&self) -> f64 {
        self.delta
    }

    pub fn fixed(&self) -> Option<f64> {
        self.fixed
    }

    pub fn global(&self) -> f64 {
        self.global
    }
}
