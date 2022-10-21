use slotmap::{new_key_type, SlotMap};

pub mod component;
pub mod system;

new_key_type! { pub struct SystemId; }

struct NativeSystem {

}

struct LuaSystem {

}

enum System {
    Native(),
    Lua(),
}

pub struct SystemScheduler {
    systems: SlotMap<SystemId, System>,
}

impl SystemScheduler {
    
    pub fn new() -> Self {
        Self { 
            systems: Default::default(),
        }
    }
}

pub struct ECS {
    scheduler: SystemScheduler,
}