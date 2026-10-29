# senn-backend-onepassword

The [1Password](https://1password.com) secrets backend of [Senn](../../README.md), and its
default backend. Senn runs it to turn a secret reference from the herd, such as
`op://infra/tofu/password`, into the secret.

## How it works

Senn starts the backend once per secret and sends one request on stdin:

```json
{"protocol": 1, "action": "read", "ref": "op://infra/tofu/password"}
```

The backend reads the secret with `op read --no-newline <ref>` and writes the response to
stdout:

```json
{"value": "the secret"}
```

The [Secrets backends](https://mrbandler.github.io/senn/secrets-backends.html) chapter of the
book has the full protocol.

## References

A reference is a
[1Password secret reference](https://developer.1password.com/docs/cli/secret-reference-syntax/):
`op://<vault>/<item>/<field>`. The backend passes it to `op` unchanged, so query parameters
work too, for example `op://infra/install-key/private key?ssh-format=openssh` for an SSH key in
OpenSSH format.

## Requirements

The backend needs a signed-in 1Password CLI (`op`):

- **On your machine:** turn on the integration with the desktop app. `op` then asks the app for
  approval, and the app shows the prompt. See
  [Get started with 1Password CLI](https://developer.1password.com/docs/cli/get-started/).
- **In CI:** use a [service account](https://developer.1password.com/docs/service-accounts/)
  and set `OP_SERVICE_ACCOUNT_TOKEN`.

You don't have to install `op`. The Nix package brings it and puts it on `PATH` after any `op`
that is already there. So a system wrapper, for example from the NixOS option
`programs._1password`, takes precedence.

## Install

Every herd uses this backend by default, so it needs no setting. It is a Nix package only and
isn't on crates.io:

```sh
nix build github:mrbandler/senn#senn-backend-onepassword
```

## Errors

Errors go to stderr as [miette](https://github.com/zkat/miette) diagnostics with a code and a
help text, for example `senn_backend_onepassword::op_failed` when `op read` fails. Errors name
the reference, never the secret. `op` writes its own errors to stderr too, so the cause stays
visible.

## Write your own backend

This crate is also the example for other backends: about 60 lines of code without the tests,
built on the protocol types in [`senn-protocol`](https://docs.rs/senn-protocol). Start with
[`src/main.rs`](src/main.rs).
