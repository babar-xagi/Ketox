//! ketox3: generated Rust bindings for Kotlin/JVM.
//!
//! Use [`kotlin_export`], [`kotlin_class`], [`kotlin_constructor`], [`kotlin_enum`], and [`kotlin_model`]
//! to expose functions, structs, and enums, then generate bindings from your build script with `ketox-codegen`.
pub use ketox_core as core;
pub use ketox_jni as runtime;
pub use ketox_jni::jni;
pub use ketox_macros::{
    kotlin_class, kotlin_constructor, kotlin_enum, kotlin_export, kotlin_model,
};

pub mod prelude {
    pub use crate::{
        kotlin_class, kotlin_constructor, kotlin_enum, kotlin_export, kotlin_model,
    };
}
