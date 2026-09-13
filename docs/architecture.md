# Ketox architecture contract

Status: initial `0.0.1` development contract. The backend is Kotlin/JVM through JNI. The existing implementation is a functions prototype; later phases in the full roadmap are not implicit API support.

## Build and call paths

```text
Rust source with #[kotlin_export]
    -> shared signature/type validation in ketox-core
    -> metadata module (schema version 1)
    -> ketox-codegen
        -> ketox_jni.rs (included by the Rust library)
        -> RustApi.kt (compiled by Kotlin)
        -> ketox-metadata.json

Kotlin object method
    -> JNI native symbol in the Rust cdylib
    -> exception/panic boundary and value conversion
    -> the original Rust function
```

`ketox` is the facade used by application crates. `ketox-macros` validates annotations using `ketox-core` and emits one hidden JSON constant per function. Macros do not write generated files or depend on invocation order. A `build.rs` calls `ketox-codegen::generate_from_file` and owns filesystem output. Both paths use the same validation logic; the build helper discovers source declarations directly rather than reading constants from compiled binaries.

`ketox-core` depends on syntax and serialization libraries, not JNI. It validates exported signatures, computes Kotlin names and JVM type descriptors, and produces portable metadata. `ketox-codegen` turns valid metadata into inspectable Rust, Kotlin, and JSON. `ketox-jni` owns JNI conversion and the panic/exception boundary. `ketox-cli` exposes the same discovery and generation path through `ketox inspect` and `ketox generate`. There is no separate handle registry or runtime scheduler yet.

## Source discovery

The prototype reads one UTF-8 Rust source file and recognizes direct top-level `#[kotlin_export]` and `#[ketox::kotlin_export]` functions. It does not expand macros, resolve imports/type aliases, discover exports in separate module files, or evaluate conditional compilation. Inline nested exports and conditional attributes on supported exports are rejected. Keep exports directly in the file supplied to codegen; an export in another file is outside this discovery contract.

Supported declarations are unrestricted `pub fn` functions with simple parameter identifiers. Async, unsafe, const, extern, variadic, generic, `where` clauses, methods, borrowed returns, and unsupported types are rejected. Generation validates its full input before writing output. The build script must emit `cargo:rerun-if-changed` for its input source. The generated Rust must be included at the same crate root as the source functions.

## Naming and loading

The configured package is a nonempty dot-separated sequence of validated identifiers; the configured object name is a validated identifier. This example uses `dev.ketox.example.RustApi`. Rust function names become lowerCamelCase by removing underscores and capitalizing the following components; `text_length` becomes `textLength`. Parameters retain their original names. ASCII identifiers are required; raw identifiers, reserved Kotlin words, and inherited object method names are rejected where applicable. Duplicate Rust names and Kotlin name collisions are errors. Overloads and custom name overrides are not supported.

Generated Kotlin uses an `object` with instance `external fun` methods. JNI entry points use `extern "system"`, a `JNIEnv`, and the object receiver. Symbol names follow JNI escaping, including underscore escaping. This follows the JVM's native-method lookup model in the [JNI design specification](https://docs.oracle.com/en/java/javase/21/docs/specs/jni/design.html).

The object initializer calls `System.loadLibrary` with the configured base name, such as `ketox_hello`. The caller supplies the native directory through `java.library.path`. Loading errors add the library name, OS, architecture, and search path while retaining the underlying cause. Automatic extraction from JARs, multiple class-loader handling, Android packaging, and platform-binary distribution remain future work.

## Compatibility and platforms

Metadata uses explicit schema version 1. Unknown fields and unsupported versions are rejected; semantic validation runs before generation. Source-discovered functions are ordered by Rust name. Identical validated input produces identical generated content. See [metadata.md](metadata.md) and the [JSON schema](metadata.schema.json).

Rust 1.88 is the declared minimum compiler version, with edition 2024. This baseline includes the workspace's compile-test dependencies. The JNI crate is pinned to `0.21.1` because the runtime and generated glue use that API deliberately; it is not a claim about the latest release. Its ownership and conversion APIs are documented in the [versioned jni crate reference](https://docs.rs/jni/0.21.1/jni/).

The initial CI configuration targets Windows, Linux, and macOS, stable Rust, JDK 21, and Kotlin 2.2.0. A separate job checks Rust 1.88 compilation. Only executed tests establish support: [ROADMAP.md](../ROADMAP.md) records the current evidence. Consumers must build compatible native binaries for their JVM's OS and architecture. This prototype promises neither a stable Rust API nor binary compatibility across Ketox versions; regenerate and rebuild both sides together.
