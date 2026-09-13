//! Ketox: generated Rust bindings for Kotlin/JVM.
//!
//! Use [`kotlin_export`] on public free functions, then generate bindings from
//! your build script with `ketox-codegen`. See `examples/hello-world`.
pub use ketox_core as core;
pub use ketox_jni as runtime;
pub use ketox_jni::jni;
pub use ketox_macros::kotlin_export;

pub mod prelude {
    pub use crate::kotlin_export;
}
