#![no_std]

pub mod disk;
pub mod ecs;
pub mod input;
pub mod io;
pub mod logger;
pub mod math;
pub mod network;
pub mod physics;
pub mod platform;
pub mod recorder;
pub mod reflection;
pub mod renderer;
pub mod script;
pub mod serialize;
pub mod simulation;
pub mod utils;

#[macro_use]
extern crate alloc;

#[cfg(test)]
extern crate std;

extern crate self as mini3d_core;
