# Ketox type contract

The initial JVM backend accepts the following types. Inputs and outputs use the same signed primitive widths on both sides.

| Rust | Kotlin | JVM descriptor | Input | Return |
| --- | --- | --- | --- | --- |
| `bool` | `Boolean` | `Z` | Yes | Yes |
| `i8` | `Byte` | `B` | Yes | Yes |
| `i16` | `Short` | `S` | Yes | Yes |
| `i32` | `Int` | `I` | Yes | Yes |
| `i64` | `Long` | `J` | Yes | Yes |
| `f32` | `Float` | `F` | Yes | Yes |
| `f64` | `Double` | `D` | Yes | Yes |
| `String` | `String` | `Ljava/lang/String;` | Yes | Yes |
| `&str` with an elided lifetime | `String` | `Ljava/lang/String;` | Yes | No |
| `()` or omitted return | `Unit` | `V` | No | Yes |

JNI method descriptors concatenate input descriptors inside parentheses followed by the result descriptor: `add(i32, i32) -> i32` has `(II)I`. The mapping follows the [JNI type-signature rules](https://docs.oracle.com/en/java/javase/21/docs/specs/jni/types.html#type-signatures).

Booleans convert through JNI's byte representation. Signed integer values and floating-point values are passed at their native JVM widths. Ketox does not choose overflow, rounding, or domain-error semantics for the body of an exported Rust function; the function defines its behavior.

## Strings

Strings are non-null at the Kotlin API. The native boundary still checks for null because JVM callers can bypass Kotlin's type checks; it raises `NullPointerException` instead of dereferencing a null reference.

JVM strings are copied from UTF-16 into owned Rust UTF-8. Embedded NULs and valid supplementary Unicode characters are preserved. Unpaired UTF-16 surrogates produce `IllegalArgumentException`, with no replacement-character conversion. An `&str` input borrows from a temporary owned Rust string for the duration of the exported call; it is not a zero-copy JVM borrow. A returned Rust `String` is copied to a new JVM string.

Inputs and outputs are limited to 16,777,216 UTF-16 code units each (`ketox::runtime::MAX_STRING_UNITS`). Exceeding the limit raises `IllegalArgumentException`. This is a prototype limit and does not promise that every allocation up to that size succeeds.

## Rejected types and declarations

Unsigned integers, pointer-sized integers, `char`, raw pointers, references other than input `&str`, arrays, tuples other than a Unit return, collections, `Option`, `Result`, structs, enums, callbacks, and futures are unsupported. Fully qualified type names and type aliases are not resolved; use the exact supported spellings in exported signatures.

Borrowed returns and explicit reference lifetimes are rejected. Exports must be public, safe, synchronous, non-generic top-level functions, with simple named parameters. Support for error/result types and collections belongs to Phase 2 and requires a separate contract before implementation.
