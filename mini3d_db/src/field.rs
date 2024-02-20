use alloc::vec;
use alloc::{string::String, vec::Vec};
use mini3d_math::{
    fixed::{FixedPoint, I32F24},
    mat::M4I32F16,
    quat::QI32F16,
};
use mini3d_utils::handle::RawHandle;
use mini3d_utils::string::AsciiArray;

use crate::{
    container::{Component, FieldIndex},
    entity::Entity,
};

pub struct Field<T: FieldType>(
    pub(crate) Component,
    pub(crate) FieldIndex,
    pub(crate) core::marker::PhantomData<T>,
);

impl<T: FieldType> Clone for Field<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: FieldType> Copy for Field<T> {}

pub enum ComponentFieldType {
    I32,
    U32,
    I32F24,
    M4I32F16,
    QI32F16,
    Entity,
    String,
    Handle,
}

pub enum ComponentFieldCollection {
    Scalar,
    Array(u32),
}

pub struct ComponentField<'a> {
    pub name: &'a str,
    pub ty: ComponentFieldType,
    pub collection: ComponentFieldCollection,
}

impl<'a> ComponentField<'a> {
    pub(crate) fn create_storage(&self) -> Storage {
        match self.ty {
            ComponentFieldType::I32 => match self.collection {
                ComponentFieldCollection::Scalar => Storage::I32(Vec::new()),
                ComponentFieldCollection::Array(n) => Storage::I32(vec![0; n as usize]),
            },
            ComponentFieldType::U32 => match self.collection {
                ComponentFieldCollection::Scalar => Storage::U32(Vec::new()),
                ComponentFieldCollection::Array(n) => Storage::U32(vec![0; n as usize]),
            },
            ComponentFieldType::I32F24 => match self.collection {
                ComponentFieldCollection::Scalar => Storage::I32F24(Vec::new()),
                ComponentFieldCollection::Array(n) => {
                    Storage::I32F24(vec![I32F24::ZERO; n as usize])
                }
            },
            ComponentFieldType::M4I32F16 => match self.collection {
                ComponentFieldCollection::Scalar => Storage::M4I32F16(Vec::new()),
                ComponentFieldCollection::Array(n) => {
                    Storage::M4I32F16(vec![M4I32F16::IDENTITY; n as usize])
                }
            },
            ComponentFieldType::QI32F16 => match self.collection {
                ComponentFieldCollection::Scalar => Storage::QI32F16(Vec::new()),
                ComponentFieldCollection::Array(n) => {
                    Storage::QI32F16(vec![QI32F16::default(); n as usize])
                }
            },
            ComponentFieldType::Entity => match self.collection {
                ComponentFieldCollection::Scalar => Storage::Entity(Vec::new()),
                ComponentFieldCollection::Array(n) => {
                    Storage::Entity(vec![Entity::null(); n as usize])
                }
            },
            ComponentFieldType::String => match self.collection {
                ComponentFieldCollection::Scalar => Storage::String(Vec::new()),
                ComponentFieldCollection::Array(n) => {
                    Storage::String(vec![String::new(); n as usize])
                }
            },
            ComponentFieldType::Handle => match self.collection {
                ComponentFieldCollection::Scalar => Storage::Handle(Vec::new()),
                ComponentFieldCollection::Array(_) => Storage::Handle(Vec::new()),
            },
        }
    }
}

pub(crate) enum Storage {
    I32(Vec<i32>),
    U32(Vec<u32>),
    I32F24(Vec<I32F24>),
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
            Storage::I32F24(v) => v.resize(size, I32F24::ZERO),
            Storage::M4I32F16(v) => v.resize(size, M4I32F16::IDENTITY),
            Storage::QI32F16(v) => v.resize(size, QI32F16::default()),
            Storage::Entity(v) => v.resize(size, Entity::null()),
            Storage::String(v) => v.resize(size, String::new()),
            Storage::Handle(v) => v.resize(size, RawHandle::null()),
        }
    }

    fn len(&self) -> usize {
        match self {
            Storage::I32(v) => v.len(),
            Storage::U32(v) => v.len(),
            Storage::I32F24(v) => v.len(),
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
            Storage::I32(v) => v[index] = 0,
            Storage::U32(v) => v[index] = 0,
            Storage::I32F24(v) => v[index] = Default::default(),
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

macro_rules! impl_field_scalar {
    ($scalar:ty, $kind:ident) => {
        impl FieldType for $scalar {
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
impl_field_scalar!(M4I32F16, M4I32F16);
impl_field_scalar!(QI32F16, QI32F16);
impl_field_scalar!(Entity, Entity);
