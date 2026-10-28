# Contributing to senn

Thanks for your interest! Please read the [Code of Conduct](CODE_OF_CONDUCT.md) first. If you
use AI tools, read the [AI policy](AI_POLICY.md) too. For questions, use
[Discussions](https://github.com/mrbandler/senn/discussions).

## Setup

Senn needs Nix, so development does too: run `direnv allow` or `devenv shell`. This installs
the toolchain, every tool and the git hooks: fast checks on commit, tests and dependency
checks on push. Linux and macOS are supported; Windows isn't, because Nix doesn't run on it.

Without Nix, `just setup` installs the Rust tools with
[cargo-binstall](https://github.com/cargo-bins/cargo-binstall#installation), but the Nix checks
still need Nix.

`just` lists all tasks, and `just check` runs everything CI runs.

## Layout

| Path | What it is |
| --- | --- |
| `crates/senn-cli/` | The `senn` command. Its manifest is the root `Cargo.toml`. |
| `crates/senn/` | The logic: runner, manifest, secrets client, OpenTofu, installer, deployer. |
| `crates/senn-protocol/` | The types of the secrets backend protocol. |
| `crates/senn-backend-onepassword/` | The 1Password secrets backend, a Nix package only. |
| `nix/` | The flake module, the base aspect, the OpenTofu generator and the presets. |
| `template/` | The herd template for `nix flake init -t github:mrbandler/senn`. |
| `tests/fixture/` | A herd that CI checks with `nix flake check`. |
| `docs/` | The book, published at <https://mrbandler.github.io/senn/>. |

## Workflow

1. Create a branch from `main` (`feat/…`, `fix/…`, `docs/…`). Commits to `main` are blocked.
2. Write [Conventional Commits](https://www.conventionalcommits.org): `feat: …`,
   `fix(nix): …`, `feat!: …` for breaking changes.
3. Run `just check` before opening a pull request.
4. Open a pull request. **The title must be a conventional commit too**: pull requests are
   squash-merged, and the title becomes the commit message that drives versioning and the
   changelog.

## Versions and releases

Senn has one version for everything: the CLI, the libraries, the Nix module and the template.
The commits decide it. Before 1.0, `feat` raises the minor version and `fix` raises the patch
version. A change in `crates/senn-cli/`, `nix/` or `template/` releases a new version; a change
only in `docs/` or `tests/` doesn't.

release-plz keeps a release pull request open. Merging it publishes the crates to crates.io and
sets the tag `vX.Y.Z`; dist then builds the binaries from that tag.

### Adding a crate

A crate that goes to crates.io has to exist there before its first release, because trusted
publishing can't create crates. CI checks this in every pull request and names the command to
run. The maintainer then runs it once:

```sh
just reserve-crate <name>
```

It publishes a placeholder 0.0.0 and sets up trusted publishing for the new crate. Add the crate
to `version_group = "senn"` in `release-plz.toml`, so it releases in lockstep with the others.

## Code style

- `cargo fmt` and `cargo clippy` (pedantic) must pass; CI treats warnings as errors.
- `unsafe` code is forbidden.
- No free functions except `main`: behavior lives in methods and types.
- A blank line between struct fields and between enum variants.
- Libraries return typed errors (`thiserror` + `miette::Diagnostic`); binaries render them with
  `miette`.
- Public items need doc comments; examples in docs are tested.
- Features must be additive; CI checks each feature on its own.
