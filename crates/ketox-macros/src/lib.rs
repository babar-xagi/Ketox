//! Compile-time validation and metadata for exported functions.
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
    let function = syn::parse_macro_input!(item as syn::ItemFn);
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
}
