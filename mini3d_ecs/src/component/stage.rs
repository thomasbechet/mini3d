use mini3d_derive::Serialize;

use crate::{container::linear::LinearContainer, scheduler::NodeKey};

use super::{NamedComponent, NativeComponent};

#[derive(Default, Clone, Serialize)]
pub struct Stage {
    #[serialize(skip)]
    pub(crate) first_node: NodeKey,
}

impl Stage {
    pub const TICK: &'static str = "tick";

    pub fn new() -> Self {
        Self::default()
    }
}

impl NamedComponent for Stage {
    const IDENT: &'static str = "stage";
}

impl NativeComponent for Stage {
    type Container = LinearContainer<Self>;
}
