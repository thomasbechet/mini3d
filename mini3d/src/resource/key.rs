use mini3d_derive::Serialize;

use crate::utils::{
    prng::PCG32,
    string::AsciiArray,
    uid::{ToUID, UID},
};

pub(crate) const MAX_RESOURCE_KEY_LEN: usize = 64;

#[derive(Default, Serialize)]
pub struct ResourceKey(AsciiArray<MAX_RESOURCE_KEY_LEN>);

impl ResourceKey {
    pub(crate) fn new(key: &str) -> Self {
        Self(AsciiArray::from(key))
    }

    pub(crate) fn random(prng: &mut PCG32) -> Self {
        let mut key = AsciiArray::default();
        for i in 0..MAX_RESOURCE_KEY_LEN {
            let c = prng.next_u32() % 26;
            let c = char::from_u32(c + 65).unwrap();
            key.push(c);
        }
        Self(key)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl ToUID for ResourceKey {
    fn to_uid(&self) -> UID {
        self.0.as_str().to_uid()
    }
}

impl AsRef<str> for ResourceKey {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for ResourceKey {
    fn from(key: &str) -> Self {
        Self::new(key)
    }
}
