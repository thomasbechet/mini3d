use std::cell::RefCell;

use mini3d_derive::Serialize;

use crate::{
    registry::{
        component::{ComponentRegistryManager, ComponentTypeTrait, PrivateComponentTableRef},
        component_type::ComponentType,
    },
    serialize::{Decoder, DecoderError, Encoder, EncoderError},
    utils::slotmap::SparseSecondaryMap,
};

use super::{
    entity::Entity,
    view::{ComponentViewMut, ComponentViewRef},
};

pub mod native;

pub(crate) trait Container {}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ComponentStatus {
    #[default]
    Unchanged = 0b00,
    Changed = 0b01,
    Added = 0b10,
    Removed = 0b11,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
struct ComponentFlags(u32);

impl ComponentFlags {
    fn added(cycle: u32) -> Self {
        let mut flags = Self(0);
        flags.set(ComponentStatus::Added, cycle);
        flags
    }

    fn cycle(&self) -> u32 {
        self.0 >> 2
    }

    fn status(&self) -> ComponentStatus {
        match self.0 & 0b11 {
            0b00 => ComponentStatus::Unchanged,
            0b01 => ComponentStatus::Changed,
            0b10 => ComponentStatus::Added,
            0b11 => ComponentStatus::Removed,
            _ => unreachable!(),
        }
    }

    fn set(&mut self, status: ComponentStatus, cycle: u32) {
        self.0 = ((cycle << 2) & (!0b11)) | status as u32;
    }
}

#[derive(Default)]
pub(crate) struct ContainerTable {
    pub(crate) containers: SparseSecondaryMap<RefCell<Box<dyn Container>>>,
}

impl ContainerTable {
    pub(crate) fn on_registry_update(&mut self, registry: &ComponentRegistryManager) {
        for (id, entry) in registry.entries.iter() {
            if !self.containers.contains(id) {
                let container = entry.reflection.create_scene_container();
                self.containers.insert(id, RefCell::new(container));
            }
        }
    }

    pub(crate) fn serialize(
        &self,
        registry: &ComponentRegistryManager,
        encoder: &mut impl Encoder,
    ) -> Result<(), EncoderError> {
        // encoder.write_u32(self.containers.len() as u32)?;
        // for (id, container) in self.containers.iter() {
        //     let uid = UID::new(&registry.definition(id.into()).unwrap().name);
        //     uid.serialize(encoder)?;
        //     container.borroComponentTypeHandlencoder)?;
        // }
        Ok(())
    }

    pub(crate) fn deserialize(
        &mut self,
        registry: &ComponentRegistryManager,
        decoder: &mut impl Decoder,
    ) -> Result<(), DecoderError> {
        // self.containers.clear();
        // let count = decoder.read_u32()?;
        // for i in 0..count {
        //     let uid = UID::deserialize(decoder, &Default::default())?;
        //     let id = registry
        //         .find_id(uid)
        //         .expect("Component ID not found while deserializing");
        //     self.preallocate(id, registry);
        //     self.containers[id.into()]
        //         .borrow_mut()
        //         .deserialize(decoder)?;
        // }
        Ok(())
    }

    pub(crate) fn remove(&mut self, entity: Entity, component: ComponentType) {
        self.containers
            .get_mut(component.0)
            .expect("Component container not found while removing entity")
            .get_mut()
            .remove(entity);
    }

    pub(crate) fn view<V: ComponentViewRef>(&self, ty: ComponentType) -> V {
        V::view(PrivateComponentTableRef(self), ty)
    }

    pub(crate) fn view_mut<V: ComponentViewMut>(&self, ty: ComponentType, cycle: u32) -> V {
        V::view_mut(PrivateComponentTableRef(self), ty, cycle)
    }
}
