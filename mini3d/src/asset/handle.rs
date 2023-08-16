use crate::{
    registry::component::Component,
    serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize},
    utils::generation::{GenerationId, VersionId},
};

use super::container::{AnyAssetContainer, StaticAssetContainer};

#[derive(Default)]
pub struct AssetBundleId(GenerationId);

impl AssetBundleId {
    pub(crate) fn new(id: GenerationId) -> Self {
        Self(id)
    }

    pub(crate) fn id(&self) -> GenerationId {
        self.0
    }
}

pub(crate) trait AssetHandle {
    type AssetRef<'a>;
    type Contructor;
    fn new(id: GenerationId) -> Self;
    fn id(&self) -> GenerationId;
    fn asset_ref<'a>(&self, container: &'a dyn AnyAssetContainer) -> Self::AssetRef<'a>;
    fn insert(
        container: &mut dyn AnyAssetContainer,
        asset: Self::Contructor,
        version: VersionId,
    ) -> Self;
    fn check_type(container: &dyn AnyAssetContainer) -> bool;
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
            _marker: self._marker.clone(),
            id: self.id.clone(),
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
    type Contructor = C;
    fn new(id: GenerationId) -> Self {
        Self {
            _marker: std::marker::PhantomData::<C>,
            id,
        }
    }
    fn id(&self) -> GenerationId {
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
        version: VersionId,
    ) -> Self {
        Self::new(GenerationId::from_slot(
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
    type Contructor = ();
    fn new(id: GenerationId) -> Self {
        Self { id }
    }
    fn id(&self) -> GenerationId {
        self.id
    }
    fn asset_ref<'a>(&self, container: &'a dyn AnyAssetContainer) -> Self::AssetRef<'a> {}
    fn insert(
        container: &mut dyn AnyAssetContainer,
        asset: Self::Contructor,
        version: VersionId,
    ) -> Self {
        Self::new(GenerationId::null())
    }
    fn check_type(container: &dyn AnyAssetContainer) -> bool {
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
