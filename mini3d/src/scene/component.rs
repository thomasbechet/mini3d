use anyhow::Result;
use serde::{Serialize, Deserialize};

use super::entity::EntityResolver;

pub trait Component: hecs::Component + Serialize + for<'de> Deserialize<'de> {
    fn resolve_entities(&mut self, resolver: &EntityResolver) -> Result<()> { Ok(()) }
}

#[derive(Serialize, Deserialize)]
pub struct TestComponent {

}

impl Component for TestComponent {}