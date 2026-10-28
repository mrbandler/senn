//! Error types of `senn-backend-onepassword`.

use miette::Diagnostic;
use thiserror::Error;

/// Errors of this backend. None of them contains a secret value.
#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    /// Stdin could not be read.
    #[error("cannot read the request from stdin")]
    #[diagnostic(code(senn_backend_onepassword::stdin))]
    Stdin(#[source] std::io::Error),

    /// The request is not valid JSON, or it has an unknown action or missing fields.
    #[error("invalid request")]
    #[diagnostic(
        code(senn_backend_onepassword::invalid_request),
        help("send one protocol 1 request as JSON, see the chapter \"Secrets backends\" in the Senn book")
    )]
    InvalidRequest(#[source] serde_json::Error),

    /// The request uses a protocol version other than [`senn::protocol::VERSION`].
    #[error("unsupported protocol version {0}")]
    #[diagnostic(
        code(senn_backend_onepassword::unsupported_protocol),
        help("this backend speaks protocol 1; update the backend or Senn so that both match")
    )]
    UnsupportedProtocol(u32),

    /// `op` could not be started, usually because it is not on `PATH`.
    #[error("cannot run `op`")]
    #[diagnostic(
        code(senn_backend_onepassword::op_missing),
        help("install the 1Password CLI, or use the Nix package of this backend, which brings it")
    )]
    OpMissing(#[source] std::io::Error),

    /// `op read` exited non-zero for the reference. `op` printed the cause to stderr.
    #[error("`op read` failed for {0}")]
    #[diagnostic(
        code(senn_backend_onepassword::op_failed),
        help("check that you are signed in (`op whoami`) and that the reference exists; `op` printed the cause above")
    )]
    OpFailed(String),

    /// The secret is binary. Protocol 1 carries UTF-8 text only.
    #[error("the secret at {0} is not valid UTF-8")]
    #[diagnostic(
        code(senn_backend_onepassword::not_utf8),
        help("protocol 1 carries text only; store binary secrets base64-encoded")
    )]
    NotUtf8(String),

    /// The response could not be encoded as JSON.
    #[error("cannot encode the response")]
    #[diagnostic(code(senn_backend_onepassword::encode))]
    Encode(#[source] serde_json::Error),

    /// Stdout could not be written, for example because Senn closed the pipe.
    #[error("cannot write the response to stdout")]
    #[diagnostic(code(senn_backend_onepassword::stdout))]
    Stdout(#[source] std::io::Error),
}
