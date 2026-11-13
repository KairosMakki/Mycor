# Mycor syntax summary

File extension: `.myc`. One statement per line is not required; semicolons separate statements. Newlines may act as statement terminators in unambiguous positions, but semicolons are always accepted.

## Items

```myc
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn square(x: i32) -> i32;   // forward declaration

struct Point {
    x: f64,
    y: f64,
}

enum Color {
    Red,
    Green,
    Blue,
}

enum Option[T] {
    Some(T),
    None,
}

trait Display {
    fn fmt(self: &Self) -> str;
}

impl Display for Point {
    fn fmt(self: &Self) -> str { "point" }
}

const MAX: usize = 1024;
static COUNTER: AtomicUsize = AtomicUsize.new(0);
type Alias = i64;
```

## Variables and control flow

```myc
let x = 5;              // immutable
let mut y = 5;          // mutable
y = y + 1;

if x > 0 {
    ...
} else {
    ...
}

while cond { ... }
for i in 0..n { ... }
loop { ... }

match value {
    Some(v) => v,
    None => 0,
}

defer cleanup();
```

## Types

`i8 i16 i32 i64 isize  u8 u16 u32 u64 usize  f32 f64  bool  char  str`

Composite: `T*`? No. Arrays `[T; N]`, slices `&[T]`, tuples `(A, B)`, references `&T` and `&mut T`, raw pointers `*T` and `*mut T` (unsafe only), generics `Foo[T, U]`, function types `fn(i32) -> i32`.

Optional values use `Option[T]`. Errors use `Result[T, E]`. Propagation: `expr?`.

## Expressions

Arithmetic `+ - * / %`, comparison `== != < <= > >=`, logical `&& || !`, bitwise `& | ^ ~ << >>`, assignment `=`, compound `+=` etc. Function call `f(a, b)`. Method call `x.method()`. Field access `p.x`. Index `a[i]`. Range `a..b`, `a..=b`. Cast `x as i64`. Closure `|a, b| a + b`.

## Modules

```myc
mod util;          // loads util.myc or util/mod.myc
use util::helper;  // import a name
use std::io::{read, write};
```

Top-level items are order independent. Forward declarations are allowed and checked for consistency.

## Concurrency

```myc
let (tx, rx) = channel[i32]();
spawn { tx.send(42); }
let v = rx.recv();
```

Scoped threads:

```myc
scope |s| {
    s.spawn(|| work(&data));
}
```

## Unsafe

```myc
unsafe {
    let p: *mut i32 = ...;
    *p = 7;
}
```

Raw pointers are only dereferenced inside `unsafe` blocks. References are checked everywhere.

## Attributes

```myc
#[test]
fn t() { ... }

#[inline]
fn hot() { ... }
```
