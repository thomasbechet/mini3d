use crate::feature::core::component_type::{
    ComponentId, PrivateComponentTableMut, PrivateComponentTableRef,
};

pub mod any;
pub mod native;

pub trait ComponentViewRef {
    fn view(table: PrivateComponentTableRef, component: ComponentId) -> Self;
}

pub trait ComponentViewMut {
    fn view_mut(table: PrivateComponentTableMut, component: ComponentId, cycle: u32) -> Self;
}
