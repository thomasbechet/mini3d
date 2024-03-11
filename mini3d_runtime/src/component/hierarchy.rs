use alloc::vec::Vec;
use mini3d_db::entity::Entity;
use mini3d_derive::{component, Error};

use crate::{self as mini3d_runtime, api::API};

#[derive(Debug, Error)]
pub enum HierarchyError {
    #[error("Circular reference")]
    CircularReference,
    #[error("Child not found")]
    ChildNotFound,
    #[error("Parent without child")]
    ParentWithoutChild,
}

#[component]
pub struct Hierarchy {
    parent: Entity,
    first_child: Entity,
    next_sibling: Entity,
}

impl Hierarchy {
    pub fn parent(&self, api: &API, e: Entity) -> Entity {
        api.read(e, self.parent).unwrap()
    }

    pub fn first_child(&self, api: &API, e: Entity) -> Entity {
        api.read(e, self.first_child).unwrap()
    }

    pub fn next_sibling(&self, api: &API, e: Entity) -> Entity {
        api.read(e, self.next_sibling).unwrap()
    }

    pub fn collect_childs(&self, api: &API, e: Entity) -> Vec<Entity> {
        let mut childs = Vec::new();
        if let Some(first_child) = self.first_child(api, e).nonnull() {
            childs.push(first_child);
            while let Some(next) = self.next_sibling(api, *childs.last().unwrap()).nonnull() {
                childs.push(next);
            }
        }
        childs
    }

    pub fn attach(&self, api: &mut API, e: Entity, child: Entity) -> Result<(), HierarchyError> {
        // Find the last child
        let mut last_child = self.first_child(api, e);
        if last_child != Entity::null() {
            while let Some(next) = self.next_sibling(api, last_child).nonnull() {
                // Prevent circular references
                if last_child == child {
                    return Err(HierarchyError::CircularReference);
                }
                last_child = next;
            }
        }

        // Append the child
        if let Some(next_sibling) = last_child.nonnull() {
            api.write(next_sibling, self.next_sibling, child);
        } else {
            api.write(e, self.first_child, child);
        }

        // Set child parent
        api.write(child, self.parent, e);

        Ok(())
    }

    pub fn detach(&self, api: &mut API, e: Entity, child: Entity) -> Result<(), HierarchyError> {
        // Find the child
        if let Some(first_child) = self.first_child(api, e).nonnull() {
            if first_child == child {
                // Remove child from the linked list
                if let Some(next_next) = self.next_sibling(api, first_child).nonnull() {
                    api.write(e, self.first_child, next_next);
                } else {
                    api.write(e, self.first_child, Entity::null());
                }
                // Unset parent
                api.write(child, self.parent, Entity::null());
                return Ok(());
            }
            let mut next_child = first_child;
            while let Some(next) = self.next_sibling(api, next_child).nonnull() {
                // Child found
                if next == child {
                    // Remove child from the linked list
                    if let Some(next_next) = self.next_sibling(api, next).nonnull() {
                        api.write(next_child, self.next_sibling, next_next);
                    } else {
                        api.write(next_child, self.next_sibling, Entity::null());
                    }
                    // Unset parent
                    api.write(child, self.parent, Entity::null());
                    return Ok(());
                }
                next_child = next;
            }
            return Err(HierarchyError::ChildNotFound);
        }
        Err(HierarchyError::ParentWithoutChild)
    }
}
