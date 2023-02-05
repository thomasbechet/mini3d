use anyhow::Result;

use crate::{feature::component::{lifecycle::Lifecycle, hierarchy::Hierarchy}, scene::{context::SystemContext, world::World, entity::Entity}};

pub fn run(_ctx: &mut SystemContext, world: &mut World) -> Result<()> {
    let mut despawn_entities: Vec<Entity> = Vec::new();
    let mut detach_entities = Vec::new();
    for (e, (lifecycle, hierarchy)) in world.query_mut::<(&Lifecycle, Option<&Hierarchy>)>() {
        if !lifecycle.alive {
            despawn_entities.push(e);
            if let Some(hierarchy) = hierarchy {
                if let Some(parent) = hierarchy.parent() {
                    detach_entities.push((parent, e));
                }
            }
        }
    }
    // Detach entities
    for (parent, entity) in detach_entities {
        for child in Hierarchy::collect_childs(entity, world)? {
            Hierarchy::detach(entity, child, world)?;
        }
        Hierarchy::detach(parent, entity, world)?;
    }
    // Despawn entities
    for entity in despawn_entities {
        world.remove_entity(entity)?;
    }
    Ok(())
}