#![feature(plugin, custom_derive)]
#![plugin(mockers_macros)]

#[cfg(test)] extern crate mockers;

#[macro_use]
extern crate log;

pub mod daemon;
pub mod client;
