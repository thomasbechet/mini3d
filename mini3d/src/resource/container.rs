use std::{any::Any, collections::HashSet};

use crate::{
    registry::resource::Resource,
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
        // C::Header::default()
        //     .serialize(&mut encoder)
        //     .map_err(|_| AssetError::SerializationError)?;
        // encoder
        //     .write_u32(set.len() as u32)
        //     .map_err(|_| AssetError::SerializationError)?;
        // for uid in set {
        //     let entry = self
        //         .0
        //         .get(uid)
        //         .ok_or(AssetError::AssetNotFound { uid: *uid })?;
        //     entry
        //         .serialize(&mut encoder)
        //         .map_err(|_| AssetError::SerializationError)?;
        // }
        Ok(())
    }

    fn deserialize_entries(
        &mut self,
        bundle: UID,
        mut decoder: &mut dyn Decoder,
    ) -> Result<(), ResourceError> {
        // let header = C::Header::deserialize(&mut decoder, &Default::default())
        //     .map_err(|_| AssetError::DeserializationError)?;
        // let len = decoder
        //     .read_u32()
        //     .map_err(|_| AssetError::DeserializationError)? as usize;
        // for _ in 0..len {
        //     let mut entry = StaticAssetEntry::<C>::deserialize(&mut decoder, &header)
        //         .map_err(|_| AssetError::DeserializationError)?;
        //     entry.bundle = bundle;
        //     self.0.insert(entry.uid(), entry);
        // }
        Ok(())
    }
}
