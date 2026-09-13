# Ketox type contract

The JVM backend accepts the following types. Inputs and outputs use the same signed primitive widths on both sides.

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
| `Vec<u8>` | `ByteArray` | `[B` | Yes | Yes |
| `&[u8]` | `ByteArray` | `[B` | Yes | No |
| `Vec<i32>` | `IntArray` | `[I` | Yes | Yes |
| `&[i32]` | `IntArray` | `[I` | Yes | No |
| `Vec<i64>` | `LongArray` | `[J` | Yes | Yes |
| `&[i64]` | `LongArray` | `[J` | Yes | No |
| `Vec<f32>` | `FloatArray` | `[F` | Yes | Yes |
| `&[f32]` | `FloatArray` | `[F` | Yes | No |
| `Vec<f64>` | `DoubleArray` | `[D` | Yes | Yes |
| `&[f64]` | `DoubleArray` | `[D` | Yes | No |
| `Vec<bool>` | `BooleanArray` | `[Z` | Yes | Yes |
| `&[bool]` | `BooleanArray` | `[Z` | Yes | No |
| `Option<T>` (where `T` is supported) | `T?` | Boxed / Object (e.g. `Ljava/lang/Integer;`) | Yes | Yes |
| `Result<T, E>` (where `T` is supported, `E: Display`) | `T` (or throws `RuntimeException`) | Return descriptor of `T` | No | Yes |

JNI method descriptors concatenate input descriptors inside parentheses followed by the result descriptor: `add(i32, i32) -> i32` has `(II)I`. The mapping follows the [JNI type-signature rules](https://docs.oracle.com/en/java/javase/21/docs/specs/jni/types.html#type-signatures).

Booleans convert through JNI's byte representation. Signed integer values and floating-point values are passed at their native JVM widths. Ketox does not choose overflow, rounding, or domain-error semantics for the body of an exported Rust function; the function defines its behavior.

## Strings

Non-optional strings are non-null at the Kotlin API. The native boundary still checks for null because JVM callers can bypass Kotlin's type checks; it raises `NullPointerException` instead of dereferencing a null reference.

JVM strings are copied from UTF-16 into owned Rust UTF-8. Embedded NULs and valid supplementary Unicode characters are preserved. Unpaired UTF-16 surrogates produce `IllegalArgumentException`, with no replacement-character conversion. An `&str` input borrows from a temporary owned Rust string for the duration of the exported call; it is not a zero-copy JVM borrow. A returned Rust `String` is copied to a new JVM string.

Inputs and outputs are limited to 16,777,216 UTF-16 code units each (`ketox::runtime::MAX_STRING_UNITS`). Exceeding the limit raises `IllegalArgumentException`.

## Byte arrays and primitive arrays

Byte arrays (`Vec<u8>`, `&[u8]`) map to Kotlin `ByteArray` (JNI `[B`). Primitive arrays (`Vec<i32>`, `Vec<i64>`, `Vec<f32>`, `Vec<f64>`, `Vec<bool>`, and their respective slice forms `&[...]`) map to Kotlin `IntArray`, `LongArray`, `FloatArray`, `DoubleArray`, and `BooleanArray`.

Array inputs copy elements into owned Rust vectors or temporary slices for the duration of the call. Array returns copy elements into newly allocated JVM primitive arrays. Arrays are bounded by `ketox::runtime::MAX_ARRAY_ELEMENTS` (16,777,216 elements). Exceeding this limit raises `IllegalArgumentException`. Passing `null` to a non-optional array raises `NullPointerException`.

## Option and Nullability

`Option<T>` maps directly to Kotlin nullable types `T?`.
- For primitive inner types (e.g. `Option<i32>`, `Option<bool>`), JNI uses boxed JVM types (`java.lang.Integer`, `java.lang.Boolean`, etc.). A `null` JVM reference converts to `None`, and `Some(v)` converts to a boxed object on return (or `null` for `None`).
- For reference types (e.g. `Option<String>`, `Option<Vec<u8>>`), a `null` JVM reference converts to `None`, and `Some(v)` converts to the corresponding JVM object.

## Results and Error Handling

`Result<T, E>` is supported as a function return type where `E` implements `std::fmt::Display`.
- On `Ok(value)`, the inner value is converted and returned to the Kotlin caller normally.
- On `Err(error)`, Ketox catches the error and raises a JVM `java.lang.RuntimeException` with the message formatted from the error.

## Rejected types and declarations

Unsigned integers (other than element type in `Vec<u8>` / `&[u8]`), pointer-sized integers (`usize`, `isize`), `char`, raw pointers, arbitrary references, tuples other than a Unit return, non-primitive collections, nested options (`Option<Option<T>>`), nested results, structs, enums, callbacks, and futures are unsupported in this phase. Fully qualified type names and type aliases are not resolved; use the exact supported spellings in exported signatures.

Borrowed returns and explicit reference lifetimes are rejected. Exports must be safe, synchronous, non-generic functions, with simple named parameters.
