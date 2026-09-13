# Ketox ownership and error contract

The Ketox runtime manages primitive values, copied strings, arrays, and native class handles through deterministic ownership boundaries.

## Value lifetimes

Primitive values are copied. A JVM string input is copied to an owned Rust `String`, which is either moved into the exported function or borrowed temporarily for an `&str` parameter. Rust code may keep an owned `String` under normal Rust rules. It cannot keep an input `&str` past the call without first making an owned copy.

A Rust string return is converted to a new JVM string before native return. JNI local references stay within the invoking thread and call; temporary string arrays use local-reference cleanup. Generated glue does not retain a `JNIEnv`, create global references, attach worker threads, or transfer JNI references between threads. These choices follow JNI's [local-reference and thread rules](https://docs.oracle.com/en/java/javase/21/docs/specs/jni/design.html).

Calls run synchronously on the calling JVM thread. Long-running Rust functions therefore block that thread. Async execution, callbacks, cancellation, and cross-thread JVM access need future contracts.

## Native Handle Registry and Struct Ownership (Phase 3)

Rust structs annotated with `#[kotlin_class]` are managed in memory through a thread-safe handle registry in `ketox-jni`:
- **64-bit Opaque Handles:** Instances are assigned a unique, non-zero 64-bit integer identifier (`jlong` in Java/Kotlin).
- **Thread Safety via `Arc<RwLock<T>>`:** Each instance is wrapped in an `Arc<RwLock<T>>`.
  - Immutable methods (`&self`) acquire a read lock, enabling safe concurrent execution across multiple JVM threads.
  - Mutable methods (`&mut self`) acquire an exclusive write lock, ensuring thread-safe mutations without data races.
- **Explicit Destruction:** Generated Kotlin classes implement `java.lang.AutoCloseable`. Calling `close()` or using Kotlin's `.use { ... }` block invokes the native destructor (`destroy_handle`), releasing the Rust memory.
- Stale-Handle & Double-Free Protection:
  - Method calls check handle validity before invocation. Attempting to use an invalid or closed handle throws `IllegalStateException("Native handle <id> is invalid or already closed")`.
  - Calling `close()` multiple times is idempotent and safe; subsequent `close()` invocations on a closed handle are no-ops.

## Pass-by-Value Models, Enums, and Collections (Phase 4)

Types annotated with `#[kotlin_model]` (or `#[kotlin_data]`) and `#[kotlin_enum]` are pass-by-value data carriers:
- **No Native Handles:** Unlike `#[kotlin_class]` instances, value models and enums do not allocate native handles or register in `HANDLE_REGISTRY`.
- **Pure JVM Objects:** Data is serialized/reconstructed across the JNI boundary as standard Kotlin `data class`, `enum class`, and `sealed class` instances.
- **Garbage Collection:** Kotlin-side instances are managed entirely by the JVM Garbage Collector. No manual `close()` or lifecycle tracking is needed.
- **String Collections:** `Vec<String>` and `&[String]` are mapped to Kotlin `Array<String>` (`[Ljava/lang/String;`). Input arrays are fully validated, copied to Rust strings, and cleaned up locally. Output arrays are allocated and populated within JNI local frame boundaries.

## Callback Ownership, Global References, and Thread Lifecycle (Phase 5)

Rust traits annotated with `#[kotlin_callback]` receive Kotlin callback instances or lambdas:
- **Global References (`GlobalRef`):** When a Kotlin callback object is passed into Rust, `ketox-jni` acquires a `JavaVM` instance and creates a `GlobalRef` protecting the Kotlin object from JVM garbage collection while Rust holds it.
- **Automatic Reference Drop:** When the Rust callback proxy drops (e.g. at the end of the function or worker thread), the `GlobalRef` drops automatically, releasing the JVM reference with zero memory leaks.
- **Cross-Thread Thread Attachment (`AttachCurrentThreadAsDaemon`):** When invoking a callback from a Rust background thread (e.g. `std::thread::spawn`), `with_callback_env` checks if the current thread is attached. If unattached, it attaches as a daemon thread and automatically detaches when the scope ends.
- **Exception Safety and Safe Detachment:** If a Kotlin callback throws an uncaught exception, JNI prohibits detaching the thread while an exception is pending. `with_callback_env` catches the exception, extracts its message via `toString()`, and clears it before thread detachment, preventing JVM fatal termination. The error is then safely caught and contained by `ketox_jni::boundary`.

## Exceptions and panics

Every generated entry point places input conversion, the Rust function call, and output conversion inside the runtime boundary.

| Condition | JVM behavior |
| --- | --- |
| Rust unwinding panic | `RuntimeException` containing a Ketox panic message |
| Kotlin callback exception | `RuntimeException` containing the original Kotlin exception message |
| Null string, array, or callback input | `NullPointerException` |
| Unpaired UTF-16 surrogate | `IllegalArgumentException` |
| String or array above configured limit | `IllegalArgumentException` |
| Invalid or closed native handle | `IllegalStateException` |
| `Result::Err(e)` return | `RuntimeException` with `e.to_string()` |
| JNI conversion failure with an existing JVM exception | Preserve the existing exception |
| Other JNI conversion failure | `RuntimeException` |

After installing an exception, native glue returns the zero/null/Unit placeholder required by the JNI signature; that placeholder is not a successful API result. A failure to inspect or install the JVM exception triggers JNI's fatal-error path rather than returning a successful-looking value.

Panic translation requires `panic = "unwind"`, which the workspace sets for release builds and Rust uses for normal development builds. `panic = "abort"`, explicit process termination, and allocation aborts cannot become catchable Kotlin exceptions. Rust's panic hook may still print the panic message even when the JVM receives and catches the translated exception.
