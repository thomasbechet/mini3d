use std::cell::RefCell;

use mini3d_derive::Serialize;

use crate::{
    feature::core::component::{
        ComponentId, ComponentType, PrivateComponentTableMut, PrivateComponentTableRef,
    },
    resource::{handle::ResourceHandle, ResourceManager},
    serialize::{Decoder, Encoder},
    utils::slotmap::SlotMap,
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

struct ContainerEntry {
    container: RefCell<Box<dyn Container>>,
    resource_type: ResourceHandle,
}

#[derive(Default)]
pub(crate) struct ContainerTable {
    pub(crate) entries: SlotMap<ContainerEntry>,
}

impl ContainerTable {
    // pub(crate) fn serialize(
    //     &self,
    //     registry: &ComponentRegistryManager,
    //     encoder: &mut impl Encoder,
    // ) -> Result<(), EncoderError> {
    //     // encoder.write_u32(self.containers.len() as u32)?;
    //     // for (id, container) in self.containers.iter() {
    //     //     let uid = UID::new(&registry.definition(id.into()).unwrap().name);
    //     //     uid.serialize(encoder)?;
    //     //     container.borroComponentTypeHandlencoder)?;
    //     // }
    //     Ok(())
    // }

    // pub(crate) fn deserialize(
    //     &mut self,
    //     registry: &ComponentRegistryManager,
    //     decoder: &mut impl Decoder,
    // ) -> Result<(), DecoderError> {
    //     // self.containers.clear();
    //     // let count = decoder.read_u32()?;
    //     // for i in 0..count {
    //     //     let uid = UID::deserialize(decoder, &Default::default())?;
    //     //     let id = registry
    //     //         .find_id(uid)
    //     //         .expect("Component ID not found while deserializing");
    //     //     self.preallocate(id, registry);
    //     //     self.containers[id.into()]
    //     //         .borrow_mut()
    //     //         .deserialize(decoder)?;
    //     // }
    //     Ok(())
    // }

    pub(crate) fn preallocate(
        &mut self,
        component: ResourceHandle,
        resources: &mut ResourceManager,
    ) -> ComponentId {
        // Find existing container
        let id = self.entries.iter().find_map(|(id, e)| {
            if e.resource_type.handle() == component {
                Some(ComponentId(id))
            } else {
                None
            }
        });
        if let Some(id) = id {
            return id;
        }
        // Create new container
        let ty = resources
            .read::<ComponentType>(component)
            .expect("Component type not found while preallocating");
        let entry = ContainerEntry {
            container: RefCell::new(ty.create_container()),
            resource_type: resources.increment_ref(component),
        };
        ComponentId(self.entries.insert(entry))
    }

    pub(crate) fn remove(&mut self, entity: Entity, component: ComponentId) {
        self.containers
            .get_mut(component.0)
            .expect("Component container not found while removing entity")
            .get_mut()
            .remove(entity);
    }

    pub(crate) fn view<V: ComponentViewRef>(&self, component: ComponentId) -> V {
        V::view(PrivateComponentTableRef(self), component)
    }

    pub(crate) fn view_mut<V: ComponentViewMut>(
        &mut self,
        component: ComponentId,
        cycle: u32,
    ) -> V {
        V::view_mut(PrivateComponentTableMut(self), component, cycle)
    }
}
