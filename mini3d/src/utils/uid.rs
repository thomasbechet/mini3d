use core::{fmt::Display, iter::Sum};

use alloc::string::String;

use crate::serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize};

/// Fast FNV1A hash algorithm taken from https://en.wikipedia.org/wiki/Fowler%E2%80%93Noll%E2%80%93Vo_hash_function

const FNV1A_HASH_32: u32 = 0x811c9dc5;
const FNV1A_HASH_64: u64 = 0xcbf29ce484222325;

const FNV1A_PRIME_32: u32 = 0x01000193;
const FNV1A_PRIME_64: u64 = 0x100000001b3;

pub const fn fnv1a_hash_32(bytes: &[u8]) -> u32 {
    let mut hash = FNV1A_HASH_32;
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        hash ^= bytes[i] as u32;
        hash = hash.wrapping_mul(FNV1A_PRIME_32);
        i += 1;
    }
    hash
}

pub const fn fnv1a_hash_32_str(s: &str) -> u32 {
    fnv1a_hash_32(s.as_bytes())
}

pub const fn fnv1a_hash_64(bytes: &[u8]) -> u64 {
    let mut hash = FNV1A_HASH_64;
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        // For loop is not supported in const fn
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(FNV1A_PRIME_64);
        i += 1;
    }
    hash
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UID(u64);

impl UID {
    pub const fn new(name: &str) -> Self {
        Self(fnv1a_hash_64(name.as_bytes()))
    }

    pub fn null() -> Self {
        Self(0)
    }

    pub fn is_null(&self) -> bool {
        self.0 == 0
    }
}

pub trait ToUID {
    fn to_uid(&self) -> UID;
}

impl ToUID for UID {
    fn to_uid(&self) -> UID {
        *self
    }
}

impl From<&str> for UID {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl ToUID for &str {
    fn to_uid(&self) -> UID {
        UID::new(self)
    }
}

impl ToUID for String {
    fn to_uid(&self) -> UID {
        UID::new(self)
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:016X}", self.0)
    }
}

impl Sum for UID {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::null(), |a, b| Self(a.0 + b.0))
    }
}

impl Serialize for UID {
    type Header = ();
    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_u64(self.0)
    }
    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        Ok(Self(decoder.read_u64()?))
    }
}

#[derive(Default)]
pub struct SequentialGenerator {
    next: u64,
}

impl SequentialGenerator {
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> UID {
        self.next += 1;
        if self.next == 0 {
            // Prevent generating null uid
            self.next += 1;
        }
        UID::from(self.next)
    }
}
