# Tooling

## Compiler (myc)

See README command table. Key commands:

```sh
myc new <name>      # new project
myc run             # compile + run main.myc
myc check           # type-check only
myc fmt             # format
myc test            # run tests
myc doc             # generate docs
myc emit-c <file>   # print C output
myc explain <code>  # error docs
myc clean           # remove target/
```

## Formatter

`myc fmt` reformats source according to the canonical style. Run in CI with `cargo fmt --check` for Rust, `myc fmt` for Mycor.

## Documentation generator

`myc doc` summarizes the current module into `target/doc/index.html`. Doc comments (`///`) are collected by `myc_doc`.

## Language server

A basic LSP server (`myc_lsp`) provides diagnostics and navigation. Editor integration is in progress.

## VS Code

A TextMate grammar is provided at `editors/vscode/syntaxes/mycor.tmLanguage.json`.

## Targets

```sh
myc targets
```

Supported: `x86_64-pc-windows-msvc`, `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`.

## Testing

`#[test]` functions are discovered by `myc test`. Compile-fail tests live in `tests/compile_fail/`. Run with `cargo test --workspace`.

## Benchmarks

`myc bench` is planned. Benchmarks (e.g. `benches/nbody.myc`) will be wired in.