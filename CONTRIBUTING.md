# Contributing to senn

Thanks for your interest! Please read the [Code of Conduct](CODE_OF_CONDUCT.md) first.

## Setup

- **Nix:** `direnv allow` or `devenv shell`.
- **Without Nix:** install rustup, [just](https://github.com/casey/just#installation) and
  [cargo-binstall](https://github.com/cargo-bins/cargo-binstall#installation), then `just setup`.

Both install the git hooks: fast checks on commit, tests and dependency checks on push.

## Workflow

1. Create a branch from `main` (`feat/…`, `fix/…`, `docs/…`). Commits to `main` are blocked.
2. Write [Conventional Commits](https://www.conventionalcommits.org): `feat: …`, `fix(core): …`,
   `feat!: …` for breaking changes.
3. Run `just check` before opening a PR.
4. Open a PR. **The PR title must be a conventional commit too**: PRs are squash-merged and the
   title becomes the commit message that drives versioning and the changelog.

## Code style

- `cargo fmt` and `cargo clippy` (pedantic) must pass; CI treats warnings as errors.
- `unsafe` code is forbidden.
- Libraries return typed errors (`thiserror` + `miette::Diagnostic`).
- Binaries render errors with `miette`.
- Public items need doc comments; examples in docs are tested.
- Features must be additive; CI checks each feature on its own.
