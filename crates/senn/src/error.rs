//! Error types of `senn`.

use miette::Diagnostic;
use thiserror::Error;

/// Errors returned by this crate.
///
/// Each variant is a [`Diagnostic`], so the CLI can render it with a code and a help text.
/// Variants are added with the modules that return them.
#[derive(Debug, Error, Diagnostic)]
pub enum Error {}

/// Result type of this crate, with [`enum@Error`] as the default error type.
pub type Result<T, E = Error> = std::result::Result<T, E>;
