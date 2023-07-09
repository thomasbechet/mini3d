use mini3d_derive::Serialize;

#[derive(Serialize, Clone, Hash)]
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

#[derive(Serialize, Clone)]
pub struct DynamicComponent {}
