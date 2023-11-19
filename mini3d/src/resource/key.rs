use crate::utils::slotmap::{Key, KeyVersion};

pub(crate) struct ResourceSlotVersion(u8);

impl KeyVersion for ResourceSlotVersion {
    fn default() -> Self {
        Self(0)
    }
    fn next(&self) -> Self {
        // Ensure we don't generate a key version
        // that can't be stored in the resource handle.
        self.0 += 1;
        if self.0 >= 64 {
            self.0 = 0;
        }
    }
}

pub(crate) struct ResourceSlotKey {
    version: ResourceSlotVersion,
    index: u16,
}

impl Key for ResourceSlotKey {
    type Version = ResourceSlotVersion;
    type Index = u16;
    fn new(index: Self::Index, version: Self::Version) -> Self {
        Self {
            version: version.0,
            index,
        }
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
            index: 0,
        }
    }
    fn is_null(&self) -> bool {
        self.index == 0
    }
}

pub(crate) struct ResourceTypeVersion(u8);

impl KeyVersion for ResourceTypeVersion {
    fn default() -> Self {
        Self(0)
    }
    fn next(&self) -> Self {
        // Ensure we don't generate a key version
        // that can't be stored in the resource handle.
        self.0 += 1;
        if self.0 >= 4 {
            self.0 = 0;
        }
    }
}

pub(crate) struct ResourceTypeKey {
    version: ResourceTypeVersion,
    index: u16,
}

impl Key for ResourceTypeKey {
    type Version = ResourceTypeVersion;
    type Index = u16;

    fn new(index: Self::Index, version: Self::Version) -> Self {
        Self {
            version: version.0,
            index,
        }
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
            index: 0,
        }
    }

    fn is_null(&self) -> bool {
        self.index == 0
    }
}
