use crate::{
    registry::component::ComponentData,
    serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize},
    utils::slotmap::SlotId,
};

use super::container::{AnyAssetContainer, StaticAssetContainer};

#[derive(Default, Clone, Copy)]
pub struct AssetBundleId(SlotId);

impl AssetBundleId {
    pub(crate) fn new(id: SlotId) -> Self {
        Self(id)
    }

    pub(crate) fn id(&self) -> SlotId {
        self.0
    }
}

pub struct PrivateAnyAssetContainerRef<'a>(pub(crate) &'a dyn AnyAssetContainer);
pub struct PrivateAnyAssetContainerMut<'a>(pub(crate) &'a mut dyn AnyAssetContainer);

pub trait AssetHandle {
    type AssetRef<'a>;
    type Data;
    fn new(id: SlotId) -> Self;
    fn id(&self) -> SlotId;
    fn asset_ref<'a>(&self, container: PrivateAnyAssetContainerRef<'a>) -> Self::AssetRef<'a>;
    fn insert_container(container: PrivateAnyAssetContainerMut, data: Self::Data) -> SlotId;
    fn remove_container(container: PrivateAnyAssetContainerMut, slot: SlotId);
    fn check_type(container: PrivateAnyAssetContainerRef) -> bool;
}

#[derive(Default)]
pub struct StaticAsset<C: ComponentData> {
    _marker: std::marker::PhantomData<C>,
    id: SlotId,
}

impl<C: ComponentData> StaticAsset<C> {
    pub fn null() -> Self {
        Default::default()
    }
}

impl<C: ComponentData> PartialEq for StaticAsset<C> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<C: ComponentData> Eq for StaticAsset<C> {}

impl<C: ComponentData> Clone for StaticAsset<C> {
    fn clone(&self) -> Self {
        Self {
            _marker: self._marker,
            id: self.id,
        }
    }
}

impl<C: ComponentData> Copy for StaticAsset<C> {}

impl<C: ComponentData> std::fmt::Debug for StaticAsset<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticAsset").field("id", &self.id).finish()
    }
}

impl<C: ComponentData> AssetHandle for StaticAsset<C> {
    type AssetRef<'a> = &'a C;
    type Data = C;
    fn new(id: SlotId) -> Self {
        Self {
            _marker: std::marker::PhantomData::<C>,
            id,
        }
    }
    fn id(&self) -> SlotId {
        self.id
    }
    fn asset_ref<'a>(&self, container: PrivateAnyAssetContainerRef<'a>) -> Self::AssetRef<'a> {
        container
            .0
            .as_any()
            .downcast_ref::<StaticAssetContainer<C>>()
            .expect("Invalid static asset container")
            .0
            .get(self.id)
            .expect("Asset not found in container")
    }
    fn insert_container(container: PrivateAnyAssetContainerMut, asset: Self::Data) -> SlotId {
        container
            .0
            .as_any_mut()
            .downcast_mut::<StaticAssetContainer<C>>()
            .expect("Invalid static asset container")
            .0
            .add(asset)
    }
    fn remove_container(container: PrivateAnyAssetContainerMut, slot: SlotId) {
        container
            .0
            .as_any_mut()
            .downcast_mut::<StaticAssetContainer<C>>()
            .expect("Invalid static asset container")
            .0
            .remove(slot);
    }
    fn check_type(container: PrivateAnyAssetContainerRef) -> bool {
        container
            .0
            .as_any()
            .downcast_ref::<StaticAssetContainer<C>>()
            .is_some()
    }
}

impl<C: ComponentData> Serialize for StaticAsset<C> {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        self.id.serialize(encoder)
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        Ok(Self {
            _marker: std::marker::PhantomData::<C>,
            id: SlotId::deserialize(decoder, &Default::default())?,
        })
    }
}

#[derive(Default, Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct Asset {
    id: SlotId,
}

impl<C: ComponentData> From<StaticAsset<C>> for Asset {
    fn from(asset: StaticAsset<C>) -> Self {
        Self { id: asset.id }
    }
}

impl AssetHandle for Asset {
    type AssetRef<'a> = ();
    type Data = ();
    fn new(id: SlotId) -> Self {
        Self { id }
    }
    fn id(&self) -> SlotId {
        self.id
    }
    fn asset_ref<'a>(&self, container: PrivateAnyAssetContainerRef<'a>) -> Self::AssetRef<'a> {}
    fn insert_container(container: PrivateAnyAssetContainerMut, asset: Self::Data) -> SlotId {
        SlotId::null()
    }
    fn remove_container(container: PrivateAnyAssetContainerMut, slot: SlotId) {}
    fn check_type(container: PrivateAnyAssetContainerRef) -> bool {
        true
    }
}

impl Serialize for Asset {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        self.id.serialize(encoder)
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        Ok(Self {
            id: SlotId::deserialize(decoder, &Default::default())?,
        })
    }
}
