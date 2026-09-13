use ketox::prelude::*;

#[kotlin_export]
pub fn hello(name: String) -> String {
    format!("Hello, {name} from Rust!")
}

#[kotlin_export]
pub fn add(a: i32, b: i32) -> i32 {
    a.wrapping_add(b)
}

#[kotlin_export]
pub fn echo(value: &str) -> String {
    value.to_owned()
}

#[kotlin_export]
pub fn invert(value: bool) -> bool {
    !value
}

#[kotlin_export]
pub fn echo_byte(value: i8) -> i8 {
    value
}

#[kotlin_export]
pub fn echo_short(value: i16) -> i16 {
    value
}

#[kotlin_export]
pub fn echo_long(value: i64) -> i64 {
    value
}

#[kotlin_export]
pub fn echo_float(value: f32) -> f32 {
    value
}

#[kotlin_export]
pub fn echo_double(value: f64) -> f64 {
    value
}

#[kotlin_export]
pub fn noop() {}

#[kotlin_export]
pub fn fail() -> i32 {
    panic!("intentional smoke-test panic")
}

include!(concat!(env!("OUT_DIR"), "/ketox_jni.rs"));

#[cfg(test)]
mod tests {
    #[test]
    fn metadata_matches_export() {
        let from_macro: ketox::core::Function =
            serde_json::from_str(super::__ketox_metadata_hello).unwrap();
        let from_source = ketox::core::parse_source(
            include_str!("lib.rs"),
            "dev.ketox.example",
            "RustApi",
            "ketox_hello",
        )
        .unwrap();
        assert_eq!(
            &from_macro,
            from_source
                .functions
                .iter()
                .find(|f| f.rust_name == "hello")
                .unwrap()
        );
    }
}
