# Unsafe code

Raw pointers and certain operations require `unsafe`:

```myc
unsafe {
    let p: *mut i32 = ...;
    *p = 7;
}
```

## Raw pointers

- `*T`: immutable raw pointer
- `*mut T`: mutable raw pointer

Raw pointers are only dereferenced inside `unsafe` blocks (`E0400`, `E0401`).

## Rules

- References are checked everywhere; raw pointers are not.
- `unsafe` does not disable the borrow checker for references.
- Invalid unsafe operations are errors (`E0403`).
- Unsafe blocks are not allowed in some contexts (`E0402`).

## FFI

Calling C functions and manipulating raw memory happens in `unsafe`. See [C interop](13-c-interop.md).

## Best practices

- Keep `unsafe` blocks as small as possible.
- Document invariants.
- Wrap unsafe in safe abstractions.