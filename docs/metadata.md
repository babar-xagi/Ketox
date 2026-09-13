# Ketox metadata version 1

`ketox-core::Module` is the in-memory contract. `ketox-metadata.json` is its serialized representation. The [JSON schema](metadata.schema.json) describes the initial structural shape; `ketox-core::validate_module` additionally enforces naming rules, derived Kotlin names, uniqueness, and supported positions for types.

```json
{
  "schema_version": 1,
  "package": "dev.ketox.example",
  "class_name": "RustApi",
  "library_name": "ketox_hello",
  "functions": [
    {
      "rust_name": "add",
      "kotlin_name": "add",
      "parameters": [
        { "name": "a", "ty": "i32" },
        { "name": "b", "ty": "i32" }
      ],
      "return_type": "i32"
    }
  ]
}
```

Types serialize as `bool`, `i8`, `i16`, `i32`, `i64`, `f32`, `f64`, `string`, `str`, or `unit`. `str` is input-only and `unit` is return-only. The shared model derives JVM method descriptors rather than trusting an extra serialized signature field.

Source discovery sorts functions by Rust name. Code generation preserves the order supplied in validated metadata and emits fixed formatting, without timestamps or machine-specific paths in generated content. Reusing identical metadata yields identical artifacts. JSON consumers must reject unsupported schema versions and unknown fields rather than guessing their meaning.

The export macro also emits a hidden `__ketox_metadata_<rust_name>` JSON string constant containing the corresponding function record. It performs no filesystem writes. Build-script codegen scans the selected source file with the same validator and wraps discovered function records in the module-level settings. The macro constants are useful for inspection; they are not a binary metadata-discovery mechanism.

Version 1 is a prototype contract, not a promise that every future release will use this format unchanged. Changes to serialized meaning must use an explicit schema/version policy and paired generator/runtime changes. Consumers should keep generated Kotlin, JNI glue, and the native library from the same build.
