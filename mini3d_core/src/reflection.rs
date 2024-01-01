use crate::{
    ecs::entity::Entity,
    math::{
        mat::M4I32F16,
        quat::QI32F16,
        vec::{V2I32, V2I32F16, V3I32, V3I32F16, V4I32, V4I32F16},
    },
    script::mir::primitive::PrimitiveType,
    utils::{string::AsciiArray, uid::UID},
};

pub enum PropertyAccess {
    Read,
    Write,
    ReadWrite,
}

pub struct PropertyId(u8);

pub struct Property {
    pub(crate) name: AsciiArray<32>,
    pub(crate) access: PropertyAccess,
    pub(crate) ty: PrimitiveType,
    pub(crate) id: PropertyId,
}

macro_rules! read_property {
    ($type:ty, $read:ident) => {
        fn $read(&self, id: PropertyId) -> Option<$type> {
            None
        }
    };
}

macro_rules! write_property {
    ($type:ty, $write:ident) => {
        fn $write(&mut self, id: PropertyId, value: $type) {}
    };
}

#[allow(unused)]
pub trait ReadProperty {
    read_property!(bool, read_bool);
    read_property!(u8, read_u8);
    read_property!(i32, read_i32);
    read_property!(u32, read_u32);
    read_property!(V2I32F16, read_v2i32f16);
    read_property!(V2I32, read_v2i32);
    read_property!(V3I32F16, read_v3i32f16);
    read_property!(V3I32, read_v3i32);
    read_property!(V4I32F16, read_v4i32f16);
    read_property!(V4I32, read_v4i32);
    read_property!(M4I32F16, read_m4i32f16);
    read_property!(QI32F16, read_q32f16);
    read_property!(Entity, read_entity);
    read_property!(UID, read_uid);
}

#[allow(unused)]
pub trait WriteProperty {
    write_property!(bool, write_bool);
    write_property!(u8, write_u8);
    write_property!(i32, write_i32);
    write_property!(u32, write_u32);
    write_property!(V2I32F16, write_v2i32f16);
    write_property!(V2I32, write_v2i32);
    write_property!(V3I32F16, write_v3i32f16);
    write_property!(V3I32, write_v3i32);
    write_property!(V4I32F16, write_v4i32f16);
    write_property!(V4I32, write_v4i32);
    write_property!(M4I32F16, write_m4i32f16);
    write_property!(QI32F16, write_q32f16);
    write_property!(Entity, write_entity);
    write_property!(UID, write_uid);
}

pub trait ReadWriteProperty: ReadProperty + WriteProperty {}

#[allow(unused)]
pub trait Reflect: ReadProperty + WriteProperty {
    fn properties() -> &'static [Property] {
        &[]
    }
}
