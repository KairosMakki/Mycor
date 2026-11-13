# Control flow

## if / else

```myc
if x > 0 {
    println("positive");
} else if x == 0 {
    println("zero");
} else {
    println("negative");
}
```

`if` is an expression when both branches yield a value:

```myc
let label = if x > 0 { "pos" } else { "nonpos" };
```

## while

```myc
let mut i = 0;
while i < 3 {
    println(i);
    i = i + 1;
}
```

## for

Range-based loops:

```myc
for i in 0..n {
    println(i);
}
for i in 0..=n {   // inclusive
    println(i);
}
```

## loop

```myc
loop {
    if done { break; }
}
```

`break` and `continue` are supported.

## match

```myc
match value {
    Some(v) => v,
    None => 0,
}
```

Matches are checked for exhaustiveness (`E0208`) and unreachable patterns (`E0209`).

## defer

Run cleanup when leaving scope:

```myc
defer cleanup();
```

## Operators

Arithmetic: `+ - * / %`
Comparison: `== != < <= > >=`
Logical: `&& || !`
Bitwise: `& | ^ ~ << >>`
Assignment: `=`, compound `+= -= *= /= %= &= |= ^= <<= >>=`
Range: `..`, `..=`

Precedence follows C-like rules. Use parentheses to disambiguate.