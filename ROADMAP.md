# Roadmap

## Done in 0.1.0

- Workspace skeleton and CI
- Lexer, parser, AST, diagnostics
- Name resolution and type checker (basic)
- Borrow checker (use-after-move heuristic)
- C backend and driver (Windows/Linux/macOS via clang)
- Interpreter and differential tests
- Standard library (partial), formatter, doc generator (basic)
- Error catalog (85+ entries)

## Done in 0.2.0

- Spawn expression and concurrency syntax
- LSP server (basic)
- VS Code TextMate grammar
- Doc and clean commands
- Standard library scaffolding
- Version bump to 0.2.0
- Expanded documentation

## Remaining

- Full standard library with doc tests
- Concurrency: channels, mutex, atomics, scoped threads
- Language server (LSP) completion and VS Code extension
- WiX installer build validation
- Benchmarks against C and Rust
- Additional error catalog tests (compile-fail per code)
- Generics monomorphization polish in backend
- Cross compilation polish
- WebAssembly target