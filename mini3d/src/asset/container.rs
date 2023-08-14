use std::{any::Any, collections::HashSet};

use crate::{
    registry::component::Component,
    serialize::{Decoder, Encoder},
    utils::{slotmap::SlotMap, uid::UID},
};

use super::error::AssetError;

#[derive(Default)]
pub(crate) struct StaticAssetContainer<C: Component>(pub(crate) SlotMap<Box<C>>);

pub(crate) trait AnyAssetContainer: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn clear(&mut self);
    fn serialize_entries(
        &self,
        set: &HashSet<UID>,
        encoder: &mut dyn Encoder,
    ) -> Result<(), AssetError>;
    fn deserialize_entries(
        &mut self,
        bundle: UID,
        decoder: &mut dyn Decoder,
    ) -> Result<(), AssetError>;
}

impl<C: Component> AnyAssetContainer for StaticAssetContainer<C> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut (dyn Any + 'static) {
        self
    }

    fn clear(&mut self) {
        self.0.clear();
    }

    fn serialize_entries(
        &self,
        set: &HashSet<UID>,
        mut encoder: &mut dyn Encoder,
    ) -> Result<(), AssetError> {
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
    ) -> Result<(), AssetError> {
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
