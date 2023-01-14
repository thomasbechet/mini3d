use anyhow::Result;
use hecs::{Entity, World};
use serde::{Serialize, Deserialize};

#[derive(Default, Serialize, Deserialize)]
pub struct HierarchyComponent {
    parent: Option<Entity>,
    first_child: Option<Entity>,
    next_sibling: Option<Entity>,
}

impl HierarchyComponent {

    pub fn parent(&self) -> Option<Entity> {
        self.parent
    }

    pub fn first_child(&self) -> Option<Entity> {
        self.first_child
    }

    pub fn next_sibling(&self) -> Option<Entity> {
        self.next_sibling
    }

    pub fn collect_childs(entity: Entity, world: &mut World) -> Result<Vec<Entity>> {
        
        // Prepare query and view
        let mut query = world.query_mut::<&mut HierarchyComponent>();
        let mut view = query.view();

        // Collect childs
        if let Some(first_child) = view.get_mut(entity).unwrap().first_child {
            let mut childs = Vec::new();
            childs.push(first_child);
            while let Some(next) = view.get_mut(*childs.last().unwrap()).unwrap().next_sibling {
                childs.push(next);
            }
            Ok(childs)
        } else {
            Ok(Vec::new())
        }
    }

    pub fn attach(entity: Entity, child: Entity, world: &mut World) -> Result<()> {

        // Prepare query and view
        let mut query = world.query_mut::<&mut HierarchyComponent>();
        let mut view = query.view();

        // Find the last child
        let mut last_child: Option<Entity> = None;
        if let Some(first_child) = view.get_mut(entity).unwrap().first_child {
            last_child = Some(first_child);
            while let Some(next) = view.get_mut(last_child.unwrap()).unwrap().next_sibling {
                // Prevent circular references
                if last_child.unwrap() == child {
                    return Err(anyhow::anyhow!("Circular reference detected"));
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

    pub fn detach(entity: Entity, child: Entity, world: &mut World) -> Result<()> {
        
        // Prepare query and view
        let mut query = world.query_mut::<&mut HierarchyComponent>();
        let mut view = query.view();

        // Find the child
        if let Some(first_child) = view.get_mut(entity).unwrap().first_child {
            if first_child == child {
                // Remove child from the linked list
                if let Some(next_next) = view.get_mut(first_child).unwrap().next_sibling {
                    view.get_mut(entity).unwrap().first_child = Some(next_next);
                } else {
                    view.get_mut(entity).unwrap().first_child = None;
                }
                // Unset parent
                view.get_mut(child).unwrap().parent = None;
                return Ok(());
            } else {
                let mut next_child = first_child;
                while let Some(next) = view.get_mut(next_child).unwrap().next_sibling {
                    // Child found 
                    if next == child {
                        // Remove child from the linked list
                        if let Some(next_next) = view.get_mut(next).unwrap().next_sibling {
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
                return Err(anyhow::anyhow!("Child not found"));
            }
        }
        Err(anyhow::anyhow!("Parent does not have childs"))
    }
}