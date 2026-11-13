# Error catalog

All compiler diagnostic codes. Run `myc explain <CODE>` to print a specific entry.

## Syntax (E0001-E0030)

E0001 unexpected token
E0002 expected expression
E0003 expected type
E0004 expected identifier
E0005 expected ';'
E0006 expected '{'
E0007 expected '}'
E0008 expected ')'
E0009 expected ']'
E0010 expected '>'
E0011 invalid literal
E0012 unterminated string
E0013 unterminated comment
E0014 invalid character
E0015 expected pattern
E0016 expected item
E0017 expected block
E0018 expected type name
E0019 expected field
E0020 expected variant
E0021 expected parameter
E0022 expected generic
E0023 expected literal
E0024 expected path
E0025 invalid escape
E0026 invalid number
E0027 invalid float
E0028 unterminated char
E0029 invalid utf8
E0030 reserved keyword

## Names (E0100-E0110)

E0100 unknown name
E0101 duplicate definition
E0102 private item
E0103 import not found
E0104 ambiguous import
E0105 unused import
E0106 did you mean
E0107 unknown module
E0108 unknown field
E0109 unknown method
E0110 unknown variant

## Types (E0200-E0215)

E0200 type mismatch
E0201 cannot infer type
E0202 wrong number of type arguments
E0203 wrong number of arguments
E0204 missing trait implementation
E0205 recursive type without indirection
E0206 invalid cast
E0207 missing match arm
E0208 non-exhaustive match
E0209 unreachable pattern
E0210 uninitialized variable
E0211 cannot call non-function
E0212 cannot index non-array
E0213 cannot field access non-struct
E0214 duplicate field
E0215 missing field

## Ownership (E0300-E0310)

E0300 use after move
E0301 double mutable borrow
E0302 shared and mutable borrow conflict
E0303 dangling reference
E0304 returning reference to local
E0305 cannot move out of borrowed
E0306 borrow lives too long
E0307 mutable borrow of immutable
E0308 borrow outlives value
E0309 cannot move out of index
E0310 cannot borrow packed field

## Unsafe (E0400-E0403)

E0400 unsafe operation outside unsafe block
E0401 dereference of raw pointer outside unsafe
E0402 unsafe block not allowed
E0403 invalid unsafe operation

## Concurrency (E0500-E0504)

E0500 data race
E0501 non-sendable value sent to thread
E0502 non-sync shared access
E0503 channel type mismatch
E0504 thread panicked

## Constants (E0600-E0603)

E0600 integer overflow in constant
E0601 division by zero in constant
E0602 constant eval failed
E0603 non-const in const context

## Forward declarations (E0700-E0703)

E0700 forward declaration mismatch
E0701 conflicting definitions
E0702 forward decl not found
E0703 signature mismatch

## Warnings (W0001-W0003)

W0001 unused variable
W0002 unreachable code
W0003 unused import