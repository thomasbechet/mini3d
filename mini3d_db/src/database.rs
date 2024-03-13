use core::fmt::{self, Formatter};

use alloc::vec::Vec;
use mini3d_utils::{slot_map_key, slotmap::SlotMap, string::AsciiArray};

use crate::{
    entity::Entity,
    error::ComponentError,
    field::{ComponentField, Field, FieldEntry, FieldType},
    query::{EntityQuery, Query},
    registry::Registry,
};

slot_map_key!(ComponentId);
slot_map_key!(FieldId);

pub(crate) struct ComponentEntry {
    pub(crate) name: AsciiArray<32>,
    pub(crate) fields: Vec<FieldId>,
}

#[derive(Default)]
pub struct Database {
    pub(crate) registry: Registry,
    pub(crate) fields: SlotMap<FieldId, FieldEntry>,
    pub(crate) components: SlotMap<ComponentId, ComponentEntry>,
}

impl Database {
    pub fn register_tag(&mut self, name: &str) -> Result<ComponentId, ComponentError> {
        if self.find_component(name).is_some() {
            return Err(ComponentError::DuplicatedEntry);
        }
        let id = self.components.add(ComponentEntry {
            name: name.into(),
            fields: Default::default(),
        });
        self.registry.add_bitset(id);
        Ok(id)
    }

    pub fn register(
        &mut self,
        name: &str,
        fields: &[ComponentField],
    ) -> Result<ComponentId, ComponentError> {
        if self.find_component(name).is_some() {
            return Err(ComponentError::DuplicatedEntry);
        }
        let id = self.components.add(ComponentEntry {
            name: name.into(),
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

    pub fn unregister(&mut self, c: ComponentId) {
        for fid in self.components.get(c).unwrap().fields.iter() {
            self.fields.remove(*fid);
            // TODO trigger events ?
        }
        self.components.remove(c);
        self.registry.remove_bitset(c);
    }

    pub fn create(&mut self) -> Entity {
        self.registry.create()
    }

    pub fn destroy(&mut self, e: Entity) {
        self.registry.destroy(e);
    }

    pub fn find_next_component(&self, e: Entity, c: Option<ComponentId>) -> Option<ComponentId> {
        self.registry.find_next_component(e, c)
    }

    pub fn add_default(&mut self, e: Entity, c: ComponentId) {
        for fid in self.components.get(c).unwrap().fields.iter() {
            self.fields[*fid].data.add_default(e);
        }
        self.registry.set(e, c);
    }

    pub fn remove(&mut self, e: Entity, c: ComponentId) {
        self.registry.unset(e, c);
    }

    pub fn has(&self, e: Entity, c: ComponentId) -> bool {
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

    pub fn find_component(&self, name: &str) -> Option<ComponentId> {
        self.components.iter().find_map(|(id, entry)| {
            if entry.name.as_str() == name {
                Some(id)
            } else {
                None
            }
        })
    }

    pub fn find_field<T: FieldType>(&self, c: ComponentId, name: &str) -> Option<Field<T>> {
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
