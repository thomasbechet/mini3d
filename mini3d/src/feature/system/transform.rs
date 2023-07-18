use glam::Mat4;

use crate::{
    ecs::{
        context::ParallelContext,
        entity::Entity,
        system::SystemResult,
        view::{
            StaticSceneComponentView, StaticSceneComponentViewMut, StaticSceneComponentViewRef,
        },
    },
    feature::component::scene::{
        hierarchy::Hierarchy, local_to_world::LocalToWorld, transform::Transform,
    },
    registry::{
        component::{Component, ComponentId},
        error::RegistryError,
        system::{ParallelResolver, ParallelSystem},
    },
};

fn recursive_propagate(
    entity: Entity,
    transforms: &StaticSceneComponentViewRef<Transform>,
    local_to_worlds: &mut StaticSceneComponentViewMut<LocalToWorld>,
    hierarchies: &StaticSceneComponentViewRef<Hierarchy>,
) -> Mat4 {
    if let Some(mut local_to_world) = local_to_worlds.get_mut(entity).cloned() {
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
    transform: ComponentId,
    hierarchy: ComponentId,
    local_to_world: ComponentId,
}

impl ParallelSystem for PropagateTransforms {
    const NAME: &'static str = "propagate_transforms";

    fn resolve(&mut self, resolver: &mut ParallelResolver) -> Result<(), RegistryError> {
        self.transform = resolver.read(Transform::UID)?;
        self.hierarchy = resolver.read(Hierarchy::UID)?;
        self.local_to_world = resolver.write(LocalToWorld::UID)?;
        Ok(())
    }

    fn run(&self, ctx: &mut ParallelContext) -> SystemResult {
        let transforms = ctx.scene.view(self.transform)?.as_static::<Transform>()?;
        let hierarchies = ctx.scene.view(self.hierarchy)?.as_static::<Hierarchy>()?;
        let mut local_to_worlds = ctx
            .scene
            .view_mut(self.local_to_world)?
            .as_static::<LocalToWorld>()?;

        // Reset all flags
        let mut entities = Vec::new();
        for e in &ctx.scene.query(&[self.local_to_world]) {
            local_to_worlds[e].dirty = true;
            entities.push(e);
        }

        for e in entities {
            let mut local_to_world = local_to_worlds.get_mut(e).cloned().unwrap();
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
