# Secrets backends

Git holds only references to secrets, such as `op://senn-infra/tailscale/credential`.
A secrets backend turns a reference into the secret. Senn runs the backend each time it
needs a secret, for example before it pushes host secrets or runs OpenTofu.

A backend is any program that speaks the protocol on this page. Senn ships one backend,
`senn-backend-onepassword`, and uses it by default.

## Use a different backend

Set `senn.herd.secrets.backend` to a function from `pkgs` to a package:

```nix
# A backend from another flake
senn.herd.secrets.backend = pkgs: inputs.my-backend.packages.${pkgs.system}.default;

# A backend defined in the herd repo
senn.herd.secrets.backend = pkgs: pkgs.callPackage ./my-backend.nix { };
```

Senn runs the main program of the package (`lib.getExe`). Set `meta.mainProgram`, or name
the binary like the package.

## Protocol, version 1

Senn starts the backend once per secret. It writes one JSON request to stdin and closes
stdin. The backend writes one JSON response to stdout and exits.

### Request

```json
{"protocol": 1, "action": "read", "ref": "op://vault/item/field"}
```

| Field      | Type    | Meaning                                                      |
| ---------- | ------- | ------------------------------------------------------------ |
| `protocol` | integer | Protocol version. Version 1 is the only version.             |
| `action`   | string  | What to do. Version 1 has one action: `read`.                |
| `ref`      | string  | The secret reference, in the format of the backend.          |

### Response

On success, exit with code 0 and write:

```json
{"value": "the secret"}
```

`value` is the secret as stored, as a JSON string. Don't add or remove characters,
trailing newlines included. Secrets must be valid UTF-8.

### Errors

On any error, exit with a non-zero code and write a message to stderr. Senn treats a
non-zero exit as a failure and ignores stdout.

A backend must reject a request when:

- `protocol` is a version it doesn't know,
- `action` is an action it doesn't know,
- the secret doesn't exist or can't be read.

### Rules

- Write secret values only to stdout, and only in the response. Never write them to
  stderr, to log files or to disk. Senn can show stderr to the operator.
- Error messages can name the reference. References aren't secret.
- Don't prompt on stdin. Stdin carries the request. A backend can still open its own
  prompt, for example the 1Password desktop app asks for approval.

## Types for Rust backends

The `senn` crate has the protocol types in `senn::protocol`: `Request`, `Action`,
`Response` and `VERSION`. `senn-backend-onepassword` in the Senn repo is a complete
example in about 60 lines, tests not counted.
