use ketox::kotlin_export;

#[kotlin_export(rename = "something")]
pub fn invalid(value: i32) -> i32 { value }

fn main() {}
