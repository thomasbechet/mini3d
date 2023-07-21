use crate::{
    registry::component::ComponentId,
    utils::slotmap::{SlotId, SlotMap},
};

enum TreeNode {
    Node { component: ComponentId },
    EdgeAdd { component: ComponentId, node: u32 },
    EdgeRemove { component: ComponentId, node: u32 },
}

pub(crate) type ArchetypeId = SlotId;

struct ArchetypeEntry {}

pub(crate) struct ArchetypeTable {
    entries: SlotMap<ArchetypeEntry>,
}

impl ArchetypeTable {
    pub(crate) fn iter_components(&self, id: ArchetypeId) -> impl Iterator<Item = ComponentId> {
        unimplemented!()
    }
}
