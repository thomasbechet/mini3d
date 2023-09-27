use crate::{
    registry::asset::{AssetData, AssetType, StaticAssetType},
    serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize},
    utils::slotmap::SlotId,
};

use super::container::AnyAssetContainer;

#[derive(Default, Clone, Copy)]
pub struct AssetBundle(pub(crate) SlotId);

impl AssetBundle {
    pub const DEFAULT: &'static str = "default";
}

pub struct PrivateAnyAssetContainerRef<'a>(pub(crate) &'a dyn AnyAssetContainer);
pub struct PrivateAnyAssetContainerMut<'a>(pub(crate) &'a mut dyn AnyAssetContainer);

pub trait AssetHandle {
    type TypeHandle;
}

#[derive(Default, Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct Asset(pub(crate) SlotId);

impl AssetHandle for Asset {
    type TypeHandle = AssetType;
}

#[derive(Default, Hash, PartialEq, Eq, Debug)]
pub struct StaticAsset<A: AssetData>(pub(crate) Asset, pub(crate) std::marker::PhantomData<A>);

impl<A: AssetData> AssetHandle for StaticAsset<A> {
    type TypeHandle = StaticAssetType<A>;
}

impl<A: AssetData> Clone for StaticAsset<A> {
    fn clone(&self) -> Self {
        Self(self.0, std::marker::PhantomData)
    }
}

impl<A: AssetData> Copy for StaticAsset<A> {}

impl<A: AssetData> Serialize for StaticAsset<A> {
    type Header = ();
    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        self.0 .0.serialize(encoder)
    }
    fn deserialize(
        decoder: &mut impl Decoder,
        header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        Ok(Self(
            Asset(SlotId::deserialize(decoder, &Default::default())?),
            std::marker::PhantomData,
        ))
    }
}
