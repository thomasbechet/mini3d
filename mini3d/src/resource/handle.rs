use std::fmt::Debug;

use mini3d_derive::Serialize;

use crate::{
    ecs::entity::Entity,
    utils::{
        prng::PCG32,
        string::AsciiArray,
        uid::{ToUID, UID},
    },
};

use super::{
    key::{ResourceSlotKey, ResourceTypeKey},
    ResourceManager,
};

pub struct ReferenceResolver;

impl ReferenceResolver {
    pub(crate) fn resolve_resource<H: ToResourceHandle + Default>(
        &mut self,
        handle: impl ToResourceHandle,
    ) -> H {
        Default::default()
    }
    pub(crate) fn resolve_entity(&mut self, entity: Entity) -> Entity {
        Default::default()
    }
}

#[derive(Default, Hash, PartialEq, Eq, Clone, Copy, Debug, Serialize)]
pub struct ResourceHandle(u32);

impl ResourceHandle {
    pub(crate) fn type_key(&self) -> ResourceTypeKey {
        ResourceTypeKey {
            version: ((self.0 >> 10) & 0x3) as u8,
            index: (self.0 & 0x3FF) as u16,
        }
    }

    pub(crate) fn slot_key(&self) -> ResourceSlotKey {
        ResourceSlotKey {
            version: ((self.0 >> 26) & 0x3F) as u8,
            index: ((self.0 >> 12) & 0x3FFF) as u16,
        }
    }

    pub(crate) fn new(type_key: ResourceTypeKey, slot_key: ResourceSlotKey) -> Self {
        Self(
            ((type_key.version as u32) << 26)
                | ((type_key.index as u32) << 12)
                | ((slot_key.version as u32) << 10)
                | (slot_key.index as u32),
        )
    }

    pub fn null() -> Self {
        Self(0xFFFFFFFF)
    }

    pub fn is_null(&self) -> bool {
        self.0 == 0xFFFFFFFF
    }

    pub fn resolve(&mut self, resolver: &mut ReferenceResolver) {
        // if !self.0.is_null() {
        //     *self = resolver.resolve_resource(*self);
        // }
    }

    pub(crate) fn release(&mut self, resources: &mut ResourceManager) {
        // if !self.0.is_null() {
        //     resources.decrement_ref(*self);
        //     self.0 = Key::null();
        // }
    }

    pub(crate) fn raw(&self) -> u32 {
        self.0
    }

    pub(crate) fn from_raw(raw: u32) -> Self {
        Self(raw)
    }
}

pub trait ToResourceHandle {
    fn to_handle(&self) -> ResourceHandle;
    fn from_handle(handle: ResourceHandle) -> Self;
}

impl ToResourceHandle for ResourceHandle {
    fn to_handle(&self) -> ResourceHandle {
        *self
    }
    fn from_handle(handle: ResourceHandle) -> Self {
        handle
    }
}

pub(crate) const MAX_RESOURCE_KEY_LEN: usize = 64;

#[derive(Default, Serialize)]
pub struct ResourceName(AsciiArray<MAX_RESOURCE_KEY_LEN>);

impl ResourceName {
    pub(crate) fn new(name: &str) -> Self {
        Self(AsciiArray::from(name))
    }

    pub(crate) fn random(prng: &mut PCG32) -> Self {
        let mut id = AsciiArray::default();
        for i in 0..MAX_RESOURCE_KEY_LEN {
            let c = prng.next_u32() % 26;
            let c = char::from_u32(c + 65).unwrap();
            id.push(c);
        }
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl ToUID for ResourceName {
    fn to_uid(&self) -> UID {
        self.0.as_str().to_uid()
    }
}

impl AsRef<str> for ResourceName {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for ResourceName {
    fn from(key: &str) -> Self {
        Self::new(key)
    }
}
