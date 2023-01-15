use rapier3d::prelude::RigidBodyHandle;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct RigidBodyComponent {
    #[serde(skip)]
    pub(crate) rigid_body_handle: Option<RigidBodyHandle>,    
}

impl RigidBodyComponent {
    
}