# Structs and enums

## Structs

```myc
struct Point {
    x: f64,
    y: f64,
}

let p = Point { x: 1.0, y: 2.0 };
let qx = p.x;
```

Fields may be private by default; use `pub` to export:

```myc
pub struct User {
    pub name: str,
    age: i32,   // private
}
```

## Enums

```myc
enum Color {
    Red,
    Green,
    Blue,
}

enum Option[T] {
    Some(T),
    None,
}
```

Construct variants:

```myc
let c = Color::Red;
let o = Option::Some(42);
```

## Pattern matching on enums

```myc
match o {
    Some(v) => v,
    None => 0,
}
```

Missing arms are errors (`E0207`, `E0208`).

## Methods

```myc
impl Point {
    fn translate(self: &mut Self, dx: f64, dy: f64) {
        self.x = self.x + dx;
        self.y = self.y + dy;
    }
}
```

## Traits and impls

```myc
trait Display {
    fn fmt(self: &Self) -> str;
}

impl Display for Point {
    fn fmt(self: &Self) -> str { "point" }
}
```

## Derives

(Planned) derive macros for common traits are on the roadmap. Currently implement traits manually.

## Memory layout

Structs are laid out in declaration order for C interop. Enums are tagged unions. Packed fields have borrowing restrictions (`E0310`).