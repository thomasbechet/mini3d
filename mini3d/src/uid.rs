use std::fmt::Display;

use serde::{Serialize, Deserialize};

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UID(u64);

impl UID {
    pub fn new(name: &str) -> Self {
        Self(const_fnv1a_hash::fnv1a_hash_str_64(name))
    }
    pub fn null() -> Self {
        Self(0)
    }
    pub fn is_null(&self) -> bool {
        self.0 == 0
    }
}

impl From<&str> for UID {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<&String> for UID {
    fn from(s: &String) -> Self {
        s.as_str().into()
    }
}

impl From<u64> for UID {
    fn from(uid: u64) -> Self {
        Self(uid)
    }
}

impl Display for UID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016X}", self.0)
    }
}