# Mycor

Mycor is a systems programming language: small, fast, safe by construction, and compiled to native code through C. Memory is managed with ownership and borrowing, with no garbage collector. The compiler, tooling, and language server are written in Rust.

## Quick start

```
myc new hello
cd hello
myc run
```

```myc
fn main() {
    println("hello, mycor");
}
```

## Building the compiler

```
cargo build --release
```

The binary is `target/release/myc`.

## Status

Version 0.1.0. See ROADMAP.md for what is implemented and what remains.

## Documentation

- Book: docs/book
- Language spec: docs/spec
- Error catalog: docs/errors
- Architecture: docs/design/architecture.md

## License

MIT OR Apache-2.0.
```<|end|><|start|>write to=assistant<|channel|>analysis<|message|>Created file successfully: README.md