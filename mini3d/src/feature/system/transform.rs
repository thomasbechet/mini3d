use anyhow::Result;
use glam::Mat4;

use crate::{feature::component::{local_to_world::LocalToWorld, hierarchy::Hierarchy, transform::Transform}, ecs::entity::Entity, context::SystemContext};

// fn recursive_propagate(
//     entity: Entity,
//     view: &mut QueryView<(&Transform, &mut LocalToWorld, Option<&Hierarchy>)>,
// ) -> Result<Mat4> {
//     if let Some((transform, global, hierarchy)) = view.get(entity) {
//         if !global.dirty {
//             return Ok(global.matrix);
//         } else if let Some(parent) = hierarchy.unwrap().parent() {
//             let parent_matrix = recursive_propagate(parent, view)?;
//             global.matrix = parent_matrix * transform.matrix();
//         } else {
//             global.matrix = transform.matrix();
//         }
//         global.dirty = false;
//         Ok(global.matrix)
//     } else {
//         Err(anyhow::anyhow!("Entity not found"))
//     }
// }

pub fn propagate(ctx: &SystemContext) -> Result<()> {
    
    let world = ctx.world().active();
    let transforms = world.view_mut::<Transform>(Transform::UID)?;
    let local_to_worlds = world.view_mut::<LocalToWorld>(LocalToWorld::UID)?;

    // Reset all flags
    let mut entities = Vec::new();
    for e in &world.query(&[LocalToWorld::UID]) {
        local_to_worlds[e].dirty = true;
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