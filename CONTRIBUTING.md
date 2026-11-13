# Contributing to Mycor

## Where does my change go?

- Lexer changes: `crates/myc_lexer`
- Parser/syntax: `crates/myc_parser`, `crates/myc_ast`
- Name resolution: `crates/myc_resolve`
- Types: `crates/myc_types`
- HIR: `crates/myc_hir`
- MIR and optimizations: `crates/myc_mir`
- Borrow checker: `crates/myc_borrowck`
- Code generation: `crates/myc_codegen_c`
- Interpreter: `crates/myc_interp`
- Driver/pipeline: `crates/myc_driver`
- Package manager: `crates/myc_pkg`
- Formatter: `crates/myc_fmt`
- Doc generator: `crates/myc_doc`
- LSP: `crates/myc_lsp`
- CLI: `crates/myc_cli`

## Build

```
cargo build
cargo test --workspace
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
```

## Documentation

When changing behavior, update:
- `README.md` and relevant `docs/book` chapters
- `docs/spec/syntax.md` and `docs/spec/grammar.ebnf`
- Error docs in `docs/errors` (keep in sync with `myc_diagnostics::codes`)
- `ROADMAP.md` and `CHANGELOG.md`

## Commit style

Short, imperative, specific. Example: "Fix borrow check for reborrowed slices".
