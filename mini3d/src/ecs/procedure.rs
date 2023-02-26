pub struct Procedure;

impl Procedure {
    pub const ENGINE_STARTUP: &'static str = "engine_startup";
    pub const UPDATE: &'static str = "update";
    pub const FIXED_UPDATE: &'static str = "fixed_update";
    pub const WORLD_CHANGED: &'static str = "world_changed";
}