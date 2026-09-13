//! JNI operations shared by generated Ketox bindings.
//!
//! Environments and local references never leave the invoking JVM thread.
pub use jni;
use jni::{
    JNIEnv,
    objects::{
        JBooleanArray, JByteArray, JCharArray, JDoubleArray, JFloatArray, JIntArray, JLongArray,
        JObject, JString,
    },
    sys::{
        jboolean, jbooleanArray, jbyte, jbyteArray, jdouble, jdoubleArray, jfloat, jfloatArray,
        jint, jintArray, jlong, jlongArray, jobject, jshort, jstring,
    },
};
use std::any::Any;
use std::collections::HashMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, OnceLock, RwLock};

/// Maximum string size accepted by this prototype, in UTF-16 code units.
pub const MAX_STRING_UNITS: usize = 16 * 1024 * 1024;

/// Maximum array element count accepted by this prototype.
pub const MAX_ARRAY_ELEMENTS: usize = 16 * 1024 * 1024;

#[derive(Debug)]
pub struct BridgeError {
    class: &'static str,
    message: String,
}

impl BridgeError {
    pub fn new(class: &'static str, message: impl Into<String>) -> Self {
        Self {
            class,
            message: message.into(),
        }
    }

    pub fn user_error(message: impl Into<String>) -> Self {
        Self::new("java/lang/RuntimeException", message)
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

pub fn read_opt_string(
    env: &mut JNIEnv<'_>,
    value: &JString<'_>,
) -> Result<Option<String>, BridgeError> {
    if value.is_null() {
        Ok(None)
    } else {
        read_string(env, value).map(Some)
    }
}

pub fn write_opt_string(
    env: &mut JNIEnv<'_>,
    value: Option<impl AsRef<str>>,
) -> Result<jstring, BridgeError> {
    match value {
        Some(s) => write_string(env, s),
        None => Ok(std::ptr::null_mut()),
    }
}

fn check_array_length(length: usize) -> Result<(), BridgeError> {
    if length > MAX_ARRAY_ELEMENTS {
        return Err(BridgeError::new(
            "java/lang/IllegalArgumentException",
            format!("Ketox array length {length} exceeds the {MAX_ARRAY_ELEMENTS} limit"),
        ));
    }
    Ok(())
}

pub fn read_opt_bool(
    env: &mut JNIEnv<'_>,
    value: &JObject<'_>,
) -> Result<Option<bool>, BridgeError> {
    if value.is_null() {
        return Ok(None);
    }
    let res = env.call_method(value, "booleanValue", "()Z", &[])?.z()?;
    Ok(Some(res))
}

pub fn write_opt_bool(env: &mut JNIEnv<'_>, value: Option<bool>) -> Result<jobject, BridgeError> {
    let Some(val) = value else {
        return Ok(std::ptr::null_mut());
    };
    let class = env.find_class("java/lang/Boolean")?;
    let boxed = env
        .call_static_method(
            class,
            "valueOf",
            "(Z)Ljava/lang/Boolean;",
            &[jni::objects::JValue::from(val as jboolean)],
        )?
        .l()?;
    Ok(boxed.into_raw())
}

pub fn read_opt_byte(env: &mut JNIEnv<'_>, value: &JObject<'_>) -> Result<Option<i8>, BridgeError> {
    if value.is_null() {
        return Ok(None);
    }
    let res = env.call_method(value, "byteValue", "()B", &[])?.b()?;
    Ok(Some(res))
}

pub fn write_opt_byte(env: &mut JNIEnv<'_>, value: Option<i8>) -> Result<jobject, BridgeError> {
    let Some(val) = value else {
        return Ok(std::ptr::null_mut());
    };
    let class = env.find_class("java/lang/Byte")?;
    let boxed = env
        .call_static_method(
            class,
            "valueOf",
            "(B)Ljava/lang/Byte;",
            &[jni::objects::JValue::from(val as jbyte)],
        )?
        .l()?;
    Ok(boxed.into_raw())
}

pub fn read_opt_short(
    env: &mut JNIEnv<'_>,
    value: &JObject<'_>,
) -> Result<Option<i16>, BridgeError> {
    if value.is_null() {
        return Ok(None);
    }
    let res = env.call_method(value, "shortValue", "()S", &[])?.s()?;
    Ok(Some(res))
}

pub fn write_opt_short(env: &mut JNIEnv<'_>, value: Option<i16>) -> Result<jobject, BridgeError> {
    let Some(val) = value else {
        return Ok(std::ptr::null_mut());
    };
    let class = env.find_class("java/lang/Short")?;
    let boxed = env
        .call_static_method(
            class,
            "valueOf",
            "(S)Ljava/lang/Short;",
            &[jni::objects::JValue::from(val as jshort)],
        )?
        .l()?;
    Ok(boxed.into_raw())
}

pub fn read_opt_int(env: &mut JNIEnv<'_>, value: &JObject<'_>) -> Result<Option<i32>, BridgeError> {
    if value.is_null() {
        return Ok(None);
    }
    let res = env.call_method(value, "intValue", "()I", &[])?.i()?;
    Ok(Some(res))
}

pub fn write_opt_int(env: &mut JNIEnv<'_>, value: Option<i32>) -> Result<jobject, BridgeError> {
    let Some(val) = value else {
        return Ok(std::ptr::null_mut());
    };
    let class = env.find_class("java/lang/Integer")?;
    let boxed = env
        .call_static_method(
            class,
            "valueOf",
            "(I)Ljava/lang/Integer;",
            &[jni::objects::JValue::from(val as jint)],
        )?
        .l()?;
    Ok(boxed.into_raw())
}

pub fn read_opt_long(
    env: &mut JNIEnv<'_>,
    value: &JObject<'_>,
) -> Result<Option<i64>, BridgeError> {
    if value.is_null() {
        return Ok(None);
    }
    let res = env.call_method(value, "longValue", "()J", &[])?.j()?;
    Ok(Some(res))
}

pub fn write_opt_long(env: &mut JNIEnv<'_>, value: Option<i64>) -> Result<jobject, BridgeError> {
    let Some(val) = value else {
        return Ok(std::ptr::null_mut());
    };
    let class = env.find_class("java/lang/Long")?;
    let boxed = env
        .call_static_method(
            class,
            "valueOf",
            "(J)Ljava/lang/Long;",
            &[jni::objects::JValue::from(val as jlong)],
        )?
        .l()?;
    Ok(boxed.into_raw())
}

pub fn read_opt_float(
    env: &mut JNIEnv<'_>,
    value: &JObject<'_>,
) -> Result<Option<f32>, BridgeError> {
    if value.is_null() {
        return Ok(None);
    }
    let res = env.call_method(value, "floatValue", "()F", &[])?.f()?;
    Ok(Some(res))
}

pub fn write_opt_float(env: &mut JNIEnv<'_>, value: Option<f32>) -> Result<jobject, BridgeError> {
    let Some(val) = value else {
        return Ok(std::ptr::null_mut());
    };
    let class = env.find_class("java/lang/Float")?;
    let boxed = env
        .call_static_method(
            class,
            "valueOf",
            "(F)Ljava/lang/Float;",
            &[jni::objects::JValue::from(val as jfloat)],
        )?
        .l()?;
    Ok(boxed.into_raw())
}

pub fn read_opt_double(
    env: &mut JNIEnv<'_>,
    value: &JObject<'_>,
) -> Result<Option<f64>, BridgeError> {
    if value.is_null() {
        return Ok(None);
    }
    let res = env.call_method(value, "doubleValue", "()D", &[])?.d()?;
    Ok(Some(res))
}

pub fn write_opt_double(env: &mut JNIEnv<'_>, value: Option<f64>) -> Result<jobject, BridgeError> {
    let Some(val) = value else {
        return Ok(std::ptr::null_mut());
    };
    let class = env.find_class("java/lang/Double")?;
    let boxed = env
        .call_static_method(
            class,
            "valueOf",
            "(D)Ljava/lang/Double;",
            &[jni::objects::JValue::from(val as jdouble)],
        )?
        .l()?;
    Ok(boxed.into_raw())
}

pub fn read_byte_vec(env: &mut JNIEnv<'_>, array: &JByteArray<'_>) -> Result<Vec<u8>, BridgeError> {
    if array.is_null() {
        return Err(BridgeError::new(
            "java/lang/NullPointerException",
            "Ketox ByteArray argument must not be null",
        ));
    }
    let len = env.get_array_length(array)? as usize;
    check_array_length(len)?;
    let mut buf = vec![0u8; len];
    let slice: &mut [i8] =
        unsafe { std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut i8, len) };
    env.get_byte_array_region(array, 0, slice)?;
    Ok(buf)
}

pub fn write_byte_array(env: &mut JNIEnv<'_>, bytes: &[u8]) -> Result<jbyteArray, BridgeError> {
    let len = bytes.len();
    check_array_length(len)?;
    let arr = env.new_byte_array(len as i32)?;
    let slice: &[i8] = unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const i8, len) };
    env.set_byte_array_region(&arr, 0, slice)?;
    Ok(arr.into_raw())
}

pub fn read_opt_byte_vec(
    env: &mut JNIEnv<'_>,
    array: &JByteArray<'_>,
) -> Result<Option<Vec<u8>>, BridgeError> {
    if array.is_null() {
        Ok(None)
    } else {
        read_byte_vec(env, array).map(Some)
    }
}

pub fn write_opt_byte_array(
    env: &mut JNIEnv<'_>,
    bytes: Option<impl AsRef<[u8]>>,
) -> Result<jbyteArray, BridgeError> {
    match bytes {
        Some(b) => write_byte_array(env, b.as_ref()),
        None => Ok(std::ptr::null_mut()),
    }
}

pub fn read_int_vec(env: &mut JNIEnv<'_>, array: &JIntArray<'_>) -> Result<Vec<i32>, BridgeError> {
    if array.is_null() {
        return Err(BridgeError::new(
            "java/lang/NullPointerException",
            "Ketox IntArray argument must not be null",
        ));
    }
    let len = env.get_array_length(array)? as usize;
    check_array_length(len)?;
    let mut buf = vec![0i32; len];
    env.get_int_array_region(array, 0, &mut buf)?;
    Ok(buf)
}

pub fn write_int_array(env: &mut JNIEnv<'_>, values: &[i32]) -> Result<jintArray, BridgeError> {
    let len = values.len();
    check_array_length(len)?;
    let arr = env.new_int_array(len as i32)?;
    env.set_int_array_region(&arr, 0, values)?;
    Ok(arr.into_raw())
}

pub fn read_opt_int_vec(
    env: &mut JNIEnv<'_>,
    array: &JIntArray<'_>,
) -> Result<Option<Vec<i32>>, BridgeError> {
    if array.is_null() {
        Ok(None)
    } else {
        read_int_vec(env, array).map(Some)
    }
}

pub fn write_opt_int_array(
    env: &mut JNIEnv<'_>,
    values: Option<impl AsRef<[i32]>>,
) -> Result<jintArray, BridgeError> {
    match values {
        Some(v) => write_int_array(env, v.as_ref()),
        None => Ok(std::ptr::null_mut()),
    }
}

pub fn read_long_vec(
    env: &mut JNIEnv<'_>,
    array: &JLongArray<'_>,
) -> Result<Vec<i64>, BridgeError> {
    if array.is_null() {
        return Err(BridgeError::new(
            "java/lang/NullPointerException",
            "Ketox LongArray argument must not be null",
        ));
    }
    let len = env.get_array_length(array)? as usize;
    check_array_length(len)?;
    let mut buf = vec![0i64; len];
    env.get_long_array_region(array, 0, &mut buf)?;
    Ok(buf)
}

pub fn write_long_array(env: &mut JNIEnv<'_>, values: &[i64]) -> Result<jlongArray, BridgeError> {
    let len = values.len();
    check_array_length(len)?;
    let arr = env.new_long_array(len as i32)?;
    env.set_long_array_region(&arr, 0, values)?;
    Ok(arr.into_raw())
}

pub fn read_opt_long_vec(
    env: &mut JNIEnv<'_>,
    array: &JLongArray<'_>,
) -> Result<Option<Vec<i64>>, BridgeError> {
    if array.is_null() {
        Ok(None)
    } else {
        read_long_vec(env, array).map(Some)
    }
}

pub fn write_opt_long_array(
    env: &mut JNIEnv<'_>,
    values: Option<impl AsRef<[i64]>>,
) -> Result<jlongArray, BridgeError> {
    match values {
        Some(v) => write_long_array(env, v.as_ref()),
        None => Ok(std::ptr::null_mut()),
    }
}

pub fn read_float_vec(
    env: &mut JNIEnv<'_>,
    array: &JFloatArray<'_>,
) -> Result<Vec<f32>, BridgeError> {
    if array.is_null() {
        return Err(BridgeError::new(
            "java/lang/NullPointerException",
            "Ketox FloatArray argument must not be null",
        ));
    }
    let len = env.get_array_length(array)? as usize;
    check_array_length(len)?;
    let mut buf = vec![0f32; len];
    env.get_float_array_region(array, 0, &mut buf)?;
    Ok(buf)
}

pub fn write_float_array(env: &mut JNIEnv<'_>, values: &[f32]) -> Result<jfloatArray, BridgeError> {
    let len = values.len();
    check_array_length(len)?;
    let arr = env.new_float_array(len as i32)?;
    env.set_float_array_region(&arr, 0, values)?;
    Ok(arr.into_raw())
}

pub fn read_opt_float_vec(
    env: &mut JNIEnv<'_>,
    array: &JFloatArray<'_>,
) -> Result<Option<Vec<f32>>, BridgeError> {
    if array.is_null() {
        Ok(None)
    } else {
        read_float_vec(env, array).map(Some)
    }
}

pub fn write_opt_float_array(
    env: &mut JNIEnv<'_>,
    values: Option<impl AsRef<[f32]>>,
) -> Result<jfloatArray, BridgeError> {
    match values {
        Some(v) => write_float_array(env, v.as_ref()),
        None => Ok(std::ptr::null_mut()),
    }
}

pub fn read_double_vec(
    env: &mut JNIEnv<'_>,
    array: &JDoubleArray<'_>,
) -> Result<Vec<f64>, BridgeError> {
    if array.is_null() {
        return Err(BridgeError::new(
            "java/lang/NullPointerException",
            "Ketox DoubleArray argument must not be null",
        ));
    }
    let len = env.get_array_length(array)? as usize;
    check_array_length(len)?;
    let mut buf = vec![0f64; len];
    env.get_double_array_region(array, 0, &mut buf)?;
    Ok(buf)
}

pub fn write_double_array(
    env: &mut JNIEnv<'_>,
    values: &[f64],
) -> Result<jdoubleArray, BridgeError> {
    let len = values.len();
    check_array_length(len)?;
    let arr = env.new_double_array(len as i32)?;
    env.set_double_array_region(&arr, 0, values)?;
    Ok(arr.into_raw())
}

pub fn read_opt_double_vec(
    env: &mut JNIEnv<'_>,
    array: &JDoubleArray<'_>,
) -> Result<Option<Vec<f64>>, BridgeError> {
    if array.is_null() {
        Ok(None)
    } else {
        read_double_vec(env, array).map(Some)
    }
}

pub fn write_opt_double_array(
    env: &mut JNIEnv<'_>,
    values: Option<impl AsRef<[f64]>>,
) -> Result<jdoubleArray, BridgeError> {
    match values {
        Some(v) => write_double_array(env, v.as_ref()),
        None => Ok(std::ptr::null_mut()),
    }
}

pub fn read_boolean_vec(
    env: &mut JNIEnv<'_>,
    array: &JBooleanArray<'_>,
) -> Result<Vec<bool>, BridgeError> {
    if array.is_null() {
        return Err(BridgeError::new(
            "java/lang/NullPointerException",
            "Ketox BooleanArray argument must not be null",
        ));
    }
    let len = env.get_array_length(array)? as usize;
    check_array_length(len)?;
    let mut raw_buf = vec![0u8; len];
    env.get_boolean_array_region(array, 0, &mut raw_buf)?;
    Ok(raw_buf.into_iter().map(|b| b != 0).collect())
}

pub fn write_boolean_array(
    env: &mut JNIEnv<'_>,
    values: &[bool],
) -> Result<jbooleanArray, BridgeError> {
    let len = values.len();
    check_array_length(len)?;
    let arr = env.new_boolean_array(len as i32)?;
    let raw_buf: Vec<u8> = values.iter().map(|&b| if b { 1u8 } else { 0u8 }).collect();
    env.set_boolean_array_region(&arr, 0, &raw_buf)?;
    Ok(arr.into_raw())
}

pub fn read_opt_boolean_vec(
    env: &mut JNIEnv<'_>,
    array: &JBooleanArray<'_>,
) -> Result<Option<Vec<bool>>, BridgeError> {
    if array.is_null() {
        Ok(None)
    } else {
        read_boolean_vec(env, array).map(Some)
    }
}

pub fn write_opt_boolean_array(
    env: &mut JNIEnv<'_>,
    values: Option<impl AsRef<[bool]>>,
) -> Result<jbooleanArray, BridgeError> {
    match values {
        Some(v) => write_boolean_array(env, v.as_ref()),
        None => Ok(std::ptr::null_mut()),
    }
}

#[derive(Clone)]
struct HandleEntry {
    type_name: &'static str,
    instance: Arc<RwLock<Box<dyn Any + Send + Sync>>>,
}

static NEXT_HANDLE: AtomicI64 = AtomicI64::new(1);
static HANDLE_REGISTRY: OnceLock<RwLock<HashMap<i64, HandleEntry>>> = OnceLock::new();

fn handle_registry() -> &'static RwLock<HashMap<i64, HandleEntry>> {
    HANDLE_REGISTRY.get_or_init(|| RwLock::new(HashMap::new()))
}

/// Register an owned Rust object into the global handle registry and return its unique positive handle ID.
pub fn register_handle<T: Any + Send + Sync + 'static>(value: T) -> i64 {
    let handle = NEXT_HANDLE.fetch_add(1, Ordering::Relaxed);
    let entry = HandleEntry {
        type_name: std::any::type_name::<T>(),
        instance: Arc::new(RwLock::new(Box::new(value))),
    };
    handle_registry().write().unwrap().insert(handle, entry);
    handle
}

/// Retrieve the shared Arc lock for the native object registered under `handle`.
pub fn get_handle_arc<T: Any + Send + Sync + 'static>(
    handle: i64,
) -> Result<Arc<RwLock<Box<dyn Any + Send + Sync>>>, BridgeError> {
    if handle == 0 {
        return Err(BridgeError::new(
            "java/lang/IllegalStateException",
            format!(
                "native handle for {} is closed or uninitialized (handle: 0)",
                std::any::type_name::<T>()
            ),
        ));
    }
    let entry = {
        let registry = handle_registry().read().unwrap();
        registry.get(&handle).cloned()
    };
    let Some(entry) = entry else {
        return Err(BridgeError::new(
            "java/lang/IllegalStateException",
            format!(
                "native handle {} for {} was not found (already closed or invalid)",
                handle,
                std::any::type_name::<T>()
            ),
        ));
    };
    Ok(entry.instance)
}

/// Execute a closure borrowing an immutable reference to the native object behind `handle`.
pub fn with_handle<T: Any + Send + Sync + 'static, R>(
    handle: i64,
    f: impl FnOnce(&T) -> Result<R, BridgeError>,
) -> Result<R, BridgeError> {
    if handle == 0 {
        return Err(BridgeError::new(
            "java/lang/IllegalStateException",
            format!(
                "native handle for {} is closed or uninitialized (handle: 0)",
                std::any::type_name::<T>()
            ),
        ));
    }
    let entry = {
        let registry = handle_registry().read().unwrap();
        registry.get(&handle).cloned()
    };
    let Some(entry) = entry else {
        return Err(BridgeError::new(
            "java/lang/IllegalStateException",
            format!(
                "native handle {} for {} was not found (already closed or invalid)",
                handle,
                std::any::type_name::<T>()
            ),
        ));
    };
    let guard = entry.instance.read().map_err(|_| {
        BridgeError::new(
            "java/lang/IllegalStateException",
            "native handle lock poisoned",
        )
    })?;
    let downcast = guard.downcast_ref::<T>().ok_or_else(|| {
        BridgeError::new(
            "java/lang/IllegalStateException",
            format!(
                "native handle {} holds {}, expected {}",
                handle,
                entry.type_name,
                std::any::type_name::<T>()
            ),
        )
    })?;
    f(downcast)
}

/// Execute a closure borrowing a mutable reference to the native object behind `handle`.
pub fn with_handle_mut<T: Any + Send + Sync + 'static, R>(
    handle: i64,
    f: impl FnOnce(&mut T) -> Result<R, BridgeError>,
) -> Result<R, BridgeError> {
    if handle == 0 {
        return Err(BridgeError::new(
            "java/lang/IllegalStateException",
            format!(
                "native handle for {} is closed or uninitialized (handle: 0)",
                std::any::type_name::<T>()
            ),
        ));
    }
    let entry = {
        let registry = handle_registry().read().unwrap();
        registry.get(&handle).cloned()
    };
    let Some(entry) = entry else {
        return Err(BridgeError::new(
            "java/lang/IllegalStateException",
            format!(
                "native handle {} for {} was not found (already closed or invalid)",
                handle,
                std::any::type_name::<T>()
            ),
        ));
    };
    let mut guard = entry.instance.write().map_err(|_| {
        BridgeError::new(
            "java/lang/IllegalStateException",
            "native handle lock poisoned",
        )
    })?;
    let downcast = guard.downcast_mut::<T>().ok_or_else(|| {
        BridgeError::new(
            "java/lang/IllegalStateException",
            format!(
                "native handle {} holds {}, expected {}",
                handle,
                entry.type_name,
                std::any::type_name::<T>()
            ),
        )
    })?;
    f(downcast)
}

/// Destroy and drop the native object registered under `handle`.
/// Closing an already closed (or 0) handle is a safe no-op.
pub fn destroy_handle<T: Any + Send + Sync + 'static>(handle: i64) -> Result<(), BridgeError> {
    if handle == 0 {
        return Ok(());
    }
    let mut registry = handle_registry().write().unwrap();
    let _ = registry.remove(&handle);
    Ok(())
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
        assert!(check_array_length(MAX_ARRAY_ELEMENTS).is_ok());
        assert!(check_array_length(MAX_ARRAY_ELEMENTS + 1).is_err());
    }

    #[test]
    fn handle_registry_lifecycle_and_stale_access() {
        struct Sample {
            value: i32,
        }

        let handle = register_handle(Sample { value: 42 });
        assert!(handle > 0);

        // Immutable read
        let res = with_handle::<Sample, _>(handle, |s| Ok(s.value)).unwrap();
        assert_eq!(res, 42);

        // Mutable write
        with_handle_mut::<Sample, _>(handle, |s| {
            s.value = 100;
            Ok(())
        })
        .unwrap();

        let res2 = with_handle::<Sample, _>(handle, |s| Ok(s.value)).unwrap();
        assert_eq!(res2, 100);

        // Destroy
        destroy_handle::<Sample>(handle).unwrap();

        // Stale handle access throws IllegalStateException
        let stale_err = with_handle::<Sample, _>(handle, |s| Ok(s.value)).unwrap_err();
        assert_eq!(stale_err.class, "java/lang/IllegalStateException");

        // Destroying 0 or already destroyed handle is safe no-op
        assert!(destroy_handle::<Sample>(handle).is_ok());
        assert!(destroy_handle::<Sample>(0).is_ok());
    }
}
