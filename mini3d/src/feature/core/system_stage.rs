#[derive(Default, Debug, Resource, Serialize, Reflect, Clone)]
pub struct SystemStage {}

impl SystemStage {
    pub const UPDATE: &'static str = "update";
    pub const FIXED_UPDATE_60HZ: &'static str = "fixed_update_60hz";
}
