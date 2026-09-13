//! Portable metadata and shared validation for the Ketox functions prototype.
//!
//! This crate describes an interface without depending on a JNI runtime. Source
//! discovery deliberately supports only directly declared, top-level functions.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use syn::{spanned::Spanned, visit::Visit};

/// The metadata version understood by this release.
pub const SCHEMA_VERSION: u32 = 1;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
}

impl Type {
    /// The corresponding Kotlin type.
    pub fn kotlin_type(self) -> &'static str {
        match self {
            Self::Bool => "Boolean",
            Self::I8 => "Byte",
            Self::I16 => "Short",
            Self::I32 => "Int",
            Self::I64 => "Long",
            Self::F32 => "Float",
            Self::F64 => "Double",
            Self::String | Self::Str => "String",
            Self::Unit => "Unit",
        }
    }

    /// The JVM descriptor for this type. Unit is valid only as a return type.
    pub fn jni_signature(self) -> &'static str {
        match self {
            Self::Bool => "Z",
            Self::I8 => "B",
            Self::I16 => "S",
            Self::I32 => "I",
            Self::I64 => "J",
            Self::F32 => "F",
            Self::F64 => "D",
            Self::String | Self::Str => "Ljava/lang/String;",
            Self::Unit => "V",
        }
    }
}

impl Function {
    /// The complete JVM method descriptor, excluding the method name.
    pub fn jni_signature(&self) -> String {
        let mut signature = String::from("(");
        for parameter in &self.parameters {
            signature.push_str(parameter.ty.jni_signature());
        }
        signature.push(')');
        signature.push_str(self.return_type.jni_signature());
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

fn parse_type(ty: &syn::Type, is_return: bool) -> syn::Result<Type> {
    let parsed = match ty {
        syn::Type::Path(path)
            if path.qself.is_none()
                && path.path.leading_colon.is_none()
                && path.path.segments.len() == 1
                && matches!(path.path.segments[0].arguments, syn::PathArguments::None) =>
        {
            match path.path.segments[0].ident.to_string().as_str() {
                "bool" => Some(Type::Bool),
                "i8" => Some(Type::I8),
                "i16" => Some(Type::I16),
                "i32" => Some(Type::I32),
                "i64" => Some(Type::I64),
                "f32" => Some(Type::F32),
                "f64" => Some(Type::F64),
                "String" => Some(Type::String),
                _ => None,
            }
        }
        syn::Type::Reference(reference)
            if !is_return && reference.mutability.is_none() && reference.lifetime.is_none() =>
        {
            match reference.elem.as_ref() {
                syn::Type::Path(path) if path.qself.is_none() && path.path.is_ident("str") => {
                    Some(Type::Str)
                }
                _ => None,
            }
        }
        syn::Type::Tuple(tuple) if is_return && tuple.elems.is_empty() => Some(Type::Unit),
        _ => None,
    };
    parsed.ok_or_else(|| syn::Error::new_spanned(
        ty,
        if is_return {
            "unsupported export return type: use bool, i8/i16/i32/i64, f32/f64, String, or ()"
        } else {
            "unsupported export parameter type: use bool, i8/i16/i32/i64, f32/f64, String, or &str with an elided lifetime"
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
    if module.schema_version != SCHEMA_VERSION {
        return Err(format!(
            "unsupported metadata schema version {}; expected {SCHEMA_VERSION}",
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
        if parameter.ty == Type::Unit {
            return Err(format!(
                "unit parameters are unsupported in `{}`",
                function.rust_name
            ));
        }
    }
    if function.return_type == Type::Str {
        return Err(format!(
            "borrowed string returns are unsupported in `{}`; return String instead",
            function.rust_name
        ));
    }
    Ok(())
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
