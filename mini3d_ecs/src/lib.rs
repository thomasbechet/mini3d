#![no_std]

#[cfg(test)]
extern crate std;

extern crate alloc;

pub mod bitset;
pub mod component;
pub mod container;
pub mod entity;
pub mod error;
pub mod query;
pub mod system;
pub mod world;
