# Ketox metadata version 5

`ketox-core::Module` is the in-memory contract. `ketox-metadata.json` is its serialized representation. The [JSON schema](metadata.schema.json) describes the structural shape for schema versions 1, 2, 3, 4, and 5; `ketox-core::validate_module` additionally enforces naming rules, derived Kotlin names, uniqueness, and supported positions for types.

```json
{
  "schema_version": 5,
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
    },
    {
      "rust_name": "download",
      "kotlin_name": "download",
      "parameters": [
        { "name": "url", "ty": "string" },
        { "name": "listener", "ty": { "callback": "ProgressListener" } }
      ],
      "return_type": "unit"
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
  ],
  "callbacks": [
    {
      "rust_name": "ProgressListener",
      "kotlin_name": "ProgressListener",
      "methods": [
        {
          "rust_name": "on_progress",
          "kotlin_name": "onProgress",
          "parameters": [
            { "name": "current", "ty": "i32" },
            { "name": "total", "ty": "i32" },
            { "name": "message", "ty": "string" }
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
- Primitive arrays/slices: `byte_array`, `byte_slice`, `int_array`, `int_slice`, `long_array`, `long_slice`, `float_array`, `float_slice`, `double_array`, `double_slice`, `boolean_array`, `boolean_slice`
- String collections: `string_array`, `string_slice`
- Option: `{"option": <type>}`
- Result: `{"result": {"ok": <type>, "err": "<error_type>"}}`
- Class: `{"class": "<class_name>"}`
- Enum: `{"enum": "<enum_name>"}`
- Model: `{"model": "<model_name>"}`
- Callback: `{"callback": "<callback_name>"}`

Source discovery sorts functions, classes, enums, models, and callbacks by their Rust names. Code generation preserves the order supplied in validated metadata and emits fixed formatting, without timestamps or machine-specific paths in generated content. Reusing identical metadata yields identical artifacts. JSON consumers must reject unsupported schema versions and unknown fields rather than guessing their meaning.

Version 5 introduces support for:
- Callback traits (`#[kotlin_callback]`)
- Single-method callbacks mapped to Kotlin `fun interface` (supporting idiomatic SAM trailing lambdas)
- Multi-method callbacks mapped to Kotlin `interface`
- Cross-thread callback invocations (`std::thread::spawn` using daemon thread attachment via `AttachCurrentThreadAsDaemon`)
- Automatic resource cleanup (GlobalRef + JavaVM) with zero leaked references
- Clean exception propagation and containment before thread detachment
