use core::cell::RefCell;

use alloc::{boxed::Box, rc::Rc, sync::Arc, vec::Vec};
use mini3d_derive::Serialize;

use crate::{
    container::{ContainerEntry, ContainerTable},
    context::Context,
    entity::Entity,
    error::SystemError,
};

pub struct SystemResolver<'a> {
    pub(crate) containers: &'a mut ContainerTable,
    pub(crate) views: &'a [Entity],
    pub(crate) index: u8,
}

impl<'a> SystemResolver<'a> {
    pub(crate) fn find_named(&mut self, name: &str) -> Result<&'_ ContainerEntry, SystemError> {
        let key = self
            .containers
            .find_container_key_by_name(name)
            .ok_or(SystemError::ConfigError)?;
        Ok(&self.containers.entries[key])
    }

    pub(crate) fn next_dynamic(&'_ mut self) -> Result<&'_ ContainerEntry, SystemError> {
        if self.index >= self.views.len() as u8 {
            return Err(SystemError::ConfigError);
        }
        let entity = self.views[self.index as usize];
        self.index += 1;
        let container = self
            .containers
            .entries
            .iter()
            .find_map(|(_, entry)| {
                if entry.entity == entity {
                    Some(entry)
                } else {
                    None
                }
            })
            .ok_or(SystemError::ConfigError)?;
        Ok(container)
    }
}

pub trait ExclusiveSystem: 'static {
    fn resolve(&mut self, resolver: &mut SystemResolver) -> Result<(), SystemError>;
    fn run(&mut self, ctx: &mut Context) -> Result<(), SystemError>;
}

pub trait ParallelSystem: 'static {
    fn resolve(&mut self, resolver: &mut SystemResolver) -> Result<(), SystemError>;
    fn run(&mut self, ctx: &Context) -> Result<(), SystemError>;
}

pub(crate) enum Instance {
    Exclusive(Rc<RefCell<Box<dyn ExclusiveSystem>>>),
    Parallel(Arc<RefCell<Box<dyn ParallelSystem>>>),
}

impl Instance {
    pub(crate) fn resolve(&mut self, resolver: &mut SystemResolver) -> Result<(), SystemError> {
        match self {
            Instance::Exclusive(system) => system.borrow_mut().resolve(resolver),
            Instance::Parallel(system) => system.borrow_mut().resolve(resolver),
            _ => unimplemented!(),
        }
    }
}

#[derive(Default, Clone, Copy, Serialize)]
pub(crate) struct InstanceIndex(pub(crate) u16);

#[derive(Default)]
pub(crate) struct InstanceTable {
    pub(crate) entries: Vec<Instance>,
}
