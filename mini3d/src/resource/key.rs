use crate::utils::slotmap::Key;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResourceSlotKey {
    pub(crate) version: u8,
    pub(crate) index: u16,
}

impl Key for ResourceSlotKey {
    fn new(index: usize) -> Self {
        Self {
            version: 0,
            index: (index & 0xFFFF) as u16,
        }
    }

    fn null() -> Self {
        Self {
            version: 0,
            index: 0xFFFF,
        }
    }

    fn update(&mut self, index: usize) {
        self.version = (self.version + 1) % 64;
        self.index = (index & 0xFFFF) as u16;
    }

    fn index(&self) -> Option<usize> {
        if self.index == 0xFFFF {
            None
        } else {
            Some(self.index as usize)
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResourceTypeKey {
    pub(crate) version: u8,
    pub(crate) index: u16,
}

impl Default for ResourceTypeKey {
    fn default() -> Self {
        Self::null()
    }
}

impl Key for ResourceTypeKey {
    fn new(index: usize) -> Self {
        Self {
            version: 0,
            index: (index & 0xFFFF) as u16,
        }
    }

    fn null() -> Self {
        Self {
            version: 0,
            index: 0xFFFF,
        }
    }

    fn update(&mut self, index: usize) {
        // Ensure we don't generate a key version
        // that can't be stored in the resource handle.
        self.version = (self.version + 1) % 4;
        self.index = (index & 0xFFFF) as u16;
    }

    fn index(&self) -> Option<usize> {
        if self.index == 0xFFFF {
            None
        } else {
            Some(self.index as usize)
        }
    }
}
