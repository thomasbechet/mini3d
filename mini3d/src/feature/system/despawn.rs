use crate::{
    ecs::{
        api::{ecs::ExclusiveECS, ExclusiveAPI},
        entity::Entity,
        instance::ExclusiveResolver,
        query::Query,
    },
    expect,
    feature::component::{common::lifecycle::Lifecycle, scene::hierarchy::Hierarchy},
    registry::{component::StaticComponent, error::RegistryError, system::ExclusiveSystem},
};

#[derive(Default)]
pub struct DespawnEntities {
    life_cycle: StaticComponent<Lifecycle>,
    hierarchy: StaticComponent<Hierarchy>,
    query: Query,
}

impl DespawnEntities {
    pub const NAME: &'static str = "despawn_entities";
}

impl ExclusiveSystem for DespawnEntities {
    fn setup(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), RegistryError> {
        self.life_cycle = resolver.find(Lifecycle::NAME)?;
        self.hierarchy = resolver.find(Hierarchy::NAME)?;
        self.query = resolver
            .query()
            .all(&[Lifecycle::NAME, Hierarchy::NAME])?
            .build();
        Ok(())
    }

    fn run(&self, ecs: &mut ExclusiveECS, api: &mut ExclusiveAPI) {
        // let mut despawn_entities: Vec<Entity> = Vec::new();
        // let mut detach_entities = Vec::new();

        // {
        //     let mut hierarchies = ecs.view_mut(self.hierarchy);
        //     let lifecycles = ecs.view(self.life_cycle);

        //     // Collect despawned entities
        //     for e in ecs.query(self.query) {
        //         if !lifecycles[e].alive {
        //             despawn_entities.push(e);
        //             if let Some(hierarchy) = hierarchies.get_mut(e) {
        //                 if let Some(parent) = hierarchy.parent() {
        //                     detach_entities.push((parent, e));
        //                 }
        //             }
        //         }
        //     }

        //     // Detach entities
        //     for (parent, entity) in detach_entities {
        //         for child in Hierarchy::collect_childs(entity, &hierarchies) {
        //             expect!(api, Hierarchy::detach(entity, child, &mut hierarchies));
        //         }
        //         expect!(api, Hierarchy::detach(parent, entity, &mut hierarchies));
        //     }
        // }

        // // Despawn entities
        // for entity in despawn_entities {
        //     ecs.remove(entity);
        // }
    }
}
