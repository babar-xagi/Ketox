# Changelog

## Unreleased — 0.0.3 development (Phase 3)

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
