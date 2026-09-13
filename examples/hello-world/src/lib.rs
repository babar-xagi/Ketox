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

#[kotlin_export]
pub fn find_user(id: i32) -> Option<String> {
    if id == 42 {
        Some("Alice".to_owned())
    } else {
        None
    }
}

#[kotlin_export]
pub fn greet_opt(name: Option<String>) -> String {
    format!("Hello, {}!", name.unwrap_or_else(|| "stranger".to_owned()))
}

#[kotlin_export]
pub fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("division by zero".to_owned())
    } else {
        Ok(a / b)
    }
}

#[kotlin_export]
pub fn process_bytes(data: &[u8]) -> Vec<u8> {
    data.iter().map(|b| b ^ 0x5a).collect()
}

#[kotlin_export]
pub fn sum_numbers(numbers: &[i32]) -> i64 {
    numbers.iter().map(|&x| x as i64).sum()
}

#[kotlin_export]
pub fn opt_add(a: Option<i32>, b: Option<i32>) -> Option<i32> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x + y),
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        (None, None) => None,
    }
}

#[kotlin_class]
pub struct Vector {
    pub x: f64,
    pub y: f64,
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

    pub fn scale(&mut self, factor: f64) {
        self.x *= factor;
        self.y *= factor;
    }

    pub fn dot(&self, other: &Vector) -> f64 {
        self.x * other.x + self.y * other.y
    }

    pub fn to_string_repr(&self) -> String {
        format!("Vector({}, {})", self.x, self.y)
    }
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
