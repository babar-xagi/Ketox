# Changelog

## 0.0.5 development (Phase 5)

- Implemented bidirectional Callbacks & JVM interaction: Rust calling Kotlin functions, interfaces, and lambdas.
- Added `#[kotlin_callback]` attribute macro for exporting Rust traits as Kotlin interfaces.
- Implemented Single-Method Callbacks: mapped to Kotlin `fun interface` (supporting idiomatic trailing lambdas via SAM conversion).
- Implemented Multi-Method Callbacks: mapped to Kotlin `interface` for complex multi-event listeners.
- Supported Rust trait object signatures: `Box<dyn Trait>`, `Box<dyn Trait + Send + Sync>`, and `Option<Box<dyn Trait>>`.
- Implemented generated proxy structs `__KetoxCallback_{Trait}` wrapping `JavaVM` and `GlobalRef`, implementing the Rust trait.
- Supported Synchronous Callbacks: direct JNI invocations on the active thread with scoped local reference frames (`with_local_frame`).
- Supported Cross-Thread Callbacks: invoked from background worker threads spawned with `std::thread::spawn`.
- Implemented Daemon Thread Lifecycle: automatic JVM thread attachment (`AttachCurrentThreadAsDaemon`) and automatic detachment on thread exit.
- Implemented Safe Exception Extraction & Clearance: uncaught Kotlin exceptions inside callbacks are captured and cleared prior to thread detachment, preventing JVM fatal termination, with safe panic containment via `ketox_jni::boundary`.
- Zero Leaked References Guarantee: `GlobalRef` is automatically freed when the Rust trait object is dropped; zero local reference table leaks.
- Bumped metadata schema to version 5 with backward compatibility for versions 1, 2, 3, 4, and 5.
- Added comprehensive unit tests in `ketox-core` and `ketox-codegen`, updated trybuild UI tests, and added end-to-end JVM integration checks in `Smoke.kt` executed with `-Xcheck:jni`.

## 0.0.4 development (Phase 4)

- Implemented Simple Enums: Rust C-like fieldless enums ↔ Kotlin `enum class` with JNI ordinal dispatch and static variant fields.
- Implemented Data-bearing Enums / ADTs: Rust enums with payload ↔ Kotlin `sealed class` with `data class` (for variants with fields) and `data object` (for fieldless variants) subclasses, supporting exhaustive `when` matching without `else`.
- Implemented Data Models / Value Structs: Rust structs annotated with `#[kotlin_model]` (or `#[kotlin_data]`) ↔ Kotlin `data class`, supporting nested models, enums, options, and collections with pass-by-value semantics and GC lifecycle.
- Added Rich Collections support: `Vec<String>` and `&[String]` mapped to Kotlin `Array<String>` (`[Ljava/lang/String;`), alongside nullable collection variants (`Option<Vec<String>>`).
- Added `#[kotlin_enum]` and `#[kotlin_model]` attribute macros to `ketox-macros` and exported them in `ketox` and `ketox::prelude`.
- Bumped metadata schema to version 4 with backward compatibility for versions 1, 2, 3, and 4.
- Added recursive field reading and writing code generation for arbitrarily nested models and enums.
- Updated `ketox-codegen` to emit type converters for enum ordinals and model constructors/field getters.
- Added comprehensive unit, contract validation, and trybuild compile tests for Phase 4 enums, models, and collections.
- Added end-to-end JVM integration tests in `integration-tests/jvm/Smoke.kt` executed with `-Xcheck:jni` verifying simple enums, sealed classes, nested models, and string arrays.
- Updated documentation across `README.md`, `ROADMAP.md`, `architecture.md`, `metadata.md`, `metadata.schema.json`, `ownership.md`, and `type-system.md`.

## 0.0.3 development (Phase 3)

- Implemented Rust Structs ↔ Kotlin Classes mapping.
- Added `#[kotlin_class]` attribute macro for exposing Rust structs.
- Added `#[kotlin_constructor]` attribute macro and constructor detection for `new`, `Self`, and `Result<Self, E>` returns.
- Extended `#[kotlin_export]` to support `impl StructName` blocks.
- Added support for methods with `&self` (immutable) and `&mut self` (mutable) receivers.
- Implemented global, thread-safe `HANDLE_REGISTRY` in `ketox-jni` storing instances in `Arc<RwLock<T>>` for safe concurrent reads and serialized writes.
- Implemented robust handle lifecycle functions: `register_handle`, `get_handle_arc`, `with_handle`, `with_handle_mut`, and `destroy_handle`.
- Added stale-handle and double-close protection: calling methods on a destroyed handle throws `IllegalStateException`, while subsequent `close()` invocations are safe no-ops.
- Added support for cross-type references (`&OtherClass`) in methods.
- Generated Kotlin classes implementing `java.lang.AutoCloseable` with `nativeHandle: Long`, `checkAlive()`, methods, and `close()`.
- Updated metadata schema to version 3 with `classes`, `constructors`, and `methods`.
- Added JVM smoke test assertions verifying struct creation, method invocation, mutation, `.use { ... }`, stale-handle exception, and double-close safety under `-Xcheck:jni`.

## 0.0.2 development (Phase 2)

- Added metadata schema version 2 supporting `Option`, `Result`, and array types.
- Implemented `Option<T>` for arguments and return types, mapping to Kotlin nullable types `T?` with JNI boxed primitive/object conversions.
- Implemented `Result<T, E>` return types, mapping directly to Kotlin return type `T` and translating `Err` into JVM `RuntimeException`.
- Added high-performance byte array support (`Vec<u8>`, `&[u8]`) mapping to Kotlin `ByteArray`.
- Added primitive array support (`IntArray`, `LongArray`, `FloatArray`, `DoubleArray`, `BooleanArray`) for `Vec<T>` and `&[T]`.
- Enforced array size limits (`MAX_ARRAY_ELEMENTS = 16M`) and null-safety across all array conversions.
- Added comprehensive unit, compile, and JVM integration tests (`-Xcheck:jni`) verifying all Phase 2 types.

## 0.0.1 development (Phase 0 & Phase 1)

- Adopted the Ketox project identity throughout the design and new workspace.
- Added a public facade, shared metadata/type validation, export attribute macro, binding generator, and JNI runtime.
- Added a build-script-driven Rust-to-Kotlin/JVM example with primitive and string conversions.
- Added `ketox generate` and `ketox inspect` CLI commands.
- Added naming validation, deterministic generated artifacts, a versioned metadata contract, and explicit unsupported-signature diagnostics.
- Added panic containment, null-string rejection, strict UTF-16 conversion, and bounded string sizes.
- Added Rust checks, JVM integration runners, and Windows/Linux/macOS CI configuration.
- Documented current contracts and separated implemented prototype behavior from the long-term roadmap.
