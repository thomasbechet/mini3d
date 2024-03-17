use core::fmt::Display;

use mini3d_derive::Serialize;

// Raw handle representation
// Null handle representation is possible but it should not be used
// as sentinel value.
#[derive(Serialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Handle(u32);

impl From<u32> for Handle {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl From<Handle> for u32 {
    fn from(handle: Handle) -> u32 {
        handle.0
    }
}

impl Handle {
    pub fn null() -> Self {
        Self(0)
    }

    pub fn raw(&self) -> u32 {
        self.0
    }
}

impl Default for Handle {
    fn default() -> Self {
        Self::null()
    }
}

impl Display for Handle {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if *self == Self::null() {
            write!(f, "null")
        } else {
            write!(f, "{:#010x}", self.0)
        }
    }
}

