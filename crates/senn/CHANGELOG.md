# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0](https://github.com/mrbandler/senn/compare/senn-v0.1.0...senn-v0.2.0) - 2026-10-29

### <!-- 1 -->🐛 Bug Fixes

- boot Hetzner servers with UEFI and wait for the tailnet ([#21](https://github.com/mrbandler/senn/pull/21))

### <!-- 4 -->♻️ Refactoring

- [**breaking**] make the error and protocol action enums non_exhaustive ([#22](https://github.com/mrbandler/senn/pull/22))

## [0.1.0](https://github.com/mrbandler/senn/compare/senn-v0.0.0...senn-v0.1.0) - 2026-10-28

### <!-- 0 -->🚀 Features

- *(cli)* add plan, apply and deploy with the 1Password secrets backend ([#13](https://github.com/mrbandler/senn/pull/13))

### <!-- 3 -->📚 Documentation

- refresh the README and the community files ([#17](https://github.com/mrbandler/senn/pull/17))

### <!-- 4 -->♻️ Refactoring

- move the secrets backend protocol into its own crate ([#15](https://github.com/mrbandler/senn/pull/15))

### <!-- 8 -->🔧 Miscellaneous

- drop Windows support ([#16](https://github.com/mrbandler/senn/pull/16))
- start senn from rust-template ([#6](https://github.com/mrbandler/senn/pull/6))
