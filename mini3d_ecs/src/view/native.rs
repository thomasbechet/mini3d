use crate::{
    component::Component,
    container::{native::NativeSingleContainer, ContainerEntry},
    context::Context,
    entity::Entity,
    error::{ComponentError, SystemError},
};

use super::{SystemView, ViewEntityPicker};

pub struct NativeSingleMut<C: Component> {
    pub(crate) ptr: *mut NativeSingleContainer<C>,
}

impl<C: Component> Default for NativeSingleMut<C> {
    fn default() -> Self {
        Self {
            ptr: core::ptr::null_mut(),
        }
    }
}

impl<C: Component> Clone for NativeSingleMut<C> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr }
    }
}

impl<C: Component> SystemView for NativeSingleMut<C> {
    fn resolve(&mut self, container: &ContainerEntry) -> Result<(), SystemError> {
        self.ptr = unsafe {
            container
                .container
                .get()
                .as_mut()
                .unwrap()
                .as_any_mut()
                .downcast_mut::<NativeSingleContainer<C>>()
                .ok_or(SystemError::ConfigError)?
        };
        Ok(())
    }
}

impl<C: Component> NativeSingleMut<C> {
    pub fn add(
        &mut self,
        ctx: &mut Context,
        entity: Entity,
        component: C,
    ) -> Result<&mut C, ComponentError> {
        if let Some(data) = unsafe { (*self.ptr).add(entity, component) } {
            data.on_added(entity, ctx)?;
            Ok(data)
        } else {
            Err(ComponentError::DuplicatedEntry)
        }
    }

    pub fn get(&self, entity: Entity) -> Option<&C> {
        unsafe { (*self.ptr).get(entity) }
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut C> {
        unsafe { (*self.ptr).get_mut(entity) }
    }
}

pub struct NativeSingleRef<C: Component> {
    pub(crate) ptr: *const NativeSingleContainer<C>,
}

impl<C: Component> Default for NativeSingleRef<C> {
    fn default() -> Self {
        Self {
            ptr: core::ptr::null(),
        }
    }
}

impl<C: Component> Clone for NativeSingleRef<C> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr }
    }
}

impl<C: Component> SystemView for NativeSingleRef<C> {
    fn resolve(&mut self, container: &ContainerEntry) -> Result<(), SystemError> {
        self.ptr = unsafe {
            container
                .container
                .get()
                .as_ref()
                .unwrap()
                .as_any()
                .downcast_ref::<NativeSingleContainer<C>>()
                .ok_or(SystemError::ConfigError)?
        };
        Ok(())
    }
}

impl<C: Component> NativeSingleRef<C> {
    pub fn get(&self, entity: Entity) -> Option<&C> {
        unsafe { (*self.ptr).get(entity) }
    }
}
