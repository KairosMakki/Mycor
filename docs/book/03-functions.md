# Functions

## Defining functions

```myc
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

The last expression in a block is the return value. Use `return` for early returns:

```myc
fn abs(x: i32) -> i32 {
    if x < 0 {
        return -x;
    }
    x
}
```

Functions with no return type return `()` (void):

```myc
fn greet(name: str) {
    println(name);
}
```

## Forward declarations

A function may be declared without a body and defined later:

```myc
fn square(x: i32) -> i32;   // forward declaration

fn square(x: i32) -> i32 {
    x * x
}
```

Forward declarations are checked for signature consistency (`E0700`-`E0703`).

## Parameters and mutability

Parameters are immutable by default. To mutate, bind to a mutable local:

```myc
fn inc(mut x: i32) -> i32 {
    x = x + 1;
    x
}
```

## Methods and impls

```myc
struct Point { x: f64, y: f64 }

impl Point {
    fn length(self: &Self) -> f64 {
        // ...
        0.0
    }
}
```

Method calls: `p.length()`.

## Traits

```myc
trait Display {
    fn fmt(self: &Self) -> str;
}

impl Display for Point {
    fn fmt(self: &Self) -> str { "point" }
}
```

## Closures

```myc
let f = |a, b| a + b;
let g = |x: i32| -> i32 { x * 2 };
```

Function types: `fn(i32) -> i32`.

## Generics

```myc
fn identity[T](x: T) -> T {
    x
}
```

See [Generics and traits](08-generics-and-traits.md).

## Attributes

```myc
#[test]
fn t() { ... }

#[inline]
fn hot() { ... }
```