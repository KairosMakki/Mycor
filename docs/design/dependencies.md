# Third-party dependencies

Every dependency is listed here with its justification. The rest of the compiler uses the Rust standard library.

- `serde`, `serde_json`: serialization for manifests, lock files, LSP JSON-RPC, and diagnostic JSON output.
- `clap`: CLI argument parsing in `myc_cli`. Keeps the binary thin.
- `toml`: parsing and writing `mycor.toml` manifests.

No other third-party crates are used in the compiler core. Editor extensions and installers may use their own toolchains.
