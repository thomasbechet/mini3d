use std::any::TypeId;

use mini3d_derive::Serialize;

use crate::{
    registry::datatype::{ReferenceResolver, StaticDataType},
    utils::{slotmap::SlotId, uid::UID},
};

use super::container::{PrivateAnyAssetContainerRef, StaticAssetContainer};

pub trait AssetRefTrait {
    fn new(entry: SlotId, ty_uid: UID, name_uid: UID) -> Self;
    fn check_static_type(container: PrivateAnyAssetContainerRef) -> bool;
}

#[derive(Default, Serialize, Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct AssetRef {
    #[serialize(skip)]
    entry: SlotId,
    ty_uid: UID,
    name_uid: UID,
}

impl AssetRef {
    pub fn resolve(&mut self, resolver: &mut ReferenceResolver) {}
}

impl AssetRefTrait for AssetRef {
    fn new(entry: SlotId, ty_uid: UID, name_uid: UID) -> Self {
        Self {
            entry,
            ty_uid,
            name_uid,
        }
    }

    fn check_static_type(_container: PrivateAnyAssetContainerRef) -> bool {
        true
    }
}

pub struct StaticAssetRef<D: StaticDataType> {
    _marker: std::marker::PhantomData<D>,
    reference: AssetRef,
}

impl<D: StaticDataType> StaticAssetRef<D> {
    pub fn resolve(&mut self, resolver: &mut ReferenceResolver) {}
}

impl<D: StaticDataType> AssetRefTrait for StaticAssetRef<D> {
    fn new(entry: SlotId, ty_uid: UID, name_uid: UID) -> Self {
        Self {
            _marker: std::marker::PhantomData,
            reference: AssetRef::new(entry, ty_uid, name_uid),
        }
    }

    fn check_static_type(container: PrivateAnyAssetContainerRef) -> bool {
        container.0.type_id() == TypeId::of::<StaticAssetContainer<D>>()
    }
}
