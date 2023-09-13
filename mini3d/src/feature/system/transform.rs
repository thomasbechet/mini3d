use glam::Mat4;

use crate::{
    ecs::{
        api::{ecs::ParallelECS, ParallelAPI},
        entity::Entity,
        instance::ParallelResolver,
        query::Query,
        view::{StaticComponentView, StaticComponentViewMut, StaticComponentViewRef},
    },
    feature::component::scene::{
        hierarchy::Hierarchy, local_to_world::LocalToWorld, transform::Transform,
    },
    registry::{component::StaticComponent, error::RegistryError, system::ParallelSystem},
};

fn recursive_propagate(
    entity: Entity,
    transforms: &StaticComponentViewRef<Transform>,
    local_to_worlds: &mut StaticComponentViewMut<LocalToWorld>,
    hierarchies: &StaticComponentViewRef<Hierarchy>,
) -> Mat4 {
    if let Some(mut local_to_world) = local_to_worlds.get(entity).cloned() {
        if !local_to_world.dirty {
            return local_to_world.matrix;
        } else if let Some(hierarchy) = hierarchies.get(entity) {
            if let Some(parent) = hierarchy.parent() {
                let parent_matrix =
                    recursive_propagate(parent, transforms, local_to_worlds, hierarchies);
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

#[derive(Default)]
pub struct PropagateTransforms {
    transform: StaticComponent<Transform>,
    hierarchy: StaticComponent<Hierarchy>,
    local_to_world: StaticComponent<LocalToWorld>,
    query: Query,
}

impl PropagateTransforms {
    pub const NAME: &'static str = "propagate_transforms";
}

impl ParallelSystem for PropagateTransforms {
    fn setup(&mut self, resolver: &mut ParallelResolver) -> Result<(), RegistryError> {
        self.transform = resolver.read(Transform::NAME.into())?;
        self.hierarchy = resolver.read(Hierarchy::NAME.into())?;
        self.local_to_world = resolver.write(LocalToWorld::NAME.into())?;
        self.query = resolver.query().all(&[LocalToWorld::NAME.into()])?.build();
        Ok(())
    }

    fn run(&self, ecs: &mut ParallelECS, api: &mut ParallelAPI) {
        let transforms = ecs.view(self.transform)?;
        let hierarchies = ecs.view(self.hierarchy)?;
        let mut local_to_worlds = ecs.view_mut(self.local_to_world)?;

        // Reset all flags
        let mut entities = Vec::new();
        for e in ecs.query(self.query) {
            local_to_worlds[e].dirty = true;
            entities.push(e);
        }

        for e in entities {
            let mut local_to_world = local_to_worlds.get(e).cloned().unwrap();
            if local_to_world.dirty {
                if let Some(hierarcy) = hierarchies.get(e) {
                    if let Some(parent) = hierarcy.parent() {
                        let parent_matrix = recursive_propagate(
                            parent,
                            &transforms,
                            &mut local_to_worlds,
                            &hierarchies,
                        );
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
}
