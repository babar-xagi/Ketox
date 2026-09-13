//! JNI operations shared by generated Ketox bindings.
//!
//! Environments and local references never leave the invoking JVM thread.
pub use jni;
use jni::{
    JNIEnv,
    objects::{JCharArray, JString},
    sys::jstring,
};
use std::panic::{AssertUnwindSafe, catch_unwind};

/// Maximum string size accepted by this prototype, in UTF-16 code units.
pub const MAX_STRING_UNITS: usize = 16 * 1024 * 1024;

#[derive(Debug)]
pub struct BridgeError {
    class: &'static str,
    message: String,
}

impl BridgeError {
    fn new(class: &'static str, message: impl Into<String>) -> Self {
        Self {
            class,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for BridgeError {}

impl From<jni::errors::Error> for BridgeError {
    fn from(error: jni::errors::Error) -> Self {
        Self::new(
            "java/lang/RuntimeException",
            format!("Ketox JNI conversion failed: {error}"),
        )
    }
}

/// Run every conversion and the Rust function behind an unwind barrier.
///
/// `panic = "unwind"` is required. Process aborts and allocation failure cannot
/// be translated into JVM exceptions. A pending JVM exception takes precedence.
pub fn boundary<'local, T>(
    env: &mut JNIEnv<'local>,
    fallback: T,
    call: impl FnOnce(&mut JNIEnv<'local>) -> Result<T, BridgeError>,
) -> T {
    let outcome = catch_unwind(AssertUnwindSafe(|| call(env)));
    let error = match outcome {
        Ok(Ok(value)) => return value,
        Ok(Err(error)) => error,
        Err(payload) => {
            let message = if let Some(message) = payload.downcast_ref::<&str>() {
                format!("Ketox: Rust panic: {message}")
            } else if let Some(message) = payload.downcast_ref::<String>() {
                format!("Ketox: Rust panic: {message}")
            } else {
                "Ketox: Rust panic (non-string payload)".to_owned()
            };
            // A user-provided panic payload can itself panic in Drop. Contain
            // that unwind too; only the second payload must be leaked.
            if let Err(second_payload) = catch_unwind(AssertUnwindSafe(|| drop(payload))) {
                std::mem::forget(second_payload);
            }
            BridgeError::new("java/lang/RuntimeException", message)
        }
    };
    match env.exception_check() {
        Ok(true) => {}
        Ok(false) => {
            if env.throw_new(error.class, &error.message).is_err()
                && !env.exception_check().unwrap_or(false)
            {
                env.fatal_error("Ketox could not report a native error to the JVM");
            }
        }
        Err(_) => env.fatal_error("Ketox could not inspect the JVM exception state"),
    }
    fallback
}

fn check_string_length(length: usize) -> Result<(), BridgeError> {
    if length > MAX_STRING_UNITS {
        return Err(BridgeError::new(
            "java/lang/IllegalArgumentException",
            format!("Ketox string exceeds the {MAX_STRING_UNITS} UTF-16 code unit limit"),
        ));
    }
    Ok(())
}

fn decode_utf16(units: &[u16]) -> Result<String, BridgeError> {
    check_string_length(units.len())?;
    String::from_utf16(units).map_err(|_| {
        BridgeError::new(
            "java/lang/IllegalArgumentException",
            "Ketox String contains unpaired UTF-16 surrogates",
        )
    })
}

/// Copy a non-null JVM string to owned Rust UTF-8, rejecting invalid UTF-16.
/// Uses safe JNI calls; temporary arrays belong to the native call's local frame.
pub fn read_string(env: &mut JNIEnv<'_>, value: &JString<'_>) -> Result<String, BridgeError> {
    if value.is_null() {
        return Err(BridgeError::new(
            "java/lang/NullPointerException",
            "Ketox String argument must not be null",
        ));
    }
    let length = env.call_method(value, "length", "()I", &[])?.i()?;
    check_string_length(length as usize)?;
    let chars: JCharArray<'_> = env
        .call_method(value, "toCharArray", "()[C", &[])?
        .l()?
        .into();
    let chars = env.auto_local(chars);
    let mut units = vec![0; length as usize];
    env.get_char_array_region(&*chars, 0, &mut units)?;
    decode_utf16(&units)
}

/// Allocate a JVM string. The JVM owns the returned reference after JNI returns.
pub fn write_string(env: &mut JNIEnv<'_>, value: impl AsRef<str>) -> Result<jstring, BridgeError> {
    let value = value.as_ref();
    check_string_length(value.encode_utf16().count())?;
    Ok(env.new_string(value)?.into_raw())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_preserves_nuls_and_supplementary_characters() {
        let text = "Hello\0 Kotlin 🦀 नमस्ते";
        assert_eq!(
            decode_utf16(&text.encode_utf16().collect::<Vec<_>>()).unwrap(),
            text
        );
    }

    #[test]
    fn malformed_utf16_is_rejected() {
        for units in [&[0xd800][..], &[0xdc00], &[0xd800, 65]] {
            let error = decode_utf16(units).unwrap_err();
            assert_eq!(error.class, "java/lang/IllegalArgumentException");
        }
    }

    #[test]
    fn allocation_limit_has_explicit_boundary() {
        assert!(check_string_length(MAX_STRING_UNITS).is_ok());
        assert!(check_string_length(MAX_STRING_UNITS + 1).is_err());
    }
}
