use mini3d_derive::Component;
use rapier3d::prelude::RigidBodyHandle;

#[derive(Default, Component)]
pub struct RigidBody {
    #[serialize(skip)]
    pub(crate) rigid_body_handle: Option<RigidBodyHandle>,
}
