use std::{collections::HashMap, any::TypeId};

use anyhow::{anyhow, Context, Result};

use crate::{uid::UID, asset::AnyAssetContainer};

pub(crate) trait AnyAssetDefinitionReflection {
    fn create_container(&self) -> Box<dyn AnyAssetContainer>;
}

pub(crate) struct AssetDefinitionReflection<A> {
    _phantom: std::marker::PhantomData<A>,
}

pub(crate) struct AssetDefinition {
    pub(crate) name: String,
    pub(crate) reflection: Box<dyn AnyAssetDefinitionReflection>,
}

#[derive(Default)]
pub struct AssetRegistry {
    assets: HashMap<UID, AssetDefinition>,
    types_to_uid: HashMap<TypeId, UID>,
}

impl AssetRegistry {

    pub(crate) fn define_compiled<A>(&mut self, name: &str) -> Result<()> {
        let type_id = TypeId::of::<A>();
        let uid: UID = name.into();
        if self.types_to_uid.contains_key(&type_id) || self.assets.contains_key(&uid) {
            return Err(anyhow!("Asset type already defined"));
        }
        self.assets.insert(uid, AssetDefinition { 
            name: name.to_owned(), 
            reflection: Box::new(AssetDefinitionReflection::<A> { _phantom: std::marker::PhantomData }) 
        });
        self.types_to_uid.insert(type_id, uid);
        Ok(())
    }

    // TODO: support runtime assets ???

    pub(crate) fn get(&self, uid: UID) -> Result<&AssetDefinition> {
        self.assets.get(&uid).with_context(|| "Asset not found")
    }

    pub(crate) fn uid_from_type<A>(&self) -> Result<UID> {
        let type_id = TypeId::of::<A>();
        self.types_to_uid.get(&type_id).to_owned().with_context(|| "Asset type not found")
    }
}