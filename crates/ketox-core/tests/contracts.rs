use ketox_core::{SCHEMA_VERSION, Type, parse_function, parse_source, validate_module};

fn source(source: &str) -> Result<ketox_core::Module, String> {
    parse_source(source, "dev.ketox.demo", "RustMath", "ketox_demo")
}

#[test]
fn primitive_and_string_type_matrix_matches_kotlin_and_jvm() {
    let expected = [
        (Type::Bool, "Boolean", "Z"),
        (Type::I8, "Byte", "B"),
        (Type::I16, "Short", "S"),
        (Type::I32, "Int", "I"),
        (Type::I64, "Long", "J"),
        (Type::F32, "Float", "F"),
        (Type::F64, "Double", "D"),
        (Type::String, "String", "Ljava/lang/String;"),
        (Type::Str, "String", "Ljava/lang/String;"),
        (Type::Unit, "Unit", "V"),
    ];
    for (ty, kotlin, jvm) in expected {
        assert_eq!(ty.kotlin_type(), kotlin);
        assert_eq!(ty.jni_signature(), jvm);
    }
}

#[test]
fn discovers_functions_with_all_supported_inputs_and_sorts_deterministically() {
    let module = source(
        r#"
        fn ignored() {}
        #[ketox::kotlin_export]
        pub fn write_all(flag: bool, tiny: i8, small: i16, count: i32,
                         big: i64, single: f32, double: f64, text: String,
                         borrowed: &str) -> () {}
        #[kotlin_export]
        pub fn add_numbers(mut left: i32, right: i32) -> i32 { left += right; left }
        #[kotlin_export]
        pub fn greet(name: &str) -> String { name.to_owned() }
    "#,
    )
    .unwrap();
    assert_eq!(module.schema_version, SCHEMA_VERSION);
    assert_eq!(
        module
            .functions
            .iter()
            .map(|f| f.rust_name.as_str())
            .collect::<Vec<_>>(),
        ["add_numbers", "greet", "write_all"]
    );
    assert_eq!(module.functions[0].kotlin_name, "addNumbers");
    assert_eq!(module.functions[0].jni_signature(), "(II)I");
    assert_eq!(
        module.functions[1].jni_signature(),
        "(Ljava/lang/String;)Ljava/lang/String;"
    );
    assert_eq!(
        module.functions[2].jni_signature(),
        "(ZBSIJFDLjava/lang/String;Ljava/lang/String;)V"
    );
}

#[test]
fn default_return_is_unit_and_contextual_kotlin_names_are_valid() {
    let function: syn::ItemFn = syn::parse_str("pub fn accept_value(value: i32) {}").unwrap();
    let parsed = parse_function(&function).unwrap();
    assert_eq!(parsed.return_type, Type::Unit);
    assert_eq!(parsed.parameters[0].name, "value");
    assert_eq!(parsed.kotlin_name, "acceptValue");
    assert_eq!(parsed.jni_signature(), "(I)V");
}

#[test]
fn accepts_each_supported_owned_return() {
    for ty in [
        "bool", "i8", "i16", "i32", "i64", "f32", "f64", "String", "()",
    ] {
        source(&format!(
            "#[kotlin_export] pub fn answer() -> {ty} {{ todo!() }}"
        ))
        .unwrap_or_else(|error| panic!("{ty}: {error}"));
    }
}

#[test]
fn rejects_unsupported_types_and_invalid_reference_lifetimes() {
    for ty in [
        "u8",
        "u16",
        "u32",
        "u64",
        "usize",
        "isize",
        "i128",
        "char",
        "Vec<i32>",
        "Option<i32>",
        "std::string::String",
        "&String",
        "&i32",
        "&mut str",
        "&'static str",
        "()",
        "[i32; 2]",
        "(i32, i32)",
        "*const i32",
        "impl Copy",
    ] {
        let result = source(&format!("#[kotlin_export] pub fn accept(value: {ty}) {{}}"));
        assert!(result.is_err(), "accepted unsupported input {ty}");
    }
    for ty in [
        "&str",
        "&'static str",
        "u32",
        "Result<i32, String>",
        "!",
        "Vec<i32>",
    ] {
        assert!(
            source(&format!(
                "#[kotlin_export] pub fn answer() -> {ty} {{ todo!() }}"
            ))
            .is_err(),
            "accepted unsupported output {ty}"
        );
    }
}

#[test]
fn rejects_unsupported_function_declarations() {
    for declaration in [
        "fn answer() {}",
        "pub(crate) fn answer() {}",
        "pub(super) fn answer() {}",
        "pub async fn answer() {}",
        "pub unsafe fn answer() {}",
        "pub const fn answer() {}",
        "pub extern \"C\" fn answer() {}",
        "pub fn answer<T>() {}",
        "pub fn answer<'a>() {}",
        "pub fn answer() where i32: Copy {}",
        "pub fn answer(self) {}",
        "pub fn answer(_: i32) {}",
        "pub fn answer((left, right): (i32, i32)) {}",
        "pub fn answer(ref value: i32) {}",
        "pub fn answer(value @ _: i32) {}",
    ] {
        assert!(
            source(&format!("#[kotlin_export] {declaration}")).is_err(),
            "accepted unsupported declaration {declaration}"
        );
    }
}

#[test]
fn rejects_configuration_attributes_and_export_arguments() {
    for declaration in [
        "#[cfg(feature = \"x\")] #[kotlin_export] pub fn answer() {}",
        "#[cfg_attr(feature = \"x\", inline)] #[kotlin_export] pub fn answer() {}",
        "#[kotlin_export] pub fn answer(#[cfg(feature = \"x\")] value: i32) {}",
        "#[kotlin_export] pub fn answer(#[allow(unused)] value: i32) {}",
        "#[kotlin_export(rename = \"other\")] pub fn answer() {}",
        "#[kotlin_export()] pub fn answer() {}",
        "#[kotlin_export = \"other\"] pub fn answer() {}",
        "#[kotlin_export] #[ketox::kotlin_export] pub fn answer() {}",
        "#[cfg_attr(feature = \"x\", kotlin_export)] pub fn answer() {}",
        "#[cfg_attr(feature = \"x\", cfg_attr(feature = \"y\", ketox::kotlin_export))] pub fn answer() {}",
        "#![cfg(feature = \"x\")] #[kotlin_export] pub fn answer() {}",
        "#[other_macro] #[kotlin_export] pub fn answer() {}",
        "#[unsafe(no_mangle)] #[kotlin_export] pub fn answer() {}",
        "#[export_name = \"answer\"] #[kotlin_export] pub fn answer() {}",
    ] {
        assert!(
            source(declaration).is_err(),
            "accepted unsupported attributes {declaration}"
        );
    }
}

#[test]
fn reports_nested_and_nonfunction_exports_instead_of_silently_ignoring_them() {
    for declaration in [
        "mod nested { #[kotlin_export] pub fn answer() {} }",
        "fn outer() { #[kotlin_export] pub fn inner() {} }",
        "#[kotlin_export] pub fn outer() { #[kotlin_export] pub fn inner() {} }",
        "#[kotlin_export] mod nested {}",
        "#[kotlin_export] struct Counter;",
        "struct Counter; impl Counter { #[kotlin_export] pub fn answer() {} }",
        "trait Counter { #[kotlin_export] fn answer(); }",
        "#[kotlin_export] const ANSWER: i32 = 42;",
        "#[kotlin_export] use std::string::String;",
    ] {
        let error = source(declaration).unwrap_err();
        assert!(
            error.contains("top-level free functions"),
            "{declaration}: {error}"
        );
    }
}

#[test]
fn rejects_nonportable_and_conflicting_names() {
    for declaration in [
        "#[kotlin_export] pub fn r#match() {}",
        "#[kotlin_export] pub fn café() {}",
        "#[kotlin_export] pub fn when() {}",
        "#[kotlin_export] pub fn to_string() -> String { String::new() }",
        "#[kotlin_export] pub fn hash_code() -> i32 { 1 }",
        "#[kotlin_export] pub fn get_class() {}",
        "#[kotlin_export] pub fn notify_all() {}",
        "#[kotlin_export] pub fn answer(r#type: i32) {}",
        "#[kotlin_export] pub fn answer(when: i32) {}",
        "#[kotlin_export] pub fn answer(first: i32, first: i32) {}",
        "#[kotlin_export] pub fn __() {}",
        "#[kotlin_export] pub fn answer() {} #[kotlin_export] pub fn answer() {}",
        "#[kotlin_export] pub fn foo_bar() {} #[kotlin_export] pub fn fooBar() {}",
    ] {
        assert!(
            source(declaration).is_err(),
            "accepted invalid name: {declaration}"
        );
    }
    for (package, class, library) in [
        ("", "Math", "math"),
        ("dev..demo", "Math", "math"),
        ("dev.class.demo", "Math", "math"),
        ("dev.demo", "when", "math"),
        ("dev.demo", "42Math", "math"),
        ("dev.demo", "Math", "../math"),
        ("dev.demo", "Math", "math\"injected"),
    ] {
        assert!(parse_source("", package, class, library).is_err());
    }
}

#[test]
fn metadata_roundtrips_and_rejects_unknown_json_fields() {
    let original =
        source("#[kotlin_export] pub fn add(left: i32, right: i32) -> i32 { left + right }")
            .unwrap();
    let json = serde_json::to_string_pretty(&original).unwrap();
    assert!(json.contains("\"return_type\": \"i32\""));
    let decoded: ketox_core::Module = serde_json::from_str(&json).unwrap();
    assert_eq!(original, decoded);
    validate_module(&decoded).unwrap();
    let mut value = serde_json::to_value(&original).unwrap();
    value["unexpected"] = serde_json::json!(true);
    assert!(serde_json::from_value::<ketox_core::Module>(value).is_err());
}

#[test]
fn validates_untrusted_deserialized_metadata_before_generation() {
    let original = source("#[kotlin_export] pub fn answer(value: i32) -> i32 { value }").unwrap();
    let mut candidate = original.clone();
    candidate.schema_version = 999;
    assert!(validate_module(&candidate).unwrap_err().contains("schema"));
    let mut candidate = original.clone();
    candidate.functions[0].kotlin_name = "injected() {}".into();
    assert!(validate_module(&candidate).is_err());
    let mut candidate = original.clone();
    candidate.functions[0].rust_name = "fn".into();
    candidate.functions[0].kotlin_name = "fn".into();
    assert!(validate_module(&candidate).is_err());
    let mut candidate = original.clone();
    candidate.functions[0].parameters[0].ty = Type::Unit;
    assert!(validate_module(&candidate).is_err());
    let mut candidate = original.clone();
    candidate.functions[0].return_type = Type::Str;
    assert!(validate_module(&candidate).is_err());
    let mut candidate = original.clone();
    candidate.functions.push(candidate.functions[0].clone());
    assert!(validate_module(&candidate).is_err());
    let mut candidate = original;
    candidate.functions[0].parameters[0].name = "bad-name".into();
    assert!(validate_module(&candidate).is_err());
}

#[test]
fn ignores_unrelated_attributes_and_allows_valid_empty_modules() {
    let module =
        source("#[allow(dead_code)] fn helper() {} #[other::kotlin_export] fn unrelated() {}")
            .unwrap();
    assert!(module.functions.is_empty());
    assert!(
        source("this is not Rust")
            .unwrap_err()
            .contains("invalid Rust source")
    );
}

#[test]
fn supports_absolute_qualified_macro_paths() {
    let module = source("#[::ketox::kotlin_export] pub fn answer() -> i32 { 42 }").unwrap();
    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.functions[0].rust_name, "answer");
}

#[test]
fn allows_documentation_and_passive_function_attributes() {
    source(
        r#"
        /// Documentation stays on the Rust function.
        #[allow(unused_variables)]
        #[inline]
        #[must_use]
        #[kotlin_export]
        pub fn answer(value: i32) -> i32 { value }
    "#,
    )
    .unwrap();
}
