use slotmap::{new_key_type, SlotMap};

pub mod component;
pub mod system;

new_key_type! { pub struct SystemId; }

pub struct ECS {
    world: hecs::World,
}