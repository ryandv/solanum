#![feature(plugin, use_extern_macros)]

#[cfg(test)]
extern crate mockers;

#[macro_use]
extern crate log;

pub mod daemon;
pub mod client;
