# C interop

Mycor compiles to C11, so interop with C is natural.

## Calling C functions

Declare foreign functions (syntax planned; currently use extern declarations as supported by the parser):

```myc
extern "C" fn puts(s: *const i8) -> i32;
```

## Types

Mycor primitives map to C types:
- `i8`/`u8` -> `int8_t`/`uint8_t`
- `i32` -> `int32_t`
- `i64` -> `int64_t`
- `f64` -> `double`
- `bool` -> `_Bool`
- `str` -> C string (null-terminated)
- Raw pointers -> C pointers

## ABI

The default ABI is C. Structs are laid out in declaration order to match C structs.

## Emitting C

Inspect generated C:

```sh
myc emit-c main.myc
```

## Linking

The driver invokes the C compiler and linker. Add C sources/libraries via the package manifest (planned). Current builds compile the single generated C file.

## Safety

All FFI calls are unsafe from Mycor's perspective; wrap them in safe functions.<|end|><|start|>write to=assistant<|channel|>analysis<|message|>Created file successfully: docs/book/13-c-interop.md