# Generics and traits

## Generic functions

```myc
fn identity[T](x: T) -> T {
    x
}

fn max[T: Ord](a: T, b: T) -> T {
    if a > b { a } else { b }
}
```

Type arguments are inferred at call sites when possible:

```myc
let x = identity(5);   // T = i32
```

Explicit:

```myc
let x = identity[i32](5);
```

## Generic structs and enums

```myc
struct Box[T] {
    value: T,
}

enum Option[T] {
    Some(T),
    None,
}
```

## Traits

Define shared behavior:

```myc
trait Display {
    fn fmt(self: &Self) -> str;
}
```

Implement for a type:

```myc
impl Display for Point {
    fn fmt(self: &Self) -> str { "point" }
}
```

Trait bounds:

```myc
fn print_it[T: Display](x: T) {
    println(x.fmt());
}
```

Multiple bounds: `T: Display + Clone`.

## Where clauses

```myc
fn foo[T, U](x: T, y: U) where T: Display, U: Clone {
    ...
}
```

## Associated types

(Planned) associated types are on the roadmap. Use generic parameters today.

## Monomorphization

Generics are monomorphized in the backend. Each concrete instantiation produces specialized code. See `ROADMAP.md` for remaining work.

## Missing trait implementation

`E0204`: a required trait is not implemented for a type.

## Wrong type arguments

`E0202`: wrong number of type arguments to a generic.