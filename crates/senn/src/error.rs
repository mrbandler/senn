//! Error types of `senn`.

use miette::Diagnostic;
use thiserror::Error;

/// Errors returned by this crate.
///
/// Each variant is a [`Diagnostic`], so the CLI can render it with a code and a help text.
/// Messages name programs, hosts and secret references, never secret values.
#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    /// A program could not be started, usually because it is not on `PATH`.
    #[error("cannot start `{program}`")]
    #[diagnostic(
        code(senn::spawn),
        help("Senn takes its tools from the herd flake; run it inside the herd repository")
    )]
    Spawn {
        /// The program Senn tried to start.
        program: String,
        /// Why it could not be started.
        #[source]
        source: std::io::Error,
    },

    /// A program exited non-zero. Its own error output is already on the terminal.
    #[error("`{program}` failed (exit code {code:?})")]
    #[diagnostic(code(senn::failed), help("the output of `{program}` above shows the cause"))]
    Failed {
        /// The program that failed.
        program: String,
        /// Its exit code, `None` if a signal ended it.
        code: Option<i32>,
    },

    /// A program wrote output that Senn cannot use.
    #[error("{0}")]
    #[diagnostic(code(senn::protocol))]
    Protocol(String),

    /// The herd has no host with this name.
    #[error("unknown host `{0}`")]
    #[diagnostic(
        code(senn::unknown_host),
        help("hosts are the files in herd/hosts/; the name is the den host name")
    )]
    UnknownHost(String),

    /// A secret name is not safe to use as a file name on a host.
    #[error("invalid secret name `{0}`")]
    #[diagnostic(
        code(senn::invalid_secret_name),
        help("use lowercase letters, digits and dashes, for example `tailscale-authkey`")
    )]
    InvalidSecretName(String),

    /// Reading or writing a file or pipe failed.
    #[error(transparent)]
    #[diagnostic(code(senn::io))]
    Io(#[from] std::io::Error),

    /// JSON from Nix or a tool could not be parsed.
    #[error("invalid JSON")]
    #[diagnostic(code(senn::json))]
    Json(#[from] serde_json::Error),
}

/// Result type of this crate, with [`enum@Error`] as the default error type.
pub type Result<T, E = Error> = std::result::Result<T, E>;
