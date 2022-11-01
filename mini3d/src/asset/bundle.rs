use serde::{Serialize, Deserialize};
use slotmap::new_key_type;

use super::Asset;

new_key_type! { pub struct BundleId; }

#[derive(Serialize, Deserialize)]
pub struct Bundle {
    
}

impl Asset for Bundle {
    type Id = BundleId;
    fn typename() -> &'static str { "bundle" }
}