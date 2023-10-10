use std::any::TypeId;

use crate::{
    feature::common::resource_definition::ResourceDefinition,
    program::{ProgramId, ProgramManager},
    reflection::{Property, Reflect},
    resource::{
        container::{NativeResourceContainer, ResourceContainer},
        handle::{ReferenceResolver, ResourceRef},
        ResourceManager,
    },
    serialize::{Decoder, DecoderError, Encoder, EncoderError},
    utils::{
        slotmap::{SlotId, SlotMap},
        uid::{ToUID, UID},
    },
};

use super::error::RegistryError;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceType(pub(crate) SlotId);

pub trait Resource: 'static + Default + Reflect {
    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError>;
    fn deserialize(&mut self, decoder: &mut impl Decoder) -> Result<(), DecoderError>;
    fn resolve_references(&mut self, references: &mut ReferenceResolver);
}

pub(crate) trait ResourceReflection {
    fn create_resource_container(&self) -> Box<dyn ResourceContainer>;
    fn find_property(&self, name: &str) -> Option<&Property>;
    fn properties(&self) -> &[Property];
    fn type_id(&self) -> TypeId;
}

pub(crate) struct NativeResourceReflection<R: Resource> {
    pub(crate) _phantom: std::marker::PhantomData<R>,
}

impl<R: Resource> ResourceReflection for NativeResourceReflection<R> {
    fn create_resource_container(&self) -> Box<dyn ResourceContainer> {
        Box::new(NativeResourceContainer::<R>::with_capacity(128))
    }

    fn find_property(&self, name: &str) -> Option<&Property> {
        R::PROPERTIES.iter().find(|p| p.name == name)
    }

    fn properties(&self) -> &[Property] {
        R::PROPERTIES
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<R>()
    }
}

pub(crate) struct ResourceEntry {
    pub(crate) definition: ResourceRef,
    pub(crate) owner: ProgramId,
}

#[derive(Default)]
pub struct ResourceRegistryManager {
    pub(crate) entries: SlotMap<ResourceEntry>,
    pub(crate) changed: bool,
}

impl ResourceRegistryManager {
    fn add(
        &mut self,
        definition: ResourceDefinition,
        owner: ProgramId,
        programs: &ProgramManager,
        resources: &ResourceManager,
    ) -> Result<SlotId, RegistryError> {
        let uid: UID = definition.name.into();
        if self.find(uid, owner, programs, resources).is_some() {
            return Err(RegistryError::DuplicatedResource);
        }
        self.changed = true;
        Ok(self.entries.add(ResourceEntry { definition, owner }))
    }

    pub(crate) fn find(
        &self,
        resource: impl ToUID,
        owner: ProgramId,
        programs: &ProgramManager,
        resources: &ResourceManager,
    ) -> Option<ResourceType> {
        let uid = resource.to_uid();
        let mut current = owner;
        while !current.0.is_null() {
            if let Some((id, e)) = self.entries.iter().find(|(_, e)| {
                e.owner == current
                    && resources
                        .read::<ResourceDefinition>(e.definition)
                        .unwrap()
                        .name
                        .to_uid()
                        == uid
            }) {
                return Some(ResourceType(id));
            }
            current = programs.entries[current.0].parent;
        }
        return None;
    }
}
