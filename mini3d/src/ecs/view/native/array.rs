use crate::{
    ecs::{
        container::native::array::NativeArrayContainer, entity::Entity, error::ResolverError,
        system::SystemResolver,
    },
    feature::ecs::component::Component,
    utils::uid::ToUID,
};

pub trait NativeArrayView<C: Component> {
    fn get(&self, entity: Entity) -> Option<&[C]>;
}

// Native array reference

pub struct NativeArrayViewRef<C: Component> {
    pub(crate) container: *const NativeArrayContainer<C>,
}

impl<C: Component> NativeArrayViewRef<C> {
    pub fn resolve(
        &mut self,
        resolver: &mut SystemResolver,
        component: impl ToUID,
    ) -> Result<(), ResolverError> {
        let id = resolver.read(component)?;
        unsafe {
            self.container = (*resolver
                .containers
                .entries
                .get(id.0)
                .unwrap()
                .container
                .get())
            .as_any()
            .downcast_ref::<NativeArrayContainer<C>>()
            .unwrap();
        }
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = &[C]> {
        unsafe { &*self.container }.iter()
    }
}

impl<C: Component> NativeArrayView<C> for NativeArrayViewRef<C> {
    fn get(&self, entity: Entity) -> Option<&[C]> {
        unsafe { &*self.container }.get(entity)
    }
}

// Native array mutable reference

pub struct NativeArrayViewMut<C: Component> {
    pub(crate) container: *mut NativeArrayContainer<C>,
}

impl<C: Component> NativeArrayViewMut<C> {
    pub fn resolve(
        &mut self,
        resolver: &mut SystemResolver,
        component: impl ToUID,
    ) -> Result<(), ResolverError> {
        let id = resolver.write(component)?;
        unsafe {
            self.container = (*resolver
                .containers
                .entries
                .get_mut(id.0)
                .unwrap()
                .container
                .get())
            .as_any_mut()
            .downcast_mut::<NativeArrayContainer<C>>()
            .unwrap();
        }
        Ok(())
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut [C]> {
        unsafe { &mut *self.container }.get_mut(entity)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut [C]> {
        unsafe { &mut *self.container }.iter_mut()
    }
}

impl<C: Component> NativeArrayView<C> for NativeArrayViewMut<C> {
    fn get(&self, entity: Entity) -> Option<&[C]> {
        unsafe { &*self.container }.get(entity)
    }
}
