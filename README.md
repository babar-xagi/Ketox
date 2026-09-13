# Ketox

Ketox generates Kotlin/JVM bindings for Rust libraries. Its goal is to make Rust functions usable from Kotlin through a small annotation and generated JNI glue.

This repository contains the **0.0.1 development prototype**: the architecture contracts and an initial Phase 1 functions implementation. The [full project design](KETOX_FULL_PROJECT_DESIGN_AND_ROADMAP.md) describes the longer-term vision; [ROADMAP.md](ROADMAP.md) tracks what exists and what remains. This is not a published or stable release.

```rust
use ketox::kotlin_export;

#[kotlin_export]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[kotlin_export]
pub fn hello(name: String) -> String {
    format!("Hello, {name}!")
}
```

The generated Kotlin object provides:

```kotlin
import dev.ketox.example.RustApi

println(RustApi.add(20, 22))
println(RustApi.hello("Kotlin"))
```

## Run the example and tests

Install Rust (minimum declared version 1.88), a native linker for your platform, JDK 21, and the standalone Kotlin compiler. CI pins [Kotlin 2.2.0](https://github.com/JetBrains/kotlin/releases/tag/v2.2.0) for reproducibility. Follow the [official compiler setup instructions](https://kotlinlang.org/docs/command-line.html), then put `kotlinc/bin` and Java on `PATH`. The Bash runner also requires Python 3 to read Cargo's artifact paths.

From the repository root:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Run the generated library on a real JVM, with JNI checking enabled:

```powershell
# Windows / PowerShell
./scripts/test-jvm.ps1

# If Kotlin is not on PATH:
./scripts/test-jvm.ps1 -Kotlinc 'C:\tools\kotlinc\bin\kotlinc.bat'
```

```bash
# Linux / macOS
bash scripts/test-jvm.sh

# If Kotlin is not on PATH:
bash scripts/test-jvm.sh '/path/to/kotlinc/bin/kotlinc'
```

Both runners also accept the `KETOX_KOTLINC` environment variable. They build `ketox-hello`, obtain the matching generated source and native library paths from Cargo JSON, compile `RustApi.kt` with the JVM integration checks, and run `java -Xcheck:jni`. The test JAR is written to Cargo's target directory under `ketox-jvm/smoke.jar`. Missing tools and failed checks produce errors; tests are never silently skipped.

## How generation works

The working example is [examples/hello-world](examples/hello-world). Its `Cargo.toml` uses `ketox` as a dependency and `ketox-codegen` as a build dependency, with `crate-type = ["cdylib", "rlib"]` and native library name `ketox_hello`.

Its `build.rs` calls:

```rust
fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    let out_dir = std::env::var("OUT_DIR").expect("Cargo sets OUT_DIR");
    ketox_codegen::generate_from_file(
        "src/lib.rs",
        "dev.ketox.example",
        "RustApi",
        "ketox_hello",
        out_dir,
    )
    .expect("generate Ketox bindings");
}
```

The Rust library includes the generated JNI entry points at its crate root:

```rust
include!(concat!(env!("OUT_DIR"), "/ketox_jni.rs"));
```

Generation writes `ketox_jni.rs`, `RustApi.kt`, and `ketox-metadata.json` to Cargo's `OUT_DIR`. The Kotlin object loads the native library with `System.loadLibrary("ketox_hello")`; the integration runner supplies its directory through `java.library.path`. Native binaries are built for the host platform. Packaging native libraries into JARs or Android artifacts is future work.

## Command-line generation

The workspace includes the `ketox` CLI. To validate the example's exports and print their metadata:

```text
cargo run --locked -p ketox-cli -- inspect --source examples/hello-world/src/lib.rs --package dev.ketox.example --class RustApi --library ketox_hello
```

To write the three generated artifacts to a directory for inspection:

```text
cargo run --locked -p ketox-cli -- generate --source examples/hello-world/src/lib.rs --package dev.ketox.example --class RustApi --library ketox_hello --out target/ketox-generated
```

Both commands require the source, package, object name (`--class`), and library name. `generate` additionally requires `--out`; `inspect` writes JSON to standard output. `cargo run --locked -p ketox-cli -- --help` lists the options. The build helper remains the example's automatic generation path.

## Current boundaries

Exports must be public, safe, synchronous, non-generic functions declared directly in the source file passed to codegen. Supported values are `bool`, signed fixed-width integers, floating-point values, and `String`; `&str` is accepted as an input and `()` as a return. Rust snake_case function names become Kotlin lowerCamelCase names. Unsupported declarations and naming collisions produce diagnostics.

Strings are copied across the boundary. Null inputs and malformed UTF-16 are rejected. Rust panics are caught and reported as JVM exceptions when compiled with `panic = "unwind"`. See the [type contract](docs/type-system.md) and [ownership and error contract](docs/ownership.md) for exact limits.

`Option`, `Result`, collections, exported classes, native handles, callbacks, async functions, Android packaging, Gradle integration, and Kotlin/Native are planned and are not implemented.

## Repository

| Path | Responsibility |
| --- | --- |
| `crates/ketox` | User-facing facade and `#[kotlin_export]` re-export |
| `crates/ketox-core` | Metadata, supported types, source validation and naming |
| `crates/ketox-macros` | Attribute validation and per-function metadata constants |
| `crates/ketox-codegen` | Deterministic Kotlin, JNI Rust, and JSON generation |
| `crates/ketox-jni` | String conversion and panic/exception boundary |
| `crates/ketox-cli` | `ketox generate` and `ketox inspect` commands |
| `examples/hello-world` | Build-script-driven native library |
| `integration-tests/jvm` | End-to-end JVM checks |
| `docs` | [Architecture](docs/architecture.md), [types](docs/type-system.md), [ownership](docs/ownership.md), [metadata](docs/metadata.md) |

CI is configured for Windows, Linux, and macOS with stable Rust, JDK 21, and Kotlin 2.2.0, plus a Rust 1.88 compile check. Configured jobs do not imply those platforms have already been verified; [ROADMAP.md](ROADMAP.md) records validation status.
