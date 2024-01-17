use mini3d_derive::Serialize;
use mini3d_utils::slotmap::Key;

use crate::{context::Context, entity::Entity, error::ComponentError, system::SystemStageKey};

use super::{Component, ComponentStorage};

#[derive(Default, Clone, Serialize)]
pub struct SystemStage {
    pub(crate) auto_enable: bool,
    #[serialize(skip)]
    pub(crate) key: SystemStageKey,
}

impl SystemStage {
    pub const IDENT: &'static str = "system_stage";
    pub const START: &'static str = "start";
    pub const TICK: &'static str = "tick";

    pub fn new(auto_enable: bool) -> Self {
        Self {
            auto_enable,
            key: Default::default(),
        }
    }

    pub fn is_enable(&self) -> bool {
        !self.key.is_null()
    }
}

impl Component for SystemStage {
    const STORAGE: ComponentStorage = ComponentStorage::Single;

    fn on_added(&mut self, entity: Entity, ctx: &mut Context) -> Result<(), ComponentError> {
        if self.auto_enable {
            Self::enable(ctx, entity);
        }
        Ok(())
    }

    fn on_removed(&mut self, entity: Entity, ctx: &mut Context) -> Result<(), ComponentError> {
        if !self.key.is_null() {
            Self::disable(ctx, entity);
        }
        Ok(())
    }
}
