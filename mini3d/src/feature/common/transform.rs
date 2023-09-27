use glam::{Mat4, Quat, Vec3};
use mini3d_derive::{Component, Reflect, Serialize};

use crate::{
    ecs::{
        api::{context::Context, ecs::ECS},
        entity::Entity,
        instance::ParallelResolver,
        query::Query,
        view::single::{StaticSingleView, StaticSingleViewMut, StaticSingleViewRef},
    },
    registry::{component::StaticComponentType, error::RegistryError, system::ParallelSystem},
};

use super::{hierarchy::Hierarchy, local_to_world::LocalToWorld};

#[derive(Default, Debug, Component, Serialize, Reflect, Clone)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn from_translation(translation: Vec3) -> Self {
        Self {
            translation,
            rotation: Quat::default(),
            scale: Vec3::ONE,
        }
    }

    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    pub fn forward(&self) -> Vec3 {
        self.rotation * Vec3::Z
    }

    pub fn backward(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Z
    }

    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    pub fn down(&self) -> Vec3 {
        self.rotation * Vec3::NEG_Y
    }

    pub fn left(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::NEG_X
    }
}

fn recursive_propagate(
    entity: Entity,
    transforms: &StaticSingleViewRef<Transform>,
    local_to_worlds: &mut StaticSingleViewMut<LocalToWorld>,
    hierarchies: &StaticSingleViewRef<Hierarchy>,
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
    transform: StaticComponentType<Transform>,
    hierarchy: StaticComponentType<Hierarchy>,
    local_to_world: StaticComponentType<LocalToWorld>,
    query: Query,
}

impl PropagateTransforms {
    pub const NAME: &'static str = "propagate_transforms";
}

impl ParallelSystem for PropagateTransforms {
    fn setup(&mut self, resolver: &mut ParallelResolver) -> Result<(), RegistryError> {
        self.transform = resolver.read(Transform::NAME)?;
        self.hierarchy = resolver.read(Hierarchy::NAME)?;
        self.local_to_world = resolver.write(LocalToWorld::NAME)?;
        self.query = resolver.query().all(&[LocalToWorld::NAME])?.build();
        Ok(())
    }

    fn run(&self, ecs: &ECS, ctx: &Context) {
        let transforms = ecs.view(self.transform);
        let hierarchies = ecs.view(self.hierarchy);
        let mut local_to_worlds = ecs.view_mut(self.local_to_world);

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
    }
}
