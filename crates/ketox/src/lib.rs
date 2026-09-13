//! ketox3: generated Rust bindings for Kotlin/JVM.
//!
//! Use [`kotlin_export`], [`kotlin_class`], [`kotlin_constructor`], [`kotlin_enum`], [`kotlin_model`], and [`kotlin_callback`]
//! to expose functions, structs, enums, models, and callbacks, then generate bindings from your build script with `ketox-codegen`.
pub use ketox_core as core;
pub use ketox_jni as runtime;
pub use ketox_jni::jni;
pub use ketox_macros::{
    kotlin_callback, kotlin_class, kotlin_constructor, kotlin_enum, kotlin_export, kotlin_model,
};

pub mod prelude {
    pub use crate::{
        kotlin_callback, kotlin_class, kotlin_constructor, kotlin_enum, kotlin_export, kotlin_model,
    };
}
