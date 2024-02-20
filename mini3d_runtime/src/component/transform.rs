use alloc::vec::Vec;
use mini3d_derive::{Component, Reflect, Serialize};

use crate::{
    ecs::{
        context::Context,
        entity::Entity,
        error::ResolverError,
        query::Query,
        system::Resolver,
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

fn rotator_system_resolve(
    resolver: &mut Resolver,
    v_transform: &mut NativeSingleViewRef<Transform>,
    v_hierarchy: &mut NativeSingleViewRef<Hierarchy>,
    v_local_to_world: &mut NativeSingleViewMut<LocalToWorld>,
    query: &mut Query,
) -> Result<(), ResolverError> {
    v_transform.resolve(resolver, Transform::NAME)?;
    v_hierarchy.resolve(resolver, Hierarchy::NAME)?;
    v_local_to_world.resolve(resolver, LocalToWorld::NAME)?;
    query
        .resolve(resolver)
        .all(&[Transform::NAME, Hierarchy::NAME])?;
    Ok(())
}

fn rotator_system(
    ctx: &Context,
    v_transform: &mut NativeSingleViewRef<Transform>,
    v_hierarchy: &mut NativeSingleViewRef<Hierarchy>,
    v_local_to_world: &mut NativeSingleViewMut<LocalToWorld>,
    query: &mut Query,
) {
    // Reset all flags
    let mut entities = Vec::new();
    for e in query.iter() {
        v_local_to_world[e].dirty = true;
        entities.push(e);
    }

    for e in entities {
        let mut local_to_world = v_local_to_world.get(e).cloned().unwrap();
        if local_to_world.dirty {
            if let Some(hierarchy) = v_hierarchy.get(e) {
                if let Some(parent) = hierarchy.parent() {
                    let parent_matrix =
                        recursive_propagate(parent, &v_transform, v_local_to_world, &v_hierarchy);
                    local_to_world.matrix = parent_matrix * v_transform[e].matrix();
                } else {
                    local_to_world.matrix = v_transform[e].matrix();
                }
            } else {
                local_to_world.matrix = v_transform[e].matrix();
            }
            local_to_world.dirty = false;
            v_local_to_world[e] = local_to_world;
        }
    }
}
