use mini3d_derive::Serialize;

use crate::{
    registry::component::Component,
    serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize},
    utils::slotmap::SlotId,
};

use super::container::{AnyAssetContainer, StaticAssetContainer};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
pub(crate) struct AssetId(u32);

pub(crate) type AssetVersion = u8;

impl AssetId {
    pub(crate) fn new(slot: SlotId, version: AssetVersion) -> Self {
        Self(u32::from(slot) | ((version as u32) << 24))
    }

    pub(crate) fn slot(&self) -> SlotId {
        (self.0 & 0x00ff_ffff).into()
    }

    pub(crate) fn version(&self) -> AssetVersion {
        (self.0 >> 24) as AssetVersion
    }

    pub fn null() -> Self {
        Self(0)
    }
}

impl Default for AssetId {
    fn default() -> Self {
        Self::null()
    }
}

pub(crate) trait AssetHandle {
    type AssetRef<'a>;
    type Contructor;
    fn new(id: AssetId) -> Self;
    fn id(&self) -> AssetId;
    fn asset_ref<'a>(&self, container: &'a dyn AnyAssetContainer) -> Self::AssetRef<'a>;
    fn insert(
        container: &mut dyn AnyAssetContainer,
        asset: Self::Contructor,
        version: AssetVersion,
    ) -> Self;
    fn check_type(container: &dyn AnyAssetContainer) -> bool;
}

#[derive(Default, Clone, Copy)]
pub struct StaticAsset<C: Component> {
    _marker: std::marker::PhantomData<C>,
    id: AssetId,
}

impl<C: Component> AssetHandle for StaticAsset<C> {
    type AssetRef<'a> = &'a C;
    type Contructor = C;
    fn new(id: AssetId) -> Self {
        Self {
            _marker: std::marker::PhantomData::<C>,
            id,
        }
    }
    fn id(&self) -> AssetId {
        self.id
    }
    fn asset_ref<'a>(&self, container: &'a dyn AnyAssetContainer) -> Self::AssetRef<'a> {
        container
            .as_any()
            .downcast_ref::<StaticAssetContainer<C>>()
            .expect("Invalid static asset container")
            .0
            .get(self.id.slot())
            .expect("Asset not found in container")
    }
    fn insert(
        container: &mut dyn AnyAssetContainer,
        asset: Self::Contructor,
        version: AssetVersion,
    ) -> Self {
        Self::new(AssetId::new(
            container
                .as_any_mut()
                .downcast_mut::<StaticAssetContainer<C>>()
                .expect("Invalid static asset container")
                .0
                .add(Box::new(asset)),
            version,
        ))
    }
    fn check_type(container: &dyn AnyAssetContainer) -> bool {
        container
            .as_any()
            .downcast_ref::<StaticAssetContainer<C>>()
            .is_some()
    }
}

impl<C: Component> Serialize for StaticAsset<C> {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_u32(self.id.0)?;
        Ok(())
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        let handle = AssetId(decoder.read_u32()?);
        Ok(Self {
            _marker: std::marker::PhantomData::<C>,
            id: handle,
        })
    }
}

#[derive(Default)]
pub struct DynamicAsset {
    id: AssetId,
}

impl AssetHandle for DynamicAsset {
    type AssetRef<'a> = ();
    type Contructor = ();
    fn new(id: AssetId) -> Self {
        Self { id }
    }
    fn id(&self) -> AssetId {
        self.id
    }
    fn asset_ref<'a>(&self, container: &'a dyn AnyAssetContainer) -> Self::AssetRef<'a> {}
    fn insert(
        container: &mut dyn AnyAssetContainer,
        asset: Self::Contructor,
        version: AssetVersion,
    ) -> Self {
        Self::new(AssetId::null())
    }
    fn check_type(container: &dyn AnyAssetContainer) -> bool {
        true
    }
}

impl Serialize for DynamicAsset {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_u32(self.id.0)?;
        Ok(())
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        let handle = AssetId(decoder.read_u32()?);
        Ok(Self { id: handle })
    }
}
