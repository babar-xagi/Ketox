use ketox::kotlin_export;

#[kotlin_export]
pub fn hello(name: &str) -> String { format!("Hello, {name}") }

fn main() {
    assert_eq!(hello("Kotlin"), "Hello, Kotlin");
    assert!(__ketox_metadata_hello.contains("hello"));
}
