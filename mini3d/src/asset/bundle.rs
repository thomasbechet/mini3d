use std::collections::HashSet;

use crate::{
    registry::component::{ComponentId, ComponentRegistry},
    serialize::{Decoder, Serialize},
    utils::{slotmap::SparseSecondaryMap, uid::UID},
};

use super::{container::AnyAssetContainer, error::AssetError};

pub struct ImportAssetBundle {
    pub(crate) name: String,
    pub(crate) containers: SparseSecondaryMap<ComponentId, Box<dyn AnyAssetContainer>>,
}

impl ImportAssetBundle {
    pub(crate) fn deserialize(
        registry: &ComponentRegistry,
        decoder: &mut impl Decoder,
    ) -> Result<ImportAssetBundle, AssetError> {
        let name = String::deserialize(decoder, &Default::default())
            .map_err(|_| AssetError::DeserializationError)?;
        let bundle = UID::new(&name);
        let len = decoder
            .read_u32()
            .map_err(|_| AssetError::DeserializationError)? as usize;
        let mut containers: SparseSecondaryMap<ComponentId, Box<dyn AnyAssetContainer>> =
            Default::default();
        for _ in 0..len {
            let asset =
                UID::deserialize(decoder, &()).map_err(|_| AssetError::DeserializationError)?;
            let (id, definition) = registry.find(asset).ok_or(AssetError::AssetTypeNotFound)?;
            let mut container = definition.reflection.create_asset_container();
            container.deserialize_entries(bundle, decoder)?;
            if containers.contains(id) {
                return Err(AssetError::DuplicatedAssetType { uid: asset });
            }
            containers.insert(id, container);
        }
        Ok(ImportAssetBundle { name, containers })
    }
}

pub(crate) struct AssetBundle {
    pub(crate) name: String,
    pub(crate) assets: SparseSecondaryMap<ComponentId, HashSet<UID>>,
}

impl AssetBundle {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            assets: Default::default(),
        }
    }
}
