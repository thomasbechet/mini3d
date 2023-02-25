use std::{collections::HashMap};

use anyhow::{Context, Result};
use serde::{Deserializer, Serializer, Serialize, de::{Visitor, DeserializeSeed}};

use crate::{uid::UID, registry::component::{Component, ComponentRegistry, AnyComponentDefinitionReflection}};

use super::{entity::Entity, container::{AnyComponentContainer, ComponentContainer}, view::{ComponentView, ComponentViewMut}, query::Query};

pub struct World {
    pub(crate) name: String,
    containers: HashMap<UID, Box<dyn AnyComponentContainer>>,
    free_entities: Vec<Entity>,
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
                    let definition = self.registry.get(*uid).with_context(|| "Component definition not found").map_err(Error::custom)?;
                    map.serialize_entry(uid, &definition.reflection.serialize_container(container.as_ref()))?;
                }
                map.end()
            }
        }
        use serde::ser::SerializeTuple;
        let tuple = serializer.serialize_tuple(3)?;
        tuple.serialize_element(&self.name)?;
        tuple.serialize_element(&ContainersSerializer { containers: &self.containers, registry })?;
        tuple.serialize_element(&self.free_entities)?;
        tuple.end()
    }

    pub(crate) fn deserialize<'a, D: Deserializer<'a>>(registry: &ComponentRegistry, deserializer: D) -> Result<World, D::Error> {
        struct WorldVisitor<'a> {
            registry: &'a ComponentRegistry,
        }
        impl<'a> Visitor<'a> for WorldVisitor<'a> {
            type Value = World;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a tuple of (containers, free_entities)")
            }
            fn visit_seq<A: serde::de::SeqAccess<'a>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                use serde::de::Error;
                struct ContainersDeserializeSeed<'a> {
                    registry: &'a ComponentRegistry,
                }
                impl<'a> DeserializeSeed<'a> for ContainersDeserializeSeed<'a> {
                    type Value = HashMap<UID, Box<dyn AnyComponentContainer>>;
                    fn deserialize<D: Deserializer<'a>>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error> {
                        struct ContainersVisitor<'a> {
                            registry: &'a ComponentRegistry,
                        }
                        impl<'a> Visitor<'a> for ContainersVisitor<'a> {
                            type Value = HashMap<UID, Box<dyn AnyComponentContainer>>;
                            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                                formatter.write_str("a map of (uid, container)")
                            }
                            fn visit_map<A: serde::de::MapAccess<'a>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                                struct ContainerDeserializeSeed<'a> {
                                    reflection: &'a dyn AnyComponentDefinitionReflection,
                                }
                                impl<'a> DeserializeSeed<'a> for ContainerDeserializeSeed<'a> {
                                    type Value = Box<dyn AnyComponentContainer>;
                                    fn deserialize<D: Deserializer<'a>>(self, deserializer: D) -> std::result::Result<Self::Value, D::Error> {
                                        self.reflection.deserialize_container(&mut <dyn erased_serde::Deserializer>::erase(deserializer)).map_err(Error::custom)
                                    }
                                }
                                let mut containers = HashMap::new();
                                while let Some(uid) = map.next_key()? {
                                    if containers.contains_key(&uid) { return Err(A::Error::duplicate_field("uid")); }
                                    let reflection = &self.registry.get(uid).with_context(|| "Component definition not found").map_err(Error::custom)?.reflection;
                                    containers.insert(uid, map.next_value_seed(ContainerDeserializeSeed { reflection: reflection.as_ref() })?);
                                }
                                Ok(containers)
                            }
                        }
                        deserializer.deserialize_map(ContainersVisitor { registry: self.registry })
                    }
                }

                let name: String = seq.next_element()?.with_context(|| "Missing name").map_err(Error::custom)?;
                let containers = seq.next_element_seed(ContainersDeserializeSeed { registry: self.registry })?.with_context(|| "Missing containers").map_err(Error::custom)?;
                let free_entities = seq.next_element()?.with_context(|| "Missing free_entities").map_err(Error::custom)?;
                Ok(World { name, containers, free_entities })
            }
        }
        deserializer.deserialize_tuple(3, WorldVisitor { registry })
    }

    pub(crate) fn new(name: String) -> World {
        World {
            name,
            containers: HashMap::new(),
            free_entities: Vec::new(),
        }
    }

    pub(crate) fn create(&mut self) -> Entity {
        if let Some(entity) = self.free_entities.pop() {
            return entity;
        }
        Entity::null()
    }

    pub(crate) fn destroy(&mut self, entity: Entity) -> Result<()> {
        for container in self.containers.values_mut() {
            container.remove(entity);
        }
        self.free_entities.push(Entity::new(entity.index(), entity.version() + 1));
        Ok(())
    }

    pub(crate) fn add<C: Component>(&mut self, registry: &ComponentRegistry, entity: Entity, component: UID, data: C) -> Result<()> {
        if !self.containers.contains_key(&component) {
            let container = registry
                .get(component).with_context(|| "Component not registered")?
                .reflection.create_container();
            self.containers.insert(component, container);
        }
        let container = self.containers.get_mut(&component).unwrap();
        container.as_any_mut()
            .downcast_mut::<ComponentContainer<C>>().with_context(|| "Component type mismatch")?
            .add(entity, data);
        Ok(())
    }
    
    pub(crate) fn remove(&mut self, registry: ComponentRegistry, entity: Entity, component: UID) -> Result<()> {
        let container = self.containers.get_mut(&component).with_context(|| "Component container not found")?;
        container.remove(entity);
        Ok(())
    }

    pub(crate) fn view<'a, C: Component>(&'a self, component: UID) -> Result<ComponentView<'a, C>> {
        let container = self.containers.get(&component).with_context(|| "Component container not found")?;
        let container = container.as_any()
            .downcast_ref::<ComponentContainer<C>>().with_context(|| "Component type mismatch")?;
        Ok(ComponentView::new(container))
    }

    pub(crate) fn view_mut<'a, C: Component>(&'a self, component: UID) -> Result<ComponentViewMut<'a, C>> {
        let container = self.containers.get(&component).with_context(|| "Component container not found")?;
        let container = container.as_any()
            .downcast_ref::<ComponentContainer<C>>().with_context(|| "Component type mismatch")?;
        Ok(ComponentViewMut::new(container))
    }

    pub(crate) fn query<'a>(&'a self, components: &[UID]) -> Query<'a> {
        let mut containers = Vec::new();
        for component in components {
            containers.push(self.containers.get(component).unwrap().as_ref());
        }
        containers.sort_by(|a, b| a.len().cmp(&b.len()));
        Query::new(containers)
    }
}