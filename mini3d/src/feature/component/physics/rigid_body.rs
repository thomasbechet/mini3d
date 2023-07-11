use mini3d_derive::{Component, Reflect, Serialize};
use rapier3d::prelude::RigidBodyHandle;

#[derive(Default, Component, Serialize, Reflect)]
pub struct RigidBody {
    #[serialize(skip)]
    pub(crate) rigid_body_handle: Option<RigidBodyHandle>,
}
