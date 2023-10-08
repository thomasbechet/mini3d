use std::any::TypeId;

use crate::{
    reflection::Property,
    resource::container::{
        AnyResourceContainer, PrivateAnyResourceContainerMut, PrivateAnyResourceContainerRef,
        StaticResourceContainer,
    },
    utils::{
        slotmap::{SlotId, SlotMap},
        string::AsciiArray,
        uid::{ToUID, UID},
    },
};

use super::{datatype::StaticDataType, error::RegistryError};

pub trait ResourceTypeTrait: Copy {
    type Ref<'a>;
    type Data: Default;
    fn new(id: SlotId) -> Self;
    fn id(&self) -> SlotId;
    fn insert_container(container: PrivateAnyResourceContainerMut, data: Self::Data) -> SlotId;
    fn resource_ref(container: PrivateAnyResourceContainerRef, slot: SlotId) -> Self::Ref<'_>;
    fn check_type_id(id: TypeId) -> bool;
}

pub trait ResourceReferenceTrait {
    type AssetType: ResourceTypeTrait;
}

impl<T: StaticDataType> ResourceReferenceTrait for T {
    type AssetType = StaticResourceType<T>;
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceType(pub(crate) SlotId);

impl ResourceTypeTrait for ResourceType {
    type Ref<'a> = ();
    type Data = ();

    fn new(id: SlotId) -> Self {
        Self(id)
    }

    fn id(&self) -> SlotId {
        self.0
    }

    fn insert_container(container: PrivateAnyResourceContainerMut, data: Self::Data) -> SlotId {
        todo!()
    }

    fn resource_ref(container: PrivateAnyResourceContainerRef, slot: SlotId) -> Self::Ref<'_> {
        todo!()
    }

    fn check_type_id(id: TypeId) -> bool {
        true
    }
}

pub struct StaticResourceType<C: Component> {
    _marker: std::marker::PhantomData<D>,
    pub(crate) id: ResourceType,
}

impl<C: Component> Clone for StaticResourceType<D> {
    fn clone(&self) -> Self {
        Self {
            _marker: std::marker::PhantomData,
            id: self.id,
        }
    }
}

impl<C: Component> Copy for StaticResourceType<D> {}

impl<C: Component> Default for StaticResourceType<D> {
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
            id: ResourceType::default(),
        }
    }
}

impl<C: Component> ResourceTypeTrait for StaticResourceType<D> {
    type Ref<'a> = &'a D;
    type Data = D;

    fn new(id: SlotId) -> Self {
        Self {
            _marker: std::marker::PhantomData,
            id: ResourceType(id),
        }
    }

    fn id(&self) -> SlotId {
        self.id.0
    }

    fn insert_container(container: PrivateAnyResourceContainerMut, resource: Self::Data) -> SlotId {
        container
            .0
            .as_any_mut()
            .downcast_mut::<StaticResourceContainer<D>>()
            .expect("Invalid static resource container")
            .0
            .add(resource)
    }

    fn resource_ref(container: PrivateAnyResourceContainerRef, slot: SlotId) -> Self::Ref<'_> {
        container
            .0
            .as_any()
            .downcast_ref::<StaticResourceContainer<D>>()
            .expect("Invalid static resource container")
            .0
            .get(slot)
            .expect("Invalid static resource slot")
    }

    fn check_type_id(id: TypeId) -> bool {
        id == TypeId::of::<D>()
    }
}

pub(crate) const MAX_RESOURCE_TYPE_NAME_LEN: usize = 64;

pub(crate) enum ResourceKind {
    Static,
    Dynamic,
}

pub(crate) trait AnyResourceReflection {
    fn create_resource_container(&self) -> Box<dyn AnyResourceContainer>;
    fn find_property(&self, name: &str) -> Option<&Property>;
    fn properties(&self) -> &[Property];
    fn type_id(&self) -> TypeId;
}

pub(crate) struct StaticResourceReflection<C: Component> {
    _phantom: std::marker::PhantomData<D>,
}

impl<C: Component> AnyResourceReflection for StaticResourceReflection<D> {
    fn create_resource_container(&self) -> Box<dyn AnyResourceContainer> {
        Box::<StaticResourceContainer<D>>::default()
    }

    fn find_property(&self, name: &str) -> Option<&Property> {
        D::PROPERTIES.iter().find(|p| p.name == name)
    }

    fn properties(&self) -> &[Property] {
        D::PROPERTIES
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<D>()
    }
}

pub(crate) struct ResourceEntry {
    pub(crate) name: AsciiArray<MAX_RESOURCE_TYPE_NAME_LEN>,
    pub(crate) reflection: Box<dyn AnyResourceReflection>,
    pub(crate) kind: ResourceKind,
}

#[derive(Default)]
pub struct ResourceRegistryManager {
    pub(crate) entries: SlotMap<ResourceEntry>,
    pub(crate) changed: bool,
}

impl ResourceRegistryManager {
    fn add(
        &mut self,
        name: &str,
        kind: ResourceKind,
        reflection: Box<dyn AnyResourceReflection>,
    ) -> Result<SlotId, RegistryError> {
        let uid: UID = name.into();
        if self.contains(uid) {
            return Err(RegistryError::DuplicatedComponent);
        }
        self.changed = true;
        Ok(self.entries.add(ResourceEntry {
            name: name.into(),
            kind,
            reflection,
        }))
    }

    pub(crate) fn add_static<C: Component>(
        &mut self,
        name: &str,
    ) -> Result<StaticResourceType<D>, RegistryError> {
        let reflection = StaticResourceReflection::<D> {
            _phantom: std::marker::PhantomData,
        };
        let id = self.add(name, ResourceKind::Static, Box::new(reflection))?;
        Ok(StaticResourceType {
            _marker: std::marker::PhantomData,
            id: ResourceType(id),
        })
    }

    pub(crate) fn find<H: ResourceTypeTrait>(&self, resource: impl ToUID) -> Option<H> {
        // Find entry
        let resource = resource.to_uid();
        let resource = self
            .entries
            .iter()
            .find(|(_, def)| UID::new(&def.name) == resource)
            .map(|(id, _)| id);
        // Check type
        if let Some(id) = resource {
            if !H::check_type_id(self.entries[id].reflection.type_id()) {
                None
            } else {
                Some(H::new(id))
            }
        } else {
            None
        }
    }

    pub(crate) fn contains(&self, resource: impl ToUID) -> bool {
        let resource = resource.to_uid();
        self.find::<ResourceType>(resource).is_some()
    }
}
