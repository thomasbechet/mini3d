use std::{any::Any, collections::HashSet};

use crate::{
    feature::core::resource::Resource,
    serialize::{Decoder, Encoder},
    utils::{
        slotmap::{SlotId, SlotMap},
        uid::UID,
    },
};

use super::error::ResourceError;

pub struct PrivateResourceContainerRef<'a>(pub(crate) &'a dyn ResourceContainer);
pub struct PrivateResourceContainerMut<'a>(pub(crate) &'a mut dyn ResourceContainer);

#[derive(Default)]
pub(crate) struct NativeResourceContainer<R: Resource>(pub(crate) SlotMap<R>);

impl<R: Resource> NativeResourceContainer<R> {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self(SlotMap::with_capacity(capacity))
    }
}

pub(crate) trait ResourceContainer: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn remove(&mut self, slot: SlotId);
    fn clear(&mut self);
    fn serialize_entries(
        &self,
        set: &HashSet<UID>,
        encoder: &mut dyn Encoder,
    ) -> Result<(), ResourceError>;
    fn deserialize_entries(
        &mut self,
        bundle: UID,
        decoder: &mut dyn Decoder,
    ) -> Result<(), ResourceError>;
}

impl<R: Resource> ResourceContainer for NativeResourceContainer<R> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut (dyn Any + 'static) {
        self
    }

    fn remove(&mut self, slot: SlotId) {
        self.0.remove(slot);
    }

    fn clear(&mut self) {
        self.0.clear();
    }

    fn serialize_entries(
        &self,
        set: &HashSet<UID>,
        mut encoder: &mut dyn Encoder,
    ) -> Result<(), ResourceError> {
        Ok(())
    }

    fn deserialize_entries(
        &mut self,
        bundle: UID,
        mut decoder: &mut dyn Decoder,
    ) -> Result<(), ResourceError> {
        Ok(())
    }
}
