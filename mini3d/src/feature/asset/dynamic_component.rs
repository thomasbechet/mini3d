use glam::{Vec2, Vec3, Vec4, Quat};
use serde::{Serialize, Deserialize};

use crate::ecs::entity::Entity;

#[derive(Serialize, Deserialize, Clone, Hash)]
pub enum FieldType {
    String,
    Integer,
    Float,
    Boolean,
    Vec2,
    Vec3,
    Vec4,
    Entity,
    Array,
}


#[derive(Serialize, Deserialize, Clone)]
pub struct DynamicComponent {

}