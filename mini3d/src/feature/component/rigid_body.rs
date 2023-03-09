use rapier3d::prelude::RigidBodyHandle;
use serde::{Serialize, Deserialize};

use crate::{uid::UID, ecs::component::Component};

#[derive(Serialize, Deserialize)]
pub struct RigidBody {
    #[serde(skip)]
    pub(crate) rigid_body_handle: Option<RigidBodyHandle>,    
}

impl Component for RigidBody {}

impl RigidBody {
    pub const NAME: &'static str = "rigid_body";
    pub const UID: UID = UID::new(RigidBody::NAME);
}