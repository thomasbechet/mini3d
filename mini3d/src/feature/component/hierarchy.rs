use anyhow::Result;
use glam::{Mat4, Vec3, Quat};
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
        let childs = Vec::new();
        Ok(childs)
    }

    pub fn set_parent(entity: Entity, parent: Entity, world: &mut World) -> Result<()> {

        let mut query = world.query_mut::<&mut HierarchyComponent>();
        let mut view = query.view();

        if let Some(first_child) = view.get_mut(parent).unwrap().first_child {
            let mut next_sibling = first_child;
            while let Some(next) = view.get_mut(next_sibling).unwrap().next_sibling {
                next_sibling = next;
            }
            
        }

        Ok(())
    }
}