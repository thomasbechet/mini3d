#[derive(Clone, Default, Resource, Reflect, Serialize)]
pub struct ComponentType {}

impl ComponentType {
    pub const MAX_NAME_LENGTH: usize = 64;
}
