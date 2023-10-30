use std::{any::Any, cell::RefCell};

use glam::{IVec2, IVec3, IVec4, Mat4, Quat, Vec2, Vec3, Vec4};

use crate::{
    feature::core::component::{ComponentId, ComponentType},
    reflection::PropertyId,
    resource::{handle::ResourceHandle, ResourceManager},
    serialize::{Decoder, DecoderError, Encoder, EncoderError},
    utils::{slotmap::SlotMap, uid::UID},
};

use super::entity::Entity;

pub mod native;

pub(crate) enum ComponentChange {
    Added(Entity),
    Removed(Entity),
}

pub(crate) trait Container {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn serialize(&self, encoder: &mut dyn Encoder) -> Result<(), EncoderError>;
    fn deserialize(&mut self, decoder: &mut dyn Decoder) -> Result<(), DecoderError>;
    fn mark_removed(&mut self, entity: Entity);
    fn remove(&mut self, entity: Entity);
    fn flush_changes(&mut self);
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

struct ContainerEntry {
    pub(crate) container: Box<dyn Container>,
    pub(crate) changes: Vec<ComponentChange>,
    resource_type: ResourceHandle,
}

#[derive(Default)]
pub(crate) struct ContainerTable {
    pub(crate) entries: SlotMap<ContainerEntry>,
}

impl ContainerTable {
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
            changes: Vec::new(),
            resource_type: resources.increment_ref(component),
        };
        ComponentId(self.entries.insert(entry))
    }

    pub(crate) fn remove(&mut self, entity: Entity, component: ComponentId) {
        self.entries
            .get_mut(component.0)
            .expect("Component container not found while removing entity")
            .container
            .get_mut()
            .as_mut()
            .remove(entity);
    }
}
