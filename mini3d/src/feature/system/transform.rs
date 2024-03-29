use anyhow::Result;
use glam::Mat4;

use crate::{feature::component::{local_to_world::LocalToWorld, hierarchy::Hierarchy, transform::Transform}, ecs::{entity::Entity, view::{ComponentViewRef, ComponentViewMut, ComponentView}}, context::SystemContext};

pub fn recursive_propagate(
    entity: Entity, 
    transforms: &ComponentViewRef<Transform>,
    local_to_worlds: &mut ComponentViewMut<LocalToWorld>,
    hierarchies: &ComponentViewRef<Hierarchy>,
) -> Mat4 {
    if let Some(mut local_to_world) = local_to_worlds.get_mut(entity).cloned() {
        if !local_to_world.dirty {
            return local_to_world.matrix;
        } else if let Some(hierarchy) = hierarchies.get(entity) {
            if let Some(parent) = hierarchy.parent() {
                let parent_matrix = recursive_propagate(parent, transforms, local_to_worlds, hierarchies);
                local_to_world.matrix = parent_matrix * transforms[entity].matrix();
            } else {
                local_to_world.matrix = transforms[entity].matrix();
            }
        } else {
            local_to_world.matrix = transforms[entity].matrix();
        }
        local_to_world.dirty = false;
        let matrix = local_to_world.matrix;
        local_to_worlds[entity] = local_to_world;
        matrix
    } else {
        panic!("Entity not found");
    }
}

pub fn propagate(ctx: &mut SystemContext) -> Result<()> {
    
    let world = ctx.world.active();
    let transforms = world.view::<Transform>(Transform::UID)?;
    let hierarchies = world.view::<Hierarchy>(Hierarchy::UID)?;
    let mut local_to_worlds = world.view_mut::<LocalToWorld>(LocalToWorld::UID)?;

    // Reset all flags
    let mut entities = Vec::new();
    for e in &world.query(&[LocalToWorld::UID]) {
        local_to_worlds[e].dirty = true;
        entities.push(e);
    }

    for e in entities {
        let mut local_to_world = local_to_worlds.get_mut(e).cloned().unwrap();
        if local_to_world.dirty {
            if let Some(hierarcy) = hierarchies.get(e) {
                if let Some(parent) = hierarcy.parent() {
                    let parent_matrix = recursive_propagate(parent, &transforms, &mut local_to_worlds, &hierarchies);
                    local_to_world.matrix = parent_matrix * transforms[e].matrix();
                } else {
                    local_to_world.matrix = transforms[e].matrix();
                }
            } else {
                local_to_world.matrix = transforms[e].matrix();
            }
            local_to_world.dirty = false;
            local_to_worlds[e] = local_to_world;
        }
    }

    Ok(())
}