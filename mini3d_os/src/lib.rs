use component::os::OS;
use mini3d::{engine::Engine, anyhow::Result};

pub mod asset;
pub mod component;
pub mod input;
pub mod system;

pub fn define_features(engine: &mut Engine) -> Result<()> {
    engine.define_static_component::<OS>(OS::NAME)?;
    engine.define_static_system("update", crate::system::update::update)?;
    Ok(())
}