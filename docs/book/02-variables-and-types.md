# Variables and types

## Variables

Declare a variable with `let`. By default bindings are immutable:

```myc
let x = 5;
let name = "mycor";
```

Mutable bindings use `mut`:

```myc
let mut counter = 0;
counter = counter + 1;
```

Type annotations are optional when the type can be inferred:

```myc
let x: i32 = 5;
let y = x + 1;   // inferred i32
```

Uninitialized variables are not allowed; every `let` must have an initializer (or be assigned before use where the checker permits).

## Primitive types

Integers:

- Signed: `i8`, `i16`, `i32`, `i64`, `isize`
- Unsigned: `u8`, `u16`, `u32`, `u64`, `usize`

Floating point: `f32`, `f64`.

Other: `bool`, `char`, `str`.

```myc
let a: i32 = -42;
let b: u64 = 1000;
let c: f64 = 3.14;
let flag: bool = true;
let ch: char = 'x';
let s: str = "hello";
```

Integer literals are `i64`-ish and coerce where needed; floats require a decimal point or exponent.

## Copy vs move

Small primitive types (`i*`, `u*`, `f*`, `bool`, `char`, raw pointers) are `Copy`: assignment copies. Larger types (structs, enums, `str` values, arrays) move by default. See [Ownership](06-ownership.md).

## Composite types

Arrays (fixed size):

```myc
let a: [i32; 4] = [1, 2, 3, 4];
let v = a[2];
```

Slices (view into an array or buffer):

```myc
let s: &[i32] = &a[1..3];
```

Tuples:

```myc
let t: (i32, f64, bool) = (1, 2.0, true);
let (x, y, z) = t;
```

References:

```myc
let r: &i32 = &x;
let m: &mut i32 = &mut y;
```

Raw pointers (unsafe only):

```myc
let p: *mut i32 = ...;
```

Generics and function types:

```myc
let f: fn(i32) -> i32 = square;
```

## Option and Result

Optional values use `Option[T]`:

```myc
enum Option[T] { Some(T), None }
```

Errors use `Result[T, E]`:

```myc
enum Result[T, E] { Ok(T), Err(E) }
```

Propagate with `?`:

```myc
fn read() -> Result[i32, str] {
    let v = parse()?; // returns Err early on failure
    Ok(v)
}
```

## Type aliases

```myc
type Alias = i64;
```

## Constants and statics

```myc
const MAX: usize = 1024;
static COUNTER: AtomicUsize = AtomicUsize.new(0);
```

Constants are evaluated at compile time. See error codes `E0600`-`E0603`.

## Casting

Use `as`:

```myc
let x: i64 = 5 as i64;
let y: usize = x as usize;
```