use std::collections::HashMap;

use crate::{uid::UID, asset::{AnyAssetContainer, AssetContainer}, serialize::Serialize};

use super::error::RegistryError;

pub trait Asset: Sized + Serialize + 'static {
    const NAME: &'static str;
    const UID: UID = UID::new(Self::NAME);
}

pub(crate) trait AnyAssetDefinitionReflection {
    fn create_container(&self) -> Box<dyn AnyAssetContainer>;
}

pub(crate) struct AssetDefinitionReflection<A: Asset> {
    _phantom: std::marker::PhantomData<A>,
}

impl<A: Asset> AnyAssetDefinitionReflection for AssetDefinitionReflection<A> {
    fn create_container(&self) -> Box<dyn AnyAssetContainer> {
        Box::<AssetContainer<A>>::default()
    }
}

pub(crate) struct AssetDefinition {
    pub(crate) name: String,
    pub(crate) reflection: Box<dyn AnyAssetDefinitionReflection>,
}

#[derive(Default)]
pub struct AssetRegistry {
    assets: HashMap<UID, AssetDefinition>,
}

impl AssetRegistry {

    pub(crate) fn define_static<A: Asset>(&mut self, name: &str) -> Result<UID, RegistryError> {
        let uid: UID = name.into();
        if self.assets.contains_key(&uid) {
            return Err(RegistryError::DuplicatedAssetDefinition { name: name.to_owned() });
        }
        self.assets.insert(uid, AssetDefinition { 
            name: name.to_owned(), 
            reflection: Box::new(AssetDefinitionReflection::<A> { _phantom: std::marker::PhantomData }) 
        });
        Ok(uid)
    }

    // TODO: support dynamic assets ???

    pub(crate) fn get(&self, uid: UID) -> Result<&AssetDefinition, RegistryError> {
        self.assets.get(&uid).ok_or(RegistryError::AssetDefinitionNotFound { uid })
    }
}