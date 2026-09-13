//! Portable metadata and shared validation for the Ketox functions prototype.
//!
//! This crate describes an interface without depending on a JNI runtime. Source
//! discovery deliberately supports only directly declared, top-level functions.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use syn::{spanned::Spanned, visit::Visit};

/// The metadata version understood by this release.
pub const SCHEMA_VERSION: u32 = 4;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Module {
    pub schema_version: u32,
    pub package: String,
    pub class_name: String,
    pub library_name: String,
    pub functions: Vec<Function>,
    #[serde(default)]
    pub classes: Vec<Class>,
    #[serde(default)]
    pub enums: Vec<Enum>,
    #[serde(default)]
    pub models: Vec<Model>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Enum {
    pub rust_name: String,
    pub kotlin_name: String,
    pub variants: Vec<EnumVariant>,
}

impl Enum {
    pub fn is_simple(&self) -> bool {
        self.variants.iter().all(|v| v.fields.is_empty())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnumVariant {
    pub rust_name: String,
    pub kotlin_name: String,
    #[serde(default)]
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    pub rust_name: String,
    pub kotlin_name: String,
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Model {
    pub rust_name: String,
    pub kotlin_name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Class {
    pub rust_name: String,
    pub kotlin_name: String,
    pub constructors: Vec<Constructor>,
    pub methods: Vec<Method>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Constructor {
    pub rust_name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
}

impl Constructor {
    pub fn jni_signature(&self) -> String {
        let mut signature = String::from("(");
        for parameter in &self.parameters {
            signature.push_str(&parameter.ty.jni_signature());
        }
        signature.push_str(")J");
        signature
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Method {
    pub rust_name: String,
    pub kotlin_name: String,
    pub is_mut: bool,
    pub parameters: Vec<Parameter>,
    pub return_type: Type,
}

impl Method {
    pub fn jni_signature(&self) -> String {
        let mut signature = String::from("(J");
        for parameter in &self.parameters {
            signature.push_str(&parameter.ty.jni_signature());
        }
        signature.push(')');
        signature.push_str(&self.return_type.jni_signature());
        signature
    }
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
    StringArray,
    StringSlice,
    Class(String),
    Enum(String),
    Model(String),
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
            Self::StringArray | Self::StringSlice => "Array<String>".to_owned(),
            Self::Class(name) | Self::Enum(name) | Self::Model(name) => name.clone(),
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
            Self::StringArray | Self::StringSlice => "[Ljava/lang/String;".to_owned(),
            Self::Class(_) => "J".to_owned(),
            Self::Enum(name) | Self::Model(name) => format!("L{name};"),
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
                Self::StringArray | Self::StringSlice => "[Ljava/lang/String;".to_owned(),
                Self::Class(_) => "Ljava/lang/Long;".to_owned(),
                Self::Enum(name) | Self::Model(name) => format!("L{name};"),
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
                    "Self" => return Ok(Type::Class("Self".to_owned())),
                    other if other.chars().next().is_some_and(|c| c.is_ascii_uppercase()) => {
                        return Ok(Type::Class(other.to_owned()));
                    }
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
                                "String" => return Ok(Type::StringArray),
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
                syn::Type::Path(path)
                    if path.qself.is_none()
                        && path.path.leading_colon.is_none()
                        && path.path.segments.len() == 1 =>
                {
                    let ident = &path.path.segments[0].ident;
                    let name = ident.to_string();
                    if name != "String"
                        && name != "Option"
                        && name != "Result"
                        && name != "Vec"
                        && name != "Self"
                        && name.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                    {
                        return Ok(Type::Class(name));
                    }
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
                            "String" => return Ok(Type::StringSlice),
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
            "unsupported export return type: use bool, i8/i16/i32/i64, f32/f64, String, Vec<u8/i32/i64/f32/f64/bool/String>, Option<T>, Result<T, E>, enums, models, classes, or ()"
        } else {
            "unsupported export parameter type: use bool, i8/i16/i32/i64, f32/f64, String, &str, Vec/&[u8/i32/i64/f32/f64/bool/String], Option<T>, enums, models, or classes"
        },
    ))
}

fn is_class(attribute: &syn::Attribute) -> bool {
    attribute.path().is_ident("kotlin_class")
        || (attribute.path().segments.len() == 2
            && attribute.path().segments[0].ident == "ketox"
            && attribute.path().segments[1].ident == "kotlin_class")
}

fn is_enum(attribute: &syn::Attribute) -> bool {
    attribute.path().is_ident("kotlin_enum")
        || (attribute.path().segments.len() == 2
            && attribute.path().segments[0].ident == "ketox"
            && attribute.path().segments[1].ident == "kotlin_enum")
}

fn is_model(attribute: &syn::Attribute) -> bool {
    attribute.path().is_ident("kotlin_model")
        || attribute.path().is_ident("kotlin_data")
        || (attribute.path().segments.len() == 2
            && attribute.path().segments[0].ident == "ketox"
            && (attribute.path().segments[1].ident == "kotlin_model"
                || attribute.path().segments[1].ident == "kotlin_data"))
}

fn resolve_type(ty: &mut Type, enum_names: &HashSet<String>, model_names: &HashSet<String>) {
    match ty {
        Type::Class(name) => {
            if enum_names.contains(name) {
                *ty = Type::Enum(name.clone());
            } else if model_names.contains(name) {
                *ty = Type::Model(name.clone());
            }
        }
        Type::Option(inner) => resolve_type(inner, enum_names, model_names),
        Type::Result { ok, .. } => resolve_type(ok, enum_names, model_names),
        _ => {}
    }
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
                "kotlin_export is supported only on top-level free functions and impl blocks; nested exports and exports on other items are unsupported",
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
    let mut classes_map: std::collections::BTreeMap<String, Class> =
        std::collections::BTreeMap::new();
    let mut enums_map: std::collections::BTreeMap<String, Enum> =
        std::collections::BTreeMap::new();
    let mut models_map: std::collections::BTreeMap<String, Model> =
        std::collections::BTreeMap::new();
    let mut nested = NestedExportDetector::default();
    for attribute in &file.attrs {
        nested.visit_attribute(attribute);
    }
    for item in &file.items {
        match item {
            syn::Item::Struct(item_struct) if item_struct.attrs.iter().any(is_class) => {
                let rust_name = item_struct.ident.to_string();
                validate_rust_identifier(&rust_name, "class name")
                    .map_err(|e| syn::Error::new_spanned(&item_struct.ident, e).to_string())?;
                let kotlin_name = rust_name.clone();
                validate_kotlin_identifier(&kotlin_name, "class name")
                    .map_err(|e| syn::Error::new_spanned(&item_struct.ident, e).to_string())?;
                classes_map
                    .entry(rust_name.clone())
                    .or_insert_with(|| Class {
                        rust_name,
                        kotlin_name,
                        constructors: Vec::new(),
                        methods: Vec::new(),
                    });
            }
            syn::Item::Struct(item_struct) if item_struct.attrs.iter().any(is_model) => {
                let rust_name = item_struct.ident.to_string();
                validate_rust_identifier(&rust_name, "model name")
                    .map_err(|e| syn::Error::new_spanned(&item_struct.ident, e).to_string())?;
                let kotlin_name = rust_name.clone();
                validate_kotlin_identifier(&kotlin_name, "model name")
                    .map_err(|e| syn::Error::new_spanned(&item_struct.ident, e).to_string())?;
                let mut fields = Vec::new();
                match &item_struct.fields {
                    syn::Fields::Named(named) => {
                        for field in &named.named {
                            let field_ident = field.ident.as_ref().unwrap();
                            let rust_field_name = field_ident.to_string();
                            validate_rust_identifier(&rust_field_name, "field name")
                                .map_err(|e| syn::Error::new_spanned(field_ident, e).to_string())?;
                            let kt_field_name = kotlin_function_name(&rust_field_name)
                                .map_err(|e| syn::Error::new_spanned(field_ident, e).to_string())?;
                            let ty = parse_type(&field.ty, false)
                                .map_err(|e| syn::Error::new_spanned(&field.ty, e.to_string()).to_string())?;
                            fields.push(Field {
                                rust_name: rust_field_name,
                                kotlin_name: kt_field_name,
                                ty,
                            });
                        }
                    }
                    _ => {
                        return Err(format!("model struct `{rust_name}` must have named fields"));
                    }
                }
                models_map.insert(
                    rust_name.clone(),
                    Model {
                        rust_name,
                        kotlin_name,
                        fields,
                    },
                );
            }
            syn::Item::Enum(item_enum) if item_enum.attrs.iter().any(is_enum) => {
                let rust_name = item_enum.ident.to_string();
                validate_rust_identifier(&rust_name, "enum name")
                    .map_err(|e| syn::Error::new_spanned(&item_enum.ident, e).to_string())?;
                let kotlin_name = rust_name.clone();
                validate_kotlin_identifier(&kotlin_name, "enum name")
                    .map_err(|e| syn::Error::new_spanned(&item_enum.ident, e).to_string())?;
                let mut variants = Vec::new();
                for variant in &item_enum.variants {
                    let var_rust_name = variant.ident.to_string();
                    validate_rust_identifier(&var_rust_name, "variant name")
                        .map_err(|e| syn::Error::new_spanned(&variant.ident, e).to_string())?;
                    let var_kotlin_name = var_rust_name.clone();
                    validate_kotlin_identifier(&var_kotlin_name, "variant name")
                        .map_err(|e| syn::Error::new_spanned(&variant.ident, e).to_string())?;
                    let mut fields = Vec::new();
                    match &variant.fields {
                        syn::Fields::Unit => {}
                        syn::Fields::Named(named) => {
                            for field in &named.named {
                                let field_ident = field.ident.as_ref().unwrap();
                                let rust_field_name = field_ident.to_string();
                                validate_rust_identifier(&rust_field_name, "field name")
                                    .map_err(|e| syn::Error::new_spanned(field_ident, e).to_string())?;
                                let kt_field_name = kotlin_function_name(&rust_field_name)
                                    .map_err(|e| syn::Error::new_spanned(field_ident, e).to_string())?;
                                let ty = parse_type(&field.ty, false)
                                    .map_err(|e| syn::Error::new_spanned(&field.ty, e.to_string()).to_string())?;
                                fields.push(Field {
                                    rust_name: rust_field_name,
                                    kotlin_name: kt_field_name,
                                    ty,
                                });
                            }
                        }
                        syn::Fields::Unnamed(unnamed) => {
                            let count = unnamed.unnamed.len();
                            for (idx, field) in unnamed.unnamed.iter().enumerate() {
                                let name = if count == 1 {
                                    "value".to_string()
                                } else {
                                    format!("v{idx}")
                                };
                                let ty = parse_type(&field.ty, false)
                                    .map_err(|e| syn::Error::new_spanned(&field.ty, e.to_string()).to_string())?;
                                fields.push(Field {
                                    rust_name: idx.to_string(),
                                    kotlin_name: name,
                                    ty,
                                });
                            }
                        }
                    }
                    variants.push(EnumVariant {
                        rust_name: var_rust_name,
                        kotlin_name: var_kotlin_name,
                        fields,
                    });
                }
                enums_map.insert(
                    rust_name.clone(),
                    Enum {
                        rust_name,
                        kotlin_name,
                        variants,
                    },
                );
            }
            syn::Item::Impl(item_impl) if item_impl.attrs.iter().any(is_export) => {
                let syn::Type::Path(self_path) = item_impl.self_ty.as_ref() else {
                    return Err(
                        "Ketox: #[kotlin_export] impl is only supported on struct types"
                            .to_string(),
                    );
                };
                let struct_ident = &self_path.path.segments.last().unwrap().ident;
                let struct_name = struct_ident.to_string();
                let class = classes_map
                    .entry(struct_name.clone())
                    .or_insert_with(|| Class {
                        rust_name: struct_name.clone(),
                        kotlin_name: struct_name.clone(),
                        constructors: Vec::new(),
                        methods: Vec::new(),
                    });
                for impl_item in &item_impl.items {
                    if let syn::ImplItem::Fn(method) = impl_item {
                        if !matches!(method.vis, syn::Visibility::Public(_)) {
                            continue;
                        }
                        let first_arg = method.sig.inputs.first();
                        let is_receiver = matches!(first_arg, Some(syn::FnArg::Receiver(_)));
                        if is_receiver {
                            let syn::FnArg::Receiver(receiver) = first_arg.unwrap() else {
                                unreachable!()
                            };
                            if receiver.reference.is_none() {
                                return Err(format!(
                                    "method `{}` in `{}`: by-value `self` is unsupported; use `&self` or `&mut self`",
                                    method.sig.ident, struct_name
                                ));
                            }
                            let is_mut = receiver.mutability.is_some();
                            let method_rust_name = method.sig.ident.to_string();
                            let method_kotlin_name = kotlin_function_name(&method_rust_name)
                                .map_err(|e| {
                                    syn::Error::new_spanned(&method.sig.ident, e).to_string()
                                })?;
                            let mut params = Vec::new();
                            for arg in method.sig.inputs.iter().skip(1) {
                                let syn::FnArg::Typed(typed) = arg else {
                                    continue;
                                };
                                let syn::Pat::Ident(pat) = typed.pat.as_ref() else {
                                    return Err(
                                        "parameter pattern must be simple identifier".to_string()
                                    );
                                };
                                let name = pat.ident.to_string();
                                let ty = parse_type(&typed.ty, false).map_err(|e| {
                                    format!(
                                        "method `{}` parameter `{}`: {e}",
                                        method_rust_name, name
                                    )
                                })?;
                                params.push(Parameter { name, ty });
                            }
                            let return_type = match &method.sig.output {
                                syn::ReturnType::Default => Type::Unit,
                                syn::ReturnType::Type(_, ty) => {
                                    let mut t = parse_type(ty, true).map_err(|e| {
                                        format!("method `{}` return type: {e}", method_rust_name)
                                    })?;
                                    if t == Type::Class("Self".to_string()) {
                                        t = Type::Class(struct_name.clone());
                                    }
                                    t
                                }
                            };
                            class.methods.push(Method {
                                rust_name: method_rust_name,
                                kotlin_name: method_kotlin_name,
                                is_mut,
                                parameters: params,
                                return_type,
                            });
                            nested.visit_block(&method.block);
                        } else {
                            let method_rust_name = method.sig.ident.to_string();
                            let mut params = Vec::new();
                            for arg in &method.sig.inputs {
                                let syn::FnArg::Typed(typed) = arg else {
                                    continue;
                                };
                                let syn::Pat::Ident(pat) = typed.pat.as_ref() else {
                                    return Err(
                                        "parameter pattern must be simple identifier".to_string()
                                    );
                                };
                                let name = pat.ident.to_string();
                                let ty = parse_type(&typed.ty, false).map_err(|e| {
                                    format!(
                                        "constructor `{}` parameter `{}`: {e}",
                                        method_rust_name, name
                                    )
                                })?;
                                params.push(Parameter { name, ty });
                            }
                            let return_type = match &method.sig.output {
                                syn::ReturnType::Default => {
                                    return Err(format!(
                                        "constructor `{}` in `{}` must return Self or Result<Self, E>",
                                        method_rust_name, struct_name
                                    ));
                                }
                                syn::ReturnType::Type(_, ty) => {
                                    let mut t = parse_type(ty, true).map_err(|e| {
                                        format!(
                                            "constructor `{}` return type: {e}",
                                            method_rust_name
                                        )
                                    })?;
                                    if t == Type::Class("Self".to_string()) {
                                        t = Type::Class(struct_name.clone());
                                    }
                                    if let Type::Result { ref mut ok, .. } = t
                                        && **ok == Type::Class("Self".to_string())
                                    {
                                        **ok = Type::Class(struct_name.clone());
                                    }
                                    t
                                }
                            };
                            class.constructors.push(Constructor {
                                rust_name: method_rust_name,
                                parameters: params,
                                return_type,
                            });
                            nested.visit_block(&method.block);
                        }
                    }
                }
            }
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
    if !functions.is_empty() || !classes_map.is_empty() || !enums_map.is_empty() || !models_map.is_empty() {
        reject_configuration(&file.attrs).map_err(|error| error.to_string())?;
    }

    let enum_names: HashSet<String> = enums_map.keys().cloned().collect();
    let model_names: HashSet<String> = models_map.keys().cloned().collect();

    // Resolve Type candidate references in functions, classes, enums, models
    for function in &mut functions {
        for param in &mut function.parameters {
            resolve_type(&mut param.ty, &enum_names, &model_names);
        }
        resolve_type(&mut function.return_type, &enum_names, &model_names);
    }
    for class in classes_map.values_mut() {
        for constructor in &mut class.constructors {
            for param in &mut constructor.parameters {
                resolve_type(&mut param.ty, &enum_names, &model_names);
            }
            resolve_type(&mut constructor.return_type, &enum_names, &model_names);
        }
        for method in &mut class.methods {
            for param in &mut method.parameters {
                resolve_type(&mut param.ty, &enum_names, &model_names);
            }
            resolve_type(&mut method.return_type, &enum_names, &model_names);
        }
    }

    let mut enums = enums_map.into_values().collect::<Vec<_>>();
    let mut models = models_map.into_values().collect::<Vec<_>>();

    for enum_def in &mut enums {
        for variant in &mut enum_def.variants {
            for field in &mut variant.fields {
                resolve_type(&mut field.ty, &enum_names, &model_names);
            }
        }
    }
    for model in &mut models {
        for field in &mut model.fields {
            resolve_type(&mut field.ty, &enum_names, &model_names);
        }
    }

    functions.sort_by(|a, b| a.rust_name.cmp(&b.rust_name));
    let mut classes = classes_map.into_values().collect::<Vec<_>>();
    for class in &mut classes {
        class.methods.sort_by(|a, b| a.rust_name.cmp(&b.rust_name));
        class
            .constructors
            .sort_by(|a, b| a.rust_name.cmp(&b.rust_name));
    }
    classes.sort_by(|a, b| a.rust_name.cmp(&b.rust_name));
    enums.sort_by(|a, b| a.rust_name.cmp(&b.rust_name));
    models.sort_by(|a, b| a.rust_name.cmp(&b.rust_name));

    let module = Module {
        schema_version: SCHEMA_VERSION,
        package: package.to_owned(),
        class_name: class_name.to_owned(),
        library_name: library_name.to_owned(),
        functions,
        classes,
        enums,
        models,
    };
    validate_module(&module)?;
    Ok(module)
}

/// Check metadata read from JSON before using it to generate code.
///
/// Unlike source discovery, this does not reorder functions or modify metadata.
pub fn validate_module(module: &Module) -> Result<(), String> {
    if module.schema_version != 1
        && module.schema_version != 2
        && module.schema_version != 3
        && module.schema_version != SCHEMA_VERSION
    {
        return Err(format!(
            "unsupported metadata schema version {}; expected 1, 2, 3, or {SCHEMA_VERSION}",
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
        for param in &function.parameters {
            validate_type_reference(
                &param.ty,
                &module.classes,
                &module.enums,
                &module.models,
                &function.rust_name,
            )?;
        }
        validate_type_reference(
            &function.return_type,
            &module.classes,
            &module.enums,
            &module.models,
            &function.rust_name,
        )?;
    }
    for class in &module.classes {
        validate_class(class)?;
        if !rust_names.insert(&class.rust_name) {
            return Err(format!("duplicate exported item `{}`", class.rust_name));
        }
        if !kotlin_names.insert(&class.kotlin_name) {
            return Err(format!(
                "exported items collide on Kotlin name `{}`",
                class.kotlin_name
            ));
        }
        for constructor in &class.constructors {
            for param in &constructor.parameters {
                validate_type_reference(
                    &param.ty,
                    &module.classes,
                    &module.enums,
                    &module.models,
                    &constructor.rust_name,
                )?;
            }
            validate_type_reference(
                &constructor.return_type,
                &module.classes,
                &module.enums,
                &module.models,
                &constructor.rust_name,
            )?;
        }
        for method in &class.methods {
            for param in &method.parameters {
                validate_type_reference(
                    &param.ty,
                    &module.classes,
                    &module.enums,
                    &module.models,
                    &method.rust_name,
                )?;
            }
            validate_type_reference(
                &method.return_type,
                &module.classes,
                &module.enums,
                &module.models,
                &method.rust_name,
            )?;
        }
    }
    for enum_def in &module.enums {
        validate_enum(enum_def)?;
        if !rust_names.insert(&enum_def.rust_name) {
            return Err(format!("duplicate exported item `{}`", enum_def.rust_name));
        }
        if !kotlin_names.insert(&enum_def.kotlin_name) {
            return Err(format!(
                "exported items collide on Kotlin name `{}`",
                enum_def.kotlin_name
            ));
        }
        for variant in &enum_def.variants {
            for field in &variant.fields {
                validate_type_reference(
                    &field.ty,
                    &module.classes,
                    &module.enums,
                    &module.models,
                    &format!("{}::{}", enum_def.rust_name, variant.rust_name),
                )?;
            }
        }
    }
    for model in &module.models {
        validate_model(model)?;
        if !rust_names.insert(&model.rust_name) {
            return Err(format!("duplicate exported item `{}`", model.rust_name));
        }
        if !kotlin_names.insert(&model.kotlin_name) {
            return Err(format!(
                "exported items collide on Kotlin name `{}`",
                model.kotlin_name
            ));
        }
        for field in &model.fields {
            validate_type_reference(
                &field.ty,
                &module.classes,
                &module.enums,
                &module.models,
                &model.rust_name,
            )?;
        }
    }
    Ok(())
}

fn validate_enum(enum_def: &Enum) -> Result<(), String> {
    validate_kotlin_identifier(&enum_def.kotlin_name, "enum name")?;
    validate_rust_identifier(&enum_def.rust_name, "enum name")?;
    if enum_def.variants.is_empty() {
        return Err(format!(
            "enum `{}` must have at least one variant",
            enum_def.rust_name
        ));
    }
    let mut var_rust_names = HashSet::new();
    let mut var_kt_names = HashSet::new();
    for variant in &enum_def.variants {
        validate_kotlin_identifier(&variant.kotlin_name, "variant name")?;
        validate_rust_identifier(&variant.rust_name, "variant name")?;
        if !var_rust_names.insert(&variant.rust_name) {
            return Err(format!(
                "duplicate variant `{}` in enum `{}`",
                variant.rust_name, enum_def.rust_name
            ));
        }
        if !var_kt_names.insert(&variant.kotlin_name) {
            return Err(format!(
                "variants collide on Kotlin name `{}` in enum `{}`",
                variant.kotlin_name, enum_def.rust_name
            ));
        }
        let mut field_names = HashSet::new();
        for field in &variant.fields {
            validate_kotlin_identifier(&field.kotlin_name, "field")?;
            validate_rust_identifier(&field.rust_name, "field")?;
            if !field_names.insert(&field.kotlin_name) {
                return Err(format!(
                    "duplicate field `{}` in variant `{}` of enum `{}`",
                    field.kotlin_name, variant.rust_name, enum_def.rust_name
                ));
            }
            validate_type_position(
                &field.ty,
                false,
                &format!("{}::{}", enum_def.rust_name, variant.rust_name),
            )?;
        }
    }
    Ok(())
}

fn validate_model(model: &Model) -> Result<(), String> {
    validate_kotlin_identifier(&model.kotlin_name, "model name")?;
    validate_rust_identifier(&model.rust_name, "model name")?;
    if model.fields.is_empty() {
        return Err(format!(
            "model struct `{}` must have at least one field",
            model.rust_name
        ));
    }
    let mut field_names = HashSet::new();
    for field in &model.fields {
        validate_kotlin_identifier(&field.kotlin_name, "field")?;
        validate_rust_identifier(&field.rust_name, "field")?;
        if !field_names.insert(&field.kotlin_name) {
            return Err(format!(
                "duplicate field `{}` in model `{}`",
                field.kotlin_name, model.rust_name
            ));
        }
        validate_type_position(&field.ty, false, &model.rust_name)?;
    }
    Ok(())
}

fn validate_type_reference(
    ty: &Type,
    classes: &[Class],
    enums: &[Enum],
    models: &[Model],
    context: &str,
) -> Result<(), String> {
    match ty {
        Type::Class(name) => {
            if !classes.iter().any(|c| c.rust_name == *name) {
                return Err(format!(
                    "unrecognized class `{name}` in `{context}`; exported classes must be defined in the same module"
                ));
            }
            Ok(())
        }
        Type::Enum(name) => {
            if !enums.iter().any(|e| e.rust_name == *name) {
                return Err(format!(
                    "unrecognized enum `{name}` in `{context}`; exported enums must be defined in the same module"
                ));
            }
            Ok(())
        }
        Type::Model(name) => {
            if !models.iter().any(|m| m.rust_name == *name) {
                return Err(format!(
                    "unrecognized model `{name}` in `{context}`; exported models must be defined in the same module"
                ));
            }
            Ok(())
        }
        Type::Option(inner) => validate_type_reference(inner, classes, enums, models, context),
        Type::Result { ok, .. } => validate_type_reference(ok, classes, enums, models, context),
        _ => Ok(()),
    }
}

fn validate_class(class: &Class) -> Result<(), String> {
    validate_kotlin_identifier(&class.kotlin_name, "class name")?;
    validate_rust_identifier(&class.rust_name, "class name")?;
    for constructor in &class.constructors {
        validate_rust_identifier(&constructor.rust_name, "constructor name")?;
        let mut param_names = HashSet::new();
        for param in &constructor.parameters {
            validate_kotlin_identifier(&param.name, "parameter")?;
            validate_rust_identifier(&param.name, "parameter")?;
            if !param_names.insert(&param.name) {
                return Err(format!(
                    "duplicate parameter `{}` in constructor `{}` of class `{}`",
                    param.name, constructor.rust_name, class.rust_name
                ));
            }
            validate_type_position(&param.ty, false, &constructor.rust_name)?;
        }
        validate_type_position(&constructor.return_type, true, &constructor.rust_name)?;
    }
    let mut method_names = HashSet::new();
    let mut method_kt_names = HashSet::new();
    for method in &class.methods {
        let expected = kotlin_function_name(&method.rust_name)?;
        if method.kotlin_name != expected {
            return Err(format!(
                "Kotlin name `{}` for Rust method `{}` must be `{expected}`",
                method.kotlin_name, method.rust_name
            ));
        }
        if !method_names.insert(&method.rust_name) {
            return Err(format!(
                "duplicate method `{}` in class `{}`",
                method.rust_name, class.rust_name
            ));
        }
        if !method_kt_names.insert(&method.kotlin_name) {
            return Err(format!(
                "methods collide on Kotlin name `{}` in class `{}`",
                method.kotlin_name, class.rust_name
            ));
        }
        let mut param_names = HashSet::new();
        for param in &method.parameters {
            validate_kotlin_identifier(&param.name, "parameter")?;
            validate_rust_identifier(&param.name, "parameter")?;
            if !param_names.insert(&param.name) {
                return Err(format!(
                    "duplicate parameter `{}` in method `{}` of class `{}`",
                    param.name, method.rust_name, class.rust_name
                ));
            }
            validate_type_position(&param.ty, false, &method.rust_name)?;
        }
        validate_type_position(&method.return_type, true, &method.rust_name)?;
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
        | Type::StringSlice
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
        Type::Class(name) => {
            if name == "Self" {
                return Err(format!("unresolved Self in `{func_name}`"));
            }
            validate_rust_identifier(name, "class")?;
            validate_kotlin_identifier(name, "class")?;
            Ok(())
        }
        Type::Enum(name) => {
            validate_rust_identifier(name, "enum")?;
            validate_kotlin_identifier(name, "enum")?;
            Ok(())
        }
        Type::Model(name) => {
            validate_rust_identifier(name, "model")?;
            validate_kotlin_identifier(name, "model")?;
            Ok(())
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
