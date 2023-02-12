use core::cell::RefCell;
use anyhow::Result;

use crate::{asset::AssetManager, uid::UID, registry::RegistryManager};

pub struct AssetContext<'a> {
    registry: &'a RefCell<RegistryManager>,
    asset: &'a RefCell<AssetManager>,   
}

impl<'a> AssetContext<'a> {

    pub fn set_default<A: 'static>(&mut self, uid: UID) -> Result<()> {
        self.asset.borrow_mut().set_default::<A>(uid)
    }

    pub fn get<A: 'static>(&'_ self, uid: UID) -> Result<&'_ A> {
        
    }
}