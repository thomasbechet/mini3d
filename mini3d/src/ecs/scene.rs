use crate::feature::component::common::prefab::Prefab;
use crate::registry::component::ComponentId;
use crate::serialize::Serialize;
use crate::utils::slotmap::SparseSecondaryMap;
use crate::utils::uid::UID;
use crate::{
    registry::component::{Component, ComponentRegistry},
    serialize::{Decoder, DecoderError, Encoder, EncoderError},
};

use super::view::{SceneComponentViewMut, SceneComponentViewRef};
use super::{
    container::{AnySceneContainer, StaticSceneContainer},
    entity::Entity,
    error::SceneError,
    query::Query,
    singleton::{AnySceneSingleton, StaticSceneSingleton, StaticSingletonMut, StaticSingletonRef},
    view::{StaticSceneComponentViewMut, StaticSceneComponentViewRef},
};

pub(crate) struct Scene {
    pub(crate) name: String,
    containers: SparseSecondaryMap<ComponentId, Box<dyn AnySceneContainer>>,
    singletons: SparseSecondaryMap<ComponentId, Box<dyn AnySceneSingleton>>,
    free_entities: Vec<Entity>,
    next_entity: Entity,
}

impl Scene {
    pub(crate) fn serialize(
        &self,
        registry: &ComponentRegistry,
        encoder: &mut impl Encoder,
    ) -> Result<(), EncoderError> {
        // Name
        self.name.serialize(encoder)?;
        // Containers
        encoder.write_u32(self.containers.len() as u32)?;
        for (id, container) in self.containers.iter() {
            UID::new(&registry.get(id).unwrap().name).serialize(encoder)?;
            container.serialize(encoder)?;
        }
        // Containers
        encoder.write_u32(self.singletons.len() as u32)?;
        for (id, singleton) in self.singletons.iter() {
            UID::new(&registry.get(id).unwrap().name).serialize(encoder)?;
            singleton.serialize(encoder)?;
        }
        // Free entities
        self.free_entities.serialize(encoder)?;
        // Next entity
        self.next_entity.serialize(encoder)?;
        Ok(())
    }

    pub(crate) fn deserialize(
        registry: &ComponentRegistry,
        decoder: &mut impl Decoder,
    ) -> Result<Self, DecoderError> {
        // Name
        let name = String::deserialize(decoder, &Default::default())?;
        // Containers
        let container_count = decoder.read_u32()?;
        let mut containers = SparseSecondaryMap::with_capacity(container_count as usize);
        for _ in 0..container_count {
            let uid = UID::deserialize(decoder, &Default::default())?;
            let (id, definition) = registry.find(uid).ok_or(DecoderError::Unsupported)?;
            let mut container = definition.reflection.create_scene_container();
            container.deserialize(decoder)?;
            containers.insert(id, container);
        }
        // Singletons
        let singleton_count = decoder.read_u32()?;
        let mut singletons = SparseSecondaryMap::with_capacity(singleton_count as usize);
        for _ in 0..singleton_count {
            let uid = UID::deserialize(decoder, &Default::default())?;
            let (id, definition) = registry.find(uid).ok_or(DecoderError::Unsupported)?;
            let mut singleton = definition.reflection.create_scene_singleton();
            singleton.deserialize(decoder)?;
            singletons.insert(id, singleton);
        }
        // Free entities
        let free_entities = Vec::<Entity>::deserialize(decoder, &Default::default())?;
        // Next entity
        let next_entity = Entity::deserialize(decoder, &Default::default())?;

        Ok(Self {
            name,
            containers,
            singletons,
            free_entities,
            next_entity,
        })
    }

    pub(crate) fn new(name: &str) -> Scene {
        Scene {
            name: name.to_string(),
            containers: SparseSecondaryMap::default(),
            singletons: SparseSecondaryMap::default(),
            free_entities: Vec::new(),
            next_entity: Entity::new(1, 0),
        }
    }

    pub(crate) fn add_entity(&mut self) -> Entity {
        if let Some(entity) = self.free_entities.pop() {
            return entity;
        }
        let entity = self.next_entity;
        self.next_entity = Entity::new(entity.key() + 1, 0);
        entity
    }

    pub(crate) fn remove_entity(&mut self, entity: Entity) -> Result<(), SceneError> {
        for container in self.containers.values_mut() {
            container.remove(entity);
        }
        self.free_entities
            .push(Entity::new(entity.key(), entity.version() + 1));
        Ok(())
    }

    pub(crate) fn add_static_component<C: Component>(
        &mut self,
        registry: &ComponentRegistry,
        entity: Entity,
        component: ComponentId,
        data: C,
    ) -> Result<(), SceneError> {
        if !self.containers.contains(component) {
            let container = registry
                .get(component)
                .unwrap()
                .reflection
                .create_scene_container();
            self.containers.insert(component, container);
        }
        let container = self.containers.get_mut(component).unwrap();
        container
            .as_any_mut()
            .downcast_mut::<StaticSceneContainer<C>>()
            .ok_or(SceneError::ComponentTypeMismatch)?
            .add(entity, data)
            .map_err(|_| SceneError::ContainerBorrowMut)?;
        Ok(())
    }

    pub(crate) fn remove_component(
        &mut self,
        entity: Entity,
        component: ComponentId,
    ) -> Result<(), SceneError> {
        let container = self
            .containers
            .get_mut(component)
            .ok_or(SceneError::ComponentContainerNotFound)?;
        container.remove(entity);
        Ok(())
    }

    // pub(crate) fn static_view<C: Component>(
    //     &self,
    //     component: ComponentId,
    // ) -> Result<StaticSceneComponentViewRef<'_, C>, SceneError> {
    //     if let Some(container) = self.containers.get(component) {
    //         let container = container
    //             .as_any()
    //             .downcast_ref::<StaticSceneContainer<C>>()
    //             .ok_or(SceneError::ComponentTypeMismatch)?;
    //         Ok(StaticSceneComponentViewRef::new(container))
    //     } else {
    //         Ok(StaticSceneComponentViewRef::none())
    //     }
    // }

    // pub(crate) fn static_view_mut<C: Component>(
    //     &self,
    //     component: ComponentId,
    // ) -> Result<StaticSceneComponentViewMut<'_, C>, SceneError> {
    //     if let Some(container) = self.containers.get(component) {
    //         let container = container
    //             .as_any()
    //             .downcast_ref::<StaticSceneContainer<C>>()
    //             .ok_or(SceneError::ComponentTypeMismatch)?;
    //         Ok(StaticSceneComponentViewMut::new(container))
    //     } else {
    //         Ok(StaticSceneComponentViewMut::none())
    //     }
    // }

    pub(crate) fn view(
        &self,
        component: ComponentId,
    ) -> Result<SceneComponentViewRef<'_>, SceneError> {
        if let Some(container) = self.containers.get(component) {
            Ok(container.any_view())
        } else {
            Ok(SceneComponentViewRef::none())
        }
    }

    pub(crate) fn view_mut(
        &self,
        component: ComponentId,
    ) -> Result<SceneComponentViewMut<'_>, SceneError> {
        if let Some(container) = self.containers.get(component) {
            Ok(container.any_view_mut())
        } else {
            Ok(SceneComponentViewMut::none())
        }
    }

    pub(crate) fn query<'a>(&'a self, components: &[ComponentId]) -> Query<'a> {
        let mut containers = Vec::new();
        for component in components {
            if let Some(container) = self.containers.get(*component) {
                containers.push(container.as_ref());
            } else {
                // One of the components is missing, so the query will return no results
                return Query::none();
            }
        }
        containers.sort_by_key(|a| a.len());
        Query::new(containers)
    }

    pub(crate) fn add_static_singleton<C: Component>(
        &mut self,
        component: ComponentId,
        data: C,
    ) -> Result<(), SceneError> {
        if self.singletons.contains(component) {
            return Err(SceneError::DuplicatedSingleton);
        }
        self.singletons
            .insert(component, Box::new(StaticSceneSingleton::new(data)));
        Ok(())
    }

    pub(crate) fn remove_singleton(&mut self, component: ComponentId) -> Result<(), SceneError> {
        self.singletons
            .remove(component)
            .ok_or(SceneError::SingletonNotFound)?;
        Ok(())
    }

    pub(crate) fn get_static_singleton<C: Component>(
        &self,
        component: ComponentId,
    ) -> Result<Option<StaticSingletonRef<'_, C>>, SceneError> {
        if let Some(singleton) = self.singletons.get(component) {
            Ok(Some(StaticSingletonRef {
                component: singleton
                    .as_any()
                    .downcast_ref::<StaticSceneSingleton<C>>()
                    .ok_or(SceneError::SingletonTypeMismatch)?
                    .component
                    .borrow(),
            }))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn get_static_singleton_mut<C: Component>(
        &self,
        component: ComponentId,
    ) -> Result<Option<StaticSingletonMut<'_, C>>, SceneError> {
        if let Some(singleton) = self.singletons.get(component) {
            Ok(Some(StaticSingletonMut {
                component: singleton
                    .as_any()
                    .downcast_ref::<StaticSceneSingleton<C>>()
                    .ok_or(SceneError::SingletonTypeMismatch)?
                    .component
                    .borrow_mut(),
            }))
        } else {
            Ok(None)
        }
    }

    // TODO: transfer entities to another scene

    pub(crate) fn export(
        &self,
        registry: &ComponentRegistry,
        entity: Entity,
        export_hierarchy: bool,
    ) -> Result<Prefab, SceneError> {
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
