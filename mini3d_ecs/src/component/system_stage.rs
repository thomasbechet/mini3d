use mini3d_derive::Serialize;
use mini3d_math::fixed::U32F16;
use mini3d_utils::{slotmap::Key, string::AsciiArray};

use crate::{
    context::{Command, Context},
    entity::Entity,
    error::ComponentError,
    scheduler::Invocation,
    system::SystemStageKey,
};

use super::{Component, ComponentStorage};

#[derive(Default, Clone, Serialize)]
pub struct SystemStage {
    pub(crate) name: AsciiArray<32>,
    pub(crate) auto_enable: bool,
    #[serialize(skip)]
    pub(crate) key: SystemStageKey,
}

impl SystemStage {
    pub const NAME: &'static str = "system_stage";
    pub const START: &'static str = "start";
    pub const TICK: &'static str = "tick";

    pub fn new(name: &str, auto_enable: bool) -> Self {
        Self {
            name: AsciiArray::from(name),
            auto_enable,
            key: Default::default(),
        }
    }

    pub fn invoke(ctx: &mut Context, stage: Entity, invocation: Invocation) {}

    pub fn is_enable(&self) -> bool {
        !self.key.is_null()
    }
}

impl Component for SystemStage {
    const STORAGE: ComponentStorage = ComponentStorage::Single;
    const NAME: &'static str = "system_stage";

    fn on_added(&mut self, entity: Entity, ctx: &mut Context) -> Result<(), ComponentError> {
        if self.auto_enable {
            Command::enable_system_stage(ctx, entity);
        }
        Ok(())
    }

    fn on_removed(&mut self, entity: Entity, ctx: &mut Context) -> Result<(), ComponentError> {
        if !self.key.is_null() {
            Command::disable_system_stage(ctx, entity);
        }
        Ok(())
    }
}
