use ketox::kotlin_export;

#[kotlin_export]
pub async fn invalid(value: i32) -> i32 { value }

fn main() {}
