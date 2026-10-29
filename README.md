<p align="center">
  <img src="docs/src/images/senn.svg" alt="Senn, a cartoon alpine herdsman" width="160">
</p>

<h1 align="center">Senn</h1>

<p align="center">
  Git-native infrastructure for single operators, built on Nix and OpenTofu, driven by one CLI.
</p>

<p align="center">
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/built_with-Rust-dea584?logo=rust&logoColor=white" alt="Built with Rust"></a>
  <a href="https://builtwithnix.org"><img src="https://img.shields.io/badge/built_with-Nix-5277c3?logo=nixos&logoColor=white" alt="Built with Nix"></a>
</p>

<p align="center">
  <a href="https://github.com/mrbandler/senn/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/mrbandler/senn/ci.yml?branch=main&label=CI&logo=github" alt="CI"></a>
  <a href="https://crates.io/crates/senn-cli"><img src="https://img.shields.io/crates/v/senn-cli.svg?logo=rust" alt="crates.io"></a>
  <a href="https://docs.rs/senn-protocol"><img src="https://img.shields.io/docsrs/senn-protocol?label=docs.rs%20senn-protocol&logo=docs.rs" alt="docs.rs: senn-protocol"></a>
  <a href="https://flakehub.com/flake/mrbandler/senn"><img src="https://img.shields.io/endpoint?url=https://flakehub.com/f/mrbandler/senn/badge" alt="FlakeHub"></a>
  <a href="https://mrbandler.github.io/senn/"><img src="https://img.shields.io/badge/docs-book-blue.svg?logo=mdbook" alt="Book"></a>
  <a href="LICENSE"><img src="https://img.shields.io/crates/l/senn-cli.svg" alt="License: MIT"></a>
</p>

> [!WARNING]
> Senn is at an early stage and under heavy development. Interfaces change without notice.
> Don't use it for anything you rely on.

## What it does

You describe each server in a single [Nix](https://nixos.org) host definition: where it runs,
what runs on it and what is public. Then `senn apply`:

1. provisions it with [OpenTofu](https://opentofu.org), using any provider,
2. installs [NixOS](https://nixos.org) on it with
   [nixos-anywhere](https://github.com/nix-community/nixos-anywhere),
3. joins it to your [Tailscale](https://tailscale.com) tailnet,
4. pushes its secrets from your secrets backend and configures it with
   [deploy-rs](https://github.com/serokell/deploy-rs).

If you change a definition and run `senn apply` again, only the changes are applied. The CLI
works the same on your laptop and in CI.

- **Provisioning is part of the host.** It doesn't need a separate tool.
- **Works with any provider.** Bring your own OpenTofu resources or modules. Presets exist for
  common providers.
- **Private by default.** Hosts are reached only over the tailnet. Anything public is declared
  explicitly and goes through review.
- **Pluggable secrets.** Git holds references only. 1Password is the first backend, and any
  program that speaks the [backend protocol](https://mrbandler.github.io/senn/secrets-backends.html)
  can be one.

Senn reuses these tools instead of replacing them. Hosts start from
[srvos](https://github.com/nix-community/srvos) server defaults, disks are laid out with
[disko](https://github.com/nix-community/disko), and host definitions are
[den](https://github.com/denful/den) aspects.

## Requirements

- Linux or macOS with [Nix](https://nixos.org/download/) and flakes enabled.
- A [Tailscale](https://tailscale.com) tailnet.
- A provider account, for example Hetzner Cloud.
- For the default secrets backend: [1Password](https://1password.com) and its CLI `op`.

## Install

Every method installs the command `senn`.

### Nix

```sh
nix profile install github:mrbandler/senn
```

To try it without installing:

```sh
nix run github:mrbandler/senn -- --help
```

### Homebrew

```sh
brew install mrbandler/tap/senn
```

### Cargo

```sh
cargo install senn-cli
```

### Shell installer

```sh
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/mrbandler/senn/releases/latest/download/senn-cli-installer.sh | sh
```

## Quick start

```sh
mkdir infra && cd infra
nix flake init -t github:mrbandler/senn
```

This creates a herd: `flake.nix`, `herd/herd.nix` for the settings of the whole herd and one
host per file in `herd/hosts/`. Fill in your keys and secret references, then:

```sh
senn plan     # show what would change, change nothing
senn apply    # create, install and configure every host
senn deploy   # push secrets and deploy NixOS configurations, without OpenTofu
```

The [book](https://mrbandler.github.io/senn/) has the details.

## The name

A [Senn](https://en.wikipedia.org/wiki/Senn_(occupation)) is the alpine herdsman who takes the herd up to the mountain pasture for the summer and
looks after it there. Your hosts are cattle, not pets, and Senn tends them.

## Contributing

Contributions are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) for the setup and the
workflow, and the [AI policy](AI_POLICY.md) if you use AI tools. For questions, use
[Discussions](https://github.com/mrbandler/senn/discussions).

## How AI is used here

I'm not a vibe coder. I use AI to speed up development, mostly for ideation and boilerplate.
I read all of the code, and every change has to meet my quality bar before it lands.

## License

Licensed under `MIT`. See [LICENSE](LICENSE).
