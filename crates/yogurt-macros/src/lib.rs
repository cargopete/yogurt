//! Procedural macros for yogurt subgraph mappings.
//!
//! The `#[handler]` attribute macro transforms idiomatic Rust functions
//! into graph-node-compatible WASM exports.

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, ItemFn, Pat, Type};

/// Transform a mapping handler function into a graph-node-compatible WASM export.
///
/// # Example
///
/// ```rust,ignore
/// use yogurt_runtime::prelude::*;
/// use crate::generated::TransferEvent;
///
/// #[handler]
/// fn handle_transfer(event: TransferEvent) {
///     // Handler logic here
/// }
/// ```
///
/// This expands to:
///
/// ```rust,ignore
/// fn handle_transfer(event: TransferEvent) {
///     // Handler logic here
/// }
///
/// #[no_mangle]
/// pub extern "C" fn handleTransfer(ptr: u32) {
///     let event = TransferEvent::from_asc_ptr(ptr);
///     handle_transfer(event);
/// }
/// ```
#[proc_macro_attribute]
pub fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let attr_args = attr.to_string();

    // Parse optional name override from #[handler(name = "customName")]
    let export_name = parse_handler_name(&attr_args, &input.sig.ident.to_string());

    // Get the function name and parameter info
    let fn_name = &input.sig.ident;
    let fn_vis = &input.vis;
    let fn_block = &input.block;
    let fn_attrs = &input.attrs;

    // Extract the event parameter (should be exactly one)
    let param = match input.sig.inputs.first() {
        Some(FnArg::Typed(pat_type)) => pat_type,
        _ => {
            return syn::Error::new_spanned(&input.sig, "handler must have exactly one parameter")
                .to_compile_error()
                .into();
        }
    };

    let param_name = match param.pat.as_ref() {
        Pat::Ident(ident) => &ident.ident,
        _ => {
            return syn::Error::new_spanned(&param.pat, "expected identifier for parameter")
                .to_compile_error()
                .into();
        }
    };

    let param_type = &param.ty;

    // Generate the wrapper function name (camelCase for WASM export)
    let wrapper_name = format_ident!("{}", export_name);

    let expanded = quote! {
        // Original function (internal, not exported)
        #(#fn_attrs)*
        #fn_vis fn #fn_name(#param_name: #param_type) #fn_block

        // WASM export wrapper
        #[no_mangle]
        pub extern "C" fn #wrapper_name(ptr: u32) {
            let #param_name = <#param_type as yogurt_runtime::asc::FromAscPtr>::from_asc_ptr(ptr);
            #fn_name(#param_name);
        }
    };

    expanded.into()
}

/// Parse the handler name from attribute arguments or derive from function name.
///
/// Supports:
/// - `#[handler]` -> converts snake_case function name to camelCase
/// - `#[handler(name = "customName")]` -> uses the provided name
fn parse_handler_name(attr_args: &str, fn_name: &str) -> String {
    // Check for name = "..." in attributes
    if let Some(start) = attr_args.find("name") {
        if let Some(eq_pos) = attr_args[start..].find('=') {
            let after_eq = &attr_args[start + eq_pos + 1..];
            if let Some(quote_start) = after_eq.find('"') {
                if let Some(quote_end) = after_eq[quote_start + 1..].find('"') {
                    return after_eq[quote_start + 1..quote_start + 1 + quote_end].to_string();
                }
            }
        }
    }

    // Default: convert snake_case to camelCase
    snake_to_camel(fn_name)
}

/// Convert snake_case to camelCase.
///
/// "handle_transfer" -> "handleTransfer"
fn snake_to_camel(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut capitalize_next = false;

    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snake_to_camel() {
        assert_eq!(snake_to_camel("handle_transfer"), "handleTransfer");
        assert_eq!(snake_to_camel("handle_pair_created"), "handlePairCreated");
        assert_eq!(snake_to_camel("on_block"), "onBlock");
        assert_eq!(snake_to_camel("simple"), "simple");
    }

    #[test]
    fn test_parse_handler_name() {
        assert_eq!(
            parse_handler_name("", "handle_transfer"),
            "handleTransfer"
        );
        assert_eq!(
            parse_handler_name("name = \"customHandler\"", "handle_transfer"),
            "customHandler"
        );
    }
}
