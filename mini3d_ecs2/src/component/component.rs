use alloc::boxed::Box;
use mini3d_derive::Serialize;

use crate::{
    container::{linear::LinearContainer, ComponentId, Container},
    ecs::ECS,
    entity::Entity,
    error::ComponentError,
};

use super::{NamedComponent, SingleComponent};

#[derive(Serialize, Default)]
pub enum ComponentStorage {
    #[default]
    Single,
    Array(u16),
    List,
    Map,
    Spatial,
}

#[derive(Default, Serialize)]
pub struct Component {
    pub(crate) storage: ComponentStorage,
    #[serialize(skip)]
    pub(crate) id: (ComponentId, Option<Box<dyn Container>>),
}

impl NamedComponent for Component {
    const IDENT: &'static str = "component";
}

impl SingleComponent for Component {
    type Container = LinearContainer<Self>;

    fn on_post_added(ecs: &mut ECS, entity: Entity) -> Result<(), ComponentError> {
        let container = ecs
            .get_mut::<Component>(entity)
            .unwrap()
            .id
            .1
            .take()
            .unwrap();
        let id = ecs.containers.add_container(entity, container)?;
        ecs.get_mut::<Component>(entity).unwrap().id.0 = id;
        Ok(())
    }

    fn on_post_removed(ecs: &mut ECS, entity: Entity) -> Result<(), ComponentError> {
        ecs.containers.remove_container(entity)
    }
}

impl Component {
    pub fn single<C: SingleComponent + NamedComponent>() -> Self {
        Self {
            storage: ComponentStorage::Single,
            id: (
                Default::default(),
                Some(Box::<<C as SingleComponent>::Container>::default()),
            ),
        }
    }

    pub fn array<C: SingleComponent + NamedComponent>(size: u16) -> Self {
        // TODO: Check if size is valid
        Self {
            storage: ComponentStorage::Array(size),
            id: Default::default(),
        }
    }
}
