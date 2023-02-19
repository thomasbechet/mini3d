use anyhow::Result;

use crate::{context::SystemContext, ecs::entity::Entity, feature::component::{lifecycle::Lifecycle, hierarchy::Hierarchy}};

pub fn run(ctx: &SystemContext) -> Result<()> {

    let mut despawn_entities: Vec<Entity> = Vec::new();
    let mut detach_entities = Vec::new();
    let world = ctx.world().active();
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
        for child in Hierarchy::collect_childs(entity, &hierarchies)? {
            Hierarchy::detach(entity, child, &mut hierarchies)?;
        }
        Hierarchy::detach(parent, entity, &mut hierarchies)?;
    }

    // Despawn entities
    for entity in despawn_entities {
        world.destroy(entity)?;
    }

    Ok(())
}