use mini3d_derive::{Reflect, Serialize};

use crate::{
    ecs::{
        context::Context,
        entity::Entity,
        scheduler::{Invocation, SystemStageKey},
    },
    math::fixed::U32F16,
};

use super::{Component, ComponentContext, ComponentError, ComponentStorage};

#[derive(Default, Clone, Reflect, Serialize)]
pub struct SystemStage {
    pub(crate) periodic: Option<U32F16>,
    #[serialize(skip)]
    pub(crate) key: SystemStageKey,
}

impl SystemStage {
    pub const NAME: &'static str = "system_stage";
    pub const START: &'static str = "start";
    pub const TICK: &'static str = "tick";

    pub fn periodic(periodic: U32F16) -> Self {
        Self {
            periodic: Some(periodic),
            key: SystemStageKey::null(),
        }
    }

    pub fn invoke(&self, ctx: &mut Context, invocation: Invocation) {
        ctx.scheduler.invoke(self.key, invocation)
    }
}

impl Component for SystemStage {
    const STORAGE: ComponentStorage = ComponentStorage::Single;
    fn on_added(&mut self, entity: Entity, ctx: ComponentContext) -> Result<(), ComponentError> {}
    fn on_removed(&mut self, entity: Entity, ctx: ComponentContext) -> Result<(), ComponentError> {}
}
