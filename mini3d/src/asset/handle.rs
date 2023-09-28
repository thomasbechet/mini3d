use std::fmt::Debug;

use crate::{
    registry::asset::{AssetData, AssetType, AssetTypeHandle, StaticAssetType},
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
pub struct Asset(pub(crate) SlotId);

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

#[derive(Default, Hash)]
pub struct StaticAsset<A: AssetData>(pub(crate) SlotId, pub(crate) std::marker::PhantomData<A>);

impl<A: AssetData> AssetHandle for StaticAsset<A> {
    type Ref<'a> = &'a A;
    type TypeHandle = StaticAssetType<A>;
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
            .downcast_ref::<StaticAssetContainer<A>>()
            .expect("Invalid static asset container")
            .0
            .get(slot)
            .expect("Asset not found in container")
    }
}

impl<A: AssetData> Clone for StaticAsset<A> {
    fn clone(&self) -> Self {
        Self(self.0, std::marker::PhantomData)
    }
}

impl<A: AssetData> Copy for StaticAsset<A> {}

impl<A: AssetData> Debug for StaticAsset<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("StaticAsset")
            .field(&self.0)
            .field(&std::any::type_name::<A>())
            .finish()
    }
}

impl<A: AssetData> PartialEq for StaticAsset<A> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<A: AssetData> Eq for StaticAsset<A> {}

impl<A: AssetData> Serialize for StaticAsset<A> {
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
