//! Native CLI support for the stc binary.

pub mod args;
pub mod config;
pub mod fs;
pub mod driver;

#[cfg(test)]
mod args_tests;
#[cfg(test)]
mod config_tests;
#[cfg(test)]
mod fs_tests;
#[cfg(test)]
mod driver_tests;
