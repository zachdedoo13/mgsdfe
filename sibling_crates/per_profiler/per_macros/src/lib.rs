extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, LitStr};

#[proc_macro_attribute]
pub fn time_function(attr: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as ItemFn);
    let name = parse_macro_input!(attr as LitStr).value();

    // Get the function's identifier, signature, and block
    let fn_sig = &input.sig;
    let fn_block = &input.block;

    // Generate the new function body with the placeholders
    let expanded = quote! {
        #fn_sig {
            performance_code::PROFILER.lock().unwrap().start_time_function(#name);
            #fn_block
            performance_code::PROFILER.lock().unwrap().end_time_function(#name);
        }
    };

    // Return the generated tokens
    TokenStream::from(expanded)
}