# ketox3 development roadmap

The project goal is to make Rust libraries natural, safe, and performant to call from Kotlin. The current implementation targets Kotlin/JVM and establishes progressive development milestones.

The [full design and roadmap](KETOX_FULL_PROJECT_DESIGN_AND_ROADMAP.md) remains the reference for long-term vision.

## Milestone: Phase 0 & Phase 1 — Functions MVP (Completed)

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

## Milestone: Phase 3 — Rust Structs ↔ Kotlin Classes (Completed)

- [x] Extend metadata schema to version 3 with `classes`, `constructors`, and `methods`.
- [x] Add `#[kotlin_class]` attribute macro for exporting Rust structs.
- [x] Add `#[kotlin_constructor]` attribute macro and recognition of `new` / `Self` / `Result<Self, E>` constructor returns.
- [x] Extend `#[kotlin_export]` to support `impl StructName` blocks.
- [x] Support methods with immutable `&self` and mutable `&mut self` receivers.
- [x] Implement thread-safe `HANDLE_REGISTRY` in `ketox-jni` storing instances in `Arc<RwLock<T>>` for concurrent reads and exclusive writes.
- [x] Implement robust handle lifecycle functions: `register_handle`, `get_handle_arc`, `with_handle`, `with_handle_mut`, and `destroy_handle`.
- [x] Implement stale-handle and double-close protection (`IllegalStateException` on stale handle use; safe idempotent double close).
- [x] Implement cross-type parameter references (e.g. `fn dot(&self, other: &Vector) -> f64`).
- [x] Generate Kotlin classes implementing `java.lang.AutoCloseable` with `nativeHandle: Long`, `checkAlive()`, methods, and `close()`.
- [x] Verify complete Phase 3 on JVM with `-Xcheck:jni` covering instantiation, methods, mutation, cross-class calls, `.use { ... }`, stale handles, and double closes.

## Milestone: Phase 4 — Enums, Models, and Collections (Completed)

- [x] Extend metadata schema to version 4 with `enums` and `models`.
- [x] Add `#[kotlin_enum]` attribute macro for exporting simple and data-bearing enums.
- [x] Add `#[kotlin_model]` (and `#[kotlin_data]`) attribute macro for exporting data structs.
- [x] Implement Simple Enums: Rust fieldless enums ↔ Kotlin `enum class` with fast ordinal dispatch and static variant fields.
- [x] Implement Data-bearing Enums / ADTs: Rust enums with payload ↔ Kotlin `sealed class` with nested `data class` and `data object` variants.
- [x] Implement Data Models / Structs: Rust structs ↔ Kotlin `data class`, supporting nested models, enums, options, and collections.
- [x] Implement Rich Collections: `Vec<String>` and `&[String]` mapped to Kotlin `Array<String>` (`[Ljava/lang/String;`).
- [x] Implement comprehensive type resolution and cycle-safe validation in `ketox-core`.
- [x] Implement JNI conversion helpers and recursive nested field readers/writers in `ketox-codegen` and `ketox-jni`.
- [x] Verify complete Phase 4 on JVM with `-Xcheck:jni` covering simple enums, sealed classes with `when` pattern matching, data models with nested structures, and string collections.

## Milestone: Phase 5 — Callbacks & JVM Interaction (Completed)

- [x] Extend metadata schema to version 5 with `callbacks` and `Type::Callback`.
- [x] Add `#[kotlin_callback]` attribute macro for exporting Rust traits as Kotlin interfaces.
- [x] Implement Single-Method Callbacks: mapped to Kotlin `fun interface` (supporting idiomatic trailing lambdas via SAM conversion).
- [x] Implement Multi-Method Callbacks: mapped to Kotlin `interface` for complex event listeners.
- [x] Implement Rust Trait Object Support: pass callbacks as `Box<dyn Trait>`, `Box<dyn Trait + Send + Sync>`, and `Option<Box<dyn Trait>>`.
- [x] Implement Generated Proxy Structs: `__KetoxCallback_{Trait}` wrapping `JavaVM` and `GlobalRef`, implementing the Rust trait.
- [x] Implement Synchronous Callbacks: direct JNI invocations on the active thread with local frame cleanup (`with_local_frame`).
- [x] Implement Cross-Thread Callbacks: worker threads spawned via `std::thread::spawn` calling Kotlin callbacks.
- [x] Implement Daemon Thread Lifecycle: automatic JVM thread attachment via `AttachCurrentThreadAsDaemon` and automatic detachment on thread exit.
- [x] Implement Exception Propagation & Safe Detach: cleanly inspect, extract, and clear pending Kotlin exceptions before thread detachment, containing errors safely via `ketox_jni::boundary`.
- [x] Implement Zero Leaks Guarantee: automatic GlobalRef destruction when Rust trait object drops; zero local reference table leaks.
- [x] Verify complete Phase 5 on JVM with `-Xcheck:jni` covering synchronous lambdas, return-value filter callbacks, cross-thread worker execution, multi-method listeners, and exception recovery.

## Validation status

| Environment | Status |
| --- | --- |
| Local Windows Rust checks | All tests, lints, and formatting pass (`cargo test --workspace`) |
| Local Windows JVM checks | Passed: primitives, strings, nulls, Unicode, limits, panics, threads, Option, Result, ByteArrays, IntArrays, Vector class, methods, mutation, AutoCloseable, stale handle protection, Simple Enums, Sealed Classes / ADTs, Data Models, String Arrays, Callbacks (SAM trailing lambdas, multi-method, cross-thread daemon threads, exception recovery) (`-Xcheck:jni`) |
| GitHub Actions Windows/Linux/macOS | Configured; tested locally |
| Rust minimum-version check | Verified |
| Android and Kotlin/Native | Planned for future phases |

## Next work (Phase 6)

1. Async integration: Kotlin coroutines and Rust futures / channels (`suspend fun` ↔ `async fn`).
2. Android distribution (AAR packaging) and Gradle plugin integration.
3. Kotlin Multiplatform / Kotlin/Native backend.

