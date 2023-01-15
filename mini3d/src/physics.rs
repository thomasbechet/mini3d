use anyhow::Result;
use rapier3d::prelude::{RigidBodySet, ColliderSet, PhysicsPipeline, QueryPipeline, IslandManager, BroadPhase, NarrowPhase, CCDSolver, ImpulseJointSet, MultibodyJointSet};

#[derive(Default)]
pub struct PhysicsManager {
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: BroadPhase,
    narrow_phase: NarrowPhase,
    ccd_solver: CCDSolver,
    query_pipeline: QueryPipeline,

    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
}

impl PhysicsManager {
    
    pub(crate) fn update(&mut self) -> Result<()> {

        Ok(())
    }
}