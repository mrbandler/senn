//! The `senn` library.

mod error;

pub use error::{Error, Result};

/// Builds a greeting for `name`.
///
/// # Errors
///
/// Returns [`Error::EmptyName`] if `name` is empty or whitespace only.
///
/// # Examples
///
/// ```
/// assert_eq!(senn::greet("world")?, "Hello, world!");
/// # Ok::<(), senn::Error>(())
/// ```
pub fn greet(name: &str) -> Result<String> {
    let name = name.trim();
    if name.is_empty() {
        return Err(Error::EmptyName);
    }
    Ok(format!("Hello, {name}!"))
}
