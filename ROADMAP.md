# Ketox development roadmap

The project goal is to make Rust libraries natural and safe to call from Kotlin. The current implementation targets Kotlin/JVM and establishes the development milestones.

The [full design and roadmap](KETOX_FULL_PROJECT_DESIGN_AND_ROADMAP.md) remains the reference for future direction. Its phase numbers and proposed release versions are planning targets, not completed features or release commitments.

## Milestone: Phase 0 & Phase 1 (Completed)

- [x] Rename the project and package namespace to Ketox.
- [x] Set up the Cargo workspace and user-facing `ketox` crate.
- [x] Document architecture, type mapping, naming, ownership, exceptions, loading, and compatibility rules.
- [x] Define metadata schema version 1 and shared validation.
- [x] Implement `#[kotlin_export]` validation and per-export metadata constants.
- [x] Implement single-source-file discovery and deterministic Kotlin/JNI/JSON generation.
- [x] Add a Cargo build helper and a native hello-world example.
- [x] Add `ketox generate` and `ketox inspect` CLI commands.
- [x] Support Phase 1 primitive values, copied strings, borrowed string inputs, and Unit returns.
- [x] Add panic containment and explicit null/UTF-16/string-size error handling.
- [x] Add Rust unit/compile tests and real JVM integration checks.
- [x] Add Windows and Unix test runners and a three-OS CI configuration.
- [x] Confirm the complete local Rust and JVM checks pass.

## Milestone: Phase 2 — Rich Types, Error Handling, and Byte Fast-Paths (Completed)

- [x] Extend metadata schema to version 2 with Option, Result, and array type variants.
- [x] Support `Option<T>` for arguments and return types mapped to Kotlin nullable `T?` (handling boxed primitives and nullable references).
- [x] Support `Result<T, E>` return types mapped to Kotlin `T`, translating `Err(e)` to JVM `java.lang.RuntimeException`.
- [x] Implement byte array fast-paths: `Vec<u8>` and `&[u8]` mapped to Kotlin `ByteArray` (JNI `[B`).
- [x] Implement primitive array support: `IntArray`, `LongArray`, `FloatArray`, `DoubleArray`, `BooleanArray` for `Vec<T>` and `&[T]`.
- [x] Implement array bounds checking (`MAX_ARRAY_ELEMENTS = 16M`) and null-safety.
- [x] Update codegen and runtime for automatic conversion between Kotlin/JNI and Rust types.
- [x] Verify end-to-end on JVM with `-Xcheck:jni` covering all Phase 2 features.

## Validation status

| Environment | Status |
| --- | --- |
| Local Windows Rust checks | All tests, lints (`clippy -D warnings`), and formatting pass |
| Local Windows JVM checks | Passed: primitives, strings, nulls, Unicode, limits, panics, threads, Option, Result, ByteArrays, IntArrays |
| GitHub Actions Windows/Linux/macOS | Configured; tested locally |
| Rust minimum-version check | Verified |
| Android and Kotlin/Native | Planned for future phases |

## Next work (Phase 3)

1. Design explicit native ownership, opaque handles, and stale-handle protection for exported Rust structs/classes.
2. Implement struct/class exports with constructor methods, methods, and automatic destructors (`AutoCloseable` in Kotlin).
3. Rich collections (e.g. `Vec<String>`, maps) and complex object graphs.
4. Callbacks, Kotlin coroutines, and async integration.
5. Android distribution and Gradle plugin integration.
