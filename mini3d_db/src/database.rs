use core::fmt::{self, Formatter};

use alloc::vec::Vec;
use mini3d_derive::Serialize;
use mini3d_utils::{slot_map_key, slotmap::SlotMap, string::AsciiArray};

use crate::{
    entity::Entity,
    error::ComponentError,
    field::{ComponentField, Field, FieldEntry, FieldType},
    query::{EntityQuery, Query},
    registry::Registry,
};

slot_map_key!(ComponentHandle);
slot_map_key!(FieldHandle);

pub trait GetComponentHandle {
    fn handle(&self) -> ComponentHandle;
}

impl GetComponentHandle for ComponentHandle {
    fn handle(&self) -> ComponentHandle {
        *self
    }
}

#[derive(Default, Serialize, PartialEq, Eq)]
pub enum ComponentState {
    #[default]
    Created,
    Active,
    Deleted,
}

pub(crate) struct ComponentEntry {
    pub(crate) name: AsciiArray<32>,
    pub(crate) state: ComponentState,
    pub(crate) fields: Vec<FieldHandle>,
}

#[derive(Default)]
pub struct Database {
    pub(crate) registry: Registry,
    pub(crate) fields: SlotMap<FieldHandle, FieldEntry>,
    pub(crate) components: SlotMap<ComponentHandle, ComponentEntry>,
}

impl Database {
    pub fn rebuild(&mut self) {
        for id in self.components_from_state(ComponentState::Deleted) {
            for fid in self.components.get(id).unwrap().fields.iter() {
                self.fields.remove(*fid);
                // TODO trigger events ?
            }
            self.components.remove(id);
            self.registry.remove_bitset(id);
        }
        for id in self.components_from_state(ComponentState::Created) {
            self.components[id].state = ComponentState::Active;
        }
    }

    pub fn components_from_state(&self, state: ComponentState) -> Vec<ComponentHandle> {
        self.components
            .iter()
            .filter_map(|(id, system)| {
                if system.state == state {
                    Some(id)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn register_tag(&mut self, name: &str) -> Result<ComponentHandle, ComponentError> {
        self.register(name, &[])
    }

    pub fn register(
        &mut self,
        name: &str,
        fields: &[ComponentField],
    ) -> Result<ComponentHandle, ComponentError> {
        if self.find_component(name).is_some() {
            return Err(ComponentError::DuplicatedEntry);
        }
        let id = self.components.add(ComponentEntry {
            name: name.into(),
            state: ComponentState::Created,
            fields: Vec::with_capacity(fields.len()),
        });
        let component = self.components.get_mut(id).unwrap();
        for field in fields {
            let fid = self.fields.add(FieldEntry {
                name: field.name.into(),
                data: field.create_storage(),
                ty: field.ty,
            });
            component.fields.push(fid);
        }
        self.registry.add_bitset(id);
        Ok(id)
    }

    pub fn unregister_component(&mut self, c: ComponentHandle) {
        self.components[c].state = ComponentState::Deleted;
    }

    pub fn create(&mut self) -> Entity {
        self.registry.create()
    }

    pub fn delete(&mut self, e: Entity) {
        self.registry.destroy(e);
    }

    pub fn find_next_component(&self, e: Entity, c: Option<ComponentHandle>) -> Option<ComponentHandle> {
        self.registry.find_next_component(e, c)
    }

    pub fn add_default(&mut self, e: Entity, c: ComponentHandle) {
        for fid in self.components.get(c).unwrap().fields.iter() {
            self.fields[*fid].data.add_default(e);
        }
        self.registry.set(e, c);
    }

    pub fn remove(&mut self, e: Entity, c: ComponentHandle) {
        self.registry.unset(e, c);
    }

    pub fn has(&self, e: Entity, c: ComponentHandle) -> bool {
        self.registry.has(e, c)
    }

    pub fn read<T: FieldType>(&self, e: Entity, f: Field<T>) -> Option<T> {
        T::read(&self.fields[f.0], e)
    }

    pub fn write<T: FieldType>(&mut self, e: Entity, f: Field<T>, v: T) {
        T::write(&mut self.fields[f.0], e, v)
    }

    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.registry.entities()
    }

    pub fn find_component(&self, name: &str) -> Option<ComponentHandle> {
        self.components.iter().find_map(|(id, entry)| {
            if entry.name.as_str() == name {
                Some(id)
            } else {
                None
            }
        })
    }

    pub fn find_field<T: FieldType>(&self, c: ComponentHandle, name: &str) -> Option<Field<T>> {
        for field in self.components[c].fields.iter() {
            if self.fields[*field].name.as_str() == name {
                return Some(Field(*field, Default::default()));
            }
        }
        None
    }

    pub fn query_entities<'a>(&self, query: &'a Query) -> EntityQuery<'a> {
        EntityQuery::new(query, self)
    }

    pub fn display(&self, f: &mut Formatter, e: Entity) -> fmt::Result {
        let mut next = None;
        writeln!(f)?;
        write!(f, "{}:", e)?;
        while let Some(component) = self.registry.find_next_component(e, next) {
            next = Some(component);
            let component = &self.components[component];
            writeln!(f)?;
            write!(f, "- {}", component.name)?;
            for field in component.fields.iter() {
                let field = &self.fields[*field];
                writeln!(f)?;
                write!(f, "  - ")?;
                field.display(f, e)?;
            }
        }
        Ok(())
    }
}
