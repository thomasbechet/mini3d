use std::fmt::Display;

use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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

impl From<String> for UID {
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

impl From<u64> for UID {
    fn from(uid: u64) -> Self {
        Self(uid)
    }
}

impl From<UID> for u64 {
    fn from(uid: UID) -> Self {
        uid.0
    }
}

impl Display for UID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016X}", self.0)
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct SequentialGenerator {
    next: u64,
}

impl SequentialGenerator {
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> UID {
        self.next += 1;
        if self.next == 0 { // Prevent generating null uid
            self.next += 1;
        }
        UID::from(self.next)
    }
}