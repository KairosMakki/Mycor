# Error handling

Mycor uses `Result[T, E]` for recoverable errors and panics for unrecoverable ones.

## Result

```myc
enum Result[T, E] {
    Ok(T),
    Err(E),
}
```

## The ? operator

Propagate errors concisely:

```myc
fn parse_and_double(s: str) -> Result[i32, str] {
    let n = parse_i32(s)?;   // returns Err early
    Ok(n * 2)
}
```

`?` requires the enclosing function to return a compatible `Result`.

## Matching on Result

```myc
match read_file(path) {
    Ok(contents) => println(contents),
    Err(e) => println(e),
}
```

## Panics

Use `panic` for unrecoverable conditions (abort). Prefer `Result` for expected failures.

## Converting errors

Use `map_err` / `and_then` (stdlib, scaffolding) to transform error types. Generic conversions are planned.

## Error codes

Compiler diagnostics use stable codes (`E0xxx`, `W0xxx`). Run `myc explain E0200` for details. See `docs/errors`.