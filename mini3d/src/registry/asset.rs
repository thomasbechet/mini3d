use std::any::TypeId;

use crate::{
    asset::{
        container::{AnyAssetContainer, StaticAssetContainer},
        handle::{PrivateAnyAssetContainerMut, PrivateAnyAssetContainerRef},
    },
    reflection::{Property, Reflect},
    serialize::Serialize,
    utils::{
        slotmap::{SlotId, SlotMap},
        string::AsciiArray,
        uid::{ToUID, UID},
    },
};

use super::error::RegistryError;

pub trait AssetData: Default + Serialize + Reflect + 'static {}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetType(pub(crate) SlotId);

pub struct StaticAssetType<A: AssetData> {
    _marker: std::marker::PhantomData<A>,
    pub(crate) id: AssetType,
}

impl<A: AssetData> Clone for StaticAssetType<A> {
    fn clone(&self) -> Self {
        Self {
            _marker: std::marker::PhantomData,
            id: self.id,
        }
    }
}

impl<A: AssetData> Copy for StaticAssetType<A> {}

impl<A: AssetData> Default for StaticAssetType<A> {
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
            id: AssetType::default(),
        }
    }
}

pub trait AssetTypeHandle: Copy {
    type Data: Default;
    fn new(id: SlotId) -> Self;
    fn id(&self) -> SlotId;
    fn insert_container(container: PrivateAnyAssetContainerMut, data: Self::Data) -> SlotId;
    fn remove_container(container: PrivateAnyAssetContainerMut, slot: SlotId);
    fn check_type(container: PrivateAnyAssetContainerRef) -> bool;
    fn check_type_id(id: TypeId) -> bool;
}

impl AssetTypeHandle for AssetType {
    type Data = ();

    fn new(id: SlotId) -> Self {
        Self(id)
    }

    fn id(&self) -> SlotId {
        self.0
    }

    fn insert_container(container: PrivateAnyAssetContainerMut, data: Self::Data) -> SlotId {
        todo!()
    }

    fn remove_container(container: PrivateAnyAssetContainerMut, slot: SlotId) {
        todo!()
    }

    fn check_type(container: PrivateAnyAssetContainerRef) -> bool {
        todo!()
    }

    fn check_type_id(id: TypeId) -> bool {
        true
    }
}

impl<A: AssetData> AssetTypeHandle for StaticAssetType<A> {
    type Data = A;

    fn new(id: SlotId) -> Self {
        Self {
            _marker: std::marker::PhantomData,
            id: AssetType(id),
        }
    }

    fn id(&self) -> SlotId {
        self.id.0
    }

    fn insert_container(container: PrivateAnyAssetContainerMut, asset: Self::Data) -> SlotId {
        container
            .0
            .as_any_mut()
            .downcast_mut::<StaticAssetContainer<A>>()
            .expect("Invalid static asset container")
            .0
            .add(asset)
    }

    fn remove_container(container: PrivateAnyAssetContainerMut, slot: SlotId) {
        container
            .0
            .as_any_mut()
            .downcast_mut::<StaticAssetContainer<A>>()
            .expect("Invalid static asset container")
            .0
            .remove(slot);
    }

    fn check_type(container: PrivateAnyAssetContainerRef) -> bool {
        container
            .0
            .as_any()
            .downcast_ref::<StaticAssetContainer<A>>()
            .is_some()
    }

    fn check_type_id(id: TypeId) -> bool {
        id == TypeId::of::<A>()
    }
}

pub(crate) const MAX_ASSET_TYPE_NAME_LEN: usize = 64;

pub(crate) enum AssetKind {
    Static,
    Dynamic,
}

pub(crate) trait AnyAssetReflection {
    fn create_asset_container(&self) -> Box<dyn AnyAssetContainer>;
    fn find_property(&self, name: &str) -> Option<&Property>;
    fn properties(&self) -> &[Property];
    fn type_id(&self) -> TypeId;
}

pub(crate) struct StaticAssetReflection<A: AssetData> {
    _phantom: std::marker::PhantomData<A>,
}

impl<A: AssetData> AnyAssetReflection for StaticAssetReflection<A> {
    fn create_asset_container(&self) -> Box<dyn AnyAssetContainer> {
        Box::<StaticAssetContainer<A>>::default()
    }

    fn find_property(&self, name: &str) -> Option<&Property> {
        A::PROPERTIES.iter().find(|p| p.name == name)
    }

    fn properties(&self) -> &[Property] {
        A::PROPERTIES
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<A>()
    }
}

pub(crate) struct AssetEntry {
    pub(crate) name: AsciiArray<MAX_ASSET_TYPE_NAME_LEN>,
    pub(crate) reflection: Box<dyn AnyAssetReflection>,
    pub(crate) kind: AssetKind,
}

#[derive(Default)]
pub struct AssetRegistry {
    pub(crate) entries: SlotMap<AssetEntry>,
    pub(crate) changed: bool,
}

impl AssetRegistry {
    fn add(
        &mut self,
        name: &str,
        kind: AssetKind,
        reflection: Box<dyn AnyAssetReflection>,
    ) -> Result<SlotId, RegistryError> {
        let uid: UID = name.into();
        if self.contains(uid) {
            return Err(RegistryError::DuplicatedComponent);
        }
        self.changed = true;
        Ok(self.entries.add(AssetEntry {
            name: name.into(),
            kind,
            reflection,
        }))
    }

    pub fn add_static<A: AssetData>(
        &mut self,
        name: &str,
    ) -> Result<StaticAssetType<A>, RegistryError> {
        let reflection = StaticAssetReflection::<A> {
            _phantom: std::marker::PhantomData,
        };
        let id = self.add(name, AssetKind::Static, Box::new(reflection))?;
        Ok(StaticAssetType {
            _marker: std::marker::PhantomData,
            id: AssetType(id),
        })
    }

    pub fn find<H: AssetTypeHandle>(&self, asset: impl ToUID) -> Option<H> {
        // Find entry
        let asset = asset.to_uid();
        let asset = self
            .entries
            .iter()
            .find(|(_, def)| UID::new(&def.name) == asset)
            .map(|(id, _)| id);
        // Check type
        if let Some(id) = asset {
            if !H::check_type_id(self.entries[id].reflection.type_id()) {
                None
            } else {
                Some(H::new(id))
            }
        } else {
            None
        }
    }

    pub fn contains(&self, asset: impl ToUID) -> bool {
        let asset = asset.to_uid();
        self.find::<AssetType>(asset).is_some()
    }
}
