// pub struct EntityResolver {
//     map: HashMap<hecs::Entity, hecs::Entity>,
// }

// pub trait ResolveEntity {
//     fn resolve(&mut self, resolver: &EntityResolver) -> Result<()>;
// }

use crate::serialize::{Serialize, Decoder, Encoder, EncoderError, DecoderError};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Entity(pub(crate) u32);

pub(crate) type EntityVersion = u8;
pub(crate) type EntityKey = u32;

impl Entity {

    pub(crate) fn new(key: EntityKey, version: EntityVersion) -> Self {
        Self(key | ((version as EntityKey) << 24))
    }

    pub(crate) fn key(&self) -> EntityKey {
        self.0 & 0x00ff_ffff
    }

    pub(crate) fn version(&self) -> EntityVersion {
        (self.0 >> 24) as EntityVersion
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

impl Serialize for Entity {
    
    type Header = ();
    
    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_u32(self.0)?;
        Ok(())
    }
    
    fn deserialize(decoder: &mut impl Decoder, _header: &Self::Header) -> Result<Self, DecoderError> {
        Ok(Self(decoder.read_u32()?))
    }
}