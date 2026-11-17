# Contributing to Mycor

Thanks for your interest in contributing to Mycor! This document outlines how to
get started.

## Development setup

- Rust 1.91+ (see `rust-toolchain.toml`)
- A C compiler (`clang` recommended; `gcc`/`cc`/`cl` also work)

Build the compiler:

```sh
cargo build
```

Run the test suite:

```sh
cargo test --workspace
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
```

## Workflow

1. Fork the repository and create a feature branch.
2. Make your changes. Keep commits focused and well-described.
3. Add or update tests when fixing bugs or adding features.
4. Run `cargo fmt` and `cargo clippy` before submitting.
5. Open a pull request against `main`.

## Code style

- Follow `rustfmt` defaults.
- Prefer small, reviewable changes.
- Document public APIs with doc comments.

## Reporting issues

Please use the issue templates in `.github/ISSUE_TEMPLATE`. For security
issues, see `SECURITY.md`.

## License

By contributing, you agree that your contributions will be licensed under the
project's MIT OR Apache-2.0 license.