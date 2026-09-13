# Ketox ownership and error contract

The functions prototype passes values and copied strings. It does not expose Rust addresses, native object handles, borrowed JVM buffers, or retained JVM objects.

## Value lifetimes

Primitive values are copied. A JVM string input is copied to an owned Rust `String`, which is either moved into the exported function or borrowed temporarily for an `&str` parameter. Rust code may keep an owned `String` under normal Rust rules. It cannot keep an input `&str` past the call without first making an owned copy.

A Rust string return is converted to a new JVM string before native return. JNI local references stay within the invoking thread and call; temporary string arrays use local-reference cleanup. Generated glue does not retain a `JNIEnv`, create global references, attach worker threads, or transfer JNI references between threads. These choices follow JNI's [local-reference and thread rules](https://docs.oracle.com/en/java/javase/21/docs/specs/jni/design.html).

Calls run synchronously on the calling JVM thread. Long-running Rust functions therefore block that thread. Shared state inside an exported Rust function remains the library author's responsibility. Async execution, callbacks, cancellation, and cross-thread JVM access need future contracts.

## Exceptions and panics

Every generated entry point places input conversion, the Rust function call, and output conversion inside the runtime boundary.

| Condition | JVM behavior |
| --- | --- |
| Rust unwinding panic | `RuntimeException` containing a Ketox panic message |
| Null string input | `NullPointerException` |
| Unpaired UTF-16 surrogate | `IllegalArgumentException` |
| String above the configured limit | `IllegalArgumentException` |
| JNI conversion failure with an existing JVM exception | Preserve the existing exception |
| Other JNI conversion failure | `RuntimeException` |

After installing an exception, native glue returns the zero/null/Unit placeholder required by the JNI signature; that placeholder is not a successful API result. A failure to inspect or install the JVM exception triggers JNI's fatal-error path rather than returning a successful-looking value.

Panic translation requires `panic = "unwind"`, which the workspace sets for release builds and Rust uses for normal development builds. `panic = "abort"`, explicit process termination, and allocation aborts cannot become catchable Kotlin exceptions. Rust's panic hook may still print the panic message even when the JVM receives and catches the translated exception.

The panic boundary is not an object transaction or rollback mechanism. Mutations already performed by Rust code remain its responsibility. Before native classes or handles are added, Ketox must separately define ownership, disposal, stale-handle rejection, borrowing, and synchronization. No such runtime is present in this milestone.
