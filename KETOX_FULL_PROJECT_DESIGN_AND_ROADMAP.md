# 🌉 Ketox — Rust ↔ Kotlin Interop Toolkit

> **Project name:** `Ketox`  
> **Status:** Initial development — see ROADMAP.md for implementation progress  
> **Project type:** Open-source developer infrastructure  
> **Primary goal:** Make Rust libraries feel natural and safe to use from Kotlin, with an experience inspired by PyO3.

---

# 1. 🚀 Project Vision

Ketox is a developer toolkit for building **high-performance Rust libraries that can be consumed naturally from Kotlin**.

The long-term goal is to make this:

```rust
#[kotlin_export]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

usable from Kotlin like this:

```kotlin
val result = RustMath.add(10, 20)
println(result)
```

without developers manually writing:

- JNI bindings
- native method signatures
- pointer-management code
- object lifecycle glue
- repetitive Kotlin wrappers
- platform-specific loading code
- error-conversion boilerplate

Ketox should provide a high-level developer experience similar in spirit to:

```text
PyO3
Rust ↔ Python
```

but designed for:

```text
Ketox
Rust ↔ Kotlin/JVM
Rust ↔ Android
Rust ↔ Kotlin/Native
Kotlin Multiplatform (long-term)
```

---

# 2. 🎯 Core Product Goal

Ketox should let developers write performance-critical code in Rust and expose it to Kotlin through a simple, safe, generated API.

Target experience:

```text
Rust library
   ↓
Ketox annotations/macros
   ↓
Binding generator
   ↓
Native library + Kotlin API
   ↓
Kotlin application
```

The Kotlin developer should not need to understand JNI internals.

The Rust developer should not need to manually generate JVM signatures.

---

# 3. 🧠 Why Build Ketox?

Kotlin is excellent for:

- Android applications
- JVM backend development
- desktop applications
- multiplatform application logic
- developer productivity
- Java ecosystem integration

Rust is excellent for:

- systems programming
- CPU-intensive workloads
- low-level memory control
- parsers
- databases
- cryptography
- networking
- compression
- image/video processing
- AI runtimes
- high-performance data processing

Ketox connects both ecosystems.

---

# 4. 🔥 Example Use Cases

Ketox could enable Kotlin libraries backed internally by Rust for:

### 🤖 AI / Machine Learning

```text
Kotlin API
   ↓
Rust inference engine
   ↓
ONNX / native runtime / custom kernels
```

### 📊 Data Processing

```text
Kotlin
   ↓
Rust dataframe engine
   ↓
Arrow / SIMD / parallel execution
```

### 🗜 Compression

```text
Kotlin
   ↓
Rust
   ↓
high-performance compression
```

### 🔐 Cryptography

```text
Kotlin-friendly API
   ↓
Rust crypto implementation
```

### 🖼 Image Processing

```text
Android/Kotlin app
   ↓
Ketox
   ↓
Rust image engine
```

### 🗄 Embedded Databases

```text
Kotlin
   ↓
Ketox
   ↓
Rust storage engine
```

### 🌐 Networking

```text
Kotlin
   ↓
Rust async/network core
```

---

# 5. 🏗 High-Level Architecture

## Kotlin/JVM Path

```text
┌─────────────────────────┐
│      Kotlin Project     │
└────────────┬────────────┘
             │
             ▼
┌─────────────────────────┐
│ Generated Kotlin API    │
│ - classes               │
│ - functions             │
│ - exceptions            │
│ - ownership wrappers    │
└────────────┬────────────┘
             │
             ▼
┌─────────────────────────┐
│       JNI Boundary      │
└────────────┬────────────┘
             │
             ▼
┌─────────────────────────┐
│ Generated Rust JNI Glue │
└────────────┬────────────┘
             │
             ▼
┌─────────────────────────┐
│      Rust Library       │
└─────────────────────────┘
```

---

# 6. 🧩 Core Components

Ketox should eventually consist of the following major components:

```text
Ketox
├── ketox-core
├── ketox-macros
├── ketox-codegen
├── ketox-runtime
├── ketox-jni
├── ketox-cli
├── ketox-gradle-plugin
├── ketox-qutivex-plugin
├── ketox-native
└── ketox-examples
```

---

# 7. 📦 Component Responsibilities

## 7.1 `ketox-core`

Pure shared models.

Responsibilities:

- type definitions
- metadata model
- exported function model
- exported class model
- method signatures
- ownership rules
- error model
- ABI description
- platform description

No JNI-specific runtime logic should live here.

---

## 7.2 `ketox-macros`

Rust procedural macros.

Example:

```rust
#[kotlin_export]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

Future:

```rust
#[kotlin_class]
pub struct User {
    id: u64,
    name: String,
}
```

Future:

```rust
#[kotlin_export]
impl User {
    pub fn display_name(&self) -> String {
        self.name.clone()
    }
}
```

The macro layer should collect metadata, not hide unsafe behavior.

---

## 7.3 `ketox-codegen`

Generates:

```text
Rust JNI glue
Kotlin wrappers
metadata files
native-loader code
optional documentation
```

Input:

```text
Rust export metadata
```

Output:

```text
generated/kotlin/
generated/rust/
generated/metadata/
```

---

## 7.4 `ketox-jni`

Low-level JVM bridge.

Responsibilities:

- JNI calls
- JNI environment handling
- JVM primitive conversion
- Java/Kotlin string conversion
- arrays
- object handles
- exception translation
- callbacks
- thread attachment
- lifecycle handling

This should contain most unsafe JNI code so the rest of the project remains safe.

---

## 7.5 `ketox-runtime`

Small Kotlin runtime library.

Responsibilities:

- native library loading
- native handle ownership
- lifecycle helpers
- callback registration
- async completion bridges
- Kotlin-friendly exceptions
- platform checks

The runtime should remain lightweight.

---

## 7.6 `ketox-cli`

Developer-facing CLI.

Potential commands:

```bash
ketox init
ketox generate
ketox build
ketox doctor
ketox inspect
ketox clean
```

Example:

```bash
ketox init rust-core
```

```bash
ketox generate
```

```bash
ketox build
```

---

# 8. ✨ Target Developer Experience

## Rust

```rust
use ketox::prelude::*;

#[kotlin_export]
pub fn multiply(a: i64, b: i64) -> i64 {
    a * b
}
```

Generated Kotlin:

```kotlin
object RustMath {
    external fun multiply(a: Long, b: Long): Long
}
```

Developer usage:

```kotlin
val result = RustMath.multiply(6, 7)
```

---

# 9. 🧱 Classes and Structs

Target Rust:

```rust
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
}
```

Generated Kotlin concept:

```kotlin
class Vector private constructor(
    private var nativeHandle: Long
) : AutoCloseable {

    constructor(x: Double, y: Double) :
        this(nativeCreate(x, y))

    fun magnitude(): Double =
        nativeMagnitude(nativeHandle)

    override fun close() {
        nativeDestroy(nativeHandle)
        nativeHandle = 0L
    }
}
```

---

# 10. 🔢 Type Mapping

Initial type system should intentionally remain small.

## Phase 1 Types

| Rust | Kotlin |
|---|---|
| `bool` | `Boolean` |
| `i8` | `Byte` |
| `i16` | `Short` |
| `i32` | `Int` |
| `i64` | `Long` |
| `f32` | `Float` |
| `f64` | `Double` |
| `String` | `String` |
| `&str` | `String` |
| `()` | `Unit` |

---

## Phase 2 Types

| Rust | Kotlin |
|---|---|
| `Option<T>` | `T?` |
| `Vec<T>` | `List<T>` / primitive arrays |
| `Result<T,E>` | return value / Kotlin exception |
| structs | Kotlin class |
| enums | Kotlin enum/sealed type |

---

# 11. 🧠 Memory Ownership Model

Memory safety is one of the most important parts of Ketox.

A Rust object exposed to Kotlin may internally be represented as:

```text
Kotlin object
     ↓
nativeHandle: Long
     ↓
Rust allocation
```

Example:

```text
Vector.kt
nativeHandle = 0x000001AF...
```

Ketox must prevent:

- double free
- use-after-free
- invalid handles
- concurrent mutation races
- Rust panic crossing FFI boundary

---

# 12. 🔒 Handle Registry

Recommended initial architecture:

```text
Kotlin
   ↓ Long handle
Ketox JNI Runtime
   ↓
Rust Handle Registry
   ↓
Arc / Mutex / owned object
```

Possible Rust representation:

```rust
HandleId -> Arc<NativeObject>
```

Benefits:

- safer than passing raw pointers directly
- easier debugging
- type validation
- leak tracking
- future concurrency control

Performance-sensitive APIs may later support direct pointer strategies behind unsafe/advanced APIs.

---

# 13. 🧹 Object Lifecycle

Generated Kotlin classes should implement:

```kotlin
AutoCloseable
```

Example:

```kotlin
Vector(1.0, 2.0).use { vector ->
    println(vector.magnitude())
}
```

Ketox should support explicit cleanup.

GC/finalizer-based cleanup should only be a fallback safety mechanism.

---

# 14. ❌ Error Handling

Rust:

```rust
#[kotlin_export]
fn read_file(path: String) -> Result<String, FileError>
```

Kotlin target:

```kotlin
try {
    val value = readFile(path)
} catch (e: FileException) {
    ...
}
```

Architecture:

```text
Rust Result::Err
      ↓
Ketox error mapper
      ↓
JNI ThrowNew / generated exception
      ↓
Kotlin exception
```

Rust panics must never unwind across the JNI boundary.

Use:

```rust
catch_unwind
```

around exported entry points where appropriate.

---

# 15. ⚡ Async Architecture

One of the most important long-term features.

Target Rust:

```rust
#[kotlin_suspend]
pub async fn download(url: String) -> Result<Vec<u8>, NetworkError> {
    ...
}
```

Generated Kotlin:

```kotlin
suspend fun download(url: String): ByteArray
```

Potential flow:

```text
Kotlin coroutine
      ↓
generated suspend bridge
      ↓
Rust async runtime
      ↓
future completes
      ↓
JNI callback
      ↓
resume Kotlin continuation
```

Initial async runtime candidate:

```text
Tokio
```

Async support should not be attempted in the first MVP.

---

# 16. 🔁 Callbacks

Future Rust API:

```rust
#[kotlin_export]
pub fn process(
    input: String,
    callback: KotlinCallback<String>
) {
    ...
}
```

Kotlin:

```kotlin
process("hello") { result ->
    println(result)
}
```

Challenges:

- global JNI references
- callback lifecycle
- thread attachment
- JVM thread safety
- callback exceptions
- cancellation

Callbacks belong in a later phase after synchronous APIs are stable.

---

# 17. 🧵 Threading

Rust may execute work outside JVM-created threads.

Before calling JVM objects from a Rust worker thread:

```text
Rust thread
   ↓
AttachCurrentThread
   ↓
JNIEnv
   ↓
Kotlin callback
```

Ketox runtime must handle:

- attach
- detach
- local references
- global references
- thread-safe handles

---

# 18. 📱 Android Support

Android should become a major target after desktop JVM works.

Architecture:

```text
Android Kotlin
      ↓
Generated Kotlin API
      ↓
JNI
      ↓
Rust .so
```

Target ABIs:

```text
arm64-v8a
armeabi-v7a
x86_64
```

Later possibly:

```text
x86
```

Packaging target:

```text
AAR
```

Example output:

```text
ketox-my-lib.aar
├── classes.jar
└── jni/
    ├── arm64-v8a/libmylib.so
    ├── armeabi-v7a/libmylib.so
    └── x86_64/libmylib.so
```

---

# 19. 🖥 Desktop JVM Platforms

Initial platforms:

```text
Windows x64
Linux x64
macOS x64
macOS ARM64
```

Native outputs:

```text
Windows  → .dll
Linux    → .so
macOS    → .dylib
```

---

# 20. 🌍 Kotlin Multiplatform Vision

Long-term:

```text
                    Rust Core
                       │
            ┌──────────┴──────────┐
            │                     │
            ▼                     ▼
          JNI                   C ABI
            │                     │
            ▼                     ▼
      Kotlin/JVM           Kotlin/Native
            │
         Android
```

Possible future targets:

```text
JVM
Android
macOS
Linux
Windows
iOS
```

This is explicitly **not** an MVP requirement.

---

# 21. 🧬 Kotlin/Native Backend

Kotlin/Native does not require JNI.

Architecture:

```text
Rust
  ↓
C ABI
  ↓
generated headers
  ↓
Kotlin cinterop
  ↓
Kotlin/Native
```

This should use a separate backend while sharing Ketox metadata and type models.

---

# 22. 🛠 Build System Integration

Ketox should not force one build system.

Potential integrations:

```text
Ketox CLI
Gradle
Qutivex
Cargo
```

Core code generation should remain independent.

---

# 23. 🔥 Qutivex Integration

Ketox can become a first-class Qutivex capability later.

Possible project:

```text
my-app/
├── qutivex.toml
├── src/
│   └── main/kotlin/
└── native/
    └── rust/
        ├── Cargo.toml
        └── src/lib.rs
```

Possible commands:

```bash
qutivex native init rust
qutivex native build
qutivex run
```

Flow:

```text
Qutivex
   ↓
detect native Rust module
   ↓
Ketox
   ↓
Cargo build
   ↓
binding generation
   ↓
Kotlin compile
   ↓
package native library
```

Ketox should remain an independent project even if Qutivex supports it.

---

# 24. 🧪 Testing Strategy

Testing must exist at several levels.

## Unit Tests

Test:

- metadata parsing
- type mapping
- naming
- signatures
- code generation
- error conversion
- handle validation

---

## Compile Tests

Use compile-test fixtures to ensure invalid APIs fail clearly.

Example invalid export:

```rust
#[kotlin_export]
fn invalid(value: UnsupportedType) {}
```

Expected:

```text
error: Unsupported Kotlin bridge type: UnsupportedType
```

---

## JNI Integration Tests

Real JVM tests:

```text
Rust DLL/SO
  ↓
JVM
  ↓
generated Kotlin wrapper
  ↓
actual method invocation
```

---

## Cross-Platform CI

Eventually:

```text
Windows
Ubuntu
macOS Intel
macOS ARM
```

---

# 25. 🔐 Security Requirements

Ketox handles unsafe FFI boundaries.

Requirements:

- validate handles
- never trust arbitrary `Long` values as pointers
- validate array lengths
- validate UTF input
- prevent panic unwinding into JVM
- bound memory allocations
- release JNI local refs
- no use-after-free
- thread-safe global reference management

All unsafe blocks should be localized and documented.

---

# 26. 📈 Performance Goals

Ketox should minimize FFI overhead.

Important principle:

> Do not cross JNI for every tiny operation when work can be batched.

Bad:

```text
Kotlin
↓ JNI
one integer
↓
Rust
```

repeated millions of times.

Better:

```text
Kotlin array/buffer
      ↓ one JNI call
Rust processes entire batch
      ↓
result
```

Primary benchmarks:

- primitive function calls
- strings
- byte arrays
- large primitive arrays
- object method calls
- callbacks
- async completion
- buffer transfer

---

# 27. 🚄 Zero-Copy Vision

Later support direct buffers:

```kotlin
ByteBuffer.allocateDirect(...)
```

Architecture:

```text
Kotlin DirectByteBuffer
       ↓
native memory view
       ↓
Rust slice
```

Useful for:

- AI tensors
- images
- audio
- databases
- networking
- Arrow data

This should be an advanced feature after basic correctness.

---

# 28. 📝 Generated API Philosophy

Generated Kotlin APIs should feel like normal Kotlin.

Prefer:

```kotlin
fun parse(input: String): Document
```

instead of:

```kotlin
external fun nativeParse(
    env: Long,
    pointer: Long,
    value: ByteArray
): Long
```

Low-level JNI details must remain hidden.

---

# 29. 🎨 Kotlin API Quality Goals

Generated code should support:

- nullable types
- Kotlin naming conventions
- exceptions
- `AutoCloseable`
- documentation
- meaningful type names
- `suspend` functions later
- sealed classes/enums later

The public API should not expose JNI terminology.

---

# 30. 📁 Proposed Repository Structure

```text
ketox/
├── Cargo.toml
├── README.md
├── LICENSE
├── CHANGELOG.md
├── ROADMAP.md
│
├── crates/
│   ├── ketox-core/
│   ├── ketox-macros/
│   ├── ketox-codegen/
│   ├── ketox-jni/
│   ├── ketox-cli/
│   └── ketox-runtime-native/
│
├── kotlin/
│   ├── runtime/
│   ├── gradle-plugin/
│   └── test-fixtures/
│
├── examples/
│   ├── hello-world/
│   ├── strings/
│   ├── classes/
│   ├── errors/
│   ├── collections/
│   └── android/
│
├── integration-tests/
│   ├── jvm/
│   └── android/
│
└── docs/
    ├── architecture.md
    ├── type-system.md
    ├── ownership.md
    ├── jni.md
    ├── android.md
    └── security.md
```

---

# 31. 🗺 Development Roadmap

---

# Phase 0 — Research & Contracts 🧭

## Goal

Freeze the initial architecture before building complex bindings.

### Deliverables

- JNI architecture decision
- metadata format
- supported type matrix
- ownership rules
- naming rules
- exception strategy
- native library loading strategy
- platform support policy
- compatibility policy

### Research

Study:

- JNI lifecycle
- JVM class loading
- JNI global/local references
- Rust panic behavior through FFI
- existing Rust JNI crates
- Kotlin/JVM bytecode signatures
- native library packaging

### Definition of Done

- [ ] architecture.md
- [ ] type-system.md
- [ ] ownership.md
- [ ] first metadata schema
- [ ] proof-of-concept Rust → JNI → Kotlin call

### Suggested Release

```text
v0.0.1
```

---

# Phase 1 — Functions MVP 🧩

## Goal

Expose simple Rust functions to Kotlin/JVM.

### Supported Types

```text
Boolean
Byte
Short
Int
Long
Float
Double
String
Unit
```

### Implement

```rust
#[kotlin_export]
fn hello(name: String) -> String
```

Generate:

```kotlin
object NativeApi {
    external fun hello(name: String): String
}
```

### Deliverables

- procedural macro
- metadata generation
- JNI glue generator
- Kotlin wrapper generator
- library loader
- Windows x64
- Linux x64
- macOS ARM64/x64 where CI allows

### Definition of Done

- [ ] exported function works
- [ ] primitives work
- [ ] strings work
- [ ] errors are readable
- [ ] generated code deterministic
- [ ] JNI smoke tests pass
- [ ] example project works

### Suggested Release

```text
v0.1.0
```

---

# Phase 2 — Errors, Option & Collections 📦

## Goal

Make real-world function APIs practical.

### Add

```text
Option<T>
Result<T,E>
Vec<T>
ByteArray
primitive arrays
```

### Example

Rust:

```rust
#[kotlin_export]
fn find_user(id: u64) -> Result<Option<UserData>, UserError>
```

### Kotlin target:

```kotlin
fun findUser(id: ULong): UserData?
```

with errors mapped to exceptions.

### Deliverables

- nullable mapping
- collection conversion
- exception model
- byte-array fast path
- bounded allocations

### Definition of Done

- [ ] Option works
- [ ] Result works
- [ ] Vec primitives work
- [ ] strings collections work
- [ ] large array tests
- [ ] error tests

### Suggested Release

```text
v0.2.0
```

---

# Phase 3 — Rust Structs ↔ Kotlin Classes 🧱

## Goal

Expose stateful Rust objects.

### Add

```rust
#[kotlin_class]
```

```rust
#[kotlin_constructor]
```

```rust
#[kotlin_export]
impl Type
```

### Runtime

Implement:

```text
Handle Registry
type-safe handles
explicit destruction
AutoCloseable wrappers
leak detection in tests
```

### Definition of Done

- [ ] constructors
- [ ] methods
- [ ] immutable methods
- [ ] mutable methods
- [ ] destruction
- [ ] invalid handle detection
- [ ] double-close safety
- [ ] stress tests

### Suggested Release

```text
v0.3.0
```

---

# Phase 4 — Enums, Data Models & Rich Types 🧬

## Goal

Improve Kotlin-native API quality.

### Add

- Rust enums
- Kotlin enums
- sealed-class mapping
- nested structs
- generic collection shapes
- custom type adapters
- better generated docs

### Definition of Done

- [ ] simple enums
- [ ] data-bearing enums
- [ ] sealed types
- [ ] nested structures
- [ ] deterministic serialization metadata

### Suggested Release

```text
v0.4.0
```

---

# Phase 5 — Callbacks & JVM Interaction 🔁

## Goal

Allow Kotlin objects/functions to be called from Rust.

### Add

```text
Kotlin callback → Rust
Rust worker thread → Kotlin callback
global reference management
thread attach/detach
```

### Requirements

- callback lifetime tracking
- JVM thread attachment
- exception propagation
- cancellation design

### Definition of Done

- [ ] synchronous callbacks
- [ ] cross-thread callbacks
- [ ] no leaked global JNI refs
- [ ] callback exception handling
- [ ] concurrency stress tests

### Suggested Release

```text
v0.5.0
```

---

# Phase 6 — Async Rust ↔ Kotlin Coroutines ⚡

## Goal

Map Rust futures to Kotlin suspend functions.

Target:

```rust
#[kotlin_suspend]
async fn fetch(...)
```

Generated:

```kotlin
suspend fun fetch(...)
```

### Implement

- future registry
- completion callback
- Kotlin continuation bridge
- cancellation
- Tokio integration
- exception propagation

### Definition of Done

- [ ] async success
- [ ] async failure
- [ ] cancellation
- [ ] concurrency
- [ ] worker thread integration
- [ ] no continuation leaks

### Suggested Release

```text
v0.6.0
```

---

# Phase 7 — Android Production Support 📱

## Goal

Make Ketox usable in real Android applications.

### Add

- Android NDK build integration
- ABI matrix
- AAR packaging
- native `.so` packaging
- Android library loader
- emulator tests
- physical-device validation

### Initial ABIs

```text
arm64-v8a
armeabi-v7a
x86_64
```

### Definition of Done

- [ ] Android sample app
- [ ] release APK works
- [ ] debug APK works
- [ ] all supported ABIs build
- [ ] AAR consumption works
- [ ] CI Android emulator test

### Suggested Release

```text
v0.7.0
```

---

# Phase 8 — Build-System Integrations 🛠

## Goal

Make Ketox easy to adopt.

### Gradle Plugin

Target:

```kotlin
plugins {
    id("dev.ketox")
}
```

Potential configuration:

```kotlin
ketox {
    crate = "../native"
}
```

### Qutivex Integration

Possible future:

```bash
qutivex native build
```

### Deliverables

- Cargo integration
- incremental native builds
- generated source integration
- artifact packaging
- cache awareness

### Suggested Release

```text
v0.8.0
```

---

# Phase 9 — Kotlin/Native Backend 🖥

## Goal

Support Kotlin/Native through C ABI instead of JNI.

Architecture:

```text
Rust metadata
   ↓
C ABI codegen
   ↓
headers
   ↓
Kotlin cinterop
```

Share:

- metadata model
- type system
- documentation
- macros

Use a separate runtime backend.

### Suggested Release

```text
v0.9.0
```

---

# Phase 10 — Stable 1.0 🏁

## Goal

Freeze stable public contracts.

### Requirements

- stable macros
- stable metadata format
- stable ownership model
- stable Kotlin runtime
- semantic versioning policy
- migration documentation
- API compatibility tests
- robust diagnostics
- complete docs
- security audit of unsafe/JNI code
- benchmarks
- Windows/Linux/macOS
- production Android support

### Release

```text
v1.0.0
```

---

# 32. 🚫 Explicit Non-Goals for Early Versions

Do not attempt all of these immediately:

- reimplementing the Kotlin compiler
- replacing the JVM
- replacing Cargo
- arbitrary Kotlin objects in Phase 1
- async before synchronous APIs are stable
- iOS before JVM/Android are stable
- zero-copy everywhere
- transparent arbitrary generics
- automatic mapping of every Rust crate
- hiding all ownership semantics at the cost of safety

---

# 33. ⚠️ Major Technical Risks

## JNI Complexity

JNI is verbose and easy to misuse.

Mitigation:

```text
centralize unsafe JNI code
generate repetitive bindings
test aggressively
```

---

## Memory Ownership

Rust and JVM have different memory models.

Mitigation:

```text
handle registry
explicit close
strong lifecycle tests
```

---

## Cross-Thread Callbacks

JNI environment values are thread-local.

Mitigation:

```text
store JavaVM
attach Rust worker threads
manage global refs
```

---

## ABI Compatibility

Native libraries differ across OS/architecture.

Mitigation:

```text
publish per-target binaries
embed target metadata
strict loader diagnostics
```

---

## API Explosion

Trying to support every Rust type could make the project unmaintainable.

Mitigation:

```text
small explicit supported type system
custom adapters later
```

---

# 34. 🧭 Design Principles

Ketox should follow these rules:

### 1. Safety Before Magic

Prefer explicit predictable behavior over clever unsafe automation.

### 2. Kotlin-Native Developer Experience

Generated APIs should look like Kotlin, not JNI.

### 3. Rust-Native Developer Experience

Macros should feel natural to Rust developers.

### 4. Small Runtime

Code generation should do most work.

### 5. Deterministic Generation

Same input must produce identical generated code.

### 6. Strong Diagnostics

Errors should explain:

```text
what failed
where
why
how to fix it
```

### 7. Platform Independence in Core

Do not couple metadata models to JNI.

### 8. Benchmark Before Optimizing

Use evidence before adding complicated zero-copy or runtime machinery.

---

# 35. 🧪 Initial Proof-of-Concept

Before building the entire framework, implement one vertical slice:

Rust:

```rust
#[kotlin_export]
pub fn hello(name: String) -> String {
    format!("Hello, {name} from Rust!")
}
```

Kotlin:

```kotlin
fun main() {
    println(RustApi.hello("Kotlin"))
}
```

Expected:

```text
Hello, Kotlin from Rust!
```

The proof-of-concept must demonstrate:

```text
macro
 ↓
metadata
 ↓
generated JNI
 ↓
generated Kotlin
 ↓
Cargo build
 ↓
native library load
 ↓
successful JVM call
```

If this vertical slice is clean, continue Phase 1.

---

# 36. 📊 Success Metrics

Track:

### Developer Experience

- lines of manual JNI required: **0**
- setup commands
- generated code size
- diagnostic quality

### Performance

- JNI call overhead
- string conversion
- byte-array transfer
- large-array transfer
- native execution throughput

### Reliability

- memory leaks
- invalid handle failures
- crashes
- panic containment
- race conditions

### Ecosystem

- supported platforms
- examples
- adoption
- downstream integrations

---

# 37. 🔮 Long-Term Vision

Eventually a developer could create:

```text
High-performance Rust core
        ↓
Ketox
        ↓
Kotlin API
        ├── Android
        ├── JVM backend
        ├── Desktop
        └── Kotlin/Native
```

with one Rust implementation and generated Kotlin-facing APIs.

The project could become infrastructure for:

```text
Rust-powered Kotlin libraries
Rust-powered Android SDKs
high-performance Kotlin data tools
AI runtimes
databases
crypto
image engines
network stacks
developer tools
```

---

# 38. 🌟 Final Product Identity

> **Ketox — Safe, ergonomic Rust bindings for Kotlin.**

Core promise:

```text
Write performance-critical code in Rust.
Expose it naturally to Kotlin.
Avoid manual JNI.
Keep ownership explicit.
Generate the bridge.
```

---

# 39. ✅ Immediate Next Steps

1. Finalize project name.
2. Create repository.
3. Write architecture decision for JNI-first approach.
4. Define Phase 1 supported type matrix.
5. Build the smallest Rust → Kotlin/JVM proof-of-concept.
6. Decide metadata generation mechanism for procedural macros.
7. Add Windows + Linux CI.
8. Publish `v0.0.1` architecture prototype.
9. Begin Phase 1 Functions MVP.

---

# 40. 🗓 Release Roadmap Summary

| Phase | Goal | Suggested Release |
|---|---|---|
| Phase 0 | Architecture + proof of concept | `v0.0.1` |
| Phase 1 | Export Rust functions to Kotlin/JVM | `v0.1.0` |
| Phase 2 | Result, Option, collections | `v0.2.0` |
| Phase 3 | Rust structs ↔ Kotlin classes | `v0.3.0` |
| Phase 4 | Enums and rich data models | `v0.4.0` |
| Phase 5 | Kotlin callbacks | `v0.5.0` |
| Phase 6 | Rust async ↔ Kotlin coroutines | `v0.6.0` |
| Phase 7 | Production Android support | `v0.7.0` |
| Phase 8 | Gradle / Qutivex integration | `v0.8.0` |
| Phase 9 | Kotlin/Native backend | `v0.9.0` |
| Phase 10 | Stable production contracts | `v1.0.0` |

---

# 41. 🏁 End State

```text
Rust Developer
      │
      ▼
#[kotlin_export]
      │
      ▼
Ketox
├── metadata
├── codegen
├── JNI backend
├── ownership runtime
└── packaging
      │
      ▼
Kotlin Developer
      │
      ├── JVM
      ├── Android
      ├── Desktop
      └── Kotlin/Native (future)
```

**No hand-written JNI for normal use.**
