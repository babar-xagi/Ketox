use ketox::kotlin_export;

#[kotlin_export]
pub fn invalid(value: Vec<i32>) -> i32 { value.len() as i32 }

fn main() {}
