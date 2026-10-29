# Introduction

<img src="images/senn.svg" alt="Senn, a cartoon alpine herdsman" width="128" align="right">

Senn is Git-native infrastructure for single operators, built on Nix and OpenTofu and driven by
one CLI.

You describe each server in one Nix host definition: where it runs, what runs on it and what
is public. `senn apply` then provisions it with OpenTofu, installs NixOS on it with
nixos-anywhere, joins it to your Tailscale tailnet, pushes its secrets and configures it with
deploy-rs. Running `senn apply` again applies only what changed.

> **Early stage.** Senn is under heavy development, and interfaces change without notice.

## A herd

A herd is one infrastructure project: its hosts, its secrets vault, its tailnet and its state.
Start one with:

```sh
nix flake init -t 'https://flakehub.com/f/mrbandler/senn/0.3.*'
```

That creates `flake.nix`, `herd/herd.nix` for the settings of the whole herd and one file per
host in `herd/hosts/`.

## Chapters

- [Secrets backends](secrets-backends.md): how Senn reads secrets, and how to write your own
  backend.

The source is on [GitHub](https://github.com/mrbandler/senn).
