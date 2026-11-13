# Mycor

Mycor (file extension `.myc`) is a small, fast, safe-by-construction systems programming language. It compiles to native code through C (C11) and manages memory with ownership and borrowing instead of a garbage collector. The compiler, tooling, and language server are written in Rust.

- **Version:** 0.2.0
- **License:** MIT OR Apache-2.0
- **Repository:** https://github.com/KairosMakki/Mycor

## Quick start

```sh
myc new hello
cd hello
myc run
```

Edit `main.myc`:

```myc
fn main() {
    println("hello, mycor");
}
```

## Building the compiler

Requirements: Rust 1.91+ (see `rust-toolchain.toml`), and a C compiler (`clang` recommended; `gcc`/`cc`/`cl` also work).

```sh
cargo build --release
```

The binary is `target/release/myc` (or `myc.exe` on Windows). Add it to your `PATH`, or invoke it directly.

```sh
cargo test --workspace
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
```

## Language at a glance

- Static typing with inference: `let x = 5;`, `let mut y: i32 = 0;`
- Primitive types: `i8 i16 i32 i64 isize  u8 u16 u32 u64 usize  f32 f64  bool  char  str`
- Composites: arrays `[T; N]`, slices `&[T]`, tuples `(A, B)`, references `&T` / `&mut T`, raw pointers `*T` / `*mut T` (unsafe only), generics `Foo[T]`, function types `fn(i32) -> i32`
- Ownership and borrowing: values move by default; small `Copy` types copy; borrow with `&` and `&mut`; one mutable borrow or many shared borrows
- `Option[T]` and `Result[T, E]` with `?` propagation
- Structs, enums, traits, impls, type aliases, constants, statics
- Pattern matching with `match`, exhaustiveness checking
- Control flow: `if`/`else`, `while`, `for i in a..b`, `loop`, `break`, `continue`, `defer`
- Modules: `mod util;`, `use util::helper;`, `pub`
- Concurrency: `spawn`, `scope`, channels
- `unsafe` blocks for raw pointer dereference
- Attributes: `#[test]`, `#[inline]`
- C interop through the C backend

## Tooling

| Command | Description |
|---|---|
| `myc new <name>` | Create a new project |
| `myc init` | Initialize a project in the current directory |
| `myc run [file] [--interp]` | Compile and run (or interpret) |
| `myc build [--release]` | Build `main.myc` into `target/myc_app.exe` |
| `myc compile <file>` | Compile a single file to an executable |
| `myc check` | Type-check `main.myc` |
| `myc test` | Run tests discovered in the project |
| `myc fmt [files...]` | Format source files |
| `myc doc` | Generate documentation into `target/doc` |
| `myc emit-c <file>` | Print generated C11 source |
| `myc repl` | Interactive REPL (basic) |
| `myc targets` | List supported targets |
| `myc explain <code>` | Show documentation for an error/warning code |
| `myc clean` | Remove `target/` |
| `myc version` | Print compiler version |

## Project layout

A `myc new` project contains:

```
hello/
  main.myc
  mycor.toml
```

`mycor.toml`:

```toml
[package]
name = "hello"
version = "0.1.0"
```

## Documentation

- Book: `docs/book`
- Language spec: `docs/spec` (`syntax.md`, `grammar.ebnf`)
- Error catalog: `docs/errors` (use `myc explain <code>`)
- Architecture: `docs/design/architecture.md`
- Dependencies: `docs/design/dependencies.md`

## Status

Version 0.2.0. See `ROADMAP.md` for implemented features and remaining work. Highlights of 0.2.0: spawn expression and concurrency syntax, basic LSP server, VS Code TextMate grammar, `doc` and `clean` commands, standard library scaffolding.

## Contributing

See `CONTRIBUTING.md`. Please follow the code of conduct in `CODE_OF_CONDUCT.md`. Report security issues privately per `SECURITY.md`.