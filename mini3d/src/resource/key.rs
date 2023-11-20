use crate::utils::slotmap::{Key, KeyIndex, KeyVersion};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResourceSlotVersion(pub(crate) u8);

impl KeyVersion for ResourceSlotVersion {
    fn next(&self) -> Self {
        // Ensure we don't generate a key version
        // that can't be stored in the resource handle.
        Self((self.0 + 1) % 64)
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResourceSlotIndex(pub(crate) u16);

impl From<usize> for ResourceSlotIndex {
    fn from(index: usize) -> Self {
        Self(index as u16)
    }
}

impl Into<usize> for ResourceSlotIndex {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl KeyIndex for ResourceSlotIndex {}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResourceSlotKey {
    version: ResourceSlotVersion,
    index: ResourceSlotIndex,
}

impl Key for ResourceSlotKey {
    type Version = ResourceSlotVersion;
    type Index = ResourceSlotIndex;

    fn new(index: Self::Index, version: Self::Version) -> Self {
        Self { version, index }
    }

    fn index(&self) -> Self::Index {
        self.index
    }

    fn version(&self) -> Self::Version {
        self.version
    }

    fn null() -> Self {
        Self {
            version: ResourceSlotVersion::default(),
            index: ResourceSlotIndex::default(),
        }
    }

    fn is_null(&self) -> bool {
        self.index.0 == 0
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct ResourceTypeVersion(pub(crate) u8);

impl KeyVersion for ResourceTypeVersion {
    fn next(&self) -> Self {
        // Ensure we don't generate a key version
        // that can't be stored in the resource handle.
        Self((self.0 + 1) % 4)
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct ResourceTypeIndex(pub(crate) u16);

impl From<usize> for ResourceTypeIndex {
    fn from(index: usize) -> Self {
        Self(index as u16)
    }
}

impl Into<usize> for ResourceTypeIndex {
    fn into(self) -> usize {
        self.0 as usize
    }
}

impl KeyIndex for ResourceTypeIndex {}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResourceTypeKey {
    version: ResourceTypeVersion,
    index: ResourceTypeIndex,
}

impl Key for ResourceTypeKey {
    type Version = ResourceTypeVersion;
    type Index = ResourceTypeIndex;

    fn new(index: Self::Index, version: Self::Version) -> Self {
        Self { version, index }
    }

    fn index(&self) -> Self::Index {
        self.index
    }

    fn version(&self) -> Self::Version {
        self.version
    }

    fn null() -> Self {
        Self {
            version: ResourceTypeVersion::default(),
            index: ResourceTypeIndex::default(),
        }
    }

    fn is_null(&self) -> bool {
        self.index.0 == 0
    }
}
