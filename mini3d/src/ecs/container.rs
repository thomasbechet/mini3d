use std::cell::RefCell;

use crate::{
    registry::component::{
        ComponentHandle, ComponentId, ComponentRegistry, PrivateComponentTableRef,
    },
    serialize::{Decoder, DecoderError, Encoder, EncoderError},
    utils::slotmap::SparseSecondaryMap,
};

use self::single::AnySingleContainer;

use super::{entity::Entity, error::ECSError};

pub mod array;
pub mod list;
pub mod map;
pub mod single;

#[derive(Default)]
pub(crate) struct ContainerTable {
    pub(crate) containers: SparseSecondaryMap<RefCell<Box<dyn AnySingleContainer>>>,
}

impl ContainerTable {
    pub(crate) fn on_registry_update(&mut self, registry: &ComponentRegistry) {
        for (id, entry) in registry.entries.iter() {
            if !self.containers.contains(id) {
                let container = entry.reflection.create_scene_container();
                self.containers.insert(id, RefCell::new(container));
            }
        }
    }

    pub(crate) fn serialize(
        &self,
        registry: &ComponentRegistry,
        encoder: &mut impl Encoder,
    ) -> Result<(), EncoderError> {
        // encoder.write_u32(self.containers.len() as u32)?;
        // for (id, container) in self.containers.iter() {
        //     let uid = UID::new(&registry.definition(id.into()).unwrap().name);
        //     uid.serialize(encoder)?;
        //     container.borrow().serialize(encoder)?;
        // }
        Ok(())
    }

    pub(crate) fn deserialize(
        &mut self,
        registry: &ComponentRegistry,
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

    pub(crate) fn remove(&mut self, entity: Entity, component: ComponentId) {
        self.containers
            .get_mut(component.0)
            .expect("Component container not found while removing entity")
            .get_mut()
            .remove(entity);
    }

    pub(crate) fn view<H: ComponentHandle>(
        &self,
        component: H,
    ) -> Result<H::SingleViewRef<'_>, ECSError> {
        component.single_view_ref(PrivateComponentTableRef(self))
    }

    pub(crate) fn view_mut<H: ComponentHandle>(
        &self,
        component: H,
        cycle: u32,
    ) -> Result<H::SingleViewMut<'_>, ECSError> {
        component.single_view_mut(PrivateComponentTableRef(self), cycle)
    }
}
