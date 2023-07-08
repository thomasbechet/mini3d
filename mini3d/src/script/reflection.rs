use glam::{IVec2, IVec3, IVec4, Mat4, Quat, Vec2, Vec3, Vec4};

use crate::{ecs::entity::Entity, uid::UID};

use super::mir::primitive::PrimitiveType;

pub enum PropertyAccess {
    Read,
    Write,
    ReadWrite,
}

pub struct PropertyId(u8);

pub struct Property {
    pub(crate) name: String,
    pub(crate) access: PropertyAccess,
    pub(crate) ty: PrimitiveType,
    pub(crate) id: PropertyId,
}

macro_rules! read_property {
    ($type:ty, $read:ident) => {
        fn $read(&self, id: PropertyId) -> Option<$type> {
            let _ = id;
            None
        }
    };
}

macro_rules! write_property {
    ($type:ty, $write:ident) => {
        fn $write(&mut self, id: PropertyId, value: $type) {
            let _ = id;
            let _ = value;
        }
    };
}

pub trait Reflect {
    const PROPERTIES: &'static [Property];

    write_property!(bool, write_bool);
    write_property!(u8, write_u8);
    write_property!(i32, write_i32);
    write_property!(u32, write_u32);
    write_property!(f32, write_f32);
    write_property!(f64, write_f64);
    write_property!(Vec2, write_vec2);
    write_property!(IVec2, write_ivec2);
    write_property!(Vec3, write_vec3);
    write_property!(IVec3, write_ivec3);
    write_property!(Vec4, write_vec4);
    write_property!(IVec4, write_ivec4);
    write_property!(Mat4, write_mat4);
    write_property!(Quat, write_quat);
    write_property!(Entity, write_entity);
    write_property!(UID, write_uid);

    read_property!(bool, read_bool);
    read_property!(u8, read_u8);
    read_property!(i32, read_i32);
    read_property!(u32, read_u32);
    read_property!(f32, read_f32);
    read_property!(f64, read_f64);
    read_property!(Vec2, read_vec2);
    read_property!(IVec2, read_ivec2);
    read_property!(Vec3, read_vec3);
    read_property!(IVec3, read_ivec3);
    read_property!(Vec4, read_vec4);
    read_property!(IVec4, read_ivec4);
    read_property!(Mat4, read_mat4);
    read_property!(Quat, read_quat);
    read_property!(Entity, read_entity);
    read_property!(UID, read_uid);
}
