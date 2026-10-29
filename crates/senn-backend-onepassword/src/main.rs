//! 1Password secrets backend of Senn, protocol 1 (see [`senn_protocol`]).
//!
//! Reads one request from stdin, fetches the secret with `op read` and writes the
//! response to stdout. Errors go to stderr as miette diagnostics and never contain a value.
//!
//! `op` is the 1Password CLI. The Nix package puts it on `PATH` after any `op` that is
//! already there, so a system wrapper (for example from NixOS `programs._1password`) wins.

mod error;

use std::io::Write;
use std::process::{Command, Stdio};

use senn_protocol::{Action, Request, Response, VERSION};

use crate::error::Error;

/// Answers the request on stdin and writes the response to stdout.
fn main() -> miette::Result<()> {
    let input = std::io::read_to_string(std::io::stdin()).map_err(Error::Stdin)?;
    let output = OnePassword.answer(&input)?;

    std::io::stdout()
        .write_all(output.as_bytes())
        .map_err(Error::Stdout)?;

    Ok(())
}

/// A store of secrets that answers protocol 1 requests. Tests implement it with a fake.
trait Vault {
    /// Reads the secret at `reference`.
    fn read(&self, reference: &str) -> Result<String, Error>;

    /// Answers one protocol 1 request and returns the response as JSON.
    fn answer(&self, input: &str) -> Result<String, Error> {
        let request: Request = serde_json::from_str(input).map_err(Error::InvalidRequest)?;

        if request.protocol != VERSION {
            return Err(Error::UnsupportedProtocol(request.protocol));
        }

        match request.action {
            Action::Read { reference } => {
                let value = self.read(&reference)?;
                serde_json::to_string(&Response { value }).map_err(Error::Encode)
            },
            // An action of a newer protocol; parsing rejects unknown names already.
            _ => Err(Error::UnsupportedAction),
        }
    }
}

/// 1Password, read through the `op` CLI.
struct OnePassword;

impl Vault for OnePassword {
    /// Reads the secret with `op read --no-newline`.
    ///
    /// `op` gets no stdin, because stdin carries the request. Its stderr goes to the
    /// operator, so sign-in prompts and errors from `op` stay visible.
    fn read(&self, reference: &str) -> Result<String, Error> {
        let output = Command::new("op")
            .args(["read", "--no-newline", reference])
            .stdin(Stdio::null())
            .stderr(Stdio::inherit())
            .output()
            .map_err(Error::OpMissing)?;

        if !output.status.success() {
            return Err(Error::OpFailed(reference.to_owned()));
        }

        String::from_utf8(output.stdout).map_err(|_| Error::NotUtf8(reference.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A [`Vault`] that answers with a closure instead of running `op`.
    struct Fake<F>(F);

    impl<F: Fn(&str) -> Result<String, Error>> Vault for Fake<F> {
        fn read(&self, reference: &str) -> Result<String, Error> {
            (self.0)(reference)
        }
    }

    #[test]
    fn read_returns_the_exact_value() {
        let vault = Fake(|r: &str| {
            assert_eq!(r, "op://v/i/f");
            Ok("line1\nline2\n".to_owned())
        });

        let output = vault
            .answer(r#"{"protocol":1,"action":"read","ref":"op://v/i/f"}"#)
            .unwrap();

        let response: Response = serde_json::from_str(&output).unwrap();
        assert_eq!(response.value, "line1\nline2\n");
    }

    #[test]
    fn unknown_protocol_is_rejected() {
        let vault = Fake(|_: &str| unreachable!());
        let err = vault
            .answer(r#"{"protocol":2,"action":"read","ref":"x"}"#)
            .unwrap_err();
        assert!(matches!(err, Error::UnsupportedProtocol(2)), "{err:?}");
    }

    #[test]
    fn unknown_action_is_rejected() {
        let vault = Fake(|_: &str| unreachable!());
        let err = vault
            .answer(r#"{"protocol":1,"action":"write","ref":"x"}"#)
            .unwrap_err();
        assert!(matches!(err, Error::InvalidRequest(_)), "{err:?}");
    }

    #[test]
    fn read_failure_is_passed_on() {
        let vault = Fake(|r: &str| Err(Error::OpFailed(r.to_owned())));
        let err = vault
            .answer(r#"{"protocol":1,"action":"read","ref":"x"}"#)
            .unwrap_err();
        assert!(matches!(err, Error::OpFailed(ref r) if r == "x"), "{err:?}");
    }
}
