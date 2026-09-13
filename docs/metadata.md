# Ketox metadata version 3

`ketox-core::Module` is the in-memory contract. `ketox-metadata.json` is its serialized representation. The [JSON schema](metadata.schema.json) describes the structural shape for schema versions 1, 2, and 3; `ketox-core::validate_module` additionally enforces naming rules, derived Kotlin names, uniqueness, and supported positions for types.

```json
{
  "schema_version": 3,
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
  ],
  "classes": [
    {
      "rust_name": "Vector",
      "kotlin_name": "Vector",
      "constructors": [
        {
          "rust_name": "new",
          "kotlin_name": "new",
          "parameters": [
            { "name": "x", "ty": "f64" },
            { "name": "y", "ty": "f64" }
          ],
          "return_type": { "class": "Vector" }
        }
      ],
      "methods": [
        {
          "rust_name": "magnitude",
          "kotlin_name": "magnitude",
          "is_mut": false,
          "parameters": [],
          "return_type": "f64"
        },
        {
          "rust_name": "scale",
          "kotlin_name": "scale",
          "is_mut": true,
          "parameters": [
            { "name": "factor", "ty": "f64" }
          ],
          "return_type": "unit"
        }
      ]
    }
  ]
}
```

Types serialize as:
- Primitives: `bool`, `i8`, `i16`, `i32`, `i64`, `f32`, `f64`, `string`, `str`, `unit`
- Arrays: `byte_array`, `byte_slice`, `int_array`, `int_slice`, `long_array`, `long_slice`, `float_array`, `float_slice`, `double_array`, `double_slice`, `boolean_array`, `boolean_slice`
- Option: `{"option": <type>}`
- Result: `{"result": {"ok": <type>, "err": "<error_type>"}}`
- Class: `{"class": "<class_name>"}`

Source discovery sorts functions and classes by their Rust names. Code generation preserves the order supplied in validated metadata and emits fixed formatting, without timestamps or machine-specific paths in generated content. Reusing identical metadata yields identical artifacts. JSON consumers must reject unsupported schema versions and unknown fields rather than guessing their meaning.

Version 3 introduces support for exported structs and classes, stateful handles, constructors, methods (`&self` and `&mut self`), and destructors.
