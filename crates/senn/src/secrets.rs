//! Client side of the secrets backend protocol (see [`crate::protocol`]).

use crate::error::{Error, Result};
use crate::protocol::{Request, Response};
use crate::runner::{Cmd, Runner, run};

/// Reads secrets through a backend program.
pub struct Secrets<'a> {
    runner: &'a dyn Runner,
    program: &'a str,
}

impl<'a> Secrets<'a> {
    /// A client for the backend at `program` (the `backend` path of the manifest).
    #[must_use]
    pub const fn new(runner: &'a dyn Runner, program: &'a str) -> Self {
        Self { runner, program }
    }

    /// Reads the secret at `reference`.
    ///
    /// The backend runs once per call. Its stderr goes to the terminal, so prompts and
    /// errors from the backend stay visible.
    ///
    /// # Errors
    ///
    /// The errors of [`run`] if the backend can't start or fails, and
    /// [`Error::Protocol`] if its response is not valid. The error names the reference,
    /// never the output, because the output can contain the secret.
    pub fn read(&self, reference: &str) -> Result<String> {
        let request = serde_json::to_string(&Request::read(reference))?;
        let out = run(self.runner, &Cmd::new(self.program).stdin(request).capture())?;
        let response: Response = serde_json::from_str(&out).map_err(|_| {
            Error::Protocol(format!(
                "the secrets backend returned an invalid response for {reference}"
            ))
        })?;

        Ok(response.value)
    }
}

/// Checks that `name` is safe as a secret name.
///
/// Secret names become file names in `/var/lib/senn/secrets` on hosts and appear in
/// remote shell commands, so only lowercase ASCII letters, digits and dashes are allowed.
///
/// # Errors
///
/// [`Error::InvalidSecretName`] if `name` is empty or has any other character.
pub fn validate_name(name: &str) -> Result<()> {
    let ok = !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    if ok {
        Ok(())
    } else {
        Err(Error::InvalidSecretName(name.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::fake::Fake;

    #[test]
    fn reads_through_the_backend_protocol() {
        let fake = Fake::new(|cmd| {
            let req: serde_json::Value = serde_json::from_str(cmd.stdin.as_deref().unwrap()).unwrap();
            assert_eq!(req["protocol"], 1);
            assert_eq!(req["action"], "read");
            assert_eq!(req["ref"], "op://v/i/f");
            Some(r#"{"value":"s3cret\n"}"#.to_owned())
        });

        let value = Secrets::new(&fake, "backend").read("op://v/i/f").unwrap();

        assert_eq!(value, "s3cret\n");
        assert_eq!(fake.programs(), ["backend"]);
        assert!(fake.calls()[0].capture);
    }

    #[test]
    fn invalid_json_names_the_reference_not_the_output() {
        let fake = Fake::new(|_| Some("leaked-value".to_owned()));
        let err = Secrets::new(&fake, "backend")
            .read("op://v/i/f")
            .unwrap_err()
            .to_string();
        assert!(err.contains("op://v/i/f"), "{err}");
        assert!(!err.contains("leaked-value"), "{err}");
    }

    #[test]
    fn backend_failure_is_an_error() {
        let fake = Fake::new(|_| None);
        let result = Secrets::new(&fake, "backend").read("x");
        assert!(matches!(result, Err(Error::Failed { .. })), "{result:?}");
    }

    #[test]
    fn secret_names_are_restricted() {
        assert!(validate_name("tailscale-authkey").is_ok());
        for bad in ["", "../x", "a b", "A", "x;rm", "a/b"] {
            assert!(validate_name(bad).is_err(), "{bad} accepted");
        }
    }
}
