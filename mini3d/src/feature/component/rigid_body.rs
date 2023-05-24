use mini3d_derive::Component;
use rapier3d::prelude::RigidBodyHandle;

#[derive(Default, Component)]
#[component(name = "rigid_body")]
pub struct RigidBody {
    #[serialize(skip)]
    pub(crate) rigid_body_handle: Option<RigidBodyHandle>,    
}