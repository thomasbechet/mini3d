use std::{marker::PhantomData, any::{type_name, TypeId}};

use anyhow::{Result, Context};
use hecs::{Archetype, ColumnBatchBuilder, ColumnBatchType, ArchetypeColumn};
use serde::{Serialize, Deserialize, de::{SeqAccess, Visitor}, Deserializer};

use super::{entity::{EntityResolver, Entity}, world::World};

pub trait Component: hecs::Component + Serialize + for<'de> Deserialize<'de> {
    fn resolve_entities(&mut self, resolver: &EntityResolver) -> Result<()> { Ok(()) }
}

pub(crate) trait AnyComponent {
    fn serialize_column<'a>(&'a self, archetype: &'a Archetype) -> Result<Box<dyn AnyArchetypeColumnIterator<'a> + 'a>>;
    fn deserialize_column(&self, batch: &mut ColumnBatchBuilder, entity_count: u32, deserializer: &mut dyn erased_serde::Deserializer) -> Result<()>;
    fn add_to_batch(&self, batch: &mut ColumnBatchType);
    fn instantiate(&self, entity: Entity, value: &serde_json::Value, world: &mut World) -> Result<()>;
}

pub(crate) struct TypeComponent<C> { marker: PhantomData<C> }

trait AnyComponentDeserializeSeed<'a> {}

pub(crate) trait AnyArchetypeColumnIterator<'a> {
    fn next<'b>(&'b mut self) -> Option<&'b (dyn erased_serde::Serialize + 'b)>;
}

impl<C: Component> AnyComponent for TypeComponent<C> {
    
    fn serialize_column<'a>(&'a self, archetype: &'a Archetype) -> Result<Box<dyn AnyArchetypeColumnIterator<'a> + 'a>> {
        struct ArchetypeColumnIterator<'a, C: hecs::Component + Serialize> {
            reference: ArchetypeColumn<'a, C>,
            next: usize,
        }
        impl<'a, C: hecs::Component + Serialize> AnyArchetypeColumnIterator<'a> for ArchetypeColumnIterator<'a, C> {
            fn next<'b>(&'b mut self) -> Option<&'b (dyn erased_serde::Serialize + 'b)> {
                let current = self.next;
                self.next += 1;
                match self.reference.get(current) {
                    Some(r) => Some(r),
                    None => None,
                }
            }
        }
        let reference = archetype.get::<&C>().with_context(|| "Archetype doesn't contain component")?;
        Ok(Box::new(ArchetypeColumnIterator { reference, next: 0 }))
    }
    
    fn deserialize_column(&self, batch: &mut ColumnBatchBuilder, entity_count: u32, deserializer: &mut dyn erased_serde::Deserializer) -> Result<()> {
        struct ColumnVisitor<'a, C: hecs::Component + for<'de> Deserialize<'de>> {
            batch: &'a mut ColumnBatchBuilder,
            entity_count: u32,
            marker: PhantomData<C>,
        }
        impl<'de, 'a, C: hecs::Component + for<'b> Deserialize<'b>> Visitor<'de> for ColumnVisitor<'a, C> {
            type Value = ();
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a set of {} {} values", self.entity_count, type_name::<C>())
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where A: SeqAccess<'de>,
            {
                use serde::de::Error;
                let mut out = self.batch.writer::<C>().expect("Unexpected component type");
                while let Some(component) = seq.next_element()? {
                    if out.push(component).is_err() {
                        return Err(A::Error::custom("Extra component"));
                    }
                }
                if out.fill() < self.entity_count {
                    return Err(A::Error::custom("Invalid tuple length"));
                }
                Ok(())
            }
        }
        deserializer.deserialize_tuple(entity_count as usize, ColumnVisitor::<C> { batch, entity_count, marker: PhantomData })?;
        Ok(())
    }

    fn add_to_batch(&self, batch: &mut ColumnBatchType) {
        batch.add::<C>();
    }

    fn instantiate(&self, entity: Entity, value: &serde_json::Value, world: &mut World) -> Result<()> {
        let component: C = serde_json::from_value(value.clone())?;
        world.add_component(entity, component)
    }
}

trait AnyArchetypeColumnSerialize {
    fn serialize_column(&self) -> Vec<&dyn erased_serde::Serialize>;
}

pub(crate) struct ComponentEntry {
    pub(crate) name: String,
    pub(crate) type_id: TypeId,
    pub(crate) component: Box<dyn AnyComponent>,
}