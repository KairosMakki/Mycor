# Modules

## Declaring modules

```myc
mod util;   // loads util.myc or util/mod.myc
```

Top-level items are order-independent. Forward declarations are allowed.

## Imports

```myc
use util::helper;
use std::io::{read, write};
use util::*;          // glob (if supported)
```

## Visibility

Items are private by default. Use `pub` to export:

```myc
pub fn helper() -> i32 { 42 }
```

Private access from outside the module is an error (`E0102`).

## Paths

Nested modules:

```myc
mod net {
    pub mod http { ... }
}
use net::http::get;
```

## Re-exports

```myc
pub use util::helper;
```

## Common errors

- `E0103`: import not found
- `E0104`: ambiguous import
- `E0105`: unused import (warning `W0003`)
- `E0106`: did you mean (suggestion)
- `E0107`: unknown module
- `E0108`: unknown field
- `E0109`: unknown method
- `E0110`: unknown variant