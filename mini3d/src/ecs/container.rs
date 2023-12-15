use core::{any::Any, cell::UnsafeCell};

use alloc::boxed::Box;

use crate::{
    ecs::resource::component::{ComponentKey, ComponentType, ComponentTypeHandle},
    math::{
        mat::M4I32F16,
        quat::QI32F16,
        vec::{V2I32, V2I32F16, V3I32, V3I32F16, V4I32, V4I32F16},
    },
    reflection::PropertyId,
    resource::ResourceManager,
    serialize::{Decoder, DecoderError, Encoder, EncoderError},
    utils::{slotmap::SlotMap, uid::UID},
};

use super::{
    entity::{Entity, EntityTable},
    query::QueryTable,
};

pub mod native;

pub(crate) trait Container {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn as_single(&self) -> &dyn SingleContainer;
    fn as_single_mut(&mut self) -> &mut dyn SingleContainer;
    fn serialize(&self, encoder: &mut dyn Encoder) -> Result<(), EncoderError>;
    fn deserialize(&mut self, decoder: &mut dyn Decoder) -> Result<(), DecoderError>;
    fn mark_removed(&mut self, entity: Entity);
    fn remove(&mut self, entity: Entity);
    fn flush_added_removed(
        &mut self,
        entities: &mut EntityTable,
        queries: &mut QueryTable,
        id: ComponentKey,
    );
    fn update_view_size(&mut self);
}

#[macro_export]
macro_rules! trait_property {
    ($type:ty, $read:ident, $write:ident) => {
        fn $read(&self, entity: Entity, id: PropertyId) -> Option<$type>;
        fn $write(&mut self, entity: Entity, id: PropertyId, value: $type);
    };
}

#[macro_export]
macro_rules! trait_property_impl {
    ($type:ty, $read:ident, $write:ident) => {
        fn $read(&self, entity: Entity, id: PropertyId) -> Option<$type> {
            None
        }
        fn $write(&mut self, entity: Entity, id: PropertyId, value: $type) {}
    };
}

pub(crate) trait SingleContainer {
    trait_property!(bool, read_bool, write_bool);
    trait_property!(u8, read_u8, write_u8);
    trait_property!(i32, read_i32, write_i32);
    trait_property!(u32, read_u32, write_u32);
    trait_property!(V2I32F16, read_v2i32f16, write_v2i32f16);
    trait_property!(V2I32, read_v2i32, write_v2i32);
    trait_property!(V3I32F16, read_v3i32f16, write_v3i32f16);
    trait_property!(V3I32, read_v3i32, write_ivec3);
    trait_property!(V4I32F16, read_v4i32f16, write_v4i32f16);
    trait_property!(V4I32, read_v4i32, write_v4i32);
    trait_property!(M4I32F16, read_m4i32f16, write_m4i32f16);
    trait_property!(QI32F16, read_qi32f16, write_qi32f16);
    trait_property!(Entity, read_entity, write_entity);
    trait_property!(UID, read_uid, write_uid);
}

pub(crate) trait ArrayContainer {}

pub(crate) struct ContainerEntry {
    pub(crate) container: UnsafeCell<Box<dyn Container>>,
    component_type: ComponentTypeHandle,
}

#[derive(Default)]
pub(crate) struct ContainerTable {
    pub(crate) entries: SlotMap<ComponentKey, ContainerEntry>,
}

impl ContainerTable {
    pub(crate) fn preallocate(
        &mut self,
        component: ComponentTypeHandle,
        resource: &mut ResourceManager,
    ) -> ComponentKey {
        // Find existing container
        let key = self.entries.iter().find_map(|(key, e)| {
            if e.component_type == component {
                Some(key)
            } else {
                None
            }
        });
        if let Some(key) = key {
            return key;
        }
        // Create new container
        let ty = resource
            .native::<ComponentType>(component)
            .expect("Component type not found while preallocating");
        let entry = ContainerEntry {
            container: UnsafeCell::new(ty.create_container()),
            component_type: component,
        };
        resource.increment_ref(component.0).unwrap();
        self.entries.add(entry)
    }

    pub(crate) fn remove(&mut self, entity: Entity, component: ComponentKey) {
        self.entries
            .get_mut(component)
            .expect("Component container not found while removing entity")
            .container
            .get_mut()
            .remove(entity);
    }
}
