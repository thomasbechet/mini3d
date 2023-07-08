use std::collections::BTreeMap;

use glam::{Quat, Vec2, Vec3, Vec4};
use mini3d_derive::{Component, Error, Reflect, Serialize};

use crate::uid::UID;

use super::entity::Entity;

#[derive(Debug, Error)]
pub enum MemberAccessError {
    #[error("Member not found")]
    NotFound,
    #[error("Wrong type")]
    WrongType,
}

#[derive(Debug, Clone, Serialize)]
pub enum Value {
    Null(()),
    Boolean(bool),
    Integer(i32),
    Float(f32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
    Quat(Quat),
    Entity(Entity),
    String(Box<String>),
    Object(Box<Vec<UID>>),
}

#[derive(Default, Clone, Component, Serialize, Reflect)]
pub struct DynamicComponent {
    values: BTreeMap<UID, (String, Value)>,
}

macro_rules! define_type {
    ($value:ident, $vtype:ty, $get:ident, $set:ident, $get_uid:ident, $set_uid:ident) => {
        pub fn $get_uid(&self, uid: UID) -> Result<$vtype> {
            if let Value::$value(value) = self.get_value(uid)? {
                Ok(*value)
            } else {
                Err(anyhow!("Wrong type"))
            }
        }
        pub fn $set_uid(&mut self, uid: UID, key: &str, value: $value) -> Result<()> {
            self.set_value(uid, key, Value::$value(*value))
        }
        pub fn $get(&self, key: &str) -> Result<$vtype> {
            self.$get_uid(UID::new(key))
        }
        pub fn $set(&mut self, key: &str, value: $value) -> Result<()> {
            self.set_value(UID::new(key), key, Value::$value(*value))
        }
    };
}

impl DynamicComponent {
    pub const SEPARATOR: char = '.';

    fn set_value(
        &mut self,
        uid: UID,
        key: &str,
        new_value: Value,
    ) -> Result<(), MemberAccessError> {
        if let Some((_, value)) = self.values.get_mut(&uid) {
            *value = new_value;
        } else {
            // Build hierarchy
            let mut current_child = uid;
            for parent_key in key
                .char_indices()
                .rev()
                .filter(|(_, c)| *c == Self::SEPARATOR)
                .map(|(i, _)| &key[..i])
            {
                let parent_uid = UID::new(parent_key);
                if let Some((_, value)) = self.values.get_mut(&parent_uid) {
                    if let Value::Object(childs) = value {
                        childs.push(current_child);
                    } else {
                        *value = Value::Object(Box::new(vec![current_child]));
                    }
                    break; // Exit hierarchy creation
                } else {
                    self.values.insert(
                        parent_uid,
                        (
                            parent_key.to_string(),
                            Value::Object(Box::new(vec![current_child])),
                        ),
                    );
                    current_child = parent_uid;
                }
            }
            // Insert value
            self.values.insert(uid, (key.to_string(), new_value));
        }
        Ok(())
    }

    pub fn get_value(&self, uid: UID) -> Result<&Value, MemberAccessError> {
        Ok(&self.values.get(&uid).ok_or(MemberAccessError::NotFound)?.1)
    }

    pub fn list_keys_uid(&self, uid: UID) -> Result<impl Iterator<Item = &str>, MemberAccessError> {
        if let (_, Value::Object(childs)) =
            self.values.get(&uid).ok_or(MemberAccessError::NotFound)?
        {
            Ok(childs
                .iter()
                .map(|uid| self.values.get(uid).unwrap().0.as_str()))
        } else {
            Err(MemberAccessError::WrongType)
        }
    }

    pub fn get_length_uid(&self, uid: UID) -> Result<usize, MemberAccessError> {
        if let (_, Value::Object(childs)) = self.values.get(&uid).unwrap() {
            Ok(childs.len())
        } else {
            Err(MemberAccessError::WrongType)
        }
    }

    pub fn clear_key_uid(&mut self, uid: UID, key: &str) -> Result<(), MemberAccessError> {
        if self.values.remove(&uid).is_some() {
            // Find parent
            if let Some(parent_key) = key
                .char_indices()
                .rev()
                .find(|(_, c)| *c == Self::SEPARATOR)
                .map(|(i, _)| &key[..i])
            {
                // Remove child from parent
                let parent_uid = UID::new(parent_key);
                // Remove child from parent
                if let Some((_, value)) = self.values.get_mut(&parent_uid) {
                    if let Value::Object(childs) = value {
                        childs.retain(|child| *child != uid);
                    }
                } else {
                    return Err(MemberAccessError::NotFound);
                }
            }
        }
        Ok(())
    }

    // define_type!(Null, (), get_null, set_null, get_null_uid, set_null_uid);

    // pub fn get_bool_uid(&self, uid: UID) -> Result<bool> {
    //     if let Value::Boolean(value) = self.get_value(uid)? {
    //         Ok(*value)
    //     } else {
    //         Err(anyhow!("Not a boolean"))
    //     }
    // }

    pub fn get_float_uid(&self, uid: UID) -> Result<f32, MemberAccessError> {
        if let Value::Float(value) = self.get_value(uid)? {
            Ok(*value)
        } else {
            Err(MemberAccessError::WrongType)
        }
    }

    pub fn get_integer_uid(&self, uid: UID) -> Result<i32, MemberAccessError> {
        if let Value::Integer(value) = self.get_value(uid)? {
            Ok(*value)
        } else {
            Err(MemberAccessError::WrongType)
        }
    }

    pub fn get_vec2_uid(&self, uid: UID) -> Result<i32, MemberAccessError> {
        if let Value::Integer(value) = self.get_value(uid)? {
            Ok(*value)
        } else {
            Err(MemberAccessError::WrongType)
        }
    }
}

// impl Component for DynamicComponent {
//     fn resolve_entities(&mut self, resolver: &EntityResolver) -> Result<(), ECSError> { Ok(()) }
// }
