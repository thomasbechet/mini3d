use crate::{
    api::Context,
    ecs::{
        container::native::array::NativeArrayContainer, entity::Entity, error::ResolverError,
        system::SystemResolver,
    },
    feature::core::component::ComponentData,
    utils::uid::ToUID,
};

pub trait NativeArrayView<C: ComponentData> {
    fn get(&self, ctx: &Context, entity: Entity) -> Option<&[C]>;
}

// Native array reference

pub struct NativeArrayViewRef<C: ComponentData> {
    pub(crate) container: *const NativeArrayContainer<C>,
}

impl<C: ComponentData> NativeArrayViewRef<C> {
    pub fn resolve(
        &mut self,
        resolver: &mut SystemResolver,
        component: impl ToUID,
    ) -> Result<(), ResolverError> {
        let id = resolver.read(component)?;
        self.container = &resolver
            .containers
            .entries
            .get(id)
            .unwrap()
            .container
            .as_any()
            .downcast_ref::<NativeArrayContainer<C>>();
        Ok(())
    }

    pub fn iter(&self, ctx: &Context) -> impl Iterator<Item = &[C]> {
        unsafe { *self.container }.iter()
    }
}

impl<C: ComponentData> NativeArrayView<C> for NativeArrayViewRef<C> {
    fn get(&self, ctx: &Context, entity: Entity) -> Option<&[C]> {
        unsafe { *self.container }.get(entity)
    }
}

// Native array mutable reference

pub struct NativeArrayViewMut<C: ComponentData> {
    pub(crate) container: *mut NativeArrayContainer<C>,
}

impl<C: ComponentData> NativeArrayViewMut<C> {
    pub fn resolve(
        &mut self,
        resolver: &mut SystemResolver,
        component: impl ToUID,
    ) -> Result<(), ResolverError> {
        let id = resolver.write(component)?;
        self.container = &resolver
            .containers
            .entries
            .get_mut(id)
            .unwrap()
            .container
            .as_any_mut()
            .downcast_mut::<NativeArrayContainer<C>>();
        Ok(())
    }

    pub fn get_mut(&mut self, ctx: &Context, entity: Entity) -> Option<&mut [C]> {
        unsafe { *self.container }.get_mut(entity)
    }

    pub fn iter_mut(&mut self, ctx: &Context) -> impl Iterator<Item = &mut [C]> {
        unsafe { *self.container }.iter_mut()
    }
}

impl<C: ComponentData> NativeArrayView<C> for NativeArrayViewMut<C> {
    fn get(&self, ctx: &Context, entity: Entity) -> Option<&[C]> {
        unsafe { *self.container }.get(entity)
    }
}
