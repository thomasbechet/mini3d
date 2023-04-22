use std::{collections::{HashMap, hash_map}};

use serde::{Deserializer, Serializer, Serialize, de::{Visitor, DeserializeSeed}};
use serde_json::json;

use crate::{uid::UID, registry::{component::{ComponentRegistry, AnyComponentReflection, Component}, error::RegistryError}, feature::{asset::prefab::Prefab}};

use super::{entity::Entity, container::{AnyComponentContainer, ComponentContainer}, view::{ComponentViewRef, ComponentViewMut}, query::Query, reference::{ComponentRef, ComponentMut}, singleton::{AnySingleton, Singleton, SingletonRef, SingletonMut}, error::WorldError};

pub(crate) struct World {
    pub(crate) name: String,
    containers: HashMap<UID, Box<dyn AnyComponentContainer>>,
    singletons: HashMap<UID, Box<dyn AnySingleton>>,
    free_entities: Vec<Entity>,
    next_entity: Entity,
}

impl World {

    pub(crate) fn serialize<S: Serializer>(&self, serializer: S, registry: &ComponentRegistry) -> Result<S::Ok, S::Error> {
        struct ContainersSerializer<'a> {
            containers: &'a HashMap<UID, Box<dyn AnyComponentContainer>>,
            registry: &'a ComponentRegistry,
        }
        impl<'a> Serialize for ContainersSerializer<'a> {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(self.containers.len()))?;
                for (uid, container) in self.containers.iter() {
                    use serde::ser::Error;
                    let entry = self.registry.definitions.get(uid).ok_or_else(|| Error::custom("Component definition not found"))?;
                    map.serialize_entry(uid, &entry.reflection.serialize_container(container.as_ref()))?;
                }
                map.end()
            }
        }
        struct SingletonsSerializer<'a> {
            singletons: &'a HashMap<UID, Box<dyn AnySingleton>>,
            registry: &'a ComponentRegistry,
        }
        impl<'a> Serialize for SingletonsSerializer<'a> {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(self.singletons.len()))?;
                for (uid, singleton) in self.singletons.iter() {
                    use serde::ser::Error;
                    let entry = self.registry.definitions.get(uid).ok_or_else(|| Error::custom("Component definition not found"))?;
                    map.serialize_entry(uid, &entry.reflection.serialize_singleton(singleton.as_ref()))?;
                }
                map.end()
            }
        }
        use serde::ser::SerializeTuple;
        let mut tuple = serializer.serialize_tuple(5)?;
        tuple.serialize_element(&self.name)?;
        tuple.serialize_element(&ContainersSerializer { containers: &self.containers, registry })?;
        tuple.serialize_element(&SingletonsSerializer { singletons: &self.singletons, registry })?;
        tuple.serialize_element(&self.free_entities)?;
        tuple.serialize_element(&self.next_entity)?;
        tuple.end()
    }

    pub(crate) fn deserialize<'a, 'de, D: Deserializer<'de>>(registry: &'a ComponentRegistry, deserializer: D) -> Result<World, D::Error> {
        struct WorldVisitor<'a> {
            registry: &'a ComponentRegistry,
        }
        impl<'a, 'de> Visitor<'de> for WorldVisitor<'a> {
            type Value = World;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a tuple of (containers, free_entities)")
            }
            fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                use serde::de::Error;

                struct ContainersDeserializeSeed<'a> {
                    registry: &'a ComponentRegistry,
                }
                impl<'a, 'de> DeserializeSeed<'de> for ContainersDeserializeSeed<'a> {
                    type Value = HashMap<UID, Box<dyn AnyComponentContainer>>;
                    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error> {
                        struct ContainersVisitor<'a> {
                            registry: &'a ComponentRegistry,
                        }
                        impl<'a, 'de> Visitor<'de> for ContainersVisitor<'a> {
                            type Value = HashMap<UID, Box<dyn AnyComponentContainer>>;
                            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                                formatter.write_str("a map of (uid, container)")
                            }
                            fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                                struct ContainerDeserializeSeed<'a> {
                                    reflection: &'a dyn AnyComponentReflection,
                                }
                                impl<'a, 'de> DeserializeSeed<'de> for ContainerDeserializeSeed<'a> {
                                    type Value = Box<dyn AnyComponentContainer>;
                                    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error> {
                                        self.reflection.deserialize_container(&mut <dyn erased_serde::Deserializer>::erase(deserializer)).map_err(Error::custom)
                                    }
                                }
                                let mut containers = HashMap::new();
                                while let Some(uid) = map.next_key::<UID>()? {
                                    if containers.contains_key(&uid) { return Err(A::Error::duplicate_field("uid")); }
                                    let reflection = &self.registry.definitions.get(&uid).ok_or_else(|| Error::custom("Component definition not found"))?.reflection;
                                    containers.insert(uid, map.next_value_seed(ContainerDeserializeSeed { reflection: reflection.as_ref() })?);
                                }
                                Ok(containers)
                            }
                        }
                        deserializer.deserialize_map(ContainersVisitor { registry: self.registry })
                    }
                }

                struct SingletonsDeserializeSeed<'a> {
                    registry: &'a ComponentRegistry,
                }
                impl<'a, 'de> DeserializeSeed<'de> for SingletonsDeserializeSeed<'a> {
                    type Value = HashMap<UID, Box<dyn AnySingleton>>;
                    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error> {
                        struct SingletonsVisitor<'a> {
                            registry: &'a ComponentRegistry,
                        }
                        impl<'a, 'de> Visitor<'de> for SingletonsVisitor<'a> {
                            type Value = HashMap<UID, Box<dyn AnySingleton>>;
                            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                                formatter.write_str("a map of (uid, singleton)")
                            }
                            fn visit_map<A: serde::de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                                struct SingletonDeserializeSeed<'a> {
                                    reflection: &'a dyn AnyComponentReflection,
                                }
                                impl<'a, 'de> DeserializeSeed<'de> for SingletonDeserializeSeed<'a> {
                                    type Value = Box<dyn AnySingleton>;
                                    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error> {
                                        self.reflection.deserialize_singleton(&mut <dyn erased_serde::Deserializer>::erase(deserializer)).map_err(Error::custom)
                                    }
                                }
                                let mut singletons = HashMap::new();
                                while let Some(uid) = map.next_key::<UID>()? {
                                    if singletons.contains_key(&uid) { return Err(A::Error::duplicate_field("uid")); }
                                    let reflection = &self.registry.definitions.get(&uid).ok_or_else(|| Error::custom("Component definition not found"))?.reflection;
                                    singletons.insert(uid, map.next_value_seed(SingletonDeserializeSeed { reflection: reflection.as_ref() })?);
                                }
                                Ok(singletons)
                            }
                        }
                        deserializer.deserialize_map(SingletonsVisitor { registry: self.registry })
                    }
                }

                let name: String = seq.next_element()?.ok_or_else(|| Error::missing_field("name"))?;
                let containers = seq.next_element_seed(ContainersDeserializeSeed { registry: self.registry })?.ok_or_else(|| Error::missing_field("containers"))?;
                let singletons = seq.next_element_seed(SingletonsDeserializeSeed { registry: self.registry })?.ok_or_else(|| Error::missing_field("singletons"))?;
                let free_entities = seq.next_element()?.ok_or_else(|| Error::missing_field("free_entities"))?;
                let next_entity = seq.next_element()?.ok_or_else(|| Error::missing_field("next_entity"))?;
                Ok(World { name, containers, singletons, free_entities, next_entity })
            }
        }
        deserializer.deserialize_tuple(5, WorldVisitor { registry })
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
                .get(&component).ok_or_else(|| WorldError::Registry(RegistryError::ComponentDefinitionNotFound { uid: component }))?
                .reflection.create_container();
            e.insert(container);
        }
        let container = self.containers.get_mut(&component).unwrap();
        container.as_any_mut()
            .downcast_mut::<ComponentContainer<C>>().ok_or_else(|| WorldError::ComponentTypeMismatch { uid: component })?
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
                .downcast_ref::<ComponentContainer<C>>().ok_or_else(|| WorldError::ComponentTypeMismatch { uid: component })?
                .get(entity))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn get_mut<C: Component>(&self, entity: Entity, component: UID) -> Result<Option<ComponentMut<'_, C>>, WorldError> {
        if let Some(container) = self.containers.get(&component) {
            Ok(container.as_any()
                .downcast_ref::<ComponentContainer<C>>().ok_or_else(|| WorldError::ComponentTypeMismatch { uid: component })?
                .get_mut(entity))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn view<C: Component>(&self, component: UID) -> Result<ComponentViewRef<'_, C>, WorldError> {
        if let Some(container) = self.containers.get(&component) {
            let container = container.as_any()
                .downcast_ref::<ComponentContainer<C>>().ok_or_else(|| WorldError::ComponentTypeMismatch { uid: component })?;
            Ok(ComponentViewRef::new(container))
        } else {
            Ok(ComponentViewRef::none())
        } 
    }

    pub(crate) fn view_mut<C: Component>(&self, component: UID) -> Result<ComponentViewMut<'_, C>, WorldError> {
        if let Some(container) = self.containers.get(&component) {
            let container = container.as_any()
                .downcast_ref::<ComponentContainer<C>>().ok_or_else(|| WorldError::ComponentTypeMismatch { uid: component })?;
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
        self.singletons.insert(component, Box::new(Singleton::new(data)));
        Ok(())
    }

    pub(crate) fn remove_singleton(&mut self, component: UID) -> Result<(), WorldError> {
        self.singletons.remove(&component).ok_or_else(|| WorldError::SingletonNotFound { uid: component })?;
        Ok(())
    }

    pub(crate) fn get_singleton<C: Component>(&self, component: UID) -> Result<Option<SingletonRef<'_, C>>, WorldError> {
        if let Some(singleton) = self.singletons.get(&component) {
            Ok(Some(SingletonRef {
                component: singleton.as_any()
                    .downcast_ref::<Singleton<C>>().ok_or_else(|| WorldError::SingletonTypeMismatch { uid: component })?
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
                    .downcast_ref::<Singleton<C>>().ok_or_else(|| WorldError::SingletonTypeMismatch { uid: component })?
                    .component.borrow_mut()
            }))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn instantiate(&mut self, registry: &ComponentRegistry, prefab: &Prefab, patch: Option<serde_json::Value>) -> Result<Entity, WorldError> {
        
        let patch = json!({
            "root": {
                "transform": {
                    "position": {
                        "x": 0.0,
                        "y": 0.0,
                        "z": 0.0
                    },
                }
            },
        });

        
        let entity = self.create();
        
        Ok(entity)
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