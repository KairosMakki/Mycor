# Pattern matching

## match expressions

```myc
match value {
    0 => "zero",
    1 => "one",
    _ => "other",
}
```

`match` is an expression. Arms are `pattern => expression` (or block).

## Patterns

- Literals: `0`, `true`, `'a'`, `"hi"`
- Wildcard: `_`
- Binding: `x` binds the whole value
- Tuple: `(a, b)`
- Struct: `Point { x, y }`, `Point { x, .. }`
- Enum variant: `Some(v)`, `None`, `Color::Red`
- Range: `1..=5`
- Or-patterns: `1 | 2 | 3`
- Guards: `x if x > 0 => ...`

## Exhaustiveness

The compiler checks that all cases are covered:

```myc
match o {
    Some(v) => v,
    // error E0208: non-exhaustive match (missing None)
}
```

Add the missing arm or a wildcard `_`.

## Unreachable patterns

Redundant arms are warnings/errors:

```myc
match x {
    _ => 0,
    1 => 1, // E0209: unreachable pattern
}
```

## Missing arms

`E0207`: a match arm body is missing.

## Destructuring in let

```myc
let (a, b) = pair;
let Point { x, y } = p;
```