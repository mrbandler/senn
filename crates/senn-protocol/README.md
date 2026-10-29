# senn-protocol

Types of the secrets backend protocol of [Senn](https://github.com/mrbandler/senn), version 1,
for backends written in Rust.

A secrets backend is a program that turns a secret reference from a herd into the secret. Senn
starts the backend once per secret, writes one request as JSON to its stdin and reads one
response as JSON from its stdout:

```json
{"protocol": 1, "action": "read", "ref": "op://vault/item/field"}
```

```json
{"value": "the secret"}
```

On an error, the backend exits with a non-zero code and writes a message to stderr, never the
secret. The [Secrets backends](https://mrbandler.github.io/senn/secrets-backends.html) chapter
of the book has the full protocol and its rules.

## Usage

```sh
cargo add senn-protocol serde_json
```

A minimal backend that reads each secret from an environment variable with the name of the
reference:

```rust
use std::io::Read;

use senn_protocol::{Action, Request, Response, VERSION};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    let request: Request = serde_json::from_str(&input)?;

    if request.protocol != VERSION {
        return Err(format!("unsupported protocol version {}", request.protocol).into());
    }

    let value = match request.action {
        // Reads the secret from an environment variable named like the reference.
        Action::Read { reference } => {
            std::env::var(&reference).map_err(|_| format!("no secret for {reference}"))?
        },
        _ => return Err("unsupported action".into()),
    };

    serde_json::to_writer(std::io::stdout(), &Response { value })?;
    Ok(())
}
```

`Action` is `#[non_exhaustive]`, because a later protocol version can add actions. Match it with
a `_` arm that rejects the actions your backend doesn't implement.

To use the backend in a herd, set `senn.herd.secrets.backend` to its Nix package, as the book
shows.

## Example

[`senn-backend-onepassword`](https://github.com/mrbandler/senn/tree/main/crates/senn-backend-onepassword)
is the default backend of Senn and a complete example, with error diagnostics and tests.

## Versions

The crate is released together with Senn and has the same version. The protocol has its own
version, `VERSION`, which changes only when the format on the wire changes.

## License

Licensed under `MIT`. See [LICENSE](https://github.com/mrbandler/senn/blob/main/LICENSE).
