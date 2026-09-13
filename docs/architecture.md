# Ketox architecture contract

Status: Phase 5 development contract. The backend is Kotlin/JVM through JNI.

## Build and call paths

```text
Rust source with #[kotlin_export], #[kotlin_class], #[kotlin_constructor], #[kotlin_enum], #[kotlin_model], #[kotlin_callback]
    -> shared signature/type validation in ketox-core
    -> metadata module (schema version 5)
    -> ketox-codegen
        -> ketox_jni.rs (included by the Rust library)
        -> RustApi.kt (compiled by Kotlin, contains object RustApi, classes, enums, models, callbacks)
        -> ketox-metadata.json

Kotlin object / class method / value types / lambdas
    -> JNI native symbol in the Rust cdylib
    -> handle lookup in ketox-jni (for class methods)
    -> value conversion / reflection reconstruction (for models and sealed enums)
    -> callback proxy wrapping (JavaVM + GlobalRef) implementing Rust traits
    -> exception/panic boundary and value conversion
    -> the original Rust function, method, constructor, or callback invocation
```

`ketox` is the facade used by application crates. `ketox-macros` validates annotations using `ketox-core` and emits hidden metadata constants. Macros do not write generated files or depend on invocation order. A `build.rs` calls `ketox_codegen::generate_from_file` and owns filesystem output. Both paths use the same validation logic; the build helper discovers source declarations directly rather than reading constants from compiled binaries.

`ketox-core` depends on syntax and serialization libraries, not JNI. It validates exported signatures, computes Kotlin names and JVM type descriptors, and produces portable metadata. `ketox-codegen` turns valid metadata into inspectable Rust, Kotlin, and JSON. `ketox-jni` owns JNI conversion, the panic/exception boundary, array/string conversions, callback thread attachment/detachment, and the global thread-safe handle registry for stateful objects. `ketox-cli` exposes the same discovery and generation path through `ketox inspect` and `ketox generate`.

## Source discovery

The generator reads one UTF-8 Rust source file and recognizes top-level exports:
- Top-level `#[kotlin_export]` functions
- Structs annotated with `#[kotlin_class]`
- Associated functions annotated with `#[kotlin_constructor]` inside `impl Struct`
- Method blocks: `#[kotlin_export] impl Struct` declaring `&self` or `&mut self` methods
- Enums annotated with `#[kotlin_enum]` (simple C-like fieldless enums or data-bearing ADTs)
- Structs annotated with `#[kotlin_model]` (or `#[kotlin_data]`) for pass-by-value data models
- Traits annotated with `#[kotlin_callback]` for Kotlin callbacks and lambdas invoked by Rust

Supported declarations are unrestricted `pub fn` functions and methods with simple parameter identifiers. Async, unsafe, const, extern, variadic, generic, and `where` clauses are rejected. Generation validates its full input before writing output. The build script must emit `cargo:rerun-if-changed` for its input source. The generated Rust must be included at the same crate root as the source functions.

## Naming and loading

The configured package is a nonempty dot-separated sequence of validated identifiers; the configured object name is a validated identifier. Example: `dev.ketox.example.RustApi`. Rust function and method names become lowerCamelCase by removing underscores and capitalizing the following components; `to_string_repr` becomes `toStringRepr`. Struct names become Kotlin class names in PascalCase. Parameters retain their original names. Duplicate names and Kotlin collisions produce errors.

Generated Kotlin contains:
- An `object RustApi` with static JNI bridge methods.
- Public Kotlin classes (`class Vector internal constructor(internal var nativeHandle: kotlin.Long) : java.lang.AutoCloseable`).
- Public Kotlin `enum class` for simple fieldless enums.
- Public Kotlin `sealed class` with nested `data class` / `data object` variants for data-bearing enums.
- Public Kotlin `data class` for pass-by-value models.
- Public Kotlin `fun interface` for single-method callback traits (enabling idiomatic trailing lambdas via SAM conversion).
- Public Kotlin `interface` for multi-method callback traits.

The object initializer calls `System.loadLibrary` with the configured base name, such as `ketox_hello`. The caller supplies the native directory through `java.library.path`. Loading errors add the library name, OS, architecture, and search path while retaining the underlying cause.

## Handle Registry and State Management

`ketox-jni` provides a thread-safe handle registry:
- State is held as `Arc<RwLock<T>>`, allowing concurrent immutable calls (`&self`) and serialized mutable calls (`&mut self`).
- `register_handle` assigns a unique 64-bit integer handle to new instances.
- `with_handle` and `with_handle_mut` look up handles safely with automatic stale-handle detection.
- `destroy_handle` releases the instance when closed from Kotlin. Double closes are safely ignored.

## Callbacks and Cross-Thread Execution

`ketox-jni` and `ketox-codegen` provide full support for bidirectional interaction:
- Callback traits (`#[kotlin_callback]`) are represented in Rust as proxy structs holding `JavaVM` and `GlobalRef`.
- Single-method traits map to Kotlin `fun interface` (supporting idiomatic trailing lambdas); multi-method traits map to Kotlin `interface`.
- Callbacks can be passed as `Box<dyn Trait>`, `Box<dyn Trait + Send + Sync>`, and `Option<Box<dyn Trait>>`.
- Both synchronous and cross-thread callbacks (`std::thread::spawn`) are supported via `with_callback_env`. If called on an unattached Rust thread, it attaches to the JVM as a daemon thread and automatically detaches upon thread completion.
- Any Kotlin exception thrown inside a callback is cleanly caught, cleared, and extracted before thread detachment, preventing JVM fatal errors and safely propagating back across the `ketox_jni::boundary`.
- Zero leaked global references: when the proxy struct drops in Rust, its `GlobalRef` is released.

## Compatibility and platforms

Metadata uses explicit schema version 5 (supporting versions 1, 2, 3, 4, and 5). Unknown fields and unsupported versions are rejected; semantic validation runs before generation. Source-discovered functions, classes, enums, models, and callbacks are deterministically ordered by their Rust names.

Rust 1.88 is the declared minimum compiler version, with edition 2024. The JNI crate is pinned to `0.21.1`. The CI configuration targets Windows, Linux, and macOS with stable Rust, JDK 21, and Kotlin 2.2.0.
