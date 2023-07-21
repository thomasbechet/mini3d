use crate::utils::uid::UID;

pub struct Signal;

impl Signal {
    pub const UPDATE: &'static str = "update";
    pub const FIXED_UPDATE: &'static str = "fixed_update";
    pub const SCENE_CHANGED: &'static str = "scene_changed";
}

pub enum SignalKind {
    Reactive,
    Frame,
    Custom,
}

pub enum ReactiveSystemKind {
    AddComponent(UID),
    RemoveComponent(UID),
    UpdateComponent(UID),
    EnterGroup(UID),
    LeaveGroup(UID),
}
