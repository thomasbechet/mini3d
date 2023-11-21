use crate::utils::slotmap::Key;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResourceSlotKey {
    pub(crate) version: u8,
    pub(crate) index: u16,
}

impl Key for ResourceSlotKey {
    fn new(index: Option<usize>) -> Self {
        Self {
            version: 0,
            index: index.map_or(0xFFFF, |index| index as u16 & 0xFFFF),
        }
    }

    fn update(&mut self, index: Option<usize>) {
        self.version = (self.version + 1) % 64;
        self.index = index.map_or(0xFFFF, |index| index as u32 & 0xFFFF);
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

impl Key for ResourceTypeKey {
    fn new(index: Option<usize>) -> Self {
        Self {
            version: 0,
            index: index.map_or(0xFFFF, |index| index as u16 & 0xFFFF),
        }
    }

    fn update(&mut self, index: Option<usize>) {
        // Ensure we don't generate a key version
        // that can't be stored in the resource handle.
        self.version = (self.version + 1) % 4;
        self.index = index.map_or(0xFFFF, |index| index as u32 & 0xFFFF);
    }

    fn index(&self) -> Option<usize> {
        if self.index == 0xFFFF {
            None
        } else {
            Some(self.index as usize)
        }
    }
}
