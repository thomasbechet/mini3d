use crate::{
    ecs::{
        context::ExclusiveContext,
        entity::Entity,
        query::QueryId,
        system::{ExclusiveResolver, SystemResult},
    },
    feature::component::{common::lifecycle::Lifecycle, scene::hierarchy::Hierarchy},
    registry::{
        component::{Component, ComponentId},
        error::RegistryError,
        system::ExclusiveSystem,
    },
};

#[derive(Default)]
pub struct DespawnEntities {
    life_cycle: ComponentId,
    hierarchy: ComponentId,
    query: QueryId,
}

impl ExclusiveSystem for DespawnEntities {
    const NAME: &'static str = "despawn_entities";

    fn resolve(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), RegistryError> {
        self.life_cycle = resolver.find(Lifecycle::UID)?;
        self.hierarchy = resolver.find(Hierarchy::UID)?;
        self.query = resolver
            .query()
            .all(&[self.life_cycle, self.hierarchy])
            .build();
        Ok(())
    }

    fn run(&self, ctx: &mut ExclusiveContext) -> SystemResult {
        let mut despawn_entities: Vec<Entity> = Vec::new();
        let mut detach_entities = Vec::new();

        {
            let mut hierarchies = ctx
                .scene
                .view_mut(self.hierarchy)?
                .as_static::<Hierarchy>()?;
            let lifecycles = ctx.scene.view(self.life_cycle)?.as_static::<Lifecycle>()?;

            // Collect despawned entities
            for e in ctx.scene.query(self.query) {
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
            ctx.scene.remove(entity);
        }

        Ok(())
    }
}
