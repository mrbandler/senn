//! 1Password secrets backend of Senn, protocol 1 (see [`senn::protocol`]).
//!
//! Reads one request from stdin, fetches the secret with `op read` and writes the
//! response to stdout. Errors go to stderr as miette diagnostics and never contain a value.
//!
//! `op` is the 1Password CLI. The Nix package puts it on `PATH` after any `op` that is
//! already there, so a system wrapper (for example from NixOS `programs._1password`) wins.

mod error;

use std::io::Write;
use std::process::{Command, Stdio};

use senn::protocol::{Action, Request, Response, VERSION};

use crate::error::Error;

/// Answers the request on stdin and writes the response to stdout.
fn main() -> miette::Result<()> {
    let input = std::io::read_to_string(std::io::stdin()).map_err(Error::Stdin)?;
    let output = handle(&input, op_read)?;

    std::io::stdout()
        .write_all(output.as_bytes())
        .map_err(Error::Stdout)?;

    Ok(())
}

/// Answers one protocol 1 request and returns the response as JSON.
///
/// `read` fetches a secret by reference. It is a parameter so tests can pass a fake
/// instead of running `op`.
fn handle(input: &str, read: impl FnOnce(&str) -> Result<String, Error>) -> Result<String, Error> {
    let request: Request = serde_json::from_str(input).map_err(Error::InvalidRequest)?;

    if request.protocol != VERSION {
        return Err(Error::UnsupportedProtocol(request.protocol));
    }

    match request.action {
        Action::Read { reference } => {
            let value = read(&reference)?;
            serde_json::to_string(&Response { value }).map_err(Error::Encode)
        },
    }
}

/// Reads the secret at `reference` with `op read --no-newline`.
///
/// `op` gets no stdin, because stdin carries the request. Its stderr goes to the
/// operator, so sign-in prompts and errors from `op` stay visible.
fn op_read(reference: &str) -> Result<String, Error> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_returns_the_exact_value() {
        let output = handle(r#"{"protocol":1,"action":"read","ref":"op://v/i/f"}"#, |r| {
            assert_eq!(r, "op://v/i/f");
            Ok("line1\nline2\n".to_owned())
        })
        .unwrap();
        let response: Response = serde_json::from_str(&output).unwrap();
        assert_eq!(response.value, "line1\nline2\n");
    }

    #[test]
    fn unknown_protocol_is_rejected() {
        let err = handle(r#"{"protocol":2,"action":"read","ref":"x"}"#, |_| unreachable!()).unwrap_err();
        assert!(matches!(err, Error::UnsupportedProtocol(2)), "{err:?}");
    }

    #[test]
    fn unknown_action_is_rejected() {
        let err = handle(r#"{"protocol":1,"action":"write","ref":"x"}"#, |_| unreachable!()).unwrap_err();
        assert!(matches!(err, Error::InvalidRequest(_)), "{err:?}");
    }

    #[test]
    fn read_failure_is_passed_on() {
        let err = handle(r#"{"protocol":1,"action":"read","ref":"x"}"#, |r| {
            Err(Error::OpFailed(r.to_owned()))
        })
        .unwrap_err();
        assert!(matches!(err, Error::OpFailed(ref r) if r == "x"), "{err:?}");
    }
}
