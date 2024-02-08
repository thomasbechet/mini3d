use core::any::Any;

use mini3d_derive::Serialize;
use mini3d_utils::string::AsciiArray;

use crate::{
    container::{linear::LinearContainer, Container, NativeContainer},
    entity::Entity,
    error::ComponentError,
};

use super::{ComponentPostCallback, NamedComponent, NativeComponent};

#[derive(Default, Serialize)]
pub struct Identifier {
    ident: AsciiArray<32>,
}

impl NamedComponent for Identifier {
    const IDENT: &'static str = "identifier";
}

impl NativeComponent for Identifier {
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

impl<Context> Container<Context> for IdentifierContainer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn add(
        &mut self,
        _entity: Entity,
        _ctx: &mut Context,
    ) -> Result<Option<ComponentPostCallback<Context>>, ComponentError> {
        todo!()
    }

    fn remove(
        &mut self,
        entity: Entity,
        ctx: &mut Context,
    ) -> Result<Option<ComponentPostCallback<Context>>, ComponentError> {
        NativeContainer::remove(self, entity, ctx)?;
        Ok(Some(Identifier::on_post_removed))
    }
}

impl NativeContainer<Identifier> for IdentifierContainer {
    fn get(&self, entity: Entity) -> Option<&Identifier> {
        self.0.get(entity)
    }

    fn get_mut(&mut self, entity: Entity) -> Option<&mut Identifier> {
        self.0.get_mut(entity)
    }

    fn add<Context>(
        &mut self,
        entity: Entity,
        component: Identifier,
        ctx: &mut Context,
    ) -> Result<&mut Identifier, ComponentError> {
        if self
            .0
            .iter()
            .any(|(_, ident)| ident.ident == component.ident)
        {
            return Err(ComponentError::DuplicatedEntry);
        }
        NativeContainer::add(&mut self.0, entity, component, ctx)
    }

    fn remove<Context>(&mut self, entity: Entity, ctx: &mut Context) -> Result<(), ComponentError> {
        NativeContainer::remove(&mut self.0, entity, ctx)
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
