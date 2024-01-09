use mini3d_derive::Serialize;
use mini3d_serialize::Serialize;

use crate::{entity::Entity, error::ComponentError};

#[derive(Serialize, Default)]
pub enum ComponentStorage {
    #[default]
    Single,
    Array(usize),
    List,
    Map,
    Spatial,
}

pub struct EntityResolver;

impl EntityResolver {
    pub(crate) fn resolve(&mut self, entity: Entity) -> Entity {
        Default::default()
    }
}

pub trait Component<Context>: 'static + Default + Serialize {
    const STORAGE: ComponentStorage;
    fn resolve_entities(&mut self, resolver: &mut EntityResolver) -> Result<(), ComponentError> {
        Ok(())
    }
    fn on_added(&mut self, entity: Entity, ctx: &mut Context) -> Result<(), ComponentError> {
        Ok(())
    }
    fn on_removed(&mut self, entity: Entity, ctx: &mut Context) -> Result<(), ComponentError> {
        Ok(())
    }
}
