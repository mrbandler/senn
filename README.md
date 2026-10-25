# senn

Git-native infrastructure for single operators, built on Nix and OpenTofu, driven by one CLI.

> [!WARNING]
> Senn is at an early stage and under heavy development. Nothing is released yet, and
> interfaces change without notice. Don't use it for anything you rely on.

## What it does

You describe each server in a single Nix host definition: where it runs, what runs on it and
what is public. Then `senn apply`:

1. provisions it with OpenTofu, using any provider,
2. installs NixOS on it with nixos-anywhere,
3. joins it to your Tailscale tailnet,
4. pushes its secrets from your secrets backend and configures it with deploy-rs.

If you change a definition and run `senn apply` again, only the changes are applied. The CLI
works the same on your laptop and in CI.

- **Provisioning is part of the host.** It doesn't need a separate tool.
- **Works with any provider.** Bring your own OpenTofu resources or modules. Presets exist for
  common providers.
- **Private by default.** Hosts are reached only over the tailnet. Anything public is declared
  explicitly and goes through review.
- **Pluggable secrets.** Git holds references only. 1Password is the first backend.

Senn reuses OpenTofu, nixos-anywhere, deploy-rs, srvos and Tailscale instead of replacing them.

## The name

A Senn is the alpine herdsman who takes the herd up to the mountain pasture for the summer and
looks after it there. Your hosts are cattle, not pets, and Senn tends them.

## Development

With Nix: `direnv allow` (or `devenv shell`). Everything, including the git hooks, is set up.

`just` lists all tasks, and `just check` runs everything CI runs.

## License

Licensed under `MIT`. See [LICENSE](LICENSE).
