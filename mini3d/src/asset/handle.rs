use std::fmt::Debug;

use crate::{
    registry::{
        asset::{AssetType, AssetTypeHandle, StaticAssetType},
        datatype::StaticDataType,
    },
    serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize},
    utils::slotmap::SlotId,
};

use super::container::{AnyAssetContainer, StaticAssetContainer};

#[derive(Default, Clone, Copy)]
pub struct AssetBundle(pub(crate) SlotId);

impl AssetBundle {
    pub const DEFAULT: &'static str = "default";
}

pub struct PrivateAnyAssetContainerRef<'a>(pub(crate) &'a dyn AnyAssetContainer);
pub struct PrivateAnyAssetContainerMut<'a>(pub(crate) &'a mut dyn AnyAssetContainer);

pub trait AssetHandle {
    type Ref<'a>;
    type TypeHandle: AssetTypeHandle;
    fn new(id: SlotId) -> Self;
    fn id(&self) -> SlotId;
    fn asset_ref<'a>(
        &self,
        slot: SlotId,
        container: PrivateAnyAssetContainerRef<'a>,
    ) -> Self::Ref<'a>;
}

#[derive(Default, Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct Asset {
    pub(crate) id: SlotId,
    pub(crate) ty: AssetType,
}

pub struct StaticAsset<D: StaticDataType> {
    pub(crate) id: SlotId,
    pub(crate) ty: StaticAssetType<D>,
}

impl AssetHandle for Asset {
    type Ref<'a> = ();
    type TypeHandle = AssetType;
    fn new(id: SlotId) -> Self {
        Self(id)
    }
    fn id(&self) -> SlotId {
        self.0
    }
    fn asset_ref<'a>(
        &self,
        slot: SlotId,
        container: PrivateAnyAssetContainerRef<'a>,
    ) -> Self::Ref<'a> {
        todo!()
    }
}

impl<D: StaticDataType> AssetHandle for StaticAsset<D> {
    type Ref<'a> = &'a D;
    type TypeHandle = StaticAssetType<D>;
    fn new(id: SlotId) -> Self {
        Self(id, std::marker::PhantomData)
    }
    fn id(&self) -> SlotId {
        self.0
    }
    fn asset_ref<'a>(
        &self,
        slot: SlotId,
        container: PrivateAnyAssetContainerRef<'a>,
    ) -> Self::Ref<'a> {
        container
            .0
            .as_any()
            .downcast_ref::<StaticAssetContainer<D>>()
            .expect("Invalid static asset container")
            .0
            .get(slot)
            .expect("Asset not found in container")
    }
}

impl<D: StaticDataType> Clone for StaticAsset<D> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            ty: self.ty,
        }
    }
}

impl<D: StaticDataType> Copy for StaticAsset<D> {}

impl<D: StaticDataType> Debug for StaticAsset<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StaticAsset")
            .field(&self.0)
            .field(&std::any::type_name::<D>())
            .finish()
    }
}

impl<D: StaticDataType> PartialEq for StaticAsset<D> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<D: StaticDataType> Eq for StaticAsset<D> {}

impl<D: StaticDataType> Serialize for StaticAsset<D> {
    type Header = ();
    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        self.0.serialize(encoder)
    }
    fn deserialize(
        decoder: &mut impl Decoder,
        header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        Ok(Self(
            SlotId::deserialize(decoder, &Default::default())?,
            std::marker::PhantomData,
        ))
    }
}
