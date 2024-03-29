use serde::{Serialize, Deserialize};

// pub struct EntityResolver {
//     map: HashMap<hecs::Entity, hecs::Entity>,
// }

// pub trait ResolveEntity {
//     fn resolve(&mut self, resolver: &EntityResolver) -> Result<()>;
// }

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Entity(pub(crate) usize);

pub(crate) type EntityVersion = u32;

impl Entity {

    pub(crate) fn new(key: usize, version: EntityVersion) -> Self {
        Self(key | ((version as usize) << 32))
    }

    pub(crate) fn key(&self) -> usize {
        self.0 & 0x0000_0000_ffff_ffff
    }

    pub(crate) fn version(&self) -> EntityVersion {
        (self.0 >> 32) as EntityVersion
    }

    pub fn null() -> Self {
        Self(0)
    }

    // pub fn resolve(&mut self, resolver: &EntityResolver) {
    //     if let Some(handle) = resolver.map.get(&self.0) {
    //         self.0 = *handle;
    //     }
    // }
}