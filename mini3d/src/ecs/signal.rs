pub struct Signal;

impl Signal {
    pub const UPDATE: &'static str = "update";
    pub const FIXED_UPDATE: &'static str = "fixed_update";
    pub const SCENE_CHANGED: &'static str = "scene_changed";
}
