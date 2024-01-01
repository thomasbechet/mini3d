use alloc::vec::Vec;
use mini3d_derive::{Component, Reflect, Serialize};

use crate::{
    ecs::{
        context::Context,
        entity::Entity,
        error::ResolverError,
        query::Query,
        system::{ParallelSystem, SystemResolver},
        view::native::single::{NativeSingleView, NativeSingleViewMut, NativeSingleViewRef},
    },
    math::{
        mat::{M4, M4I32F16},
        quat::{Q, QI32F16},
        vec::{V3, V3I32F16},
    },
};

use super::{hierarchy::Hierarchy, local_to_world::LocalToWorld};

#[derive(Default, Debug, Component, Serialize, Reflect, Clone)]
pub struct Transform {
    pub translation: V3I32F16,
    pub rotation: QI32F16,
    pub scale: V3I32F16,
}

impl Transform {
    pub fn from_translation(translation: V3I32F16) -> Self {
        Self {
            translation,
            rotation: Q::default(),
            scale: V3::ONE,
        }
    }

    pub fn matrix(&self) -> M4I32F16 {
        M4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }

    pub fn forward(&self) -> V3I32F16 {
        self.rotation * V3::Z
    }

    pub fn backward(&self) -> V3I32F16 {
        self.rotation * V3::NEG_Z
    }

    pub fn up(&self) -> V3I32F16 {
        self.rotation * V3::Y
    }

    pub fn down(&self) -> V3I32F16 {
        self.rotation * V3::NEG_Y
    }

    pub fn left(&self) -> V3I32F16 {
        self.rotation * V3::X
    }

    pub fn right(&self) -> V3I32F16 {
        self.rotation * V3::NEG_X
    }
}

fn recursive_propagate(
    entity: Entity,
    transforms: &NativeSingleViewRef<Transform>,
    local_to_worlds: &mut NativeSingleViewMut<LocalToWorld>,
    hierarchies: &NativeSingleViewRef<Hierarchy>,
) -> M4I32F16 {
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

#[derive(Default, Clone)]
pub struct PropagateTransforms {
    transform: NativeSingleViewRef<Transform>,
    hierarchy: NativeSingleViewRef<Hierarchy>,
    local_to_world: NativeSingleViewMut<LocalToWorld>,
    query: Query,
}

impl PropagateTransforms {
    pub const NAME: &'static str = "SYS_PropagateTransforms";
}

impl ParallelSystem for PropagateTransforms {
    fn setup(&mut self, resolver: &mut SystemResolver) -> Result<(), ResolverError> {
        self.transform.resolve(resolver, Transform::NAME)?;
        self.hierarchy.resolve(resolver, Hierarchy::NAME)?;
        self.local_to_world.resolve(resolver, LocalToWorld::NAME)?;
        self.query.resolve(resolver).all(&[LocalToWorld::NAME])?;
        Ok(())
    }

    fn run(mut self, _ctx: &Context) {
        // Reset all flags
        let mut entities = Vec::new();
        for e in self.query.iter() {
            self.local_to_world[e].dirty = true;
            entities.push(e);
        }

        for e in entities {
            let mut local_to_world = self.local_to_world.get(e).cloned().unwrap();
            if local_to_world.dirty {
                if let Some(hierarchy) = self.hierarchy.get(e) {
                    if let Some(parent) = hierarchy.parent() {
                        let parent_matrix = recursive_propagate(
                            parent,
                            &self.transform,
                            &mut self.local_to_world,
                            &self.hierarchy,
                        );
                        local_to_world.matrix = parent_matrix * self.transform[e].matrix();
                    } else {
                        local_to_world.matrix = self.transform[e].matrix();
                    }
                } else {
                    local_to_world.matrix = self.transform[e].matrix();
                }
                local_to_world.dirty = false;
                self.local_to_world[e] = local_to_world;
            }
        }
    }
}
