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
| `Vec<String>` | `Array<String>` | `[Ljava/lang/String;` | Yes | Yes |
| `&[String]` | `Array<String>` | `[Ljava/lang/String;` | Yes | No |
| `EnumName` (simple enum) | `EnumName` | `L<package>/<EnumName>;` | Yes | Yes |
| `EnumName` (data-bearing enum) | `EnumName` | `L<package>/<EnumName>;` | Yes | Yes |
| `ModelName` (`#[kotlin_model]`) | `ModelName` | `L<package>/<ModelName>;` | Yes | Yes |
| `Option<T>` (where `T` is supported) | `T?` | Boxed / Object (e.g. `Ljava/lang/Integer;`) | Yes | Yes |
| `Result<T, E>` (where `T` is supported, `E: Display`) | `T` (or throws `RuntimeException`) | Return descriptor of `T` | No | Yes |
| `ClassName` (annotated with `#[kotlin_class]`) | `ClassName` | `J` (handle) / `L<package>/<ClassName>;` | Yes (as `&ClassName`) | Yes (as `ClassName` or `Result<ClassName, E>`) |

JNI method descriptors concatenate input descriptors inside parentheses followed by the result descriptor: `add(i32, i32) -> i32` has `(II)I`. The mapping follows the [JNI type-signature rules](https://docs.oracle.com/en/java/javase/21/docs/specs/jni/types.html#type-signatures).

Booleans convert through JNI's byte representation. Signed integer values and floating-point values are passed at their native JVM widths. Ketox does not choose overflow, rounding, or domain-error semantics for the body of an exported Rust function; the function defines its behavior.

## Strings and String Collections

Non-optional strings are non-null at the Kotlin API. The native boundary still checks for null because JVM callers can bypass Kotlin's type checks; it raises `NullPointerException` instead of dereferencing a null reference.

JVM strings are copied from UTF-16 into owned Rust UTF-8. Embedded NULs and valid supplementary Unicode characters are preserved. Unpaired UTF-16 surrogates produce `IllegalArgumentException`, with no replacement-character conversion. An `&str` input borrows from a temporary owned Rust string for the duration of the exported call; it is not a zero-copy JVM borrow. A returned Rust `String` is copied to a new JVM string.

String collections (`Vec<String>` and `&[String]`) map to Kotlin `Array<String>` (`[Ljava/lang/String;`):
- `&[String]` inputs are read from JVM string object arrays into temporary Rust vectors and passed as slices.
- `Vec<String>` returns are converted to newly allocated JVM string object arrays.
- Null arrays raise `NullPointerException` unless wrapped in `Option<T>`.

Inputs and outputs are limited to 16,777,216 UTF-16 code units each (`ketox::runtime::MAX_STRING_UNITS`). Exceeding the limit raises `IllegalArgumentException`.

## Byte arrays and primitive arrays

Byte arrays (`Vec<u8>`, `&[u8]`) map to Kotlin `ByteArray` (JNI `[B`). Primitive arrays (`Vec<i32>`, `Vec<i64>`, `Vec<f32>`, `Vec<f64>`, `Vec<bool>`, and their respective slice forms `&[...]`) map to Kotlin `IntArray`, `LongArray`, `FloatArray`, `DoubleArray`, and `BooleanArray`.

Array inputs copy elements into owned Rust vectors or temporary slices for the duration of the call. Array returns copy elements into newly allocated JVM primitive arrays. Arrays are bounded by `ketox::runtime::MAX_ARRAY_ELEMENTS` (16,777,216 elements). Exceeding this limit raises `IllegalArgumentException`. Passing `null` to a non-optional array raises `NullPointerException`.

## Option and Nullability

`Option<T>` maps directly to Kotlin nullable types `T?`.
- For primitive inner types (e.g. `Option<i32>`, `Option<bool>`), JNI uses boxed JVM types (`java.lang.Integer`, `java.lang.Boolean`, etc.). A `null` JVM reference converts to `None`, and `Some(v)` converts to a boxed object on return (or `null` for `None`).
- For reference types (e.g. `Option<String>`, `Option<Vec<String>>`, `Option<Enum>`, `Option<Model>`), a `null` JVM reference converts to `None`, and `Some(v)` converts to the corresponding JVM object.

## Results and Error Handling

`Result<T, E>` is supported as a function or constructor return type where `E` implements `std::fmt::Display`.
- On `Ok(value)`, the inner value is converted and returned to the Kotlin caller normally.
- On `Err(error)`, Ketox catches the error and raises a JVM `java.lang.RuntimeException` with the message formatted from the error.

## Rust Structs ↔ Kotlin Classes (Phase 3)

Rust structs annotated with `#[kotlin_class]` are exposed as Kotlin classes implementing `java.lang.AutoCloseable`:
- **Constructors:** Associated functions annotated with `#[kotlin_constructor]` returning `Self` or `Result<Self, E>` become Kotlin class constructors.
- **Methods:** Functions declared inside `#[kotlin_export] impl StructName` blocks taking `&self` or `&mut self` become instance methods on the Kotlin class.
- **Cross-Type Parameters:** Methods can take borrowed references to exported structs (e.g. `fn dot(&self, other: &Vector) -> f64`).
- **Memory & Handles:** Instances are held in a thread-safe registry (`HANDLE_REGISTRY`) managed by `ketox-jni`, keyed by a 64-bit integer handle (`nativeHandle: Long`).
- **Safe Concurrency:** Handlers are wrapped in `Arc<RwLock<T>>`, allowing concurrent `&self` read access and safe serialized `&mut self` write access across threads.
- **Lifecycle & Disposal:** The Kotlin class implements `java.lang.AutoCloseable`. Calling `close()` or using Kotlin's `.use { ... }` block invokes the native destructor and safely releases the handle.
- **Stale Handle Protection:** Calling methods on a closed or invalid handle throws `IllegalStateException`. Double-closing is safe and idempotent.

## Enums and Sealed Classes (Phase 4)

Rust enums annotated with `#[kotlin_enum]` are exposed to Kotlin:
- **Simple Enums:** Fieldless Rust enums map to Kotlin `enum class`. Conversions use fast JNI `ordinal()` dispatch when converting to Rust, and direct static field access when returning to Kotlin.
- **Data-bearing Enums / ADTs:** Rust enums with variant data map to Kotlin `sealed class` hierarchies:
  - Unit variants map to `data object Variant : SealedClass()`.
  - Data-bearing variants map to `data class Variant(val f: T, ...) : SealedClass()`.
  - Enables idiomatic Kotlin `when` exhaustive pattern matching and smart-casting.

## Data Models / Value Structs (Phase 4)

Rust structs annotated with `#[kotlin_model]` (or `#[kotlin_data]`) map to Kotlin `data class`:
- Properties are generated with public read-only `val` properties and primary constructors.
- Supports nested structs, enums, optionals, and collection fields.
- Converted recursively across the JNI boundary using generated field readers and constructor invocation.

## Rejected types and declarations

Unsigned integers (other than element type in `Vec<u8>` / `&[u8]`), pointer-sized integers (`usize`, `isize`), `char`, raw pointers, arbitrary unannotated references, tuples other than a Unit return, maps/dictionaries, nested options (`Option<Option<T>>`), nested results, callbacks, and futures are unsupported in this phase. Fully qualified type names and type aliases are not resolved; use the exact supported spellings in exported signatures.

Borrowed returns and explicit reference lifetimes are rejected. Exports must be safe, synchronous, non-generic functions, with simple named parameters.
