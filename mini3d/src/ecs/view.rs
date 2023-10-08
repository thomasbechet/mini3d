use crate::registry::{
    component::{PrivateComponentTableMut, PrivateComponentTableRef},
    component_type::ComponentType,
};

pub mod native;
pub mod property;

pub trait ComponentViewRef {
    fn view(table: PrivateComponentTableRef, ty: ComponentType) -> Self;
}

pub trait ComponentViewMut {
    fn view_mut(table: PrivateComponentTableMut, ty: ComponentType, cycle: u32) -> Self;
}
