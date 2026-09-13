//! Compile-time validation and metadata for exported functions and classes.
use proc_macro::TokenStream;
use quote::{format_ident, quote};

#[proc_macro_attribute]
pub fn kotlin_export(args: TokenStream, item: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return syn::Error::new(
            proc_macro::Span::call_site().into(),
            "Ketox: #[kotlin_export] does not accept arguments",
        )
        .to_compile_error()
        .into();
    }
    let item_clone = item.clone();
    if let Ok(function) = syn::parse::<syn::ItemFn>(item) {
        let metadata = match ketox_core::parse_function(&function) {
            Ok(metadata) => metadata,
            Err(error) => return error.to_compile_error().into(),
        };
        let json = match serde_json::to_string(&metadata) {
            Ok(json) => json,
            Err(error) => {
                return syn::Error::new_spanned(&function.sig, error.to_string())
                    .to_compile_error()
                    .into();
            }
        };
        let name = format_ident!("__ketox_metadata_{}", function.sig.ident);
        quote! {
            #function
            #[doc(hidden)]
            #[allow(non_upper_case_globals)]
            pub const #name: &str = #json;
        }
        .into()
    } else if let Ok(impl_block) = syn::parse::<syn::ItemImpl>(item_clone) {
        quote! {
            #impl_block
        }
        .into()
    } else {
        syn::Error::new(
            proc_macro::Span::call_site().into(),
            "Ketox: #[kotlin_export] is supported on functions and impl blocks",
        )
        .to_compile_error()
        .into()
    }
}

/// Marks a Rust struct for export as a Kotlin class.
#[proc_macro_attribute]
pub fn kotlin_class(args: TokenStream, item: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return syn::Error::new(
            proc_macro::Span::call_site().into(),
            "Ketox: #[kotlin_class] does not accept arguments",
        )
        .to_compile_error()
        .into();
    }
    let item_struct = syn::parse_macro_input!(item as syn::ItemStruct);
    quote! {
        #item_struct
    }
    .into()
}

/// Marks an exported function as a constructor for a Kotlin class.
#[proc_macro_attribute]
pub fn kotlin_constructor(args: TokenStream, item: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return syn::Error::new(
            proc_macro::Span::call_site().into(),
            "Ketox: #[kotlin_constructor] does not accept arguments",
        )
        .to_compile_error()
        .into();
    }
    item
}
