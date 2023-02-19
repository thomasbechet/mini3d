pub struct TimeContext {
    time: f64,
    delta_time: f64,
}

impl TimeContext {

    pub(crate) fn new(time: f64, delta_time: f64) -> Self {
        Self { time, delta_time }
    }

    pub fn global(&self) -> f64 {
        self.time
    }

    pub fn delta(&self) -> f64 {
        self.delta_time
    }
}