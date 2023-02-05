use rapier3d::prelude::RigidBodyHandle;
use serde::{Serialize, Deserialize};

use crate::scene::component::Component;

#[derive(Serialize, Deserialize)]
pub struct RigidBody {
    #[serde(skip)]
    pub(crate) rigid_body_handle: Option<RigidBodyHandle>,    
}

impl Component for RigidBody {}

impl RigidBody {
    
}