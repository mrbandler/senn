# CLAUDE.md

senn — Rust workspace (edition 2024).

## Layout

- `crates/senn` — library: typed errors with `thiserror` + `miette::Diagnostic`.
- `crates/senn-cli` — sources of binary `senn` (`clap`, renders errors with `miette` fancy, logs via `tracing`/`RUST_LOG`). Its manifest is the root `Cargo.toml` (package `senn-cli`, like ripgrep), so `nix/`, `template/` and `flake.nix` belong to the CLI package and release with it.
- `crates/senn-backend-onepassword` — 1Password secrets backend, a Nix package only (`publish = false`).
- One version for everything (`version_group` in `release-plz.toml`); one tag `vX.Y.Z` per release.

## Commands

```sh
just            # list tasks
just check      # everything CI runs
just test       # nextest + doc tests
just lint       # all pre-commit hooks
```

## Conventions

- Conventional commits; PR titles too (squash merges drive release-plz).
- Never commit to `main`; work in feature branches.
- Toolchain: `rust-toolchain.toml`. MSRV: `rust-version` in `Cargo.toml`. Hooks: `.pre-commit-config.yaml`.
- Clippy pedantic + nursery, warnings are errors in CI; `unsafe` is forbidden.
- New crates go in `crates/`; set `publish = false` for crates that must not reach crates.io.
