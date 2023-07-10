use std::ops::{Deref, DerefMut};

use crate::{
    script::reflection::{ReadProperty, ReadWriteProperty},
    uid::UID,
};

use super::entity::Entity;

pub struct ComponentRef {
    entity: Entity,
    index: usize,
    ty: UID,
}
