# ketox3

ketox3 generates Kotlin/JVM bindings for Rust libraries. Its goal is to make Rust functions and data structures feel natural, ergonomic, and safe to call from Kotlin through annotations and generated JNI glue.

This repository contains **ketox3** with completed:
- **Phase 1**: Functions and primitive/string conversions
- **Phase 2**: Rich types, `Option<T>`, `Result<T, E>`, and primitive array slices
- **Phase 3**: Rust Structs ↔ Kotlin Classes, thread-safe Handle Registry, and AutoCloseable object lifecycle
- **Phase 4**: Enums, Sealed Classes / ADTs, Pass-by-Value Data Models, and String Collections
- **Phase 5**: Callbacks & JVM Interaction: Rust calling Kotlin functions, interfaces, and lambdas with cross-thread execution

The [full project design](KETOX_FULL_PROJECT_DESIGN_AND_ROADMAP.md) describes the long-term vision; [ROADMAP.md](ROADMAP.md) tracks milestones and implementation progress.

```rust
use ketox::{
    kotlin_callback, kotlin_class, kotlin_constructor, kotlin_enum, kotlin_export, kotlin_model,
};

// Top-level exported functions
#[kotlin_export]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Simple C-like fieldless enum -> Kotlin enum class
#[kotlin_enum]
pub enum Status {
    Idle,
    Running,
    Completed,
}

// Data-bearing ADT enum -> Kotlin sealed class with data class / object variants
#[kotlin_enum]
pub enum Shape {
    Circle(f64),
    Rectangle { w: f64, h: f64 },
    Point,
}

// Pass-by-value data model -> Kotlin data class
#[kotlin_model]
pub struct UserProfile {
    pub id: i64,
    pub username: String,
    pub status: Status,
}

// Stateful class with lifecycle -> Kotlin class implementing AutoCloseable
#[kotlin_class]
pub struct Vector {
    x: f64,
    y: f64,
}

#[kotlin_export]
impl Vector {
    #[kotlin_constructor]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn scale(&mut self, factor: f64) {
        self.x *= factor;
        self.y *= factor;
    }

    pub fn dot(&self, other: &Vector) -> f64 {
        self.x * other.x + self.y * other.y
    }
}

// Rich collections -> Kotlin Array<String>
#[kotlin_export]
pub fn filter_names(names: &[String], prefix: &str) -> Vec<String> {
    names.iter().filter(|n| n.starts_with(prefix)).cloned().collect()
}

// Single-method callback -> Kotlin fun interface (SAM trailing lambdas)
#[kotlin_callback]
pub trait ProgressListener {
    fn on_progress(&self, current: i32, total: i32, message: String);
}

#[kotlin_export]
pub fn download(url: String, listener: Box<dyn ProgressListener>) {
    listener.on_progress(100, 100, format!("Downloaded {url}"));
}

// Cross-thread callback executed from background thread
#[kotlin_export]
pub fn run_in_background(listener: Box<dyn ProgressListener + Send + Sync + 'static>) {
    std::thread::spawn(move || {
        listener.on_progress(100, 100, "Background worker done".to_string());
    })
    .join()
    .unwrap();
}
```

The generated Kotlin bindings provide:

```kotlin
import dev.ketox.example.RustApi
import dev.ketox.example.Status
import dev.ketox.example.Shape
import dev.ketox.example.UserProfile
import dev.ketox.example.Vector

// 1. Exported functions
println(RustApi.add(20, 22))

// 2. Simple enums
val status = Status.Running
println("Status: $status")

// 3. Sealed classes / ADTs with exhaustive pattern matching
val shape: Shape = Shape.Circle(5.0)
val area = when (shape) {
    is Shape.Circle -> Math.PI * shape.radius * shape.radius
    is Shape.Rectangle -> shape.width * shape.height
    is Shape.Point -> 0.0
}
println("Area: $area")

// 4. Pass-by-value data models
val user = UserProfile(1L, "alice", null, Status.Idle)
val updatedUser = user.copy(status = Status.Running)

// 5. Stateful objects implement java.lang.AutoCloseable
Vector(3.0, 4.0).use { v1 ->
    println("Magnitude: ${v1.magnitude()}") // 5.0
    v1.scale(2.0)
    println("Scaled magnitude: ${v1.magnitude()}") // 10.0

    Vector(1.0, 2.0).use { v2 ->
        println("Dot product: ${v1.dot(v2)}")
    }
}

// 6. Rich collections
val filtered = RustApi.filterNames(arrayOf("apple", "banana", "apricot"), "ap")
println(filtered.toList()) // [apple, apricot]

// 7. Callbacks with idiomatic trailing lambdas (SAM conversion)
RustApi.download("https://example.com/file") { current, total, msg ->
    println("$current/$total: $msg")
}

// 8. Cross-thread callbacks from background worker threads
RustApi.runInBackground { current, total, msg ->
    println("From worker: $msg")
}
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

Generation writes `ketox_jni.rs`, `RustApi.kt`, and `ketox-metadata.json` to Cargo's `OUT_DIR`. The Kotlin object loads the native library with `System.loadLibrary("ketox_hello")`; the integration runner supplies its directory through `java.library.path`. Native binaries are built for the host platform. Packaging native libraries into JARs or Android artifacts is planned for subsequent phases.

## Command-line generation

The workspace includes the `ketox` CLI. To validate the example's exports and print their metadata:

```text
cargo run --locked -p ketox-cli -- inspect --source examples/hello-world/src/lib.rs --package dev.ketox.example --class RustApi --library ketox_hello
```

To write the three generated artifacts to a directory for inspection:

```text
cargo run --locked -p ketox-cli -- generate --source examples/hello-world/src/lib.rs --package dev.ketox.example --class RustApi --library ketox_hello --out target/ketox-generated
```

Both commands require the source, package, object name (`--class`), and library name. `generate` additionally requires `--out`; `inspect` writes JSON to standard output. `cargo run --locked -p ketox-cli -- --help` lists the options.

## Features and boundaries

### Supported types:
- **Primitives:** `bool`, signed fixed-width integers (`i8`, `i16`, `i32`, `i64`), and floating-point values (`f32`, `f64`)
- **Strings:** `String`, and borrowed `&str` inputs
- **Unit:** `()` as a return type
- **Nullability:** `Option<T>` for parameters and returns, mapped to Kotlin `T?` (with boxed primitives or nullable references)
- **Errors:** `Result<T, E>` returns, returning `T` directly to Kotlin and converting `Err` to JVM `RuntimeException`
- **Primitive Arrays:** Byte arrays (`Vec<u8>`, `&[u8]`) mapped to Kotlin `ByteArray`, and primitive arrays (`IntArray`, `LongArray`, `FloatArray`, `DoubleArray`, `BooleanArray`) for `Vec<T>` and `&[T]`
- **String Collections:** `Vec<String>` and `&[String]` mapped to Kotlin `Array<String>`
- **Simple Enums:** `#[kotlin_enum]` C-like fieldless enums mapped to Kotlin `enum class` with JNI ordinal dispatch
- **Data-Bearing Enums / ADTs:** `#[kotlin_enum]` enums with payloads mapped to Kotlin `sealed class` with `data class` / `data object` variants for exhaustive pattern matching
- **Data Models / Value Structs:** `#[kotlin_model]` (or `#[kotlin_data]`) structs mapped to Kotlin `data class`, supporting recursive nesting of models and enums
- **Classes & Structs:** `#[kotlin_class]` structs, `#[kotlin_constructor]` associated functions, and `#[kotlin_export] impl` methods (`&self` and `&mut self`). Managed via thread-safe `Arc<RwLock<T>>` handle registry with safe double-close and stale handle protection.
- **Callbacks & Lambdas:** `#[kotlin_callback]` traits mapped to Kotlin `fun interface` (single method SAM lambdas) and `interface` (multi-method), supported synchronously and across background threads (`std::thread::spawn`) with daemon thread lifecycle, zero reference leaks, and exception recovery.

Rust snake_case function and method names become Kotlin lowerCamelCase names. Struct and enum names become PascalCase Kotlin types. Unsupported declarations and naming collisions produce diagnostics.

Rust panics are caught and reported as JVM exceptions when compiled with `panic = "unwind"`. See the [type contract](docs/type-system.md), [ownership and error contract](docs/ownership.md), and [metadata specification](docs/metadata.md) for exact details.

## Repository

| Path | Responsibility |
| --- | --- |
| `crates/ketox` | User-facing facade, macros re-export, and prelude |
| `crates/ketox-core` | Metadata, supported types, source validation, and naming |
| `crates/ketox-macros` | Attribute macros (`#[kotlin_export]`, `#[kotlin_class]`, `#[kotlin_constructor]`, `#[kotlin_enum]`, `#[kotlin_model]`, `#[kotlin_callback]`) |
| `crates/ketox-codegen` | Deterministic Kotlin, JNI Rust, and JSON generation |
| `crates/ketox-jni` | String & collection conversion, panic boundary, thread-safe handle registry, and callback environment |
| `crates/ketox-cli` | `ketox generate` and `ketox inspect` commands |
| `examples/hello-world` | Build-script-driven native library with functions, classes, enums, and models |
| `integration-tests/jvm` | End-to-end JVM checks (`-Xcheck:jni`) |
| `docs` | [Architecture](docs/architecture.md), [types](docs/type-system.md), [ownership](docs/ownership.md), [metadata](docs/metadata.md) |

CI is configured for Windows, Linux, and macOS with stable Rust, JDK 21, and Kotlin 2.2.0, plus a Rust 1.88 compile check.
