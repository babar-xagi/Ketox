# Ketox metadata version 4

`ketox-core::Module` is the in-memory contract. `ketox-metadata.json` is its serialized representation. The [JSON schema](metadata.schema.json) describes the structural shape for schema versions 1, 2, 3, and 4; `ketox-core::validate_module` additionally enforces naming rules, derived Kotlin names, uniqueness, and supported positions for types.

```json
{
  "schema_version": 4,
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
  ],
  "enums": [
    {
      "rust_name": "Status",
      "kotlin_name": "Status",
      "variants": [
        { "name": "Idle", "fields": [] },
        { "name": "Running", "fields": [] }
      ]
    },
    {
      "rust_name": "Shape",
      "kotlin_name": "Shape",
      "variants": [
        { "name": "Circle", "fields": [{ "name": "radius", "ty": "f64" }] },
        { "name": "Rectangle", "fields": [{ "name": "w", "ty": "f64" }, { "name": "h", "ty": "f64" }] }
      ]
    }
  ],
  "models": [
    {
      "rust_name": "UserProfile",
      "kotlin_name": "UserProfile",
      "fields": [
        { "name": "id", "ty": "i64" },
        { "name": "username", "ty": "string" },
        { "name": "status", "ty": { "enum": "Status" } }
      ]
    }
  ]
}
```

Types serialize as:
- Primitives: `bool`, `i8`, `i16`, `i32`, `i64`, `f32`, `f64`, `string`, `str`, `unit`
- Primitive arrays/slices: `byte_array`, `byte_slice`, `int_array`, `int_slice`, `long_array`, `long_slice`, `float_array`, `float_slice`, `double_array`, `double_slice`, `boolean_array`, `boolean_slice`
- String collections: `string_array`, `string_slice`
- Option: `{"option": <type>}`
- Result: `{"result": {"ok": <type>, "err": "<error_type>"}}`
- Class: `{"class": "<class_name>"}`
- Enum: `{"enum": "<enum_name>"}`
- Model: `{"model": "<model_name>"}`

Source discovery sorts functions, classes, enums, and models by their Rust names. Code generation preserves the order supplied in validated metadata and emits fixed formatting, without timestamps or machine-specific paths in generated content. Reusing identical metadata yields identical artifacts. JSON consumers must reject unsupported schema versions and unknown fields rather than guessing their meaning.

Version 4 introduces support for:
- Simple fieldless enums (mapped to Kotlin `enum class`)
- Data-bearing enums / ADTs (mapped to Kotlin `sealed class` with `data class` / `data object` variants)
- Data models / value structs (annotated with `#[kotlin_model]`, mapped to Kotlin `data class`)
- String arrays and slices (`Vec<String>` and `&[String]` mapped to Kotlin `Array<String>`)
