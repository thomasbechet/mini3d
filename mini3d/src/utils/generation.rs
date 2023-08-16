use mini3d_derive::Serialize;

use super::slotmap::SlotId;

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize)]
pub struct GenerationId(u32);

#[derive(Default, Copy, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct VersionId(u8);

impl VersionId {
    pub fn next(&mut self) -> Self {
        let old = *self;
        self.0 = self.0.wrapping_add(1);
        old
    }
}

impl GenerationId {
    pub(crate) fn new(id: u32, version: VersionId) -> Self {
        Self(id | ((version.0 as u32) << 24))
    }

    pub(crate) fn from_slot(slot: SlotId, version: VersionId) -> Self {
        Self::new(slot.into(), version)
    }

    pub(crate) fn id(&self) -> u32 {
        self.0 & 0x00ff_ffff
    }

    pub(crate) fn slot(&self) -> SlotId {
        self.id().into()
    }

    pub(crate) fn version(&self) -> VersionId {
        VersionId((self.0 >> 24) as u8)
    }

    pub fn null() -> Self {
        Self(0)
    }
}

impl From<GenerationId> for SlotId {
    fn from(id: GenerationId) -> SlotId {
        id.slot()
    }
}
