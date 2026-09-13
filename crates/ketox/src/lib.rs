//! ketox3: generated Rust bindings for Kotlin/JVM.
//!
//! Use [`kotlin_export`], [`kotlin_class`], and [`kotlin_constructor`] to expose
//! functions and structs, then generate bindings from your build script with `ketox-codegen`.
pub use ketox_core as core;
pub use ketox_jni as runtime;
pub use ketox_jni::jni;
pub use ketox_macros::{kotlin_class, kotlin_constructor, kotlin_export};

pub mod prelude {
    pub use crate::{kotlin_class, kotlin_constructor, kotlin_export};
}
