use crate::{
    registry::component::Component,
    serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct AssetId(u32);

pub(crate) type AssetVersion = u8;
pub(crate) type AssetKey = u32;

impl AssetId {
    pub(crate) fn new(key: AssetKey, version: AssetVersion) -> Self {
        Self(key | ((version as AssetKey) << 24))
    }

    pub(crate) fn key(&self) -> AssetKey {
        self.0 & 0x00ff_ffff
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
    fn new(id: AssetId) -> Self;
    fn id(&self) -> AssetId;
}

#[derive(Default, Clone, Copy)]
pub struct StaticAsset<C: Component> {
    _marker: std::marker::PhantomData<C>,
    id: AssetId,
}

impl<C: Component> AssetHandle for StaticAsset<C> {
    type AssetRef<'a> = &'a C;
    fn new(id: AssetId) -> Self {
        Self {
            _marker: std::marker::PhantomData::<C>,
            id,
        }
    }
    fn id(&self) -> AssetId {
        self.id
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
    handle: AssetId,
}

impl Serialize for DynamicAsset {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_u32(self.handle.0)?;
        Ok(())
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        let handle = AssetId(decoder.read_u32()?);
        Ok(Self { handle })
    }
}
