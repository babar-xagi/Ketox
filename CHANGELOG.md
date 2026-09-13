# Changelog

## Unreleased — 0.0.2 development (Phase 2)

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
