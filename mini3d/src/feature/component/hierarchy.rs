use anyhow::Result;
use serde::{Serialize, Deserialize};

use crate::{scene::{world::World, entity::Entity, container::Component}, uid::UID};

#[derive(Default, Serialize, Deserialize)]
pub struct Hierarchy {
    parent: Option<Entity>,
    first_child: Option<Entity>,
    next_sibling: Option<Entity>,
}

impl Component for Hierarchy {}

impl Hierarchy {

    pub const NAME: &'static str = "hierarchy";
    pub const UID: UID = Hierarchy::NAME.into();

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
        
        // // Prepare query and view
        // let mut view = world.view_mut::<&mut Hierarchy>();

        // // Collect childs
        // if let Some(first_child) = view.get(entity).unwrap().first_child {
        //     let mut childs = Vec::new();
        //     childs.push(first_child);
        //     while let Some(next) = view.get(*childs.last().unwrap()).unwrap().next_sibling {
        //         childs.push(next);
        //     }
        //     Ok(childs)
        // } else {
        //     Ok(Vec::new())
        // }
        Ok(Vec::new())
    }

    pub fn attach(entity: Entity, child: Entity, world: &mut World) -> Result<()> {

        Ok(())
        // // Prepare query and view
        // let mut view = world.view_mut::<&mut Hierarchy>();

        // // Find the last child
        // let mut last_child: Option<Entity> = None;
        // if let Some(first_child) = view.get(entity).unwrap().first_child {
        //     last_child = Some(first_child);
        //     while let Some(next) = view.get(last_child.unwrap()).unwrap().next_sibling {
        //         // Prevent circular references
        //         if last_child.unwrap() == child {
        //             return Err(anyhow::anyhow!("Circular reference detected"));
        //         }
        //         last_child = Some(next);
        //     }
        // }

        // // Append the child
        // if let Some(next_sibling) = last_child {
        //     view.get(next_sibling).unwrap().next_sibling = Some(child);
        // } else {
        //     view.get(entity).unwrap().first_child = Some(child);
        // }

        // // Set child parent
        // view.get(child).unwrap().parent = Some(entity);

        // Ok(())
    }

    pub fn detach(entity: Entity, child: Entity, world: &mut World) -> Result<()> {
        
        // // Prepare query and view
        // let mut view = world.view_mut::<&mut Hierarchy>();

        // // Find the child
        // if let Some(first_child) = view.get(entity).unwrap().first_child {
        //     if first_child == child {
        //         // Remove child from the linked list
        //         if let Some(next_next) = view.get(first_child).unwrap().next_sibling {
        //             view.get(entity).unwrap().first_child = Some(next_next);
        //         } else {
        //             view.get(entity).unwrap().first_child = None;
        //         }
        //         // Unset parent
        //         view.get(child).unwrap().parent = None;
        //         return Ok(());
        //     } else {
        //         let mut next_child = first_child;
        //         while let Some(next) = view.get(next_child).unwrap().next_sibling {
        //             // Child found 
        //             if next == child {
        //                 // Remove child from the linked list
        //                 if let Some(next_next) = view.get(next).unwrap().next_sibling {
        //                     view.get(next_child).unwrap().next_sibling = Some(next_next);
        //                 } else {
        //                     view.get(next_child).unwrap().next_sibling = None;
        //                 }
        //                 // Unset parent
        //                 view.get(child).unwrap().parent = None;
        //                 return Ok(());
        //             }
        //             next_child = next;
        //         }
        //         return Err(anyhow::anyhow!("Child not found"));
        //     }
        // }
        Err(anyhow::anyhow!("Parent does not have childs"))
    }
}