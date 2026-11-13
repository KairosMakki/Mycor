# Getting started

## Installation

Build the compiler from source:

```sh
git clone https://github.com/KairosMakki/Mycor
cd Mycor
cargo build --release
```

The binary is `target/release/myc`. Put it on your `PATH`.

You also need a C compiler. `clang` is recommended; `gcc`, `cc`, and MSVC `cl` are also supported by the driver.

## Your first program

Create a project:

```sh
myc new hello
cd hello
```

This creates:

```
hello/
  main.myc
  mycor.toml
```

`main.myc`:

```myc
fn main() {
    println("hello, mycor");
}
```

Run it:

```sh
myc run
```

Or compile to an executable:

```sh
myc build
./target/myc_app.exe   # Windows: target\myc_app.exe
```

## Running a single file

```sh
myc run path/to/file.myc
myc compile path/to/file.myc
```

## Interpreted mode

For fast iteration without invoking the C compiler:

```sh
myc run --interp
```

The interpreter consumes the same HIR/MIR as the compiler, so results match native builds for supported features.

## Project manifest

`mycor.toml`:

```toml
[package]
name = "hello"
version = "0.1.0"
```

## Next steps

- [Variables and types](02-variables-and-types.md)
- [Functions](03-functions.md)