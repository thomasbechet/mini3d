use mini3d_derive::Serialize;

use crate::{
    container::ContainerTable, context::Context, entity::Entity, error::ComponentError,
    scheduler::NodeKey,
};

use super::{Component, ComponentStorage};

#[derive(Default, Clone, Serialize)]
pub struct SystemStage {
    pub(crate) auto_enable: bool,
    pub(crate) active: bool,
    #[serialize(skip)]
    pub(crate) first_node: NodeKey,
}

impl SystemStage {
    pub const START: &'static str = "stage_start";
    pub const TICK: &'static str = "stage_tick";

    pub fn new(auto_enable: bool) -> Self {
        Self {
            auto_enable,
            active: false,
            first_node: Default::default(),
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
}

impl Component for SystemStage {
    const NAME: &'static str = "system_stage";
    const STORAGE: ComponentStorage = ComponentStorage::Single;

    fn on_added(&mut self, entity: Entity, ctx: &mut Context) -> Result<(), ComponentError> {
        if self.auto_enable {
            Self::enable(ctx, entity);
        }
        Ok(())
    }

    fn on_removed(&mut self, entity: Entity, ctx: &mut Context) -> Result<(), ComponentError> {
        if self.active {
            Self::disable(ctx, entity);
        }
        Ok(())
    }
}

pub(crate) fn enable_system_stage(
    entity: Entity,
    containers: &mut ContainerTable,
) -> Result<(), ComponentError> {
    let stages = containers.system_stages();
    let stage = stages
        .get_mut(entity)
        .ok_or(ComponentError::EntryNotFound)?;
    stage.active = true;
    Ok(())
}

pub(crate) fn disable_system_stage(entity: Entity, containers: &mut ContainerTable) {
    unimplemented!()
}
