use std::any::TypeId;

use crate::{
    asset::container::{
        AnyAssetContainer, PrivateAnyAssetContainerMut, PrivateAnyAssetContainerRef,
        StaticAssetContainer,
    },
    reflection::Property,
    utils::{
        slotmap::{SlotId, SlotMap},
        string::AsciiArray,
        uid::{ToUID, UID},
    },
};

use super::{datatype::StaticDataType, error::RegistryError};

pub trait AssetTypeTrait: Copy {
    type Ref<'a>;
    type Data: Default;
    fn new(id: SlotId) -> Self;
    fn id(&self) -> SlotId;
    fn insert_container(container: PrivateAnyAssetContainerMut, data: Self::Data) -> SlotId;
    fn asset_ref(container: PrivateAnyAssetContainerRef, slot: SlotId) -> Self::Ref<'_>;
    fn check_type_id(id: TypeId) -> bool;
}

pub trait AssetReferenceTrait {
    type AssetType: AssetTypeTrait;
}

impl<T: StaticDataType> AssetReferenceTrait for T {
    type AssetType = StaticAssetType<T>;
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetType(pub(crate) SlotId);

impl AssetTypeTrait for AssetType {
    type Ref<'a> = ();
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

    fn asset_ref(container: PrivateAnyAssetContainerRef, slot: SlotId) -> Self::Ref<'_> {
        todo!()
    }

    fn check_type_id(id: TypeId) -> bool {
        true
    }
}

pub struct StaticAssetType<D: StaticDataType> {
    _marker: std::marker::PhantomData<D>,
    pub(crate) id: AssetType,
}

impl<D: StaticDataType> Clone for StaticAssetType<D> {
    fn clone(&self) -> Self {
        Self {
            _marker: std::marker::PhantomData,
            id: self.id,
        }
    }
}

impl<D: StaticDataType> Copy for StaticAssetType<D> {}

impl<D: StaticDataType> Default for StaticAssetType<D> {
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
            id: AssetType::default(),
        }
    }
}

impl<D: StaticDataType> AssetTypeTrait for StaticAssetType<D> {
    type Ref<'a> = &'a D;
    type Data = D;

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
            .downcast_mut::<StaticAssetContainer<D>>()
            .expect("Invalid static asset container")
            .0
            .add(asset)
    }

    fn asset_ref(container: PrivateAnyAssetContainerRef, slot: SlotId) -> Self::Ref<'_> {
        container
            .0
            .as_any()
            .downcast_ref::<StaticAssetContainer<D>>()
            .expect("Invalid static asset container")
            .0
            .get(slot)
            .expect("Invalid static asset slot")
    }

    fn check_type_id(id: TypeId) -> bool {
        id == TypeId::of::<D>()
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

pub(crate) struct StaticAssetReflection<D: StaticDataType> {
    _phantom: std::marker::PhantomData<D>,
}

impl<D: StaticDataType> AnyAssetReflection for StaticAssetReflection<D> {
    fn create_asset_container(&self) -> Box<dyn AnyAssetContainer> {
        Box::<StaticAssetContainer<D>>::default()
    }

    fn find_property(&self, name: &str) -> Option<&Property> {
        D::PROPERTIES.iter().find(|p| p.name == name)
    }

    fn properties(&self) -> &[Property] {
        D::PROPERTIES
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<D>()
    }
}

pub(crate) struct AssetEntry {
    pub(crate) name: AsciiArray<MAX_ASSET_TYPE_NAME_LEN>,
    pub(crate) reflection: Box<dyn AnyAssetReflection>,
    pub(crate) kind: AssetKind,
}

#[derive(Default)]
pub struct AssetRegistryManager {
    pub(crate) entries: SlotMap<AssetEntry>,
    pub(crate) changed: bool,
}

impl AssetRegistryManager {
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

    pub(crate) fn add_static<D: StaticDataType>(
        &mut self,
        name: &str,
    ) -> Result<StaticAssetType<D>, RegistryError> {
        let reflection = StaticAssetReflection::<D> {
            _phantom: std::marker::PhantomData,
        };
        let id = self.add(name, AssetKind::Static, Box::new(reflection))?;
        Ok(StaticAssetType {
            _marker: std::marker::PhantomData,
            id: AssetType(id),
        })
    }

    pub(crate) fn find<H: AssetTypeTrait>(&self, asset: impl ToUID) -> Option<H> {
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

    pub(crate) fn contains(&self, asset: impl ToUID) -> bool {
        let asset = asset.to_uid();
        self.find::<AssetType>(asset).is_some()
    }
}
