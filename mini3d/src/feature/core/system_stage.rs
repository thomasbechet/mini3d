#[derive(Default, Debug, Clone)]
pub struct SystemStage {
    periodic_invoke: Option<f64>,
}

impl SystemStage {
    pub const UPDATE: &'static str = "update";
    pub const FIXED_UPDATE_60HZ: &'static str = "fixed_update_60hz";
}
