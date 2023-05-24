use crate::{context::{SystemContext, error::ContextError}, ecs::{entity::Entity, system::SystemResult}, feature::component::{lifecycle::Lifecycle, hierarchy::Hierarchy}, registry::component::Component};

pub fn run(ctx: &mut SystemContext) -> SystemResult {

    let mut despawn_entities: Vec<Entity> = Vec::new();
    let mut detach_entities = Vec::new();
    
    let mut world = ctx.world.active();
    
    {
        let mut hierarchies = world.view_mut::<Hierarchy>(Hierarchy::UID).with_context(|| "Failed to get hierarchy view")?;
        let lifecycles = world.view::<Lifecycle>(Lifecycle::UID)?;
        
        // Collect despawned entities
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
    }

    // Despawn entities
    for entity in despawn_entities {
        world.destroy(entity)?;
    }

    Ok(())
}