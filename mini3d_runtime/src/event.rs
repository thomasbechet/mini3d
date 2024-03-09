use mini3d_db::{container::ComponentId, entity::Entity};
use mini3d_scheduler::StageId;

pub enum EventStage {
    Tick,
    ComponentAdded(ComponentId),
    ComponentRemoved(ComponentId),
}

pub struct EventData {
    pub entity: Entity,
    pub component: ComponentId,
}

#[derive(Default)]
pub(crate) struct ComponentEventStages {
    pub(crate) on_added: StageId,
    pub(crate) on_removed: StageId,
}
