use mini3d_derive::Serialize;

use crate::utils::{
    string::AsciiArray,
    uid::{ToUID, UID},
};

pub(crate) const MAX_RESOURCE_PATH_LEN: usize = 64;

#[derive(Default, Serialize)]
pub struct AssetKey(AsciiArray<MAX_RESOURCE_PATH_LEN>);

impl AssetKey {
    pub(crate) fn new(path: &str) -> Self {
        let mut p = Self(Default::default());
        p.0.set(path).unwrap();
        p
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl ToUID for AssetKey {
    fn to_uid(&self) -> UID {
        self.0.as_str().to_uid()
    }
}

impl AsRef<str> for AssetKey {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}