//! Portable metadata and shared validation for the Ketox functions prototype.
//!
//! This crate describes an interface without depending on a JNI runtime. Source
//! discovery deliberately supports only directly declared, top-level functions.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use syn::{spanned::Spanned, visit::Visit};

/// The metadata version understood by this release.
pub const SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Module {
    pub schema_version: u32,
    pub package: String,
    pub class_name: String,
    pub library_name: String,
    pub functions: Vec<Function>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Function {
    pub rust_name: String,
    pub kotlin_name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Type {
    Bool,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    String,
    Str,
    Unit,
    ByteArray,
    ByteSlice,
    IntArray,
    IntSlice,
    LongArray,
    LongSlice,
    FloatArray,
    FloatSlice,
    DoubleArray,
    DoubleSlice,
    BooleanArray,
    BooleanSlice,
    Option(Box<Type>),
    Result { ok: Box<Type>, err: String },
}

impl Type {
    /// The corresponding Kotlin type.
    pub fn kotlin_type(&self) -> String {
        match self {
            Self::Bool => "Boolean".to_owned(),
            Self::I8 => "Byte".to_owned(),
            Self::I16 => "Short".to_owned(),
            Self::I32 => "Int".to_owned(),
            Self::I64 => "Long".to_owned(),
            Self::F32 => "Float".to_owned(),
            Self::F64 => "Double".to_owned(),
            Self::String | Self::Str => "String".to_owned(),
            Self::Unit => "Unit".to_owned(),
            Self::ByteArray | Self::ByteSlice => "ByteArray".to_owned(),
            Self::IntArray | Self::IntSlice => "IntArray".to_owned(),
            Self::LongArray | Self::LongSlice => "LongArray".to_owned(),
            Self::FloatArray | Self::FloatSlice => "FloatArray".to_owned(),
            Self::DoubleArray | Self::DoubleSlice => "DoubleArray".to_owned(),
            Self::BooleanArray | Self::BooleanSlice => "BooleanArray".to_owned(),
            Self::Option(inner) => format!("{}?", inner.kotlin_type()),
            Self::Result { ok, .. } => ok.kotlin_type(),
        }
    }

    /// The JVM descriptor for this type. Unit is valid only as a return type.
    pub fn jni_signature(&self) -> String {
        match self {
            Self::Bool => "Z".to_owned(),
            Self::I8 => "B".to_owned(),
            Self::I16 => "S".to_owned(),
            Self::I32 => "I".to_owned(),
            Self::I64 => "J".to_owned(),
            Self::F32 => "F".to_owned(),
            Self::F64 => "D".to_owned(),
            Self::String | Self::Str => "Ljava/lang/String;".to_owned(),
            Self::Unit => "V".to_owned(),
            Self::ByteArray | Self::ByteSlice => "[B".to_owned(),
            Self::IntArray | Self::IntSlice => "[I".to_owned(),
            Self::LongArray | Self::LongSlice => "[J".to_owned(),
            Self::FloatArray | Self::FloatSlice => "[F".to_owned(),
            Self::DoubleArray | Self::DoubleSlice => "[D".to_owned(),
            Self::BooleanArray | Self::BooleanSlice => "[Z".to_owned(),
            Self::Option(inner) => match inner.as_ref() {
                Self::Bool => "Ljava/lang/Boolean;".to_owned(),
                Self::I8 => "Ljava/lang/Byte;".to_owned(),
                Self::I16 => "Ljava/lang/Short;".to_owned(),
                Self::I32 => "Ljava/lang/Integer;".to_owned(),
                Self::I64 => "Ljava/lang/Long;".to_owned(),
                Self::F32 => "Ljava/lang/Float;".to_owned(),
                Self::F64 => "Ljava/lang/Double;".to_owned(),
                Self::String | Self::Str => "Ljava/lang/String;".to_owned(),
                Self::ByteArray | Self::ByteSlice => "[B".to_owned(),
                Self::IntArray | Self::IntSlice => "[I".to_owned(),
                Self::LongArray | Self::LongSlice => "[J".to_owned(),
                Self::FloatArray | Self::FloatSlice => "[F".to_owned(),
                Self::DoubleArray | Self::DoubleSlice => "[D".to_owned(),
                Self::BooleanArray | Self::BooleanSlice => "[Z".to_owned(),
                _ => "Ljava/lang/Object;".to_owned(),
            },
            Self::Result { ok, .. } => ok.jni_signature(),
        }
    }
}

impl Function {
    /// The complete JVM method descriptor, excluding the method name.
    pub fn jni_signature(&self) -> String {
        let mut signature = String::from("(");
        for parameter in &self.parameters {
            signature.push_str(&parameter.ty.jni_signature());
        }
        signature.push(')');
        signature.push_str(&self.return_type.jni_signature());
        signature
    }
}

fn is_export(attribute: &syn::Attribute) -> bool {
    is_export_path(attribute.path())
}

fn is_export_path(path: &syn::Path) -> bool {
    path.is_ident("kotlin_export")
        || (path.segments.len() == 2
            && path.segments[0].ident == "ketox"
            && path.segments[1].ident == "kotlin_export")
}

fn contains_conditional_export(meta: &syn::Meta) -> bool {
    if is_export_path(meta.path()) {
        return true;
    }
    let syn::Meta::List(list) = meta else {
        return false;
    };
    if !list.path.is_ident("cfg_attr") {
        return false;
    }
    let Ok(arguments) = list.parse_args_with(
        syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
    ) else {
        return false;
    };
    arguments.iter().skip(1).any(contains_conditional_export)
}

fn reject_configuration(attributes: &[syn::Attribute]) -> syn::Result<()> {
    if let Some(attribute) = attributes
        .iter()
        .find(|attribute| attribute.path().is_ident("cfg") || attribute.path().is_ident("cfg_attr"))
    {
        return Err(syn::Error::new_spanned(
            attribute,
            "conditional compilation on exported functions or parameters is unsupported: source discovery does not expand cfg",
        ));
    }
    Ok(())
}

/// Validate and describe an exported function.
///
/// Accepts safe, synchronous, non-generic `pub fn` declarations. Borrowed `&str`
/// inputs must have an elided lifetime because generated wrappers own the
/// temporary Rust strings. Borrowed outputs and unit inputs are unsupported.
pub fn parse_function(function: &syn::ItemFn) -> syn::Result<Function> {
    reject_configuration(&function.attrs)?;
    for attribute in &function.attrs {
        let passive = [
            "doc",
            "allow",
            "warn",
            "deny",
            "forbid",
            "expect",
            "inline",
            "cold",
            "must_use",
            "deprecated",
            "track_caller",
        ]
        .iter()
        .any(|name| attribute.path().is_ident(name));
        if !is_export(attribute) && !passive {
            return Err(syn::Error::new_spanned(
                attribute,
                "unsupported attribute on an exported function: only documentation, lint, inline, cold, must_use, deprecated, and track_caller attributes are allowed; source discovery cannot expand procedural attributes",
            ));
        }
    }
    let exports: Vec<_> = function
        .attrs
        .iter()
        .filter(|attr| is_export(attr))
        .collect();
    if exports.len() > 1 {
        return Err(syn::Error::new_spanned(
            exports[1],
            "kotlin_export may appear only once",
        ));
    }
    for attribute in exports {
        if !matches!(attribute.meta, syn::Meta::Path(_)) {
            return Err(syn::Error::new_spanned(
                attribute,
                "kotlin_export takes no arguments",
            ));
        }
    }
    if !matches!(function.vis, syn::Visibility::Public(_)) {
        return Err(syn::Error::new_spanned(
            &function.vis,
            "exported functions must use unrestricted pub visibility",
        ));
    }
    let signature = &function.sig;
    if signature.asyncness.is_some()
        || signature.unsafety.is_some()
        || signature.constness.is_some()
        || signature.abi.is_some()
        || signature.variadic.is_some()
        || !signature.generics.params.is_empty()
        || signature.generics.where_clause.is_some()
    {
        return Err(syn::Error::new_spanned(
            signature,
            "exports must be safe, synchronous, non-const Rust functions without generics, where clauses, an extern ABI, or variadic arguments",
        ));
    }
    let rust_name = signature.ident.to_string();
    let kotlin_name = kotlin_function_name(&rust_name)
        .map_err(|message| syn::Error::new_spanned(&signature.ident, message))?;
    let mut parameters = Vec::new();
    for argument in &signature.inputs {
        let syn::FnArg::Typed(argument) = argument else {
            return Err(syn::Error::new_spanned(
                argument,
                "only free functions are supported; self parameters are unsupported",
            ));
        };
        reject_configuration(&argument.attrs)?;
        if !argument.attrs.is_empty() {
            return Err(syn::Error::new_spanned(
                &argument.attrs[0],
                "attributes on exported parameters are unsupported",
            ));
        }
        let syn::Pat::Ident(pattern) = argument.pat.as_ref() else {
            return Err(syn::Error::new_spanned(
                &argument.pat,
                "exported parameters must use simple identifier patterns",
            ));
        };
        if pattern.by_ref.is_some() || pattern.subpat.is_some() {
            return Err(syn::Error::new_spanned(
                pattern,
                "exported parameters must use simple identifier patterns",
            ));
        }
        let name = pattern.ident.to_string();
        validate_kotlin_identifier(&name, "parameter")
            .map_err(|message| syn::Error::new_spanned(&pattern.ident, message))?;
        let ty = parse_type(&argument.ty, false)?;
        parameters.push(Parameter { name, ty });
    }
    let return_type = match &signature.output {
        syn::ReturnType::Default => Type::Unit,
        syn::ReturnType::Type(_, ty) => parse_type(ty, true)?,
    };
    let metadata = Function {
        rust_name,
        kotlin_name,
        parameters,
        return_type,
    };
    validate_function(&metadata).map_err(|message| syn::Error::new_spanned(signature, message))?;
    Ok(metadata)
}

fn primitive_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(path)
            if path.qself.is_none()
                && path.path.leading_colon.is_none()
                && path.path.segments.len() == 1
                && matches!(path.path.segments[0].arguments, syn::PathArguments::None) =>
        {
            Some(path.path.segments[0].ident.to_string())
        }
        _ => None,
    }
}

fn parse_type(ty: &syn::Type, is_return: bool) -> syn::Result<Type> {
    match ty {
        syn::Type::Path(path)
            if path.qself.is_none()
                && path.path.leading_colon.is_none()
                && path.path.segments.len() == 1 =>
        {
            let segment = &path.path.segments[0];
            let ident_str = segment.ident.to_string();
            match &segment.arguments {
                syn::PathArguments::None => match ident_str.as_str() {
                    "bool" => return Ok(Type::Bool),
                    "i8" => return Ok(Type::I8),
                    "i16" => return Ok(Type::I16),
                    "i32" => return Ok(Type::I32),
                    "i64" => return Ok(Type::I64),
                    "f32" => return Ok(Type::F32),
                    "f64" => return Ok(Type::F64),
                    "String" => return Ok(Type::String),
                    _ => {}
                },
                syn::PathArguments::AngleBracketed(args) => {
                    if ident_str == "Option" && args.args.len() == 1 {
                        if let syn::GenericArgument::Type(inner_ty) = &args.args[0] {
                            let inner = parse_type(inner_ty, is_return)?;
                            if inner == Type::Unit {
                                return Err(syn::Error::new_spanned(
                                    inner_ty,
                                    "Option<()> is unsupported: use () directly",
                                ));
                            }
                            if matches!(inner, Type::Option(_)) {
                                return Err(syn::Error::new_spanned(
                                    inner_ty,
                                    "nested Option<Option<T>> is unsupported",
                                ));
                            }
                            if matches!(inner, Type::Result { .. }) {
                                return Err(syn::Error::new_spanned(
                                    inner_ty,
                                    "Option<Result<T, E>> is unsupported; use Result<Option<T>, E> instead",
                                ));
                            }
                            return Ok(Type::Option(Box::new(inner)));
                        }
                    } else if ident_str == "Result" && args.args.len() == 2 {
                        if !is_return {
                            return Err(syn::Error::new_spanned(
                                ty,
                                "Result is only supported as a return type",
                            ));
                        }
                        if let (
                            syn::GenericArgument::Type(ok_ty),
                            syn::GenericArgument::Type(err_ty),
                        ) = (&args.args[0], &args.args[1])
                        {
                            let ok = parse_type(ok_ty, true)?;
                            if matches!(ok, Type::Result { .. }) {
                                return Err(syn::Error::new_spanned(
                                    ok_ty,
                                    "nested Result<Result<T, E>, E2> is unsupported",
                                ));
                            }
                            let err_name = quote::quote!(#err_ty).to_string();
                            return Ok(Type::Result {
                                ok: Box::new(ok),
                                err: err_name,
                            });
                        }
                    } else if ident_str == "Vec" && args.args.len() == 1 {
                        let name = match &args.args[0] {
                            syn::GenericArgument::Type(elem_ty) => primitive_name(elem_ty),
                            _ => None,
                        };
                        if let Some(name) = name {
                            match name.as_str() {
                                "u8" => return Ok(Type::ByteArray),
                                "i32" => return Ok(Type::IntArray),
                                "i64" => return Ok(Type::LongArray),
                                "f32" => return Ok(Type::FloatArray),
                                "f64" => return Ok(Type::DoubleArray),
                                "bool" => return Ok(Type::BooleanArray),
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        syn::Type::Reference(reference)
            if reference.mutability.is_none() && reference.lifetime.is_none() =>
        {
            if is_return {
                return Err(syn::Error::new_spanned(
                    ty,
                    "borrowed returns are unsupported: return an owned type instead",
                ));
            }
            match reference.elem.as_ref() {
                syn::Type::Path(path) if path.qself.is_none() && path.path.is_ident("str") => {
                    return Ok(Type::Str);
                }
                syn::Type::Slice(slice) => {
                    if let Some(name) = primitive_name(slice.elem.as_ref()) {
                        match name.as_str() {
                            "u8" => return Ok(Type::ByteSlice),
                            "i32" => return Ok(Type::IntSlice),
                            "i64" => return Ok(Type::LongSlice),
                            "f32" => return Ok(Type::FloatSlice),
                            "f64" => return Ok(Type::DoubleSlice),
                            "bool" => return Ok(Type::BooleanSlice),
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
        syn::Type::Tuple(tuple) if is_return && tuple.elems.is_empty() => return Ok(Type::Unit),
        _ => {}
    }

    Err(syn::Error::new_spanned(
        ty,
        if is_return {
            "unsupported export return type: use bool, i8/i16/i32/i64, f32/f64, String, Vec<u8/i32/i64/f32/f64/bool>, Option<T>, Result<T, E>, or ()"
        } else {
            "unsupported export parameter type: use bool, i8/i16/i32/i64, f32/f64, String, &str, Vec/&[u8/i32/i64/f32/f64/bool], or Option<T>"
        },
    ))
}

#[derive(Default)]
struct NestedExportDetector {
    error: Option<syn::Error>,
}

impl<'ast> Visit<'ast> for NestedExportDetector {
    fn visit_attribute(&mut self, attribute: &'ast syn::Attribute) {
        if self.error.is_none() && is_export(attribute) {
            self.error = Some(syn::Error::new(
                attribute.span(),
                "kotlin_export is supported only on top-level free functions; nested exports and exports on other items are unsupported",
            ));
        } else if self.error.is_none()
            && attribute.path().is_ident("cfg_attr")
            && contains_conditional_export(&attribute.meta)
        {
            self.error = Some(syn::Error::new_spanned(
                attribute,
                "conditional kotlin_export attributes are unsupported: source discovery does not expand cfg_attr",
            ));
        }
        syn::visit::visit_attribute(self, attribute);
    }
}

/// Discover directly declared top-level exports in one Rust source file.
///
/// This does not expand macros, resolve out-of-line modules, or evaluate cfg.
/// Nested exports and conditional export declarations are rejected explicitly.
pub fn parse_source(
    source: &str,
    package: &str,
    class_name: &str,
    library_name: &str,
) -> Result<Module, String> {
    let file = syn::parse_file(source).map_err(|error| format!("invalid Rust source: {error}"))?;
    let mut functions = Vec::new();
    let mut nested = NestedExportDetector::default();
    for attribute in &file.attrs {
        nested.visit_attribute(attribute);
    }
    for item in &file.items {
        match item {
            syn::Item::Fn(function) if function.attrs.iter().any(is_export) => {
                functions.push(
                    parse_function(function)
                        .map_err(|error| format!("export {}: {error}", function.sig.ident))?,
                );
                nested.visit_block(&function.block);
            }
            other => nested.visit_item(other),
        }
    }
    if let Some(error) = nested.error {
        return Err(error.to_string());
    }
    if !functions.is_empty() {
        reject_configuration(&file.attrs).map_err(|error| error.to_string())?;
    }
    functions.sort_by(|a, b| a.rust_name.cmp(&b.rust_name));
    let module = Module {
        schema_version: SCHEMA_VERSION,
        package: package.to_owned(),
        class_name: class_name.to_owned(),
        library_name: library_name.to_owned(),
        functions,
    };
    validate_module(&module)?;
    Ok(module)
}

/// Check metadata read from JSON before using it to generate code.
///
/// Unlike source discovery, this does not reorder functions or modify metadata.
pub fn validate_module(module: &Module) -> Result<(), String> {
    if module.schema_version != 1 && module.schema_version != SCHEMA_VERSION {
        return Err(format!(
            "unsupported metadata schema version {}; expected 1 or {SCHEMA_VERSION}",
            module.schema_version
        ));
    }
    for segment in module.package.split('.') {
        validate_kotlin_identifier(segment, "package segment")?;
    }
    validate_kotlin_identifier(&module.class_name, "class name")?;
    validate_ascii_identifier(&module.library_name, "library name")?;
    let mut rust_names = HashSet::new();
    let mut kotlin_names = HashSet::new();
    for function in &module.functions {
        validate_function(function)?;
        if !rust_names.insert(&function.rust_name) {
            return Err(format!(
                "duplicate exported Rust function `{}`",
                function.rust_name
            ));
        }
        if !kotlin_names.insert(&function.kotlin_name) {
            return Err(format!(
                "exported functions collide on Kotlin name `{}`",
                function.kotlin_name
            ));
        }
    }
    Ok(())
}

fn validate_function(function: &Function) -> Result<(), String> {
    let expected = kotlin_function_name(&function.rust_name)?;
    if function.kotlin_name != expected {
        return Err(format!(
            "Kotlin name `{}` for Rust function `{}` must be `{expected}`",
            function.kotlin_name, function.rust_name
        ));
    }
    let mut names = HashSet::new();
    for parameter in &function.parameters {
        validate_kotlin_identifier(&parameter.name, "parameter")?;
        validate_rust_identifier(&parameter.name, "parameter")?;
        if !names.insert(&parameter.name) {
            return Err(format!(
                "duplicate parameter `{}` in `{}`",
                parameter.name, function.rust_name
            ));
        }
        validate_type_position(&parameter.ty, false, &function.rust_name)?;
    }
    validate_type_position(&function.return_type, true, &function.rust_name)?;
    Ok(())
}

fn validate_type_position(ty: &Type, is_return: bool, func_name: &str) -> Result<(), String> {
    match ty {
        Type::Unit if !is_return => {
            Err(format!("unit parameters are unsupported in `{func_name}`"))
        }
        Type::Str if is_return => Err(format!(
            "borrowed string returns are unsupported in `{func_name}`; return String instead"
        )),
        Type::ByteSlice
        | Type::IntSlice
        | Type::LongSlice
        | Type::FloatSlice
        | Type::DoubleSlice
        | Type::BooleanSlice
            if is_return =>
        {
            Err(format!(
                "borrowed slice returns are unsupported in `{func_name}`; return Vec instead"
            ))
        }
        Type::Option(inner) => {
            if **inner == Type::Unit {
                return Err(format!("Option<Unit> is unsupported in `{func_name}`"));
            }
            if matches!(**inner, Type::Option(_)) {
                return Err(format!(
                    "nested Option<Option<T>> is unsupported in `{func_name}`"
                ));
            }
            if matches!(**inner, Type::Result { .. }) {
                return Err(format!(
                    "Option<Result<T, E>> is unsupported in `{func_name}`"
                ));
            }
            validate_type_position(inner, is_return, func_name)
        }
        Type::Result { ok, .. } => {
            if !is_return {
                return Err(format!(
                    "Result is only supported as a return type in `{func_name}`"
                ));
            }
            if matches!(**ok, Type::Result { .. }) {
                return Err(format!(
                    "nested Result<Result<T, E>, E2> is unsupported in `{func_name}`"
                ));
            }
            validate_type_position(ok, is_return, func_name)
        }
        _ => Ok(()),
    }
}

fn kotlin_function_name(rust_name: &str) -> Result<String, String> {
    validate_rust_identifier(rust_name, "Rust function name")?;
    let mut result = String::new();
    for part in rust_name.split('_').filter(|part| !part.is_empty()) {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            result.push(if result.is_empty() {
                first.to_ascii_lowercase()
            } else {
                first.to_ascii_uppercase()
            });
            result.extend(chars);
        }
    }
    validate_kotlin_identifier(&result, "Kotlin function name")?;
    if matches!(
        result.as_str(),
        "equals"
            | "hashCode"
            | "toString"
            | "clone"
            | "finalize"
            | "getClass"
            | "notify"
            | "notifyAll"
            | "wait"
    ) {
        return Err(format!(
            "Kotlin function name `{result}` conflicts with an inherited object method"
        ));
    }
    Ok(result)
}

fn validate_rust_identifier(name: &str, kind: &str) -> Result<(), String> {
    validate_ascii_identifier(name, kind)?;
    syn::parse_str::<syn::Ident>(name)
        .map(|_| ())
        .map_err(|_| format!("invalid {kind} `{name}`: Rust keywords are unsupported"))
}

fn validate_ascii_identifier(name: &str, kind: &str) -> Result<(), String> {
    let mut chars = name.chars();
    let valid = chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        && name.chars().any(|c| c != '_');
    if !valid {
        return Err(format!(
            "invalid {kind} `{name}`: use ASCII letters, digits, and underscores, starting with a letter or underscore; raw identifiers are unsupported"
        ));
    }
    Ok(())
}

fn validate_kotlin_identifier(name: &str, kind: &str) -> Result<(), String> {
    validate_ascii_identifier(name, kind)?;
    // Kotlin's contextual keywords (such as `value` and `get`) are valid names.
    // Hard keywords need escaping, which this prototype deliberately rejects.
    if matches!(
        name,
        "as" | "break"
            | "class"
            | "continue"
            | "do"
            | "else"
            | "false"
            | "for"
            | "fun"
            | "if"
            | "in"
            | "interface"
            | "is"
            | "null"
            | "object"
            | "package"
            | "return"
            | "super"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typealias"
            | "typeof"
            | "val"
            | "var"
            | "when"
            | "while"
    ) {
        return Err(format!("{kind} `{name}` is a reserved Kotlin keyword"));
    }
    Ok(())
}
