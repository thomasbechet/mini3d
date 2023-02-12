use anyhow::Result;

use crate::{feature::component::{lifecycle::Lifecycle, hierarchy::Hierarchy}, scene::{context::SystemContext, world::World, entity::Entity}};

pub fn run(ctx: &mut SystemContext) -> Result<()> {
    
    let mut despawn_entities: Vec<Entity> = Vec::new();
    let mut detach_entities = Vec::new();
    let lifecycles = world.view::<Lifecycle>(Lifecycle::UID)?;
    let hierarchies = world.view_mut::<Hierarchy>(Hierarchy::UID)?;
    for e in &world.query(&[Lifecycle::UID, Hierarchy::UID]) {
        if !lifecycles[e].alive {
            despawn_entities.push(e);
            if let Some(hierarchy) = hierarchies.get_mut(e) {
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
        world.destroy(entity)?;
    }

    Ok(())
}