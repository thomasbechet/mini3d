use std::collections::HashMap;

use glam::{IVec2, IVec3, IVec4, Mat4, Quat, Vec2, Vec3, Vec4};

use crate::{
    ecs::{
        container::{AnyComponentContainer, ComponentContainer},
        dynamic::DynamicComponent,
        entity::Entity,
        error::ECSError,
        singleton::{AnyComponentSingleton, ComponentSingleton},
    },
    serialize::Serialize,
    uid::UID,
};

use super::error::RegistryError;

pub struct EntityResolver;

impl EntityResolver {
    pub fn resolve(&self, entity: Entity) -> Result<Entity, ECSError> {
        // TODO: Resolve entity
        Ok(entity)
    }
}

macro_rules! read_write_property {
    ($type:ty, $read:ident, $write:ident) => {
        fn $read(&self, id: ComponentPropertyId) -> Option<$type> {
            let _ = id;
            None
        }
        fn $write(&mut self, id: ComponentPropertyId, value: $type) {
            let _ = id;
            let _ = value;
        }
    };
}

pub trait Component: Serialize + Default + 'static {
    const NAME: &'static str;
    const UID: UID = UID::new(Self::NAME);
    fn resolve_entities(&mut self, resolver: &EntityResolver) -> Result<(), ECSError> {
        let _ = resolver;
        Ok(())
    }
    fn properties() -> &'static [ComponentProperty] {
        &[]
    }
    read_write_property!(bool, read_bool, write_bool);
    read_write_property!(u8, read_u8, write_u8);
    read_write_property!(i32, read_i32, write_i32);
    read_write_property!(u32, read_u32, write_u32);
    read_write_property!(f32, read_f32, write_f32);
    read_write_property!(f64, read_f64, write_f64);
    read_write_property!(Vec2, read_vec2, write_vec2);
    read_write_property!(IVec2, read_ivec2, write_ivec2);
    read_write_property!(Vec3, read_vec3, write_vec3);
    read_write_property!(IVec3, read_ivec3, write_ivec3);
    read_write_property!(Vec4, read_vec4, write_vec4);
    read_write_property!(IVec4, read_ivec4, write_ivec4);
    read_write_property!(Mat4, read_mat4, write_mat4);
    read_write_property!(Quat, read_quat, write_quat);
    read_write_property!(Entity, read_entity, write_entity);
    read_write_property!(UID, read_uid, write_uid);
}

pub(crate) enum ComponentKind {
    Static,
    Dynamic,
}

pub(crate) trait AnyComponentReflection {
    fn create_container(&self) -> Box<dyn AnyComponentContainer>;
    fn create_singleton(&self) -> Box<dyn AnyComponentSingleton>;
    fn properties(&self) -> &[ComponentProperty];
}

pub(crate) struct ComponentReflection<C: Component> {
    _phantom: std::marker::PhantomData<C>,
}

impl<C: Component> AnyComponentReflection for ComponentReflection<C> {
    fn create_container(&self) -> Box<dyn AnyComponentContainer> {
        Box::new(ComponentContainer::<C>::new())
    }

    fn create_singleton(&self) -> Box<dyn AnyComponentSingleton> {
        Box::new(ComponentSingleton::<C>::new(C::default()))
    }

    fn properties(&self) -> &[ComponentProperty] {
        C::properties()
    }
}

pub enum ComponentPropertyAccess {
    Read,
    Write,
    ReadWrite,
}

pub enum ComponentPropertyType {
    Bool,
    I32,
    F32,
    F64,
    Vec2,
    IVec2,
    Vec3,
    IVec3,
    Vec4,
    IVec4,
    Mat4,
    Quat,
    Entity,
    UID,
}

pub struct ComponentPropertyId(u8);

pub struct ComponentProperty {
    pub(crate) name: String,
    pub(crate) access: ComponentPropertyAccess,
    pub(crate) ty: ComponentPropertyType,
    pub(crate) id: ComponentPropertyId,
}

pub(crate) struct ComponentDefinition {
    pub(crate) name: String,
    pub(crate) reflection: Box<dyn AnyComponentReflection>,
    pub(crate) kind: ComponentKind,
}

#[derive(Default)]
pub(crate) struct ComponentRegistry {
    pub(crate) definitions: HashMap<UID, ComponentDefinition>,
}

impl ComponentRegistry {
    fn define(
        &mut self,
        name: &str,
        kind: ComponentKind,
        reflection: Box<dyn AnyComponentReflection>,
    ) -> Result<UID, RegistryError> {
        let uid: UID = name.into();
        if self.definitions.contains_key(&uid) {
            return Err(RegistryError::DuplicatedComponentDefinition {
                name: name.to_string(),
            });
        }
        self.definitions.insert(
            uid,
            ComponentDefinition {
                name: name.to_string(),
                kind,
                reflection,
            },
        );
        Ok(uid)
    }

    pub(crate) fn define_static<C: Component>(&mut self, name: &str) -> Result<UID, RegistryError> {
        let reflection = ComponentReflection::<C> {
            _phantom: std::marker::PhantomData,
        };
        self.define(name, ComponentKind::Static, Box::new(reflection))
    }

    pub(crate) fn define_dynamic(&mut self, name: &str) -> Result<UID, RegistryError> {
        let reflection = ComponentReflection::<DynamicComponent> {
            _phantom: std::marker::PhantomData,
        };
        self.define(name, ComponentKind::Dynamic, Box::new(reflection))
    }

    pub(crate) fn get(&self, uid: UID) -> Option<&ComponentDefinition> {
        self.definitions.get(&uid)
    }
}
