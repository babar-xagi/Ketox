# Ketox architecture contract

Status: Phase 3 development contract. The backend is Kotlin/JVM through JNI.

## Build and call paths

```text
Rust source with #[kotlin_export], #[kotlin_class], #[kotlin_constructor]
    -> shared signature/type validation in ketox-core
    -> metadata module (schema version 3)
    -> ketox-codegen
        -> ketox_jni.rs (included by the Rust library)
        -> RustApi.kt (compiled by Kotlin, contains object RustApi and classes)
        -> ketox-metadata.json

Kotlin object / class method
    -> JNI native symbol in the Rust cdylib
    -> handle lookup in ketox-jni (for class methods)
    -> exception/panic boundary and value conversion
    -> the original Rust function or method
```

`ketox` is the facade used by application crates. `ketox-macros` validates annotations using `ketox-core` and emits hidden metadata constants. Macros do not write generated files or depend on invocation order. A `build.rs` calls `ketox_codegen::generate_from_file` and owns filesystem output. Both paths use the same validation logic; the build helper discovers source declarations directly rather than reading constants from compiled binaries.

`ketox-core` depends on syntax and serialization libraries, not JNI. It validates exported signatures, computes Kotlin names and JVM type descriptors, and produces portable metadata. `ketox-codegen` turns valid metadata into inspectable Rust, Kotlin, and JSON. `ketox-jni` owns JNI conversion, the panic/exception boundary, and the global thread-safe handle registry for stateful objects. `ketox-cli` exposes the same discovery and generation path through `ketox inspect` and `ketox generate`.

## Source discovery

The generator reads one UTF-8 Rust source file and recognizes top-level exports:
- Top-level `#[kotlin_export]` functions
- Structs annotated with `#[kotlin_class]`
- Associated functions annotated with `#[kotlin_constructor]` inside `impl Struct`
- Method blocks: `#[kotlin_export] impl Struct` declaring `&self` or `&mut self` methods

Supported declarations are unrestricted `pub fn` functions and methods with simple parameter identifiers. Async, unsafe, const, extern, variadic, generic, and `where` clauses are rejected. Generation validates its full input before writing output. The build script must emit `cargo:rerun-if-changed` for its input source. The generated Rust must be included at the same crate root as the source functions.

## Naming and loading

The configured package is a nonempty dot-separated sequence of validated identifiers; the configured object name is a validated identifier. Example: `dev.ketox.example.RustApi`. Rust function and method names become lowerCamelCase by removing underscores and capitalizing the following components; `to_string_repr` becomes `toStringRepr`. Struct names become Kotlin class names in PascalCase. Parameters retain their original names. Duplicate names and Kotlin collisions produce errors.

Generated Kotlin contains an `object RustApi` with static JNI bridge methods, and public Kotlin classes (`class Vector internal constructor(internal var nativeHandle: kotlin.Long) : java.lang.AutoCloseable`).

The object initializer calls `System.loadLibrary` with the configured base name, such as `ketox_hello`. The caller supplies the native directory through `java.library.path`. Loading errors add the library name, OS, architecture, and search path while retaining the underlying cause.

## Handle Registry and State Management

`ketox-jni` provides a thread-safe handle registry:
- State is held as `Arc<RwLock<T>>`, allowing concurrent immutable calls (`&self`) and serialized mutable calls (`&mut self`).
- `register_handle` assigns a unique 64-bit integer handle to new instances.
- `with_handle` and `with_handle_mut` look up handles safely with automatic stale-handle detection.
- `destroy_handle` releases the instance when closed from Kotlin. Double closes are safely ignored.

## Compatibility and platforms

Metadata uses explicit schema version 3 (supporting versions 1, 2, and 3). Unknown fields and unsupported versions are rejected; semantic validation runs before generation. Source-discovered functions and classes are deterministically ordered by their Rust names.

Rust 1.88 is the declared minimum compiler version, with edition 2024. The JNI crate is pinned to `0.21.1`. The CI configuration targets Windows, Linux, and macOS with stable Rust, JDK 21, and Kotlin 2.2.0.
