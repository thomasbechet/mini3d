use anyhow::Result;
use hecs::{CommandBuffer, World};

use crate::{scene::SystemContext, feature::component::{lifecycle::LifecycleComponent, hierarchy::HierarchyComponent}};

pub fn run(_ctx: &mut SystemContext, world: &mut World) -> Result<()> {
    let mut command_buffer = CommandBuffer::default();
    let mut detach_entities = Vec::new();
    for (e, (lifecycle, hierarchy)) in world.query_mut::<(&LifecycleComponent, Option<&HierarchyComponent>)>() {
        if !lifecycle.alive {
            command_buffer.despawn(e);
            if let Some(hierarchy) = hierarchy {
                if let Some(parent) = hierarchy.parent() {
                    detach_entities.push((parent, e));
                }
            }
        }
    }
    // Detach entities
    for (parent, entity) in detach_entities {
        for child in HierarchyComponent::collect_childs(entity, world)? {
            HierarchyComponent::detach(entity, child, world)?;
        }
        HierarchyComponent::detach(parent, entity, world)?;
    }
    // Despawn entities
    command_buffer.run_on(world);
    Ok(())
}