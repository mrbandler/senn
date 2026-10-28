//! The logic of Senn, a Git-native infrastructure tool for single operators.
//!
//! The `senn` command (crate `senn-cli`) is a thin layer over this crate. Most modules
//! are internal to Senn and have no stable API. [`protocol`] is the exception: it holds
//! the types of the secrets backend protocol, for backends written in Rust.

mod error;
pub mod manifest;
pub mod protocol;
pub mod runner;
pub mod secrets;
pub mod tofu;

pub use error::{Error, Result};
