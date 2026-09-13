use ketox::kotlin_export;

#[kotlin_export]
pub fn invalid(value: usize) -> i32 { value as i32 }

fn main() {}
