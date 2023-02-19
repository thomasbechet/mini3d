use glam::{Vec2, Vec3, Vec4};
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
pub enum FieldValue {
    String(String),
    Integer(i32),
    Float(f32),
    Boolean(bool),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
    Entity(Entity),
    Array(Vec<FieldValue>),
}