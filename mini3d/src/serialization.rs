use std::{error::Error, fmt::Display};

use crate::registry::component::Component;

#[derive(Debug)]
pub enum SerializeError {
    Unsupported,
}

impl Error for SerializeError {}

impl Display for SerializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SerializeError::Unsupported => write!(f, "Unsupported"),
        }
    }
}

#[derive(Debug)]
pub enum DeserializeError {
    Unsupported,
}

impl Error for DeserializeError {}

impl Display for DeserializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeserializeError::Unsupported => write!(f, "Unsupported"),
        }
    }
}

pub trait BinarySerializer {
    fn serialize_f32(&mut self, value: f32) -> Result<(), SerializeError>;
    fn serialize_u32(&mut self, value: u32) -> Result<(), SerializeError>;
    fn serialize_bool(&mut self, value: bool) -> Result<(), SerializeError>;
    fn serialize_str(&mut self, value: &str) -> Result<(), SerializeError>;
}

pub trait ComponentSerializer {
    type Component: Component;
    fn serialize(&self, value: &Self::Component, ser: &mut dyn BinarySerializer) -> Result<(), SerializeError>;
}

struct MyStruct {
    a: f32,
    b: u32,
}

// struct MyStructSerializer {
//     version: u32,
// }

// impl ComponentSerializer for MyStructSerializer {
//     type Component = MyStruct;

//     fn serialize(&self, value: &Self::Component, ser: &mut dyn BinarySerializer) -> Result<(), SerializeError> {
//         ser.serialize_f32(value.a)?;
//         if self.version > 1 {
//             ser.serialize_u32(value.b)?;
//         }
//         Ok(())
//     }
// }