//! The logic of Senn, a Git-native infrastructure tool for single operators.
//!
//! The `senn` command (crate `senn-cli`) is a thin layer over this crate. Its modules are
//! internal to Senn and have no stable API. Backends written in Rust use the crate
//! [`senn-protocol`](https://docs.rs/senn-protocol) instead. The user documentation is the [book](https://mrbandler.github.io/senn/).

pub mod deployer;
mod error;
pub mod herd;
pub mod installer;
pub mod manifest;
pub mod runner;
pub mod secrets;
pub mod tofu;

pub use error::{Error, Result};
