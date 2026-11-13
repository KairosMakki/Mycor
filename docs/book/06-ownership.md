# Ownership and borrowing

Mycor has no garbage collector. Memory safety is enforced at compile time through ownership and borrowing.

## Ownership

Each value has a single owner. When the owner goes out of scope, the value is dropped.

```myc
fn main() {
    let s = String::from("hi"); // (illustrative)
    take(s);                    // s moves into take
    // s is no longer usable here
}
```

## Move semantics

Values move by default on assignment and function calls:

```myc
struct Foo { x: i32 }

fn take(f: Foo) {}

fn main() {
    let f = Foo { x: 1 };
    take(f);
    // println(f.x); // error E0300: use after move
}
```

## Copy types

Small types are `Copy` and are duplicated instead of moved: all integers, floats, `bool`, `char`, raw pointers. Assignment copies them.

```myc
let x = 5;
let y = x;   // copy
println(x);  // ok
```

## Borrowing

Borrow a value with `&` (shared) or `&mut` (mutable):

```myc
fn read(r: &Foo) {}
fn write(r: &mut Foo) {}

fn main() {
    let mut f = Foo { x: 1 };
    read(&f);        // shared borrow
    write(&mut f);   // mutable borrow
}
```

Rules:
- One mutable borrow OR many shared borrows at a time.
- Borrows must not outlive the borrowed value.
- Cannot mutate while borrowed.

Violations: `E0301`, `E0302`, `E0303`, `E0306`, `E0307`, `E0308`.

## References in functions

```myc
fn first(s: &str) -> &str { s }

fn bad() -> &i32 {
    let x = 5;
    &x   // error E0304: returning reference to local
}
```

## Moving out of borrows

Cannot move out of a borrowed value or an indexed location:

- `E0305`: cannot move out of borrowed
- `E0309`: cannot move out of index

## Reborrowing and slices

Slices `&[T]` borrow the underlying array. Packed struct fields have special borrow rules (`E0310`).

## Lifetimes

Mycor infers lifetimes. Returning references ties the output lifetime to an input. Dangling references are rejected (`E0303`, `E0304`).

## Practical tips

- Prefer borrowing over cloning for large values.
- Keep mutable borrows short-lived.
- Use `Copy` types freely; they never move.