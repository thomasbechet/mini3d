use alloc::vec;
use alloc::{string::String, vec::Vec};
use mini3d_math::vec::{V3I32F16, V3I32F24, V4I32F16, V4I32F24};
use mini3d_math::{
    fixed::{FixedPoint, I32F24},
    mat::M4I32F16,
    quat::QI32F16,
};
use mini3d_utils::handle::RawHandle;
use mini3d_utils::string::AsciiArray;

use crate::{
    container::{ComponentId, FieldIndex},
    entity::Entity,
};

pub struct Field<T: FieldType>(
    pub(crate) ComponentId,
    pub(crate) FieldIndex,
    pub(crate) core::marker::PhantomData<T>,
);

impl<T: FieldType> Clone for Field<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: FieldType> Copy for Field<T> {}

pub enum Primitive {
    I32,
    U32,
    I32F24,
    V3I32F16,
    V4I32F16,
    M4I32F16,
    QI32F16,
    Entity,
    String,
    Handle,
}

pub enum DataType {
    Scalar(Primitive),
    Array(Primitive, u32),
}

pub struct ComponentField<'a> {
    pub name: &'a str,
    pub ty: DataType,
}

impl<'a> ComponentField<'a> {
    pub(crate) fn create_storage(&self) -> Storage {
        match self.ty {
            // Scalar types
            DataType::Scalar(Primitive::I32) => Storage::I32(Default::default()),
            DataType::Scalar(Primitive::U32) => Storage::U32(Default::default()),
            DataType::Scalar(Primitive::I32F24) => Storage::I32F24(Default::default()),
            DataType::Scalar(Primitive::V3I32F16) => Storage::V3I32F16(Default::default()),
            DataType::Scalar(Primitive::V4I32F16) => Storage::V4I32F16(Default::default()),
            DataType::Scalar(Primitive::M4I32F16) => Storage::M4I32F16(Default::default()),
            DataType::Scalar(Primitive::QI32F16) => Storage::QI32F16(Default::default()),
            DataType::Scalar(Primitive::Entity) => Storage::Entity(Default::default()),
            DataType::Scalar(Primitive::String) => Storage::String(Default::default()),
            DataType::Scalar(Primitive::Handle) => Storage::Handle(Default::default()),
            // Array types
            DataType::Array(Primitive::I32, n) => {
                Storage::I32(vec![Default::default(); n as usize])
            }
            DataType::Array(Primitive::U32, n) => {
                Storage::U32(vec![Default::default(); n as usize])
            }
            DataType::Array(Primitive::I32F24, n) => {
                Storage::I32F24(vec![Default::default(); n as usize])
            }
            DataType::Array(Primitive::V3I32F16, n) => {
                Storage::V3I32F16(vec![Default::default(); n as usize])
            }
            DataType::Array(Primitive::V4I32F16, n) => {
                Storage::V4I32F16(vec![Default::default(); n as usize])
            }
            DataType::Array(Primitive::M4I32F16, n) => {
                Storage::M4I32F16(vec![Default::default(); n as usize])
            }
            DataType::Array(Primitive::QI32F16, n) => {
                Storage::QI32F16(vec![Default::default(); n as usize])
            }
            DataType::Array(Primitive::Entity, n) => {
                Storage::Entity(vec![Default::default(); n as usize])
            }
            DataType::Array(Primitive::String, n) => {
                Storage::String(vec![Default::default(); n as usize])
            }
            DataType::Array(Primitive::Handle, n) => {
                Storage::Handle(vec![Default::default(); n as usize])
            }
        }
    }
}

pub(crate) enum Storage {
    I32(Vec<i32>),
    U32(Vec<u32>),
    I32F24(Vec<I32F24>),
    V3I32F16(Vec<V3I32F16>),
    V4I32F16(Vec<V4I32F16>),
    M4I32F16(Vec<M4I32F16>),
    QI32F16(Vec<QI32F16>),
    Entity(Vec<Entity>),
    String(Vec<String>),
    Handle(Vec<RawHandle>),
}

impl Storage {
    fn resize(&mut self, size: usize) {
        match self {
            Storage::I32(v) => v.resize(size, 0),
            Storage::U32(v) => v.resize(size, 0),
            Storage::I32F24(v) => v.resize(size, Default::default()),
            Storage::V3I32F16(v) => v.resize(size, Default::default()),
            Storage::V4I32F16(v) => v.resize(size, Default::default()),
            Storage::M4I32F16(v) => v.resize(size, Default::default()),
            Storage::QI32F16(v) => v.resize(size, Default::default()),
            Storage::Entity(v) => v.resize(size, Default::default()),
            Storage::String(v) => v.resize(size, Default::default()),
            Storage::Handle(v) => v.resize(size, Default::default()),
        }
    }

    fn len(&self) -> usize {
        match self {
            Storage::I32(v) => v.len(),
            Storage::U32(v) => v.len(),
            Storage::I32F24(v) => v.len(),
            Storage::V3I32F16(v) => v.len(),
            Storage::V4I32F16(v) => v.len(),
            Storage::M4I32F16(v) => v.len(),
            Storage::QI32F16(v) => v.len(),
            Storage::Entity(v) => v.len(),
            Storage::String(v) => v.len(),
            Storage::Handle(v) => v.len(),
        }
    }

    pub(crate) fn add_default(&mut self, e: Entity) {
        let index = e.index() as usize;
        if index >= self.len() {
            self.resize(index + 1);
        }
        match self {
            Storage::I32(v) => v[index] = Default::default(),
            Storage::U32(v) => v[index] = Default::default(),
            Storage::I32F24(v) => v[index] = Default::default(),
            Storage::V3I32F16(v) => v[index] = Default::default(),
            Storage::V4I32F16(v) => v[index] = Default::default(),
            Storage::M4I32F16(v) => v[index] = Default::default(),
            Storage::QI32F16(v) => v[index] = Default::default(),
            Storage::Entity(v) => v[index] = Default::default(),
            Storage::String(v) => v[index] = Default::default(),
            Storage::Handle(v) => v[index] = Default::default(),
        }
    }

    pub(crate) fn remove(&mut self, e: Entity) {}
}

pub struct FieldEntry {
    pub(crate) name: AsciiArray<32>,
    pub(crate) data: Storage,
}

pub trait FieldType: Sized {
    fn named(name: &str) -> ComponentField;
    fn read(entry: &FieldEntry, e: Entity) -> Option<Self>;
    fn write(entry: &mut FieldEntry, e: Entity, v: Self);
}

// impl<const I: usize, T: FieldValue> FieldValue for [T; I] {
//     fn get(data: &FieldData, entity: Entity) -> Option<&Self> {
//         match &data {
//             FieldData::Array(Storage::$kind(v)) => v.get(entity.index()),
//             _ => None,
//         }
//     }
//     fn get_mut(data: &mut FieldData, entity: Entity) -> Option<&mut Self> {
//         match data {
//             FieldData::Array(Storage::$kind(v)) => v.get_mut(entity.index()),
//             _ => None,
//         }
//     }
// }

// impl<T: FieldType> FieldType for Option<T> {
//
// }

macro_rules! impl_field_scalar {
    ($scalar:ty, $kind:ident) => {
        impl FieldType for $scalar {
            fn named(name: &str) -> ComponentField {
                ComponentField {
                    name,
                    ty: DataType::Scalar(Primitive::$kind),
                }
            }
            fn read(entry: &FieldEntry, e: Entity) -> Option<Self> {
                match &entry.data {
                    Storage::$kind(s) => s.get(e.index() as usize).copied(),
                    _ => None,
                }
            }
            fn write(entry: &mut FieldEntry, e: Entity, v: Self) {
                match entry.data {
                    Storage::$kind(ref mut s) => s[e.index() as usize] = v,
                    _ => {}
                }
            }
        }
    };
}

impl_field_scalar!(i32, I32);
impl_field_scalar!(u32, U32);
impl_field_scalar!(I32F24, I32F24);
impl_field_scalar!(V3I32F16, V3I32F16);
impl_field_scalar!(M4I32F16, M4I32F16);
impl_field_scalar!(QI32F16, QI32F16);
impl_field_scalar!(Entity, Entity);
