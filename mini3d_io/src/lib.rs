#![no_std]

pub mod disk;
pub mod io;
pub mod provider;

extern crate alloc;

#[cfg(test)]
extern crate std;
