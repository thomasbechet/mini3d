use mini3d_derive::Error;

use crate::{reflection::Reflect, serialize::Serialize};

pub struct ReferenceResolver {}

#[derive(Error)]
pub enum ReferenceError {
    #[error("Failed to resolve reference")]
    ResolveError,
}

pub trait StaticDataType: Default + Serialize + Reflect + 'static {
    fn resolve_references(
        &mut self,
        resolver: &mut ReferenceResolver,
    ) -> Result<(), ReferenceError> {
        Ok(())
    }
}
