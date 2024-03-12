use alloc::boxed::Box;
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

use crate::database::FieldId;
use crate::entity::Entity;

pub struct Field<T: FieldType>(pub(crate) FieldId, pub(crate) core::marker::PhantomData<T>);

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
            DataType::Scalar(Primitive::Handle) => Storage::Handle(Default::default()),
            _ => unreachable!()
        }
    }
}

#[derive(Default)]
struct RawStorage<T> {
    chunks: Vec<Option<Box<[T]>>>,
}

impl<T: Default + Copy> RawStorage<T> {
    const CHUNK_SIZE: usize = 32;

    fn indices(e: Entity) -> (usize, usize) {
        let ei = e.index();
        let ci = ei as usize / Self::CHUNK_SIZE; 
        let ei = ei as usize % Self::CHUNK_SIZE;
        (ci, ei)
    }

    fn get(&self, e: Entity) -> &T {
        let (ci, ei) = Self::indices(e); 
        self.chunks[ci].as_ref().unwrap().get(ei).unwrap()
    }

    fn get_mut(&mut self, e: Entity) -> &mut T {
        let (ci, ei) = Self::indices(e); 
        self.chunks[ci].as_mut().unwrap().get_mut(ei).unwrap()
    }

    fn set(&mut self, e: Entity, v: T) -> &mut T {
        let (ci, ei) = Self::indices(e); 
        if ci >= self.chunks.len() {
            self.chunks.resize(ci + 1, Default::default());
        } 
        let chunk = &mut self.chunks[ci];
        let chunk = chunk.get_or_insert(Box::new([Default::default(); Self::CHUNK_SIZE]));
        let data = chunk.get_mut(ei).unwrap();
        *data = v;
        data
    }
}

pub(crate) enum Storage {
    I32(RawStorage<i32>),
    U32(RawStorage<u32>),
    I32F24(RawStorage<I32F24>),
    V3I32F16(RawStorage<V3I32F16>),
    V4I32F16(RawStorage<V4I32F16>),
    M4I32F16(RawStorage<M4I32F16>),
    QI32F16(RawStorage<QI32F16>),
    Entity(RawStorage<Entity>),
    Handle(RawStorage<RawHandle>),
}

impl Storage {
    pub(crate) fn add_default(&mut self, e: Entity) {
        match self {
            Storage::I32(s) => {
                s.set(e, 0);
            },
            Storage::U32(s) => {
                s.set(e, 0);
            },
            Storage::I32F24(s) => {
                s.set(e, Default::default());
            },
            Storage::V3I32F16(s) => {
                s.set(e, Default::default());
            },
            Storage::V4I32F16(s) => {
                s.set(e, Default::default());
            },
            Storage::M4I32F16(s) => {
                s.set(e, Default::default());
            },
            Storage::QI32F16(s) => {
                s.set(e, Default::default());
            },
            Storage::Entity(s) => {
                s.set(e, Default::default());
            },
            Storage::Handle(s) => {
                s.set(e, Default::default());
            },
        }
    }
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
                    Storage::$kind(s) => Some(*s.get(e)),
                    _ => None,
                }
            }
            fn write(entry: &mut FieldEntry, e: Entity, v: Self) {
                match entry.data {
                    Storage::$kind(ref mut s) => *s.get_mut(e) = v,
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
