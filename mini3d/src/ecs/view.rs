use std::{
    cell::{Ref, RefMut},
    ops::{Deref, DerefMut, Index, IndexMut},
};

use crate::registry::component::ComponentData;

use super::{container::single::StaticSingleContainer, entity::Entity};

pub mod array;
pub mod list;
pub mod map;
pub mod single;
