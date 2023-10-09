use crate::registry::component::{
    ComponentType, PrivateComponentTableMut, PrivateComponentTableRef,
};

pub mod any;
pub mod native;

pub trait ComponentViewRef {
    fn view(table: PrivateComponentTableRef, ty: ComponentType) -> Self;
}

pub trait ComponentViewMut {
    fn view_mut(table: PrivateComponentTableMut, ty: ComponentType, cycle: u32) -> Self;
}
