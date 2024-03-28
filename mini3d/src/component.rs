pub mod event;
pub mod hierarchy;
pub mod input;
pub mod texture;
pub mod transform;

#[macro_export]
macro_rules! handle_component {
    ($type:ident, $handle:ident, $name:tt) => {
        pub struct $type(mini3d_db::database::ComponentHandle);

        impl $crate::system::SystemParam for $type {
            fn resolve(db: &mini3d_db::database::Database) -> Self {
                Self(db.find_component(Self::NAME).unwrap())
            }
        }

        impl mini3d_db::database::GetComponentHandle for &$type {
            fn handle(&self) -> mini3d_db::database::ComponentHandle {
                self.0
            }
        }

        impl $type {
            pub const NAME: &'static str = $name;

            pub fn handle(
                &self,
                api: &$crate::api::API,
                e: mini3d_db::entity::Entity,
            ) -> Option<$handle> {
                let h = api.read_handle(e, self);
                if let Some(h) = h.nonnull() {
                    Some(h.into())
                } else {
                    None
                }
            }
        }
    };
}
