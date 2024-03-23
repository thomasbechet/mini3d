use mini3d_db::{database::ComponentHandle, entity::Entity};
use mini3d_scheduler::StageHandle;

pub enum EventStage {
    Tick,
    ComponentAdded(ComponentHandle),
    ComponentRemoved(ComponentHandle),
}

pub struct EventData {
    pub entity: Entity,
    pub component: ComponentHandle,
}

#[derive(Default)]
pub(crate) struct ComponentEventStages {
    pub(crate) on_added: Option<StageHandle>,
    pub(crate) on_removed: Option<StageHandle>,
}
