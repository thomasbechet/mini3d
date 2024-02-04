use core::any::Any;

use mini3d_derive::Serialize;
use mini3d_utils::string::AsciiArray;

use crate::{
    container::{linear::LinearContainer, Container, SingleContainer},
    entity::Entity,
    error::ComponentError,
};

use super::{ComponentPostCallback, NamedComponent, SingleComponent};

#[derive(Default, Serialize)]
pub struct Identifier {
    ident: AsciiArray<32>,
}

impl NamedComponent for Identifier {
    const IDENT: &'static str = "identifier";
}

impl SingleComponent for Identifier {
    type Container = IdentifierContainer;
}

impl Identifier {
    pub fn new(ident: &str) -> Self {
        Self {
            ident: ident.into(),
        }
    }

    pub fn ident(&self) -> &str {
        self.ident.as_str()
    }
}

pub struct IdentifierContainer(pub(crate) LinearContainer<Identifier>);

impl Default for IdentifierContainer {
    fn default() -> Self {
        Self(LinearContainer::<Identifier>::with_capacity(64))
    }
}

impl Container for IdentifierContainer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn add(
        &mut self,
        _entity: Entity,
        _user: &mut dyn Any,
    ) -> Result<Option<ComponentPostCallback>, ComponentError> {
        todo!()
    }

    fn remove(
        &mut self,
        entity: Entity,
        user: &mut dyn Any,
    ) -> Result<Option<ComponentPostCallback>, ComponentError> {
        SingleContainer::remove(self, entity, user)?;
        Ok(Some(Identifier::on_post_removed))
    }
}

impl SingleContainer<Identifier> for IdentifierContainer {
    fn get(&self, entity: Entity) -> Option<&Identifier> {
        self.0.get(entity)
    }

    fn get_mut(&mut self, entity: Entity) -> Option<&mut Identifier> {
        self.0.get_mut(entity)
    }

    fn add(
        &mut self,
        entity: Entity,
        component: Identifier,
        user: &mut dyn Any,
    ) -> Result<&mut Identifier, ComponentError> {
        if self
            .0
            .iter()
            .any(|(_, ident)| ident.ident == component.ident)
        {
            return Err(ComponentError::DuplicatedEntry);
        }
        SingleContainer::add(&mut self.0, entity, component, user)
    }

    fn remove(&mut self, entity: Entity, user: &mut dyn Any) -> Result<(), ComponentError> {
        SingleContainer::remove(&mut self.0, entity, user)
    }
}

impl IdentifierContainer {
    pub fn find(&self, ident: &str) -> Option<Entity> {
        self.0.iter().find_map(|(entity, identifier)| {
            if identifier.ident == ident {
                Some(entity)
            } else {
                None
            }
        })
    }
}
