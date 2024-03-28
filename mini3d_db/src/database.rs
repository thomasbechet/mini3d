use core::{
    fmt::{self, Formatter},
    num::NonZeroU32,
};

use alloc::vec::Vec;
use mini3d_derive::Serialize;
use mini3d_utils::{
    handle::Handle, slot_map_key, slotmap::{DefaultKey, SlotMap}, string::AsciiArray
};

use crate::{
    entity::Entity,
    error::ComponentError,
    field::{ComponentField, Field, FieldEntry, FieldType, RawStorage},
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

pub(crate) enum ComponentData {
    Fields(Vec<FieldHandle>),
    Handle(RawStorage<Handle>),
    Tag,
}

pub(crate) struct ComponentEntry {
    pub(crate) name: AsciiArray<32>,
    pub(crate) state: ComponentState,
    pub(crate) data: ComponentData,
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
            if let ComponentData::Fields(fields) = &mut self.components.get_mut(id).unwrap().data {
                for fid in fields.iter() {
                    self.fields.remove(*fid);
                    // TODO trigger events ?
                }
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
        if self.find_component(name).is_some() {
            return Err(ComponentError::DuplicatedEntry);
        }
        let id = self.components.add(ComponentEntry {
            name: name.into(),
            state: ComponentState::Created,
            data: ComponentData::Tag,
        });
        self.registry.add_bitset(id);
        Ok(id)
    }

    pub fn register_handle(&mut self, name: &str) -> Result<ComponentHandle, ComponentError> {
        if self.find_component(name).is_some() {
            return Err(ComponentError::DuplicatedEntry);
        }
        let id = self.components.add(ComponentEntry {
            name: name.into(),
            state: ComponentState::Created,
            data: ComponentData::Handle(Default::default()),
        });
        self.registry.add_bitset(id);
        Ok(id)
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
            data: ComponentData::Fields(Vec::with_capacity(fields.len())),
        });
        let component = self.components.get_mut(id).unwrap();
        for field in fields {
            let fid = self.fields.add(FieldEntry {
                name: field.name.into(),
                data: field.create_storage(),
                ty: field.ty,
            });
            if let ComponentData::Fields(component_fields) = &mut component.data {
                component_fields.push(fid);
            }
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

    pub fn find_next_component(
        &self,
        e: Entity,
        c: Option<ComponentHandle>,
    ) -> Option<ComponentHandle> {
        self.registry.find_next_component(e, c)
    }

    pub fn add_default(&mut self, e: Entity, c: ComponentHandle) {
        let component = self.components.get(c).expect("Component not found");
        if component.state == ComponentState::Created {
            panic!("Trying to in creation component");
        }
        match &mut self.components.get_mut(c).unwrap().data {
            ComponentData::Fields(fields) => {
                for fid in fields.iter() {
                    self.fields[*fid].data.add_default(e);
                }
            }
            ComponentData::Handle(handles) => {
                handles.set(e, Default::default());
            }
            ComponentData::Tag => {}
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

    pub fn read_handle(&self, e: Entity, id: ComponentHandle) -> Handle {
        if let ComponentData::Handle(handles) = &self.components[id].data {
            return *handles.get(e);
        }
        panic!("not a key component")
    }

    pub fn write_handle(&mut self, e: Entity, id: ComponentHandle, h: Handle) {
        if let ComponentData::Handle(handles) = &mut self.components[id].data {
            handles.set(e, h);
        } else {
            panic!("not a key component");
        }
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
        if let ComponentData::Fields(fields) = &self.components[c].data {
            for field in fields.iter() {
                if self.fields[*field].name.as_str() == name {
                    return Some(Field(*field, Default::default()));
                }
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
            match &component.data {
                ComponentData::Fields(fields) => {
                    write!(f, "- {} (fields)", component.name)?;
                    for field in fields.iter() {
                        let field = &self.fields[*field];
                        writeln!(f)?;
                        write!(f, "  - ")?;
                        field.display(f, e)?;
                    }
                }
                ComponentData::Handle(handles) => {
                    write!(f, "- {} (handle)", component.name)?;
                    let handle = handles.get(e);
                    if let Some(handle) = handle.nonnull() {
                        write!(f, ": {:08X}", handle.raw())?;
                    } else {
                        write!(f, ": null")?;
                    }
                }
                ComponentData::Tag => {
                    write!(f, "- {} (tag)", component.name)?;
                }
            }
        }
        Ok(())
    }
}
