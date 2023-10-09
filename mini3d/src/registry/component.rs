use std::any::TypeId;

use crate::{
    ecs::container::{
        native::single::{NativeSingleContainer, SingleContainer},
        ContainerTable,
    },
    feature::common::component_definition::ComponentDefinition,
    program::{ProgramId, ProgramManager},
    reflection::{Property, Reflect},
    resource::{
        handle::{ReferenceResolver, ResourceHandle},
        ResourceManager,
    },
    serialize::{Decoder, DecoderError, Encoder, EncoderError},
    utils::{
        slotmap::{SlotId, SlotMap},
        uid::{ToUID, UID},
    },
};

use super::{component_type::ComponentType, error::RegistryError};

pub trait Component: 'static + Default + Reflect {
    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError>;
    fn deserialize(&mut self, decoder: &mut impl Decoder) -> Result<(), DecoderError>;
    fn resolve_references(&mut self, references: &mut ReferenceResolver);
}

pub struct PrivateComponentTableRef<'a>(pub(crate) &'a ContainerTable);
pub struct PrivateComponentTableMut<'a>(pub(crate) &'a mut ContainerTable);

pub(crate) trait ComponentReflection {
    fn create_scene_container(&self) -> Box<dyn SingleContainer>;
    fn find_property(&self, name: &str) -> Option<&Property>;
    fn properties(&self) -> &[Property];
    fn type_id(&self) -> TypeId;
}

pub(crate) struct NativeComponentReflection<C: Component> {
    pub(crate) _phantom: std::marker::PhantomData<C>,
}

impl<C: Component> ComponentReflection for NativeComponentReflection<C> {
    fn create_scene_container(&self) -> Box<dyn SingleContainer> {
        Box::new(NativeSingleContainer::<C>::with_capacity(128))
    }

    fn find_property(&self, name: &str) -> Option<&Property> {
        C::PROPERTIES.iter().find(|p| p.name == name)
    }

    fn properties(&self) -> &[Property] {
        C::PROPERTIES
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<C>()
    }
}

pub struct ComponentEntry {
    definition: ResourceHandle,
    owner: ProgramId,
}

#[derive(Default)]
pub struct ComponentRegistryManager {
    pub(crate) entries: SlotMap<ComponentEntry>,
    pub(crate) changed: bool,
}

impl ComponentRegistryManager {
    pub(crate) fn add(
        &mut self,
        definition: ComponentDefinition,
        owner: ProgramId,
        programs: &ProgramManager,
        resources: &ResourceManager,
    ) -> Result<SlotId, RegistryError> {
        let uid: UID = definition.name.into();
        let mut current = owner;
        if self.find(uid, owner, programs, resources).is_some() {
            return Err(RegistryError::DuplicatedComponent);
        }
        self.changed = true;
        Ok(self.entries.add(ComponentEntry { definition, owner }))
    }

    pub(crate) fn find(
        &self,
        component: impl ToUID,
        owner: ProgramId,
        programs: &ProgramManager,
        resources: &ResourceManager,
    ) -> Option<ComponentType> {
        // Find entry
        let uid = component.to_uid();
        let mut current = owner;
        while !current.0.is_null() {
            if let Some((id, e)) = self.entries.iter().find(|(_, e)| {
                e.owner == current
                    && resources
                        .read::<ComponentDefinition>(e.definition)
                        .unwrap()
                        .name
                        .to_uid()
                        == uid
            }) {
                return Some(ComponentType(id));
            }
            current = programs.entries[current.0].parent;
        }
        return None;
    }
}
