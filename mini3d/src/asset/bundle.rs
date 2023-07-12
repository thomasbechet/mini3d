use std::collections::{HashMap, HashSet};

use crate::{
    registry::component::ComponentRegistry,
    serialize::{Decoder, Serialize},
    uid::UID,
};

use super::{container::AnyAssetContainer, error::AssetError};

pub struct ImportAssetBundle {
    pub(crate) name: String,
    pub(crate) containers: HashMap<UID, Box<dyn AnyAssetContainer>>,
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
        let mut containers: HashMap<UID, Box<dyn AnyAssetContainer>> = Default::default();
        for _ in 0..len {
            let asset =
                UID::deserialize(decoder, &()).map_err(|_| AssetError::DeserializationError)?;
            let definition = registry
                .get(asset)
                .ok_or(AssetError::AssetTypeNotFound { uid: asset })?;
            let mut container = definition.reflection.create_asset_container();
            container.deserialize_entries(bundle, decoder)?;
            if containers.contains_key(&asset) {
                return Err(AssetError::DuplicatedAssetType { uid: asset });
            }
            containers.insert(asset, container);
        }
        Ok(ImportAssetBundle { name, containers })
    }
}

pub(crate) struct AssetBundle {
    pub(crate) name: String,
    pub(crate) assets: HashMap<UID, HashSet<UID>>,
}

impl AssetBundle {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            assets: Default::default(),
        }
    }
}
