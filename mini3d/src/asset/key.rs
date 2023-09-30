use mini3d_derive::Serialize;

use crate::utils::{
    string::AsciiArray,
    uid::{ToUID, UID},
};

pub(crate) const MAX_ASSET_KEY_LEN: usize = 64;

#[derive(Default, Serialize)]
pub struct AssetKey(AsciiArray<MAX_ASSET_KEY_LEN>);

impl AssetKey {
    pub(crate) fn new(key: &str) -> Self {
        let mut asset_key = Self(Default::default());
        asset_key.0.set(key).unwrap();
        asset_key
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
