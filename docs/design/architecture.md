# Architecture

## Pipeline

```
source (.myc)
  -> lexer (myc_lexer) -> tokens with spans
  -> parser (myc_parser) -> AST (myc_ast)
  -> name resolution (myc_resolve) -> resolved AST / HIR (myc_hir)
  -> type checking (myc_types) -> typed HIR
  -> MIR lowering (myc_mir) -> MIR
  -> borrow checking (myc_borrowck)
  -> optimization passes (myc_mir)
  -> codegen (myc_codegen_c) -> C11 source
  -> driver (myc_driver) -> invokes C compiler and linker -> native binary
```

The interpreter (`myc_interp`) consumes HIR/MIR for `myc run --interp`, the REPL, and compile-time evaluation. The doc generator (`myc_doc`) and formatter (`myc_fmt`) operate on the AST/HIR. The LSP (`myc_lsp`) reuses the same pipeline for diagnostics.

## Crate graph

Dependencies point downward only. `myc_cli` is the only binary crate and depends on library crates.

```
myc_cli
  myc_driver
    myc_pkg
    myc_fmt
    myc_doc
    myc_codegen_c
    myc_interp
    myc_borrowck
    myc_mir
    myc_hir
    myc_types
    myc_resolve
    myc_parser
    myc_lexer
    myc_diagnostics
    myc_span
  (myc_lsp is separate, used by editors)
```

## Data flow

Phases communicate through well-defined data types (AST, HIR, MIR). There is no shared mutable global state. Nodes use typed indices and arenas rather than `Rc<RefCell<...>>`.

## Error handling

User errors are diagnostics with stable codes (`E0xxx`, `W0xxx`). Internal failures use `Result` with typed errors. Panics are reserved for compiler bugs and carry a "this is a compiler bug" message.

## Spans and diagnostics

`myc_span` provides `FileId` and `Span` (byte offsets). `myc_diagnostics` renders human-readable and JSON output. Every diagnostic carries a stable code documented in `docs/errors`.

## Backends

The primary backend is C11 (`myc_codegen_c`). The driver locates a C compiler (clang/gcc/cc/cl) and invokes it. Additional targets are listed by `myc targets`.

## Testing

- Unit tests per crate.
- Integration: `tests/run_pass`, `tests/compile_fail`.
- Interpreter differential tests compare interp vs native.