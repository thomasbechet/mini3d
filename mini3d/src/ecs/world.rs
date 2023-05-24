use std::{collections::{HashMap, hash_map}};

use crate::{uid::UID, registry::{component::{ComponentRegistry, Component}, error::RegistryError}, feature::{asset::prefab::Prefab}, serialize::{Encoder, EncoderError, Decoder, DecoderError}};
use crate::serialize::Serialize;

use super::{entity::Entity, container::{AnyComponentContainer, ComponentContainer}, view::{ComponentViewRef, ComponentViewMut}, query::Query, reference::{ComponentRef, ComponentMut}, singleton::{AnyComponentSingleton, ComponentSingleton, SingletonRef, SingletonMut}, error::WorldError};

pub(crate) struct World {
    pub(crate) name: String,
    containers: HashMap<UID, Box<dyn AnyComponentContainer>>,
    singletons: HashMap<UID, Box<dyn AnyComponentSingleton>>,
    free_entities: Vec<Entity>,
    next_entity: Entity,
}

impl World {

    pub(crate) fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        // Name
        self.name.serialize(encoder)?;
        // Containers
        encoder.write_u32(self.containers.len() as u32)?;
        for (uid, container) in self.containers.iter() {
            uid.serialize(encoder)?;
            container.serialize(encoder)?;
        }
        // Containers
        encoder.write_u32(self.singletons.len() as u32)?;
        for (uid, singleton) in self.singletons.iter() {
            uid.serialize(encoder)?;
            singleton.serialize(encoder)?;
        }
        // Free entities
        self.free_entities.serialize(encoder)?;
        // Next entity
        self.next_entity.serialize(encoder)?;
        Ok(())
    }

    pub(crate) fn deserialize(registry: &ComponentRegistry, decoder: &mut impl Decoder) -> Result<Self, DecoderError> {
        // Name
        let name = String::deserialize(decoder, &Default::default())?;
        // Containers
        let container_count = decoder.read_u32()?;
        let mut containers = HashMap::with_capacity(container_count as usize);
        for _ in 0..container_count {
            let uid = UID::deserialize(decoder, &Default::default())?;
            let entry = registry.definitions.get(&uid).ok_or(DecoderError::Unsupported)?;
            let mut container = entry.reflection.create_container();
            container.deserialize(decoder)?;
            containers.insert(uid, container);
        }
        // Singletons
        let singleton_count = decoder.read_u32()?;
        let mut singletons = HashMap::with_capacity(singleton_count as usize);
        for _ in 0..singleton_count {
            let uid = UID::deserialize(decoder, &Default::default())?;
            let entry = registry.definitions.get(&uid).ok_or(DecoderError::Unsupported)?;
            let mut singleton = entry.reflection.create_singleton();
            singleton.deserialize(decoder)?;
            singletons.insert(uid, singleton);
        }
        // Free entities
        let free_entities = Vec::<Entity>::deserialize(decoder, &Default::default())?;
        // Next entity
        let next_entity = Entity::deserialize(decoder, &Default::default())?;

        Ok(Self { name, containers, singletons, free_entities, next_entity })
    }

    pub(crate) fn new(name: &str) -> World {
        World {
            name: name.to_string(),
            containers: HashMap::new(),
            singletons: HashMap::new(),
            free_entities: Vec::new(),
            next_entity: Entity::new(1, 0),
        }
    }

    pub(crate) fn create(&mut self) -> Entity {
        if let Some(entity) = self.free_entities.pop() {
            return entity;
        }
        let entity = self.next_entity;
        self.next_entity = Entity::new(entity.key() + 1, 0);
        entity
    }

    pub(crate) fn destroy(&mut self, entity: Entity) -> Result<(), WorldError> {
        for container in self.containers.values_mut() {
            container.remove(entity);
        }
        self.free_entities.push(Entity::new(entity.key(), entity.version() + 1));
        Ok(())
    }

    pub(crate) fn add<C: Component>(&mut self, registry: &ComponentRegistry, entity: Entity, component: UID, data: C) -> Result<(), WorldError> {
        if let hash_map::Entry::Vacant(e) = self.containers.entry(component) {
            let container = registry.definitions
                .get(&component).ok_or(WorldError::Registry(RegistryError::ComponentDefinitionNotFound { uid: component }))?
                .reflection.create_container();
            e.insert(container);
        }
        let container = self.containers.get_mut(&component).unwrap();
        container.as_any_mut()
            .downcast_mut::<ComponentContainer<C>>().ok_or(WorldError::ComponentTypeMismatch { uid: component })?
            .add(entity, data).map_err(|_| WorldError::ContainerBorrowMut)?;
        Ok(())
    }
    
    pub(crate) fn remove(&mut self, entity: Entity, component: UID) -> Result<(), WorldError> {
        let container = self.containers.get_mut(&component).ok_or_else(|| WorldError::ComponentContainerNotFound { uid: component } )?;
        container.remove(entity);
        Ok(())
    }

    pub(crate) fn get<C: Component>(&self, entity: Entity, component: UID) -> Result<Option<ComponentRef<'_, C>>, WorldError> {
        if let Some(container) = self.containers.get(&component) {
            Ok(container.as_any()
                .downcast_ref::<ComponentContainer<C>>().ok_or(WorldError::ComponentTypeMismatch { uid: component })?
                .get(entity))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn get_mut<C: Component>(&self, entity: Entity, component: UID) -> Result<Option<ComponentMut<'_, C>>, WorldError> {
        if let Some(container) = self.containers.get(&component) {
            Ok(container.as_any()
                .downcast_ref::<ComponentContainer<C>>().ok_or(WorldError::ComponentTypeMismatch { uid: component })?
                .get_mut(entity))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn view<C: Component>(&self, component: UID) -> Result<ComponentViewRef<'_, C>, WorldError> {
        if let Some(container) = self.containers.get(&component) {
            let container = container.as_any()
                .downcast_ref::<ComponentContainer<C>>().ok_or(WorldError::ComponentTypeMismatch { uid: component })?;
            Ok(ComponentViewRef::new(container))
        } else {
            Ok(ComponentViewRef::none())
        } 
    }

    pub(crate) fn view_mut<C: Component>(&self, component: UID) -> Result<ComponentViewMut<'_, C>, WorldError> {
        if let Some(container) = self.containers.get(&component) {
            let container = container.as_any()
                .downcast_ref::<ComponentContainer<C>>().ok_or(WorldError::ComponentTypeMismatch { uid: component })?;
            Ok(ComponentViewMut::new(container))
        } else {
            Ok(ComponentViewMut::none())
        }
    }

    pub(crate) fn query<'a>(&'a self, components: &[UID]) -> Query<'a> {
        let mut containers = Vec::new();
        for component in components {
            if let Some(container) = self.containers.get(component) {
                containers.push(container.as_ref());
            } else {
                // One of the components is missing, so the query will return no results
                return Query::none();
            }
        }
        containers.sort_by_key(|a| a.len());
        Query::new(containers)
    }

    pub(crate) fn add_singleton<C: Component>(&mut self, component: UID, data: C) -> Result<(), WorldError> {
        if self.singletons.contains_key(&component) {
            return Err(WorldError::DuplicatedSingleton { uid: component });
        }
        self.singletons.insert(component, Box::new(ComponentSingleton::new(data)));
        Ok(())
    }

    pub(crate) fn remove_singleton(&mut self, component: UID) -> Result<(), WorldError> {
        self.singletons.remove(&component).ok_or(WorldError::SingletonNotFound { uid: component })?;
        Ok(())
    }

    pub(crate) fn get_singleton<C: Component>(&self, component: UID) -> Result<Option<SingletonRef<'_, C>>, WorldError> {
        if let Some(singleton) = self.singletons.get(&component) {
            Ok(Some(SingletonRef {
                component: singleton.as_any()
                    .downcast_ref::<ComponentSingleton<C>>().ok_or(WorldError::SingletonTypeMismatch { uid: component })?
                    .component.borrow()
            }))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn get_singleton_mut<C: Component>(&self, component: UID) -> Result<Option<SingletonMut<'_, C>>, WorldError> {
        if let Some(singleton) = self.singletons.get(&component) {
            Ok(Some(SingletonMut {
                component: singleton.as_any()
                    .downcast_ref::<ComponentSingleton<C>>().ok_or(WorldError::SingletonTypeMismatch { uid: component })?
                    .component.borrow_mut()
            }))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn export(&self, registry: &ComponentRegistry, entity: Entity, export_hierarchy: bool) -> Result<Prefab, WorldError> {
        
        // let hierarchies = self.view::<Hierarchy>(Hierarchy::UID)
        //     .with_context(|| "Hierarchy component not registered")?;

        // let mut prefab = Prefab::empty();

        // prefab.root = entity;
        // for (component, container) in self.containers.iter() {
        //     if let Some(data) = container. get(entity) {
        //         let component = registry.get(*component).with_context(|| "Component not registered")?;
        //         let data = component.reflection.export(data)?;
        //         prefab.components.insert(*component, data);
        //     }
        // }

        Ok(Prefab::empty())
    }
}