use std::{any::Any, cell::UnsafeCell};

use glam::{IVec2, IVec3, IVec4, Mat4, Quat, Vec2, Vec3, Vec4};

use crate::{
    feature::ecs::component::{ComponentId, ComponentType, ComponentTypeHandle},
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
    fn flush_changes(
        &mut self,
        entities: &mut EntityTable,
        queries: &mut QueryTable,
        id: ComponentId,
    );
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
    trait_property!(f32, read_f32, write_f32);
    trait_property!(f64, read_f64, write_f64);
    trait_property!(Vec2, read_vec2, write_vec2);
    trait_property!(IVec2, read_ivec2, write_ivec2);
    trait_property!(Vec3, read_vec3, write_vec3);
    trait_property!(IVec3, read_ivec3, write_ivec3);
    trait_property!(Vec4, read_vec4, write_vec4);
    trait_property!(IVec4, read_ivec4, write_ivec4);
    trait_property!(Mat4, read_mat4, write_mat4);
    trait_property!(Quat, read_quat, write_quat);
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
    pub(crate) entries: SlotMap<ContainerEntry>,
}

impl ContainerTable {
    pub(crate) fn preallocate(
        &mut self,
        component: ComponentTypeHandle,
        resource: &mut ResourceManager,
    ) -> ComponentId {
        // Find existing container
        let id = self.entries.iter().find_map(|(id, e)| {
            if e.component_type == component {
                Some(ComponentId(id))
            } else {
                None
            }
        });
        if let Some(id) = id {
            return id;
        }
        // Create new container
        let ty = resource
            .get::<ComponentType>(component)
            .expect("Component type not found while preallocating");
        let entry = ContainerEntry {
            container: UnsafeCell::new(ty.create_container()),
            component_type: component,
        };
        resource.increment_ref(component.0).unwrap();
        ComponentId(self.entries.add(entry))
    }

    pub(crate) fn remove(&mut self, entity: Entity, component: ComponentId) {
        self.entries
            .get_mut(component.0)
            .expect("Component container not found while removing entity")
            .container
            .get_mut()
            .remove(entity);
    }
}
