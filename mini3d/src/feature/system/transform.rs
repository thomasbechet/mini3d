use anyhow::Result;
use glam::Mat4;

use crate::{feature::component::{local_to_world::{Transform, LocalToWorld}, hierarchy::Hierarchy}, scene::{entity::Entity, context::SystemContext, world::World}};

fn recursive_propagate(
    entity: Entity,
    view: &mut QueryView<(&Transform, &mut LocalToWorld, Option<&Hierarchy>)>,
) -> Result<Mat4> {
    if let Some((transform, global, hierarchy)) = view.get(entity) {
        if !global.dirty {
            return Ok(global.matrix);
        } else if let Some(parent) = hierarchy.unwrap().parent() {
            let parent_matrix = recursive_propagate(parent, view)?;
            global.matrix = parent_matrix * transform.matrix();
        } else {
            global.matrix = transform.matrix();
        }
        global.dirty = false;
        Ok(global.matrix)
    } else {
        Err(anyhow::anyhow!("Entity not found"))
    }
}

pub fn propagate(_: &mut SystemContext, world: &mut World) -> Result<()> {
    
    // Reset all flags
    let mut entities = Vec::new();
    for (e, global) in world.query_mut::<&mut LocalToWorld>() {
        global.dirty = true;
        entities.push(e);
    }

    // // Prepare view
    // let mut view = world.view_mut::<(&Transform, &mut LocalToWorld, Option<&Hierarchy>)>();
    
    // // Propagate
    // for e in entities {
    //     let (transform, global, hierarchy) = view.get(e).unwrap();
    //     if !global.dirty { continue; }
    //     if let Some(hierarchy) = hierarchy {
    //         if let Some(parent) = hierarchy.parent() {
    //             let parent_matrix = recursive_propagate(parent, &mut view)?;
    //             global.matrix = parent_matrix * transform.matrix();
    //         } else {
    //             global.matrix = transform.matrix();
    //         }
    //     } else {
    //         global.matrix = transform.matrix();
    //     }
    //     global.dirty = false;
    // }

    Ok(())
}