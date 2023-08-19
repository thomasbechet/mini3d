use crate::{
    registry::component::Component,
    serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize},
    utils::{generation::GenerationId, slotmap::SlotId},
};

use super::container::{AnyAssetContainer, StaticAssetContainer};

#[derive(Default, Clone, Copy)]
pub struct AssetBundleId(GenerationId);

impl AssetBundleId {
    pub(crate) fn new(id: GenerationId) -> Self {
        Self(id)
    }

    pub(crate) fn id(&self) -> GenerationId {
        self.0
    }
}

pub struct PrivateAnyAssetContainerRef<'a>(pub(crate) &'a dyn AnyAssetContainer);
pub struct PrivateAnyAssetContainerMut<'a>(pub(crate) &'a mut dyn AnyAssetContainer);

pub trait AssetHandle {
    type AssetRef<'a>;
    type Data;
    fn new(id: GenerationId) -> Self;
    fn id(&self) -> GenerationId;
    fn asset_ref<'a>(&self, container: PrivateAnyAssetContainerRef<'a>) -> Self::AssetRef<'a>;
    fn insert_container(container: PrivateAnyAssetContainerMut, data: Self::Data) -> SlotId;
    fn remove_container(container: PrivateAnyAssetContainerMut, slot: SlotId);
    fn check_type(container: PrivateAnyAssetContainerRef) -> bool;
}

#[derive(Default)]
pub struct StaticAsset<C: Component> {
    _marker: std::marker::PhantomData<C>,
    id: GenerationId,
}

impl<C: Component> StaticAsset<C> {
    pub fn null() -> Self {
        Default::default()
    }
}

impl<C: Component> PartialEq for StaticAsset<C> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<C: Component> Eq for StaticAsset<C> {}

impl<C: Component> Clone for StaticAsset<C> {
    fn clone(&self) -> Self {
        Self {
            _marker: self._marker,
            id: self.id,
        }
    }
}

impl<C: Component> Copy for StaticAsset<C> {}

impl<C: Component> std::fmt::Debug for StaticAsset<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticAsset").field("id", &self.id).finish()
    }
}

impl<C: Component> AssetHandle for StaticAsset<C> {
    type AssetRef<'a> = &'a C;
    type Data = C;
    fn new(id: GenerationId) -> Self {
        Self {
            _marker: std::marker::PhantomData::<C>,
            id,
        }
    }
    fn id(&self) -> GenerationId {
        self.id
    }
    fn asset_ref<'a>(&self, container: PrivateAnyAssetContainerRef<'a>) -> Self::AssetRef<'a> {
        container
            .0
            .as_any()
            .downcast_ref::<StaticAssetContainer<C>>()
            .expect("Invalid static asset container")
            .0
            .get(self.id.slot())
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

impl<C: Component> Serialize for StaticAsset<C> {
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
            id: GenerationId::deserialize(decoder, &Default::default())?,
        })
    }
}

#[derive(Default)]
pub struct DynamicAsset {
    id: GenerationId,
}

impl AssetHandle for DynamicAsset {
    type AssetRef<'a> = ();
    type Data = ();
    fn new(id: GenerationId) -> Self {
        Self { id }
    }
    fn id(&self) -> GenerationId {
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

impl Serialize for DynamicAsset {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        self.id.serialize(encoder)
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        Ok(Self {
            id: GenerationId::deserialize(decoder, &Default::default())?,
        })
    }
}
