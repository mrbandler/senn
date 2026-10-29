# senn

The logic of [Senn](https://github.com/mrbandler/senn), Git-native infrastructure for single
operators, built on Nix and OpenTofu.

This crate is internal to Senn. The `senn` command is a thin layer over it, and its API can
change in any release. You probably want one of these instead:

- **The command:** install [`senn-cli`](https://crates.io/crates/senn-cli), or see the
  [install options](https://github.com/mrbandler/senn#install).
- **A secrets backend in Rust:** use [`senn-protocol`](https://crates.io/crates/senn-protocol).

## What is in it

| Module | What it does |
| --- | --- |
| `herd` | Loads a herd and runs `plan`, `apply` and `deploy`. |
| `manifest` | The manifest that the herd flake writes for the CLI. |
| `tofu` | Runs OpenTofu with encrypted state. |
| `installer` | Installs NixOS on new hosts with nixos-anywhere. |
| `deployer` | Pushes host secrets and deploys with deploy-rs. |
| `secrets` | Reads secrets through the secrets backend. |
| `runner` | Runs the external tools, with a fake for tests. |

Errors are typed, with `thiserror` and `miette::Diagnostic`. They name programs, hosts and
secret references, never secret values.

## License

Licensed under `MIT`. See [LICENSE](https://github.com/mrbandler/senn/blob/main/LICENSE).
