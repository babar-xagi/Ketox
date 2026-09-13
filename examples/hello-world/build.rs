fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    let output = std::env::var_os("OUT_DIR").expect("Cargo must set OUT_DIR");
    ketox_codegen::generate_from_file(
        "src/lib.rs",
        "dev.ketox.example",
        "RustApi",
        "ketox_hello",
        output,
    )
    .expect("Ketox binding generation failed");
}
