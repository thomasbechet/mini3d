use mini3d_derive::{Component, Error, Reflect, Serialize};

use crate::ecs::{
    entity::Entity,
    view::single::{NativeSingleView, NativeSingleViewMut},
};

#[derive(Debug, Error)]
pub enum HierarchyError {
    #[error("Circular reference")]
    CircularReference,
    #[error("Child not found")]
    ChildNotFound,
    #[error("Parent without child")]
    ParentWithoutChild,
}

#[derive(Default, Component, Reflect, Serialize, Clone)]
pub struct Hierarchy {
    parent: Option<Entity>,
    first_child: Option<Entity>,
    next_sibling: Option<Entity>,
}

impl Hierarchy {
    pub fn parent(&self) -> Option<Entity> {
        self.parent
    }

    pub fn first_child(&self) -> Option<Entity> {
        self.first_child
    }

    pub fn next_sibling(&self) -> Option<Entity> {
        self.next_sibling
    }

    pub fn collect_childs<V: NativeSingleView<Hierarchy>>(entity: Entity, view: &V) -> Vec<Entity> {
        if let Some(first_child) = view.get(entity).unwrap().first_child {
            let mut childs = Vec::new();
            childs.push(first_child);
            while let Some(next) = view.get(*childs.last().unwrap()).unwrap().next_sibling {
                childs.push(next);
            }
            childs
        } else {
            Vec::new()
        }
    }

    pub fn attach(
        entity: Entity,
        child: Entity,
        view: &mut NativeSingleViewMut<Hierarchy>,
    ) -> Result<(), HierarchyError> {
        // Find the last child
        let mut last_child: Option<Entity> = None;
        if let Some(first_child) = view.get(entity).unwrap().first_child {
            last_child = Some(first_child);
            while let Some(next) = view.get(last_child.unwrap()).unwrap().next_sibling {
                // Prevent circular references
                if last_child.unwrap() == child {
                    return Err(HierarchyError::CircularReference);
                }
                last_child = Some(next);
            }
        }

        // Append the child
        if let Some(next_sibling) = last_child {
            view.get_mut(next_sibling).unwrap().next_sibling = Some(child);
        } else {
            view.get_mut(entity).unwrap().first_child = Some(child);
        }

        // Set child parent
        view.get_mut(child).unwrap().parent = Some(entity);

        Ok(())
    }

    pub fn detach(
        entity: Entity,
        child: Entity,
        view: &mut NativeSingleViewMut<Hierarchy>,
    ) -> Result<(), HierarchyError> {
        // Find the child
        if let Some(first_child) = view.get(entity).unwrap().first_child {
            if first_child == child {
                // Remove child from the linked list
                if let Some(next_next) = view.get(first_child).unwrap().next_sibling {
                    view.get_mut(entity).unwrap().first_child = Some(next_next);
                } else {
                    view.get_mut(entity).unwrap().first_child = None;
                }
                // Unset parent
                view.get_mut(child).unwrap().parent = None;
                return Ok(());
            } else {
                let mut next_child = first_child;
                while let Some(next) = view.get(next_child).unwrap().next_sibling {
                    // Child found
                    if next == child {
                        // Remove child from the linked list
                        if let Some(next_next) = view.get(next).unwrap().next_sibling {
                            view.get_mut(next_child).unwrap().next_sibling = Some(next_next);
                        } else {
                            view.get_mut(next_child).unwrap().next_sibling = None;
                        }
                        // Unset parent
                        view.get_mut(child).unwrap().parent = None;
                        return Ok(());
                    }
                    next_child = next;
                }
                return Err(HierarchyError::ChildNotFound);
            }
        }
        Err(HierarchyError::ParentWithoutChild)
    }
}
